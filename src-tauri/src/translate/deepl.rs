//! Official DeepL Text Translation API (Free and Pro).

use async_trait::async_trait;
use reqwest::{Client, Request};
use serde::Deserialize;
use serde_json::{json, Value};

use super::{credentials::BuiltinCredentials, TranslationRequest, TranslationResult, Translator};
use crate::lang::normalize_lang_code;

pub struct DeepLTranslator {
    client: Client,
    api_key: String,
}

impl DeepLTranslator {
    pub fn from_credentials(creds: &BuiltinCredentials, client: Client) -> Result<Self, String> {
        let api_key = creds
            .deepl_api_key
            .as_deref()
            .map(str::trim)
            .filter(|key| !key.is_empty())
            .ok_or_else(|| "未配置 DeepL API Key，请在服务设置中填写后重试".to_string())?;
        Ok(Self {
            client,
            api_key: api_key.into(),
        })
    }

    fn request(&self, req: &TranslationRequest) -> Result<Request, String> {
        if req.text.trim().is_empty() {
            return Err("请输入要翻译的文本".into());
        }
        let target = normalize_lang_code(&req.target_lang);
        if target.is_empty() || target == "auto" {
            return Err("请选择 DeepL 目标语言".into());
        }
        let target = match target.as_str() {
            "zh-CN" => "ZH-HANS".into(),
            "zh-TW" => "ZH-HANT".into(),
            _ => target.to_ascii_uppercase(),
        };
        let mut body = json!({ "text": [&req.text], "target_lang": target });
        let source = normalize_lang_code(&req.source_lang);
        if !source.is_empty() && source != "auto" {
            let source = match source.as_str() {
                "zh-CN" | "zh-TW" => "ZH".into(),
                "en-US" | "en-GB" => "EN".into(),
                "pt-BR" | "pt-PT" => "PT".into(),
                _ => source.to_ascii_uppercase(),
            };
            body["source_lang"] = Value::String(source);
        }
        let endpoint = if self.api_key.ends_with(":fx") {
            "https://api-free.deepl.com/v2/translate"
        } else {
            "https://api.deepl.com/v2/translate"
        };
        self.client
            .post(endpoint)
            .header("Authorization", format!("DeepL-Auth-Key {}", self.api_key))
            .json(&body)
            .build()
            .map_err(|_| "构建 DeepL 请求失败，请检查 API Key 格式".into())
    }
}

#[async_trait]
impl Translator for DeepLTranslator {
    fn id(&self) -> &'static str {
        "deepl"
    }

    async fn translate(&self, req: &TranslationRequest) -> Result<TranslationResult, String> {
        let response = self
            .client
            .execute(self.request(req)?)
            .await
            .map_err(|e| format!("DeepL 翻译请求失败: {e}"))?;
        let status = response.status();
        let raw = response
            .text()
            .await
            .map_err(|e| format!("读取 DeepL 响应失败: {e}"))?;
        if !status.is_success() {
            return Err(format_error(status.as_u16(), &raw));
        }
        parse_translation(&raw, req)
    }
}

#[derive(Deserialize)]
struct DeepLResponse {
    translations: Vec<DeepLTranslation>,
}

#[derive(Deserialize)]
struct DeepLTranslation {
    text: String,
    detected_source_language: Option<String>,
}

fn parse_translation(raw: &str, req: &TranslationRequest) -> Result<TranslationResult, String> {
    let parsed: DeepLResponse =
        serde_json::from_str(raw).map_err(|_| "DeepL 返回了无效的翻译响应".to_string())?;
    let item = parsed
        .translations
        .into_iter()
        .next()
        .filter(|item| !item.text.trim().is_empty())
        .ok_or_else(|| "DeepL 返回空翻译结果".to_string())?;
    Ok(TranslationResult {
        engine: "deepl".into(),
        text: item.text,
        source_lang: req.source_lang.clone(),
        target_lang: normalize_lang_code(&req.target_lang),
        detected_source_lang: item
            .detected_source_language
            .map(|lang| normalize_lang_code(&lang.to_ascii_lowercase())),
    })
}

