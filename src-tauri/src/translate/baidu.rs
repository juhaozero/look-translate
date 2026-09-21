//! Baidu Translate API (通用翻译 / VIP).

use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use md5::{Digest, Md5};
use reqwest::Client;
use serde::Deserialize;

use crate::config::AppConfig;
use crate::lang::{from_baidu_lang, normalize_lang_code, to_baidu_lang};

use super::{credentials::BuiltinCredentials, TranslationRequest, TranslationResult, Translator};

const ENDPOINT: &str = "https://fanyi-api.baidu.com/api/trans/vip/translate";

pub struct BaiduTranslator {
    client: Client,
    app_id: String,
    secret: String,
}

impl BaiduTranslator {
    pub fn from_credentials(creds: &BuiltinCredentials, client: Client) -> Result<Self, String> {
        let app_id = creds
            .baidu_app_id
            .clone()
            .ok_or_else(|| "未配置百度 App ID，请在设置中填写后重试".to_string())?;
        let secret = creds
            .baidu_secret
            .clone()
            .ok_or_else(|| "未配置百度密钥，请在设置中填写后重试".to_string())?;
        Ok(Self {
            client,
            app_id,
            secret,
        })
    }

    pub fn from_config(config: &AppConfig, client: Client) -> Result<Self, String> {
        Self::from_credentials(&BuiltinCredentials::from_engine_config(&config.engine), client)
    }
}

#[async_trait]
impl Translator for BaiduTranslator {
    fn id(&self) -> &'static str {
        "baidu"
    }

    async fn translate(&self, req: &TranslationRequest) -> Result<TranslationResult, String> {
        if req.text.trim().is_empty() {
            return Err("翻译文本为空".into());
        }
        if req.target_lang.trim().is_empty() {
            return Err("目标语言不能为空".into());
        }

        let q = req.text.clone();
        let from = to_baidu_lang(&req.source_lang);
        let to = to_baidu_lang(&req.target_lang);
        let salt = unix_millis_salt();
        let sign = baidu_sign(&self.app_id, &q, &salt, &self.secret);

        let response = self
            .client
            .post(ENDPOINT)
            .form(&[
                ("q", q.as_str()),
                ("from", from.as_str()),
                ("to", to.as_str()),
                ("appid", self.app_id.as_str()),
                ("salt", salt.as_str()),
                ("sign", sign.as_str()),
            ])
            .send()
            .await
            .map_err(|e| format!("百度翻译请求失败: {e}"))?;

        let status = response.status();
        let raw = response
            .text()
            .await
            .map_err(|e| format!("读取百度翻译响应失败: {e}"))?;

        if !status.is_success() {
            return Err(format!(
                "百度翻译错误 (HTTP {}): {}",
                status.as_u16(),
                truncate(&raw, 300)
            ));
        }

        parse_baidu_response(&raw, self.id(), &req.source_lang, &req.target_lang)
    }
}

/// MD5(appid + q + salt + secret), lowercase hex.
pub fn baidu_sign(app_id: &str, q: &str, salt: &str, secret: &str) -> String {
    let mut hasher = Md5::new();
    hasher.update(app_id.as_bytes());
    hasher.update(q.as_bytes());
    hasher.update(salt.as_bytes());
    hasher.update(secret.as_bytes());
    hex::encode(hasher.finalize())
}

fn unix_millis_salt() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis().to_string())
        .unwrap_or_else(|_| "0".into())
}

fn parse_baidu_response(
    raw: &str,
    engine: &str,
    source_lang: &str,
    target_lang: &str,
) -> Result<TranslationResult, String> {
    let parsed: BaiduResponse = serde_json::from_str(raw)
        .map_err(|e| format!("解析百度翻译响应失败: {e}; body={}", truncate(raw, 200)))?;

    if let Some(code) = parsed.error_code.as_deref() {
        if code != "0" {
            return Err(format_baidu_error(code, parsed.error_msg.as_deref()));
        }
    }

    let parts = parsed
        .trans_result
        .ok_or_else(|| "百度翻译返回空结果".to_string())?;
    if parts.is_empty() {
        return Err("百度翻译返回空结果".into());
    }

    let text = parts
        .into_iter()
        .map(|p| p.dst)
        .collect::<Vec<_>>()
        .join("\n");
    if text.trim().is_empty() {
        return Err("百度翻译返回空译文".into());
    }

    Ok(TranslationResult {
        engine: engine.into(),
        text,
        source_lang: source_lang.into(),
        target_lang: normalize_lang_code(target_lang),
        detected_source_lang: parsed.from.map(|s| from_baidu_lang(&s)),
    })
}

fn format_baidu_error(code: &str, msg: Option<&str>) -> String {
    let hint = match code {
        "52001" => "请求超时，请重试",
        "52002" => "系统错误，请稍后重试",
        "52003" => "未授权用户，请检查 App ID",
        "54000" => "必填参数为空",
        "54001" => "签名错误，请检查 App ID 与密钥",
        "54003" => "访问频率受限",
        "54004" => "账户余额不足或免费额度已用尽",
        "54005" => "长查询请求过于频繁",
        "58000" => "客户端 IP 非法",
        "58001" => "译文语言方向不支持",
        "58002" => "服务当前已关闭",
        "90107" => "认证未通过或未生效",
        _ => "翻译失败",
    };
    match msg {
        Some(m) if !m.is_empty() => format!("百度翻译错误 ({code}): {hint}（{m}）"),
        _ => format!("百度翻译错误 ({code}): {hint}"),
    }
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
struct BaiduResponse {
    from: Option<String>,
    #[serde(default)]
    trans_result: Option<Vec<BaiduTransItem>>,
    error_code: Option<String>,
    error_msg: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BaiduTransItem {
    dst: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_matches_baidu_doc_example() {
        // Official doc sample: appid=2015063000000001, q=apple, salt=1435660288, secret=12345678
        let sign = baidu_sign("2015063000000001", "apple", "1435660288", "12345678");
        assert_eq!(sign, "f89f9594663708c1605f3d736d01d2d4");
    }

    #[test]
    fn parses_success_and_maps_lang() {
        let raw = r#"{"from":"en","to":"zh","trans_result":[{"src":"hello","dst":"你好"}]}"#;
        let result = parse_baidu_response(raw, "baidu", "auto", "zh-CN").unwrap();
        assert_eq!(result.text, "你好");
        assert_eq!(result.detected_source_lang.as_deref(), Some("en"));
    }

    #[test]
    fn maps_common_error_codes() {
        let raw = r#"{"error_code":"54001","error_msg":"Invalid Sign"}"#;
        let err = parse_baidu_response(raw, "baidu", "auto", "zh-CN").unwrap_err();
        assert!(err.contains("54001"));
        assert!(err.contains("签名错误"));
    }
}
