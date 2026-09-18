//! Config-driven HTTP translation engine (ADR 0001).

use std::collections::BTreeMap;

use async_trait::async_trait;
use base64::Engine as _;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use reqwest::Client;
use serde_json::Value;

use crate::config::{is_builtin_engine, AppConfig, EngineProfile};
use crate::lang::normalize_lang_code;

use super::{TranslationRequest, TranslationResult, Translator};

pub struct ConfigDrivenTranslator {
    id: String,
    profile: EngineProfile,
    client: Client,
}

impl ConfigDrivenTranslator {
    pub fn from_config(config: &AppConfig, client: Client) -> Result<Self, String> {
        let id = config.engine.active.trim().to_string();
        if id.is_empty() {
            return Err("未选择翻译引擎".into());
        }
        if is_builtin_engine(&id) {
            return Err(format!("`{id}` 是内置引擎，不应走 Config-driven 路径"));
        }
        let profile = config
            .engines
            .get(&id)
            .cloned()
            .ok_or_else(|| {
                format!(
                    "未找到引擎配置 `[engines.{id}]`。请在 data/config.toml 中添加 Engine Profile。"
                )
            })?;
        validate_profile(&id, &profile)?;
        Ok(Self {
            id,
            profile,
            client,
        })
    }
}

#[async_trait]
impl Translator for ConfigDrivenTranslator {
    fn id(&self) -> &str {
        &self.id
    }

    async fn translate(&self, req: &TranslationRequest) -> Result<TranslationResult, String> {
        if req.text.trim().is_empty() {
            return Err("翻译文本为空".into());
        }
        if req.target_lang.trim().is_empty() {
            return Err("目标语言不能为空".into());
        }

        let ctx = TemplateContext::from_request(&self.profile, req);
        let method = self.profile.method.trim().to_ascii_uppercase();
        if method != "GET" && method != "POST" {
            return Err(format!("不支持的 HTTP 方法 `{}`（仅 GET/POST）", self.profile.method));
        }

        let url = render_template(&self.profile.url, &ctx)?;
        let mut url = reqwest::Url::parse(&url).map_err(|e| format!("引擎 URL 无效: {e}"))?;

        let mut query_pairs: Vec<(String, String)> = Vec::new();
        for (k, v) in &self.profile.query {
            let rendered = render_template(v, &ctx)?;
            if rendered.is_empty() {
                continue;
            }
            query_pairs.push((k.clone(), rendered));
        }
        apply_query_auth(&self.profile, &ctx, &mut query_pairs)?;
        if !query_pairs.is_empty() {
            let mut pairs = url.query_pairs_mut();
            for (k, v) in &query_pairs {
                pairs.append_pair(k, v);
            }
        }

        let mut headers = HeaderMap::new();
        for (k, v) in &self.profile.headers {
            let name = HeaderName::from_bytes(k.as_bytes())
                .map_err(|e| format!("无效 Header 名 `{k}`: {e}"))?;
            let rendered = render_template(v, &ctx)?;
            let value = HeaderValue::from_str(&rendered)
                .map_err(|e| format!("无效 Header 值 `{k}`: {e}"))?;
            headers.insert(name, value);
        }
        apply_header_auth(&self.profile, &ctx, &mut headers)?;

        let body_type = self.profile.body_type.trim().to_ascii_lowercase();
        let mut request = match method.as_str() {
            "GET" => self.client.get(url),
            _ => self.client.post(url),
        };
        request = request.headers(headers);

        request = match body_type.as_str() {
            "none" | "" => request,
            "form" => {
                let form = render_body_map(&self.profile.body, &ctx)?;
                let mut encoded = url::form_urlencoded::Serializer::new(String::new());
                for (k, v) in &form {
                    encoded.append_pair(k, v);
                }
                request
                    .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(encoded.finish())
            }
            "json" => {
                let map = render_body_map(&self.profile.body, &ctx)?;
                let json = Value::Object(
                    map.into_iter()
                        .map(|(k, v)| (k, Value::String(v)))
                        .collect(),
                );
                request.json(&json)
            }
            other => {
                return Err(format!(
                    "不支持的 body_type `{other}`（支持：json / form / none）"
                ))
            }
        };

        let response = request
            .send()
            .await
            .map_err(|e| format!("Config-driven 翻译请求失败: {e}"))?;
        let status = response.status();
        let raw = response
            .text()
            .await
            .map_err(|e| format!("读取翻译响应失败: {e}"))?;

        if let Some(err_path) = self
            .profile
            .error_path
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            if let Ok(json) = serde_json::from_str::<Value>(&raw) {
                if let Ok(Some(msg)) = json_path_string(&json, err_path) {
                    if !msg.trim().is_empty() && !status.is_success() {
                        return Err(format!("翻译失败: {}", msg.trim()));
                    }
                }
            }
        }