fn format_error(status: u16, raw: &str) -> String {
    let hint = match status {
        403 => "API Key 无效或无权访问，请检查 DeepL API 密钥",
        429 => "请求过于频繁，请稍后重试",
        456 => "翻译额度已用尽或已达到用量上限",
        500..=599 => "DeepL 服务暂时不可用，请稍后重试",
        _ => "翻译请求失败",
    };
    let message = serde_json::from_str::<Value>(raw)
        .ok()
        .and_then(|value| value.get("message")?.as_str().map(str::to_string));
    match message {
        Some(message) => format!("DeepL HTTP {status}: {hint}（{message}）"),
        None => format!("DeepL HTTP {status}: {hint}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn translator(key: &str) -> DeepLTranslator {
        DeepLTranslator::from_credentials(
            &BuiltinCredentials {
                deepl_api_key: Some(key.into()),
                ..Default::default()
            },
            Client::new(),
        )
        .unwrap()
    }

    fn request(source: &str, target: &str) -> TranslationRequest {
        TranslationRequest {
            text: "Hello & 你好".into(),
            source_lang: source.into(),
            target_lang: target.into(),
        }
    }

    #[test]
    fn free_request_uses_header_auth_and_automatic_source() {
        let req = translator(" test-key:fx ")
            .request(&request("auto", "zh-CN"))
            .unwrap();
        assert_eq!(
            req.url().as_str(),
            "https://api-free.deepl.com/v2/translate"
        );
        assert_eq!(req.headers()["Authorization"], "DeepL-Auth-Key test-key:fx");
        let body: Value = serde_json::from_slice(req.body().unwrap().as_bytes().unwrap()).unwrap();
        assert_eq!(body["text"], json!(["Hello & 你好"]));
        assert_eq!(body["target_lang"], "ZH-HANS");
        assert!(body.get("source_lang").is_none());
    }

    #[test]
    fn pro_request_maps_chinese_and_english() {
        let adapter = translator("pro-key");
        let req = adapter.request(&request("zh-TW", "en")).unwrap();
        assert_eq!(req.url().host_str(), Some("api.deepl.com"));
        let body: Value = serde_json::from_slice(req.body().unwrap().as_bytes().unwrap()).unwrap();
        assert_eq!(body["source_lang"], "ZH");
        assert_eq!(body["target_lang"], "EN");
        let req = adapter.request(&request("en", "zh-TW")).unwrap();
        let body: Value = serde_json::from_slice(req.body().unwrap().as_bytes().unwrap()).unwrap();
        assert_eq!(body["target_lang"], "ZH-HANT");
    }

    #[test]
    fn parses_result_and_rejects_empty_or_malformed_responses() {
        let req = request("auto", "zh-CN");
        let result = parse_translation(
            r#"{"translations":[{"text":"你好","detected_source_language":"EN"}]}"#,
            &req,
        )
        .unwrap();
        assert_eq!(result.text, "你好");
        assert_eq!(result.detected_source_lang.as_deref(), Some("en"));
        assert_eq!(result.target_lang, "zh-CN");
        assert!(parse_translation(r#"{"translations":[]}"#, &req).is_err());
        assert!(parse_translation("<html>error</html>", &req).is_err());
    }

    #[test]
    fn rejects_missing_credentials_and_reports_actionable_errors() {
        assert!(
            DeepLTranslator::from_credentials(&BuiltinCredentials::default(), Client::new())
                .is_err()
        );
        assert!(format_error(403, "").contains("API Key"));
        assert!(format_error(456, "").contains("额度"));
        assert!(format_error(429, "").contains("频繁"));
        assert!(format_error(400, r#"{"message":"Unsupported language"}"#)
            .contains("Unsupported language"));
    }
}
