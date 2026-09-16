//! Translator trait and engine implementations.

mod microsoft;
mod state;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::config::AppConfig;

pub use microsoft::MicrosoftTranslator;
pub use state::TranslationState;

#[derive(Debug, Clone)]
pub struct TranslationRequest {
    pub text: String,
    pub source_lang: String,
    pub target_lang: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslationResult {
    pub engine: String,
    pub text: String,
    pub source_lang: String,
    pub target_lang: String,
    pub detected_source_lang: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslationPayload {
    pub status: String,
    pub source_text: String,
    pub translated_text: Option<String>,
    pub engine: Option<String>,
    pub source_lang: String,
    pub target_lang: String,
    pub detected_source_lang: Option<String>,
    pub error: Option<String>,
    pub cached: bool,
}

impl TranslationPayload {
    pub fn loading(source_text: &str, source_lang: &str, target_lang: &str) -> Self {
        Self {
            status: "loading".into(),
            source_text: source_text.into(),
            translated_text: None,
            engine: None,
            source_lang: source_lang.into(),
            target_lang: target_lang.into(),
            detected_source_lang: None,
            error: None,
            cached: false,
        }
    }

    pub fn ok(source_text: &str, result: TranslationResult, cached: bool) -> Self {
        Self {
            status: "ok".into(),
            source_text: source_text.into(),
            translated_text: Some(result.text),
            engine: Some(result.engine),
            source_lang: result.source_lang,
            target_lang: result.target_lang,
            detected_source_lang: result.detected_source_lang,
            error: None,
            cached,
        }
    }

    pub fn fail(
        source_text: &str,
        source_lang: &str,
        target_lang: &str,
        error: impl Into<String>,
    ) -> Self {
        Self {
            status: "error".into(),
            source_text: source_text.into(),
            translated_text: None,
            engine: None,
            source_lang: source_lang.into(),
            target_lang: target_lang.into(),
            detected_source_lang: None,
            error: Some(error.into()),
            cached: false,
        }
    }
}

#[async_trait]
pub trait Translator: Send + Sync {
    fn id(&self) -> &'static str;
    async fn translate(&self, req: &TranslationRequest) -> Result<TranslationResult, String>;
}

pub fn build_http_client(follow_system_proxy: bool) -> Result<reqwest::Client, String> {
    let mut builder = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(25))
        .user_agent(concat!("look-translate/", env!("CARGO_PKG_VERSION")));

    if !follow_system_proxy {
        builder = builder.no_proxy();
    }

    builder.build().map_err(|e| format!("创建 HTTP 客户端失败: {e}"))
}

pub async fn translate_with_config(
    config: &AppConfig,
    req: TranslationRequest,
) -> Result<TranslationResult, String> {
    let client = build_http_client(config.general.follow_system_proxy)?;
    match config.engine.active.as_str() {
        "microsoft" => {
            let translator = MicrosoftTranslator::from_config(config, client)?;
            translator.translate(&req).await
        }
        other => Err(format!(
            "未知翻译引擎 `{other}`（当前支持：microsoft）"
        )),
    }
}

pub fn request_from_config(config: &AppConfig, text: String, target_lang: Option<String>) -> TranslationRequest {
    TranslationRequest {
        text,
        source_lang: config.general.source_lang.clone(),
        target_lang: target_lang.unwrap_or_else(|| config.general.target_lang.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_loading_status() {
        let p = TranslationPayload::loading("hi", "auto", "zh-Hans");
        assert_eq!(p.status, "loading");
        assert!(p.translated_text.is_none());
    }
}