        if !status.is_success() {
            return Err(format!(
                "翻译 HTTP 错误 ({status})：{}",
                truncate(&raw, 200)
            ));
        }

        let json: Value = serde_json::from_str(&raw).map_err(|e| {
            format!(
                "解析翻译 JSON 失败: {e}; body={}",
                truncate(&raw, 200)
            )
        })?;

        let text = json_path_string(&json, &self.profile.text_path)?
            .unwrap_or_default()
            .trim()
            .to_string();
        if text.is_empty() {
            return Err(format!(
                "响应路径 `{}` 未得到译文",
                self.profile.text_path
            ));
        }

        Ok(TranslationResult {
            engine: self.id.clone(),
            text,
            source_lang: req.source_lang.clone(),
            target_lang: normalize_lang_code(&req.target_lang),
            detected_source_lang: None,
        })
    }
}

fn validate_profile(id: &str, profile: &EngineProfile) -> Result<(), String> {
    if profile.url.trim().is_empty() {
        return Err(format!("`[engines.{id}]` 缺少 url"));
    }
    if profile.text_path.trim().is_empty() {
        return Err(format!("`[engines.{id}]` 缺少 text_path"));
    }
    let auth = profile.auth.trim().to_ascii_lowercase();
    match auth.as_str() {
        "none" | "header" | "query" | "bearer" | "basic" => Ok(()),
        other => Err(format!(
            "`[engines.{id}]` 不支持的 auth `{other}`（支持：none / header / query / bearer / basic）"
        )),
    }
}

struct TemplateContext {
    text: String,
    source_lang: String,
    target_lang: String,
    extra: BTreeMap<String, String>,
}

impl TemplateContext {
    fn from_request(profile: &EngineProfile, req: &TranslationRequest) -> Self {
        let source_raw = normalize_lang_code(&req.source_lang);
        let target_raw = normalize_lang_code(&req.target_lang);
        Self {
            text: req.text.trim().to_string(),
            source_lang: map_lang(&profile.lang_map, &source_raw),
            target_lang: map_lang(&profile.lang_map, &target_raw),
            extra: profile.extra.clone(),
        }
    }
}

fn map_lang(lang_map: &BTreeMap<String, String>, code: &str) -> String {
    if let Some(mapped) = lang_map.get(code) {
        return mapped.clone();
    }
    // Case-insensitive key fallback.
    if let Some((_, mapped)) = lang_map
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(code))
    {
        return mapped.clone();
    }
    code.to_string()
}

