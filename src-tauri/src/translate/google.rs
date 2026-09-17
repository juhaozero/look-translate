//! Google translation engines: Cloud Translation API v2 + unofficial gtx web.

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::config::AppConfig;
use crate::lang::normalize_lang_code;

use super::{TranslationRequest, TranslationResult, Translator};

const CLOUD_V2_ENDPOINT: &str = "https://translation.googleapis.com/language/translate/v2";
const GTX_ENDPOINT: &str = "https://translate.googleapis.com/translate_a/single";

pub struct GoogleCloudTranslator {
    client: Client,
    api_key: String,
}

impl GoogleCloudTranslator {
    pub fn from_config(config: &AppConfig, client: Client) -> Result<Self, String> {
        let api_key = config
            .engine
            .google_api_key
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| "未配置 Google API Key，请在设置中填写后重试".to_string())?;

        Ok(Self { client, api_key })
    }
}

#[async_trait]
impl Translator for GoogleCloudTranslator {
    fn id(&self) -> &'static str {
        "google"
    }

    async fn translate(&self, req: &TranslationRequest) -> Result<TranslationResult, String> {
        validate_request(req)?;

        let target = normalize_lang_code(&req.target_lang);
        let source = normalize_lang_code(&req.source_lang);

        let mut body = CloudV2Request {
            q: req.text.clone(),
            target,
            format: "text",
            source: None,
        };
        if !source.is_empty() && !source.eq_ignore_ascii_case("auto") {
            body.source = Some(source);
        }

        let mut url = reqwest::Url::parse(CLOUD_V2_ENDPOINT).map_err(|e| e.to_string())?;
        url.query_pairs_mut().append_pair("key", &self.api_key);

        let response = self
            .client
            .post(url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Google 翻译请求失败: {e}"))?;

        let status = response.status();
        let raw = response
            .text()
            .await
            .map_err(|e| format!("读取 Google 响应失败: {e}"))?;

        if !status.is_success() {
            return Err(format_cloud_error(status.as_u16(), &raw));
        }

        let parsed: CloudV2Response = serde_json::from_str(&raw)
            .map_err(|e| format!("解析 Google 响应失败: {e}; body={raw}"))?;

        let item = parsed
            .data
            .translations
            .into_iter()
            .next()
            .ok_or_else(|| "Google 返回空结果".to_string())?;

        Ok(TranslationResult {
            engine: self.id().into(),
            text: decode_html_entities(&item.translated_text),
            source_lang: req.source_lang.clone(),
            target_lang: normalize_lang_code(&req.target_lang),
            detected_source_lang: item
                .detected_source_language
                .map(|s| normalize_lang_code(&s)),
        })
    }
}

pub struct GoogleWebTranslator {
    client: Client,
}

impl GoogleWebTranslator {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl Translator for GoogleWebTranslator {
    fn id(&self) -> &'static str {
        "google_web"
    }

    async fn translate(&self, req: &TranslationRequest) -> Result<TranslationResult, String> {
        validate_request(req)?;

        let target = normalize_lang_code(&req.target_lang);
        let mut source = normalize_lang_code(&req.source_lang);
        if source.is_empty() || source.eq_ignore_ascii_case("auto") {
            source = "auto".into();
        }

        let mut url = reqwest::Url::parse(GTX_ENDPOINT).map_err(|e| e.to_string())?;
        {
            let mut pairs = url.query_pairs_mut();
            pairs.append_pair("client", "gtx");
            pairs.append_pair("sl", &source);
            pairs.append_pair("tl", &target);
            pairs.append_pair("dt", "t");
            pairs.append_pair("q", req.text.trim());
        }

        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("Google 网页翻译请求失败: {e}"))?;

        let status = response.status();
        let raw = response
            .text()
            .await
            .map_err(|e| format!("读取 Google 网页翻译响应失败: {e}"))?;

        if !status.is_success() {
            return Err(format!(
                "Google 网页翻译错误 ({status})：非官方接口可能限流或失效。详情：{}",
                truncate(&raw, 200)
            ));
        }

        let (text, detected) = parse_gtx_response(&raw)?;
        if text.trim().is_empty() {
            return Err("Google 网页翻译返回空译文".into());
        }

        Ok(TranslationResult {
            engine: self.id().into(),
            text,
            source_lang: req.source_lang.clone(),
            target_lang: target,
            detected_source_lang: detected.map(|s| normalize_lang_code(&s)),
        })
    }
}

fn validate_request(req: &TranslationRequest) -> Result<(), String> {
    if req.text.trim().is_empty() {
        return Err("翻译文本为空".into());
    }
    if req.target_lang.trim().is_empty() {
        return Err("目标语言不能为空".into());
    }
    Ok(())
}

fn parse_gtx_response(raw: &str) -> Result<(String, Option<String>), String> {
    let value: Value = serde_json::from_str(raw)
        .map_err(|e| format!("解析 Google 网页翻译响应失败: {e}; body={}", truncate(raw, 200)))?;

    let segments = value
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Google 网页翻译响应格式异常".to_string())?;

    let mut translated = String::new();
    for segment in segments {
        if let Some(piece) = segment.as_array().and_then(|s| s.first()).and_then(|v| v.as_str())
        {
            translated.push_str(piece);
        }
    }

    let detected = value
        .as_array()
        .and_then(|arr| arr.get(2))
        .and_then(|v| v.as_str())
        .map(str::to_string);

    Ok((translated, detected))
}

fn format_cloud_error(status: u16, body: &str) -> String {
    if let Ok(err) = serde_json::from_str::<CloudErrorBody>(body) {
        if let Some(error) = err.error {
            let message = error.message.unwrap_or_else(|| body.to_string());
            let status_text = error.status.unwrap_or_default();
            return format!("Google 翻译错误 ({status}): {status_text} {message}");
        }
    }
    format!("Google 翻译错误 ({status}): {}", truncate(body, 300))
}

fn decode_html_entities(input: &str) -> String {
    input
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
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

#[derive(Serialize)]
struct CloudV2Request {
    q: String,
    target: String,
    format: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CloudV2Response {
    data: CloudV2Data,
}

#[derive(Debug, Deserialize)]
struct CloudV2Data {
    translations: Vec<CloudV2Translation>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CloudV2Translation {
    translated_text: String,
    detected_source_language: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CloudErrorBody {
    error: Option<CloudErrorDetail>,
}

#[derive(Debug, Deserialize)]
struct CloudErrorDetail {
    message: Option<String>,
    status: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_gtx_segments() {
        let raw = r#"[[["你好","hello",null,null,10]],null,"en",null,null,null,null,[]]"#;
        let (text, detected) = parse_gtx_response(raw).unwrap();
        assert_eq!(text, "你好");
        assert_eq!(detected.as_deref(), Some("en"));
    }

    #[test]
    fn formats_cloud_error() {
        let body = r#"{"error":{"message":"API key not valid.","status":"INVALID_ARGUMENT"}}"#;
        let msg = format_cloud_error(400, body);
        assert!(msg.contains("INVALID_ARGUMENT"));
        assert!(msg.contains("not valid"));
    }
}
