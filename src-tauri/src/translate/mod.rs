//! Translator trait and engine implementations.

mod baidu;
mod cloudflare;
mod config_driven;
mod credentials;
mod google;
mod microsoft;
mod microsoft_web;
mod pipeline;
mod registry;
mod state;
mod youdao;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::config::AppConfig;
use crate::lang::normalize_lang_code;

pub use pipeline::{run_parallel, BuiltinEngineRunner};
pub use registry::{list_engine_catalog, resolve_translator, EngineCatalogItem};
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
pub struct EngineTranslationResult {
    pub engine: String,
    pub status: String,
    pub text: Option<String>,
    pub error: Option<String>,
    pub cached: bool,
    pub detected_source_lang: Option<String>,
}

impl EngineTranslationResult {
    pub fn loading(engine: impl Into<String>) -> Self {
        Self {
            engine: engine.into(),
            status: "loading".into(),
            text: None,
            error: None,
            cached: false,
            detected_source_lang: None,
        }
    }

    pub fn ok(result: TranslationResult, cached: bool) -> Self {
        Self {
            engine: result.engine,
            status: "ok".into(),
            text: Some(result.text),
            error: None,
            cached,
            detected_source_lang: result.detected_source_lang,
        }
    }

    pub fn fail(engine: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            engine: engine.into(),
            status: "error".into(),
            text: None,
            error: Some(error.into()),
            cached: false,
            detected_source_lang: None,
        }
    }
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
    pub dictionary_text: Option<String>,
    pub dictionary_source: Option<String>,
    pub results: Vec<EngineTranslationResult>,
}

impl TranslationPayload {
    pub fn loading(
        source_text: &str,
        source_lang: &str,
        target_lang: &str,
        engines: &[String],
    ) -> Self {
        let results = engines
            .iter()
            .map(|id| EngineTranslationResult::loading(id.clone()))
            .collect();
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
            dictionary_text: None,
            dictionary_source: None,
            results,
        }
    }

    pub fn from_engine_results(
        source_text: &str,
        source_lang: &str,
        target_lang: &str,
        results: Vec<EngineTranslationResult>,
    ) -> Self {
        let first_ok = results.iter().find(|r| r.status == "ok");
        let still_loading = results.iter().any(|r| r.status == "loading");
        let all_failed = !results.is_empty() && results.iter().all(|r| r.status == "error");
        let status = if still_loading {
            "loading"
        } else if first_ok.is_some() {
            "ok"
        } else if all_failed {
            "error"
        } else {
            "error"
        };

        let error = if status == "error" {
            results
                .iter()
                .find_map(|r| r.error.clone())
                .or_else(|| Some("全部翻译引擎均失败".into()))
        } else {
            None
        };

        Self {
            status: status.into(),
            source_text: source_text.into(),
            translated_text: first_ok.and_then(|r| r.text.clone()),
            engine: first_ok.map(|r| r.engine.clone()),
            source_lang: source_lang.into(),
            target_lang: target_lang.into(),
            detected_source_lang: first_ok.and_then(|r| r.detected_source_lang.clone()),
            error,
            cached: first_ok.map(|r| r.cached).unwrap_or(false),
            dictionary_text: None,
            dictionary_source: None,
            results,
        }
    }

    pub fn with_dictionary(mut self, entry: Option<crate::dictionary::DictEntry>) -> Self {
        if let Some(entry) = entry {
            self.dictionary_text = Some(entry.text);
            self.dictionary_source = Some(entry.source);
        }
        self
    }
}

#[async_trait]
pub trait Translator: Send + Sync {
    fn id(&self) -> &str;
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
    let engine_id = config.engine.active.trim();
    translate_with_engine(config, engine_id, req).await
}

pub async fn translate_with_engine(
    config: &AppConfig,
    engine_id: &str,
    req: TranslationRequest,
) -> Result<TranslationResult, String> {
    let client = build_http_client(config.general.follow_system_proxy)?;
    let translator = resolve_translator(config, engine_id, client)?;
    translator.translate(&req).await
}

pub fn request_from_config(
    config: &AppConfig,
    text: String,
    target_lang: Option<String>,
) -> TranslationRequest {
    let source_lang = normalize_lang_code(&config.general.source_lang);
    let target = target_lang
        .as_deref()
        .map(normalize_lang_code)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| normalize_lang_code(&config.general.target_lang));
    TranslationRequest {
        text,
        source_lang: if source_lang.is_empty() {
            "auto".into()
        } else {
            source_lang
        },
        target_lang: if target.is_empty() {
            "zh-CN".into()
        } else {
            target
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_loading_status() {
        let engines = vec!["microsoft".into(), "google_web".into()];
        let p = TranslationPayload::loading("hi", "auto", "zh-CN", &engines);
        assert_eq!(p.status, "loading");
        assert!(p.translated_text.is_none());
        assert_eq!(p.results.len(), 2);
        assert!(p.results.iter().all(|r| r.status == "loading"));
    }

    #[test]
    fn payload_picks_first_ok() {
        let results = vec![
            EngineTranslationResult::fail("microsoft", "no key"),
            EngineTranslationResult::ok(
                TranslationResult {
                    engine: "google_web".into(),
                    text: "你好".into(),
                    source_lang: "auto".into(),
                    target_lang: "zh-CN".into(),
                    detected_source_lang: Some("en".into()),
                },
                false,
            ),
        ];
        let p = TranslationPayload::from_engine_results("hi", "auto", "zh-CN", results);
        assert_eq!(p.status, "ok");
        assert_eq!(p.translated_text.as_deref(), Some("你好"));
        assert_eq!(p.engine.as_deref(), Some("google_web"));
    }

    #[test]
    fn payload_keeps_loading_while_partial_ok() {
        let results = vec![
            EngineTranslationResult::ok(
                TranslationResult {
                    engine: "google_web".into(),
                    text: "你好".into(),
                    source_lang: "auto".into(),
                    target_lang: "zh-CN".into(),
                    detected_source_lang: Some("en".into()),
                },
                false,
            ),
            EngineTranslationResult::loading("cloudflare"),
        ];
        let p = TranslationPayload::from_engine_results("hi", "auto", "zh-CN", results);
        assert_eq!(p.status, "loading");
        assert_eq!(p.translated_text.as_deref(), Some("你好"));
        assert_eq!(p.engine.as_deref(), Some("google_web"));
    }
}