fn render_template(input: &str, ctx: &TemplateContext) -> Result<String, String> {
    let mut out = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '{' && i + 1 < chars.len() && chars[i + 1] == '{' {
            if let Some(end) = find_close(&chars, i + 2) {
                let key: String = chars[i + 2..end].iter().collect();
                let key = key.trim();
                let value = resolve_placeholder(key, ctx)?;
                out.push_str(&value);
                i = end + 2;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    Ok(out)
}

fn find_close(chars: &[char], from: usize) -> Option<usize> {
    let mut j = from;
    while j + 1 < chars.len() {
        if chars[j] == '}' && chars[j + 1] == '}' {
            return Some(j);
        }
        j += 1;
    }
    None
}

fn resolve_placeholder(key: &str, ctx: &TemplateContext) -> Result<String, String> {
    if key == "text" {
        return Ok(ctx.text.clone());
    }
    if key == "source_lang" {
        return Ok(ctx.source_lang.clone());
    }
    if key == "target_lang" {
        return Ok(ctx.target_lang.clone());
    }
    if let Some(rest) = key.strip_prefix("extra.") {
        let name = rest.trim();
        if name.is_empty() {
            return Err("占位符 `{{extra.}}` 无效".into());
        }
        return Ok(ctx.extra.get(name).cloned().unwrap_or_default());
    }
    Err(format!(
        "未知占位符 `{{{{{key}}}}}`（支持：text / source_lang / target_lang / extra.*）"
    ))
}

fn render_body_map(
    body: &BTreeMap<String, String>,
    ctx: &TemplateContext,
) -> Result<BTreeMap<String, String>, String> {
    let mut out = BTreeMap::new();
    for (k, v) in body {
        let rendered = render_template(v, &ctx)?;
        if rendered.is_empty() {
            continue;
        }
        out.insert(k.clone(), rendered);
    }
    Ok(out)
}

fn apply_header_auth(
    profile: &EngineProfile,
    ctx: &TemplateContext,
    headers: &mut HeaderMap,
) -> Result<(), String> {
    let auth = profile.auth.trim().to_ascii_lowercase();
    match auth.as_str() {
        "none" => Ok(()),
        "header" => {
            let name = profile
                .auth_header
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .unwrap_or("Authorization");
            let raw = profile
                .auth_value
                .as_deref()
                .ok_or_else(|| "auth=header 需要 auth_value".to_string())?;
            let value = render_template(raw, ctx)?;
            let header_name = HeaderName::from_bytes(name.as_bytes())
                .map_err(|e| format!("无效 auth_header: {e}"))?;
            let header_value = HeaderValue::from_str(&value)
                .map_err(|e| format!("无效 auth_value: {e}"))?;
            headers.insert(header_name, header_value);
            Ok(())
        }
        "bearer" => {
            let token = profile
                .token
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .ok_or_else(|| "auth=bearer 需要 token".to_string())?;
            let token = render_template(token, ctx)?;
            let value = format!("Bearer {token}");
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&value).map_err(|e| format!("无效 bearer token: {e}"))?,
            );
            Ok(())
        }
        "basic" => {
            let user = profile
                .username
                .as_deref()
                .ok_or_else(|| "auth=basic 需要 username".to_string())?;
            let pass = profile
                .password
                .as_deref()
                .ok_or_else(|| "auth=basic 需要 password".to_string())?;
            let user = render_template(user, ctx)?;
            let pass = render_template(pass, ctx)?;
            let encoded =
                base64::engine::general_purpose::STANDARD.encode(format!("{user}:{pass}"));
            let value = format!("Basic {encoded}");
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&value).map_err(|e| format!("无效 basic 凭证: {e}"))?,
            );
            Ok(())
        }
        "query" => Ok(()),
        other => Err(format!("未知 auth `{other}`")),
    }
}

fn apply_query_auth(
    profile: &EngineProfile,
    ctx: &TemplateContext,
    query: &mut Vec<(String, String)>,
) -> Result<(), String> {
    if !profile.auth.trim().eq_ignore_ascii_case("query") {
        return Ok(());
    }
    let key = profile
        .auth_query_key
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "auth=query 需要 auth_query_key".to_string())?;
    let raw = profile
        .auth_query_value
        .as_deref()
        .ok_or_else(|| "auth=query 需要 auth_query_value".to_string())?;
    let value = render_template(raw, ctx)?;
    if !value.is_empty() {
        query.push((key.to_string(), value));
    }
    Ok(())
}

