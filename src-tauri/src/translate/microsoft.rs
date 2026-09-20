//! Microsoft Azure Translator Text API v3.

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::AppConfig;
use crate::lang::{from_microsoft_lang, to_microsoft_lang};

use super::{credentials::BuiltinCredentials, TranslationRequest, TranslationResult, Translator};

const ENDPOINT: &str = "https://api.cognitive.microsofttranslator.com/translate";
const API_VERSION: &str = "3.0";

pub struct MicrosoftTranslator {
    client: Client,
    api_key: String,
    region: Option<String>,
}

impl MicrosoftTranslator {
    pub fn from_credentials(creds: &BuiltinCredentials, client: Client) -> Result<Self, String> {
        let api_key = creds
            .microsoft_api_key
            .clone()
            .ok_or_else(|| "未配置 Microsoft API Key，请在设置中填写后重试".to_string())?;

        Ok(Self {
            client,
            api_key,
            region: creds.microsoft_region.clone(),
        })
    }

    pub fn from_config(config: &AppConfig, client: Client) -> Result<Self, String> {
        Self::from_credentials(&BuiltinCredentials::from_engine_config(&config.engine), client)
    }
}

#[async_trait]
impl Translator for MicrosoftTranslator {
    fn id(&self) -> &'static str {
        "microsoft"
    }

    async fn translate(&self, req: &TranslationRequest) -> Result<TranslationResult, String> {
        if req.text.trim().is_empty() {
            return Err("翻译文本为空".into());
        }
        if req.target_lang.trim().is_empty() {
            return Err("目标语言不能为空".into());
        }

        let target = to_microsoft_lang(&req.target_lang);
        let source = to_microsoft_lang(&req.source_lang);

        let mut url = reqwest::Url::parse(ENDPOINT).map_err(|e| e.to_string())?;
        {
            let mut pairs = url.query_pairs_mut();
            pairs.append_pair("api-version", API_VERSION);
            pairs.append_pair("to", &target);
            if !source.is_empty() && !source.eq_ignore_ascii_case("auto") {
                pairs.append_pair("from", &source);
            }
        }

        let body = vec![MicrosoftTextBody {
            text: req.text.clone(),
        }];

        let mut request = self
            .client
            .post(url)
            .header("Ocp-Apim-Subscription-Key", &self.api_key)
            .header("Content-Type", "application/json; charset=UTF-8")
            .json(&body);

        if let Some(region) = &self.region {
            request = request.header("Ocp-Apim-Subscription-Region", region);
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("Microsoft 翻译请求失败: {e}"))?;

        let status = response.status();
        let raw = response
            .text()
            .await
            .map_err(|e| format!("读取 Microsoft 响应失败: {e}"))?;

        if !status.is_success() {
            return Err(format_microsoft_error(status.as_u16(), &raw));
        }

        let parsed: Vec<MicrosoftTranslateItem> = serde_json::from_str(&raw)
            .map_err(|e| format!("解析 Microsoft 响应失败: {e}; body={raw}"))?;

        let item = parsed
            .into_iter()
            .next()
            .ok_or_else(|| "Microsoft 返回空结果".to_string())?;

        let translated = item
            .translations
            .into_iter()
            .next()
            .ok_or_else(|| "Microsoft 未返回译文".to_string())?;

        Ok(TranslationResult {
            engine: self.id().into(),
            text: translated.text,
            source_lang: req.source_lang.clone(),
            target_lang: translated
                .to
                .map(|code| from_microsoft_lang(&code))
                .unwrap_or_else(|| req.target_lang.clone()),
            detected_source_lang: item
                .detected_language
                .map(|d| from_microsoft_lang(&d.language)),
        })
    }
}

#[derive(Serialize)]
struct MicrosoftTextBody {
    #[serde(rename = "Text")]
    text: String,
}

#[derive(Debug, Deserialize)]
struct MicrosoftTranslateItem {
    #[serde(rename = "detectedLanguage")]
    detected_language: Option<MicrosoftDetectedLanguage>,
    translations: Vec<MicrosoftTranslation>,
}

#[derive(Debug, Deserialize)]
struct MicrosoftDetectedLanguage {
    language: String,
}

#[derive(Debug, Deserialize)]
struct MicrosoftTranslation {
    text: String,
    to: Option<String>,
}

fn format_microsoft_error(status: u16, body: &str) -> String {
    if let Ok(err) = serde_json::from_str::<MicrosoftErrorBody>(body) {
        if let Some(error) = err.error {
            return format!(
                "Microsoft 翻译错误 ({status}): {} {}",
                error.code.unwrap_or_default(),
                error.message.unwrap_or_else(|| body.to_string())
            );
        }
    }
    format!("Microsoft 翻译错误 ({status}): {body}")
}

#[derive(Debug, Deserialize)]
struct MicrosoftErrorBody {
    error: Option<MicrosoftErrorDetail>,
}

#[derive(Debug, Deserialize)]
struct MicrosoftErrorDetail {
    code: Option<u32>,
    message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_error_json() {
        let body = r#"{"error":{"code":401000,"message":"The request is not authorized."}}"#;
        let msg = format_microsoft_error(401, body);
        assert!(msg.contains("401"));
        assert!(msg.contains("not authorized"));
    }
}
