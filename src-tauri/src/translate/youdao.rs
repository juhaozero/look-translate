//! Youdao Zhiyun text translation API (signType=v3).

use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::config::AppConfig;
use crate::lang::{from_youdao_lang, normalize_lang_code, to_youdao_lang};

use super::{credentials::BuiltinCredentials, TranslationRequest, TranslationResult, Translator};

const ENDPOINT: &str = "https://openapi.youdao.com/api";

pub struct YoudaoTranslator {
    client: Client,
    app_key: String,
    app_secret: String,
}

impl YoudaoTranslator {
    pub fn from_credentials(creds: &BuiltinCredentials, client: Client) -> Result<Self, String> {
        let app_key = creds
            .youdao_app_key
            .clone()
            .ok_or_else(|| "未配置有道应用 ID，请在设置中填写后重试".to_string())?;
        let app_secret = creds
            .youdao_app_secret
            .clone()
            .ok_or_else(|| "未配置有道应用密钥，请在设置中填写后重试".to_string())?;
        Ok(Self {
            client,
            app_key,
            app_secret,
        })
    }

    pub fn from_config(config: &AppConfig, client: Client) -> Result<Self, String> {
        Self::from_credentials(&BuiltinCredentials::from_engine_config(&config.engine), client)
    }
}

#[async_trait]
impl Translator for YoudaoTranslator {
    fn id(&self) -> &'static str {
        "youdao"
    }

    async fn translate(&self, req: &TranslationRequest) -> Result<TranslationResult, String> {
        if req.text.trim().is_empty() {
            return Err("翻译文本为空".into());
        }
        if req.target_lang.trim().is_empty() {
            return Err("目标语言不能为空".into());
        }

        let q = req.text.clone();
        let from = to_youdao_lang(&req.source_lang);
        let to = to_youdao_lang(&req.target_lang);
        let salt = unix_millis_salt();
        let curtime = unix_secs_curtime();
        let sign = youdao_sign_v3(&self.app_key, &q, &salt, &curtime, &self.app_secret);

        let response = self
            .client
            .post(ENDPOINT)
            .form(&[
                ("q", q.as_str()),
                ("from", from.as_str()),
                ("to", to.as_str()),
                ("appKey", self.app_key.as_str()),
                ("salt", salt.as_str()),
                ("sign", sign.as_str()),
                ("signType", "v3"),
                ("curtime", curtime.as_str()),
            ])
            .send()
            .await
            .map_err(|e| format!("有道翻译请求失败: {e}"))?;

        let status = response.status();
        let raw = response
            .text()
            .await
            .map_err(|e| format!("读取有道翻译响应失败: {e}"))?;

        if !status.is_success() {
            return Err(format!(
                "有道翻译错误 (HTTP {}): {}",
                status.as_u16(),
                truncate(&raw, 300)
            ));
        }

        parse_youdao_response(&raw, self.id(), &req.source_lang, &req.target_lang)
    }
}

/// SHA256(appKey + truncate(q) + salt + curtime + appSecret), lowercase hex.
pub fn youdao_sign_v3(app_key: &str, q: &str, salt: &str, curtime: &str, app_secret: &str) -> String {
    let input = youdao_truncate(q);
    let mut hasher = Sha256::new();
    hasher.update(app_key.as_bytes());
    hasher.update(input.as_bytes());
    hasher.update(salt.as_bytes());
    hasher.update(curtime.as_bytes());
    hasher.update(app_secret.as_bytes());
    hex::encode(hasher.finalize())
}

/// Youdao v3 input truncation: len<=20 keep as-is; else first10 + len + last10.
pub fn youdao_truncate(q: &str) -> String {
    let chars: Vec<char> = q.chars().collect();
    let len = chars.len();
    if len <= 20 {
        return q.to_string();
    }
    let head: String = chars[..10].iter().collect();
    let tail: String = chars[len - 10..].iter().collect();
    format!("{head}{len}{tail}")
}

fn unix_millis_salt() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis().to_string())
        .unwrap_or_else(|_| "0".into())
}

fn unix_secs_curtime() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".into())
}