/// Resolve a dotted JSON path; numeric segments index arrays.
pub fn json_path_string(root: &Value, path: &str) -> Result<Option<String>, String> {
    let path = path.trim();
    if path.is_empty() {
        return Err("text_path 为空".into());
    }
    let mut cur = root;
    for seg in path.split('.') {
        let seg = seg.trim();
        if seg.is_empty() {
            return Err(format!("无效 JSON 路径 `{path}`"));
        }
        if let Ok(idx) = seg.parse::<usize>() {
            cur = cur
                .as_array()
                .and_then(|arr| arr.get(idx))
                .ok_or_else(|| format!("JSON 路径 `{path}` 在数组索引 {idx} 处无效"))?;
        } else {
            cur = cur
                .as_object()
                .and_then(|obj| obj.get(seg))
                .ok_or_else(|| format!("JSON 路径 `{path}` 缺少字段 `{seg}`"))?;
        }
    }
    match cur {
        Value::String(s) => Ok(Some(s.clone())),
        Value::Number(n) => Ok(Some(n.to_string())),
        Value::Bool(b) => Ok(Some(b.to_string())),
        Value::Null => Ok(None),
        other => Err(format!(
            "JSON 路径 `{path}` 不是字符串（得到 {other}）"
        )),
    }
}

fn truncate(s: &str, max: usize) -> String {
    let mut t: String = s.chars().take(max).collect();
    if s.chars().count() > max {
        t.push('…');
    }
    t
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::EngineProfile;

    fn ctx(profile: &EngineProfile, text: &str, source: &str, target: &str) -> TemplateContext {
        TemplateContext::from_request(
            profile,
            &TranslationRequest {
                text: text.into(),
                source_lang: source.into(),
                target_lang: target.into(),
            },
        )
    }

    #[test]
    fn renders_placeholders_and_lang_map() {
        let mut profile = EngineProfile::default();
        profile.lang_map.insert("zh-CN".into(), "ZH".into());
        profile.lang_map.insert("auto".into(), String::new());
        profile.extra.insert("api_key".into(), "k".into());
        let c = ctx(&profile, "Hello", "auto", "zh-CN");
        assert_eq!(c.source_lang, "");
        assert_eq!(c.target_lang, "ZH");
        assert_eq!(
            render_template("DeepL-Auth-Key {{extra.api_key}}", &c).unwrap(),
            "DeepL-Auth-Key k"
        );
        assert_eq!(render_template("{{text}}", &c).unwrap(), "Hello");
    }

    #[test]
    fn omits_empty_body_fields() {
        let mut profile = EngineProfile::default();
        profile.lang_map.insert("auto".into(), String::new());
        profile.body.insert("text".into(), "{{text}}".into());
        profile
            .body
            .insert("source_lang".into(), "{{source_lang}}".into());
        profile
            .body
            .insert("target_lang".into(), "{{target_lang}}".into());
        let c = ctx(&profile, "Hi", "auto", "en");
        let map = render_body_map(&profile.body, &c).unwrap();
        assert_eq!(map.get("text").map(String::as_str), Some("Hi"));
        assert_eq!(map.get("target_lang").map(String::as_str), Some("en"));
        assert!(!map.contains_key("source_lang"));
    }

    #[test]
    fn extracts_deepl_style_path() {
        let json: Value = serde_json::from_str(
            r#"{"translations":[{"detected_source_language":"EN","text":"你好"}]}"#,
        )
        .unwrap();
        assert_eq!(
            json_path_string(&json, "translations.0.text")
                .unwrap()
                .as_deref(),
            Some("你好")
        );
    }

    #[test]
    fn rejects_unknown_placeholder() {
        let profile = EngineProfile::default();
        let c = ctx(&profile, "x", "en", "zh-CN");
        assert!(render_template("{{foo}}", &c).is_err());
    }
}
