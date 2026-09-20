//! Builtin engine catalog + translator resolution.

use reqwest::Client;
use serde::Serialize;

use crate::config::{is_builtin_engine, AppConfig, EngineProfile};

use super::cloudflare::CloudflareTranslator;
use super::config_driven::ConfigDrivenTranslator;
use super::credentials::BuiltinCredentials;
use super::google::{GoogleCloudTranslator, GoogleWebTranslator};
use super::microsoft::MicrosoftTranslator;
use super::microsoft_web::MicrosoftWebTranslator;
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
    /// `builtin` | `profile`
    pub kind: String,
}

/// Builtin engines first (fixed order), then Config-driven profiles (sorted).
/// Same-named profiles are skipped (Builtin wins).
pub fn list_engine_catalog(config: &AppConfig) -> Vec<EngineCatalogItem> {
    let mut list: Vec<EngineCatalogItem> = BUILTIN_ENGINES
        .iter()
        .map(|meta| EngineCatalogItem {
            id: meta.id.into(),
            label: meta.label.into(),
            subtitle: meta.subtitle.into(),
            hint: meta.hint.into(),
            configurable: meta.configurable,
            kind: "builtin".into(),
        })
        .collect();

    let mut profile_ids: Vec<&String> = config.engines.keys().collect();
    profile_ids.sort();
    for id in profile_ids {
        if is_builtin_engine(id) {
            continue;
        }
        let profile: &EngineProfile = &config.engines[id];
        list.push(EngineCatalogItem {
            id: id.clone(),
            label: profile.display_label(id).to_string(),
            subtitle: "自定义（config.toml）".into(),
            hint: format!(
                "请在 data/config.toml 的 [engines.{id}] 中编辑；此处仅可切换启用。"
            ),
            configurable: true,
            kind: "profile".into(),
        });
    }
    list
}

/// Build a Translator adapter for `engine_id` (Builtin or Config-driven Profile).
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
        other if !other.is_empty() && !is_builtin_engine(other) => Ok(Box::new(
            ConfigDrivenTranslator::from_profile(config, other, client)?,
        )),
        other => Err(format!(
            "未知翻译引擎 `{other}`（内置：microsoft / microsoft_web / google / google_web / cloudflare；或配置 `[engines.<id>]`）"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::EngineProfile;

    #[test]
    fn catalog_builtins_then_profiles_skips_shadowed() {
        let mut config = AppConfig::default();
        config.engines.insert(
            "deepl".into(),
            EngineProfile {
                label: Some("DeepL".into()),
                ..EngineProfile::default()
            },
        );
        config.engines.insert(
            "microsoft".into(),
            EngineProfile {
                label: Some("shadow".into()),
                ..EngineProfile::default()
            },
        );

        let catalog = list_engine_catalog(&config);
        assert_eq!(catalog[0].id, "microsoft");
        assert_eq!(catalog[0].kind, "builtin");
        assert!(catalog.iter().any(|i| i.id == "deepl" && i.kind == "profile"));
        assert_eq!(
            catalog.iter().filter(|i| i.id == "microsoft").count(),
            1
        );
    }

    #[test]
    fn builtin_ids_match_config_table() {
        use crate::config::BUILTIN_ENGINE_IDS;
        let meta_ids: Vec<&str> = BUILTIN_ENGINES.iter().map(|m| m.id).collect();
        assert_eq!(meta_ids, BUILTIN_ENGINE_IDS);
    }
}