fn parse_youdao_response(
    raw: &str,
    engine: &str,
    source_lang: &str,
    target_lang: &str,
) -> Result<TranslationResult, String> {
    let parsed: YoudaoResponse = serde_json::from_str(raw)
        .map_err(|e| format!("解析有道翻译响应失败: {e}; body={}", truncate(raw, 200)))?;

    let code = parsed.error_code.as_deref().unwrap_or("0");
    if code != "0" {
        return Err(format_youdao_error(code));
    }

    let text = parsed
        .translation
        .filter(|v| !v.is_empty())
        .map(|v| v.join("\n"))
        .ok_or_else(|| "有道翻译返回空结果".to_string())?;
    if text.trim().is_empty() {
        return Err("有道翻译返回空译文".into());
    }

    let detected = parsed
        .l
        .as_deref()
        .and_then(detect_from_youdao_l)
        .map(|s| from_youdao_lang(&s));

    Ok(TranslationResult {
        engine: engine.into(),
        text,
        source_lang: source_lang.into(),
        target_lang: normalize_lang_code(target_lang),
        detected_source_lang: detected,
    })
}

/// `l` looks like `en2zh-CHS` — take the source side.
fn detect_from_youdao_l(l: &str) -> Option<String> {
    l.split_once('2').map(|(from, _)| from.to_string())
}

fn format_youdao_error(code: &str) -> String {
    let hint = match code {
        "101" => "缺少必填参数",
        "102" => "不支持的语言类型",
        "103" => "翻译文本过长",
        "104" => "不支持的 API 类型",
        "105" => "不支持的签名类型",
        "106" => "不支持的响应类型",
        "107" => "不支持的传输加密类型",
        "108" => "应用 ID 无效，请检查应用 ID",
        "109" => "batchLog 格式不正确",
        "110" => "无相关服务的有效实例",
        "111" => "开发者账号无效",
        "112" => "请求频率受限",
        "113" => "查询内容不能为空",
        "201" => "解密失败",
        "202" => "签名检验失败，请检查应用 ID 与密钥",
        "203" => "访问 IP 地址不在可访问名单中",
        "301" => "辞典查询失败",
        "302" => "翻译查询失败",
        "303" => "服务端异常",
        "401" => "账户已经欠费或额度不足",
        "411" => "访问频率受限",
        "412" => "长请求过于频繁",
        _ => "翻译失败",
    };
    format!("有道翻译错误 ({code}): {hint}")
}

fn truncate(s: &str, max: usize) -> String {
    let mut iter = s.chars();
    let truncated: String = iter.by_ref().take(max).collect();
    if iter.next().is_some() {
        format!("{truncated}…")
    } else {
        truncated
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct YoudaoResponse {
    error_code: Option<String>,
    translation: Option<Vec<String>>,
    /// Language pair like `en2zh-CHS`.
    l: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_short_unchanged() {
        assert_eq!(youdao_truncate("hello"), "hello");
    }

    #[test]
    fn truncate_long_uses_len() {
        let q = "abcdefghijklmnopqrstuvwxyz"; // 26 chars
        assert_eq!(youdao_truncate(q), "abcdefghij26qrstuvwxyz");
    }

    #[test]
    fn sign_v3_matches_sha256_of_concat() {
        let sign = youdao_sign_v3("app", "hello", "1", "2", "secret");
        let mut h = Sha256::new();
        h.update(b"apphello12secret");
        assert_eq!(sign, hex::encode(h.finalize()));
    }

    #[test]
    fn parses_success() {
        let raw = r#"{"errorCode":"0","translation":["你好"],"l":"en2zh-CHS"}"#;
        let result = parse_youdao_response(raw, "youdao", "auto", "zh-CN").unwrap();
        assert_eq!(result.text, "你好");
        assert_eq!(result.detected_source_lang.as_deref(), Some("en"));
    }

    #[test]
    fn maps_common_error() {
        let raw = r#"{"errorCode":"202"}"#;
        let err = parse_youdao_response(raw, "youdao", "auto", "zh-CN").unwrap_err();
        assert!(err.contains("202"));
        assert!(err.contains("签名"));
    }
}
