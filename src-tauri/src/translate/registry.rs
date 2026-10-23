//! Builtin engine catalog + translator resolution.

use reqwest::Client;
use serde::Serialize;

use crate::config::AppConfig;

use super::baidu::BaiduTranslator;
use super::cloudflare::CloudflareTranslator;
use super::credentials::BuiltinCredentials;
use super::deepl::DeepLTranslator;
use super::google::{GoogleCloudTranslator, GoogleWebTranslator};
use super::microsoft::MicrosoftTranslator;
use super::microsoft_web::MicrosoftWebTranslator;
use super::youdao::YoudaoTranslator;
use super::Translator;

/// Static metadata for one Builtin Engine (settings / catalog SSOT).
#[derive(Debug, Clone, Copy)]
pub struct BuiltinEngineMeta {
    pub id: &'static str,
    pub label: &'static str,
    pub subtitle: &'static str,
    pub hint: &'static str,
    pub configurable: bool,
}

pub const BUILTIN_ENGINES: &[BuiltinEngineMeta] = &[
    BuiltinEngineMeta {
        id: "microsoft",
        label: "Microsoft 翻译",
        subtitle: "需 Azure Key",
        hint: "需填写 Azure Translator Key；区域资源再填 Region",
        configurable: true,
    },
    BuiltinEngineMeta {
        id: "microsoft_web",
        label: "必应翻译",
        subtitle: "免费网页接口",
        hint: "非官方 Bing 网页接口，无需 Key；可能限流或失效",
        configurable: false,
    },
    BuiltinEngineMeta {
        id: "google",
        label: "Google 翻译",
        subtitle: "Cloud API，需 Key",
        hint: "需填写 Google Cloud Translation API v2 Key",
        configurable: true,
    },
    BuiltinEngineMeta {
        id: "google_web",
        label: "Google 翻译（网页）",
        subtitle: "免费接口，可能限流",
        hint: "非官方 gtx 接口，无需 Key；可能限流或失效",
        configurable: false,
    },
    BuiltinEngineMeta {
        id: "cloudflare",
        label: "Cloudflare 翻译",
        subtitle: "Workers / translate-api",
        hint: "兼容本仓库 work.js Worker：POST JSON + Authorization Bearer；密钥用 wrangler secret put SECRET_PASS，勿写进源码。源语言为 auto 时按正文脚本猜测（中/日/韩/英）；简繁均映射为 zh。",
        configurable: true,
    },
    BuiltinEngineMeta {
        id: "baidu",
        label: "百度翻译",
        subtitle: "开放平台，需 App ID",
        hint: "需填写百度翻译开放平台 App ID 与密钥（通用翻译 API）",
        configurable: true,
    },
    BuiltinEngineMeta {
        id: "youdao",
        label: "有道翻译",
        subtitle: "智云，需应用密钥",
        hint: "需填写有道智云应用 ID（appKey）与应用密钥（文本翻译 API）",
        configurable: true,
    },
    BuiltinEngineMeta {
        id: "deepl",
        label: "DeepL 翻译",
        subtitle: "官方 API，支持 Free / Pro",
        hint: "填写 DeepL API Key；根据密钥自动选择 Free 或 Pro 接口",
        configurable: true,
    },
];

pub fn builtin_meta(id: &str) -> Option<&'static BuiltinEngineMeta> {
    let trimmed = id.trim();
    BUILTIN_ENGINES
        .iter()
        .find(|meta| meta.id.eq_ignore_ascii_case(trimmed))
}

/// One row in the settings / popup engine catalog.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineCatalogItem {
    pub id: String,
    pub label: String,
    pub subtitle: String,
    pub hint: String,
    pub configurable: bool,
    /// `builtin`
    pub kind: String,
}

/// List supported translation services.
pub fn list_engine_catalog(_config: &AppConfig) -> Vec<EngineCatalogItem> {
    BUILTIN_ENGINES
        .iter()
        .map(|meta| EngineCatalogItem {
            id: meta.id.into(),
            label: meta.label.into(),
            subtitle: meta.subtitle.into(),
            hint: meta.hint.into(),
            configurable: meta.configurable,
            kind: "builtin".into(),
        })
        .collect()
}

/// Build a Translator adapter for `engine_id` (supported service).
pub fn resolve_translator(
    config: &AppConfig,
    engine_id: &str,
    client: Client,
) -> Result<Box<dyn Translator>, String> {
    let active = engine_id.trim();
    let creds = BuiltinCredentials::from_engine_config(&config.engine);

    match active {
        "microsoft" => Ok(Box::new(MicrosoftTranslator::from_credentials(
            &creds, client,
        )?)),
        "microsoft_web" => Ok(Box::new(MicrosoftWebTranslator::new(client))),
        "google" => Ok(Box::new(GoogleCloudTranslator::from_credentials(
            &creds, client,
        )?)),
        "google_web" => Ok(Box::new(GoogleWebTranslator::new(client))),
        "cloudflare" => Ok(Box::new(CloudflareTranslator::from_credentials(
            &creds, client,
        )?)),
        "baidu" => Ok(Box::new(BaiduTranslator::from_credentials(
            &creds, client,
        )?)),
        "youdao" => Ok(Box::new(YoudaoTranslator::from_credentials(
            &creds, client,
        )?)),
        "deepl" => Ok(Box::new(DeepLTranslator::from_credentials(&creds, client)?)),
        other => Err(format!(
            "未知翻译引擎 `{other}`（内置：microsoft / microsoft_web / google / google_web / cloudflare / baidu / youdao / deepl）"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn catalog_contains_deepl_service() {
        let catalog = list_engine_catalog(&AppConfig::default());
        assert!(catalog
            .iter()
            .any(|item| item.id == "deepl" && item.configurable));
        assert!(catalog.iter().all(|item| item.kind == "builtin"));
    }

    #[test]
    fn builtin_ids_match_config_table() {
        use crate::config::BUILTIN_ENGINE_IDS;
        let meta_ids: Vec<&str> = BUILTIN_ENGINES.iter().map(|m| m.id).collect();
        assert_eq!(meta_ids, BUILTIN_ENGINE_IDS);
    }
}
