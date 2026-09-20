//! Portable config under install-dir `data/`.

mod templates;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use serde::{Deserialize, Serialize};

pub use templates::{custom_http_profile, CUSTOM_PROFILE_ID};

pub const CONFIG_FILE_NAME: &str = "config.toml";
pub const DATA_DIR_NAME: &str = "data";

/// Builtin engine ids — always win over same-named Engine Profiles.
pub const BUILTIN_ENGINE_IDS: &[&str] = &[
    "microsoft",
    "microsoft_web",
    "google",
    "google_web",
    "cloudflare",
];

pub fn is_builtin_engine(id: &str) -> bool {
    BUILTIN_ENGINE_IDS
        .iter()
        .any(|builtin| builtin.eq_ignore_ascii_case(id.trim()))
}

#[derive(Debug, Clone)]
pub struct ConfigPaths {
    pub data_dir: PathBuf,
    pub config_path: PathBuf,
}

#[derive(Debug)]
pub struct ConfigState {
    pub paths: ConfigPaths,
    pub config: RwLock<AppConfig>,
}

impl ConfigState {
    pub fn new(paths: ConfigPaths, config: AppConfig) -> Self {
        Self {
            paths,
            config: RwLock::new(config),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct AppConfig {
    pub general: GeneralConfig,
    pub engine: EngineConfig,
    /// Named Config-driven Engine profiles (`[engines.<id>]`).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub engines: BTreeMap<String, EngineProfile>,
    pub ocr: OcrConfig,
    pub dictionary: DictionaryConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            engine: EngineConfig::default(),
            engines: BTreeMap::new(),
            ocr: OcrConfig::default(),
            dictionary: DictionaryConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct GeneralConfig {
    pub target_lang: String,
    pub source_lang: String,
    pub hotkey_translate: String,
    pub hotkey_ocr: String,
    pub hotkey_enabled: bool,
    pub follow_system_proxy: bool,
    /// Launch Look Translate when the OS user logs in.
    pub launch_at_startup: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            target_lang: "zh-CN".into(),
            source_lang: "auto".into(),
            hotkey_translate: "Ctrl+Shift+D".into(),
            hotkey_ocr: "Ctrl+Shift+S".into(),
            hotkey_enabled: true,
            follow_system_proxy: true,
            launch_at_startup: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct EngineConfig {
    /// Primary / first active engine (kept in sync with `actives[0]`).
    pub active: String,
    /// Engines that run in parallel (order preserved; at least one after normalize).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actives: Vec<String>,
    /// Microsoft Translator subscription key (stored locally only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub microsoft_api_key: Option<String>,
    /// Required for regional / multi-service Azure resources.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub microsoft_region: Option<String>,
    /// Google Cloud Translation API v2 key (stored locally only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub google_api_key: Option<String>,
    /// Cloudflare Workers / translate-api compatible endpoint.
    #[serde(
        default,
        alias = "self_hosted_endpoint",
        skip_serializing_if = "Option::is_none"
    )]
    pub cloudflare_endpoint: Option<String>,
    /// Optional access secret for the Cloudflare Worker (`SECRET_PASS`).
    #[serde(
        default,
        alias = "self_hosted_secret",
        skip_serializing_if = "Option::is_none"
    )]
    pub cloudflare_secret: Option<String>,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            active: "microsoft".into(),
            actives: vec!["microsoft".into()],
            microsoft_api_key: None,
            microsoft_region: None,
            google_api_key: None,
            cloudflare_endpoint: None,
            cloudflare_secret: None,
        }
    }
}

impl EngineConfig {
    /// Normalized list of engines to run (never empty after config normalize).
    pub fn resolved_actives(&self) -> Vec<String> {
        if !self.actives.is_empty() {
            return self.actives.clone();
        }
        let active = self.active.trim();
        if active.is_empty() {
            vec!["microsoft".into()]
        } else {
            vec![active.to_string()]
        }
    }
}

/// One Config-driven Engine profile (`[engines.<id>]`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct EngineProfile {
    /// Display name in settings; falls back to profile id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// HTTP method: `GET` | `POST` (default POST).
    pub method: String,
    pub url: String,
    /// `none` | `header` | `query` | `bearer` | `basic`
    pub auth: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_header: Option<String>,
    /// Header value template (e.g. `DeepL-Auth-Key {{extra.api_key}}`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_query_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_query_value: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub headers: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub query: BTreeMap<String, String>,
    /// `json` | `form` | `none`
    pub body_type: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub body: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, String>,
    /// Map app lang codes → vendor codes (`zh-CN` → `ZH`).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub lang_map: BTreeMap<String, String>,
    /// Dot path into JSON for translated text (`translations.0.text`).
    pub text_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_path: Option<String>,
}

impl Default for EngineProfile {
    fn default() -> Self {
        Self {
            label: None,
            method: "POST".into(),
            url: String::new(),
            auth: "none".into(),
            auth_header: None,
            auth_value: None,
            token: None,
            username: None,
            password: None,
            auth_query_key: None,
            auth_query_value: None,
            headers: BTreeMap::new(),
            query: BTreeMap::new(),
            body_type: "json".into(),
            body: BTreeMap::new(),
            extra: BTreeMap::new(),
            lang_map: BTreeMap::new(),
            text_path: String::new(),
            error_path: None,
        }
    }
}

impl EngineProfile {
    pub fn display_label<'a>(&'a self, id: &'a str) -> &'a str {
        self.label
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or(id)
    }
}

/// OCR backend: Windows system OCR or Tesseract.js (frontend WASM).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct OcrConfig {
    /// `system` | `tesseract`
    pub engine: String,
}

impl Default for OcrConfig {
    fn default() -> Self {
        Self {
            engine: "system".into(),
        }
    }
}

impl OcrConfig {
    pub fn is_tesseract(&self) -> bool {
        self.engine.eq_ignore_ascii_case("tesseract")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct DictionaryConfig {
    pub enabled: bool,
    pub paths: Vec<String>,
}

impl Default for DictionaryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            paths: Vec::new(),
        }
    }
}

/// Resolve portable `data/` next to the running executable.
pub fn resolve_paths() -> Result<ConfigPaths, String> {
    let exe = std::env::current_exe().map_err(|e| format!("resolve executable path: {e}"))?;
    let install_dir = exe
        .parent()
        .ok_or_else(|| "executable has no parent directory".to_string())?;
    let data_dir = install_dir.join(DATA_DIR_NAME);
    let config_path = data_dir.join(CONFIG_FILE_NAME);
    Ok(ConfigPaths {
        data_dir,
        config_path,
    })
}

pub fn ensure_data_dir(paths: &ConfigPaths) -> Result<(), String> {
    fs::create_dir_all(&paths.data_dir)
        .map_err(|e| format!("create data dir {}: {e}", paths.data_dir.display()))
}

pub fn load_or_init(paths: &ConfigPaths) -> Result<AppConfig, String> {
    ensure_data_dir(paths)?;
    if paths.config_path.exists() {
        load_from_path(&paths.config_path)
    } else {
        let config = AppConfig::default();
        save_to_path(&paths.config_path, &config)?;
        Ok(config)
    }
}

pub fn load_from_path(path: &Path) -> Result<AppConfig, String> {
    let raw =
        fs::read_to_string(path).map_err(|e| format!("read config {}: {e}", path.display()))?;
    parse_config(&raw)
}

pub fn save_to_path(path: &Path, config: &AppConfig) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("create config parent {}: {e}", parent.display()))?;
    }
    let raw = serialize_config(config)?;
    fs::write(path, raw).map_err(|e| format!("write config {}: {e}", path.display()))
}

pub fn parse_config(raw: &str) -> Result<AppConfig, String> {
    let mut config: AppConfig =
        toml::from_str(raw).map_err(|e| format!("parse config: {e}"))?;
    normalize_config_langs(&mut config);
    Ok(config)
}

pub fn serialize_config(config: &AppConfig) -> Result<String, String> {
    let mut normalized = config.clone();
    normalize_config_langs(&mut normalized);
    let body =
        toml::to_string_pretty(&normalized).map_err(|e| format!("serialize config: {e}"))?;
    Ok(format!(
        "# Look Translate portable config. Do not commit API keys.\n{body}"
    ))
}

/// Rewrite legacy Microsoft-style Chinese tags to Google-style.
fn normalize_config_langs(config: &mut AppConfig) {
    use crate::lang::normalize_lang_code;
    use std::collections::HashSet;

    config.general.target_lang = normalize_lang_code(&config.general.target_lang);
    if config.general.target_lang.is_empty() {
        config.general.target_lang = "zh-CN".into();
    }
    config.general.source_lang = normalize_lang_code(&config.general.source_lang);
    if config.general.source_lang.is_empty() {
        config.general.source_lang = "auto".into();
    }
    let engine = config.ocr.engine.trim().to_ascii_lowercase();
    config.ocr.engine = match engine.as_str() {
        "tesseract" => "tesseract".into(),
        _ => "system".into(),
    };

    // Parallel engine list: empty actives → fall back to single `active`.
    let mut raw: Vec<String> = if config.engine.actives.is_empty() {
        let a = config.engine.active.trim();
        if a.is_empty() {
            vec!["microsoft".into()]
        } else {
            vec![a.to_string()]
        }
    } else {
        config.engine.actives.clone()
    };

    let mut seen = HashSet::new();
    let mut cleaned = Vec::new();
    for id in raw.drain(..) {
        let mut id = id.trim().to_string();
        if id.is_empty() {
            continue;
        }
        // Legacy engine id from earlier builds.
        if id.eq_ignore_ascii_case("self_hosted") {
            id = "cloudflare".into();
        }
        let key = id.to_ascii_lowercase();
        if seen.insert(key) {
            cleaned.push(id);
        }
    }
    if cleaned.is_empty() {
        cleaned.push("microsoft".into());
    }
    config.engine.actives = cleaned.clone();
    config.engine.active = cleaned[0].clone();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn default_roundtrip() {
        let config = AppConfig::default();
        let raw = serialize_config(&config).unwrap();
        let parsed = parse_config(&raw).unwrap();
        assert_eq!(parsed, config);
    }

    #[test]
    fn partial_toml_fills_defaults() {
        let raw = r#"
[general]
target_lang = "en"

[engine]
active = "microsoft"
"#;
        let parsed = parse_config(raw).unwrap();
        assert_eq!(parsed.general.target_lang, "en");
        assert_eq!(parsed.general.source_lang, "auto");
        assert!(parsed.general.hotkey_enabled);
        assert!(parsed.dictionary.enabled);
        assert!(parsed.dictionary.paths.is_empty());
        assert_eq!(parsed.ocr.engine, "system");
        assert_eq!(parsed.engine.actives, vec!["microsoft".to_string()]);
        assert_eq!(parsed.engine.active, "microsoft");
    }

    #[test]
    fn load_or_init_creates_file() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let data_dir = std::env::temp_dir().join(format!("look-translate-config-{nanos}"));
        let paths = ConfigPaths {
            config_path: data_dir.join(CONFIG_FILE_NAME),
            data_dir,
        };

        let config = load_or_init(&paths).unwrap();
        assert!(paths.config_path.exists());
        assert_eq!(config.general.target_lang, "zh-CN");

        let reloaded = load_from_path(&paths.config_path).unwrap();
        assert_eq!(reloaded, config);

        let _ = fs::remove_dir_all(&paths.data_dir);
    }

    #[test]
    fn legacy_chinese_codes_normalize() {
        let raw = r#"
[general]
target_lang = "zh-Hans"
source_lang = "zh-Hant"
"#;
        let parsed = parse_config(raw).unwrap();
        assert_eq!(parsed.general.target_lang, "zh-CN");
        assert_eq!(parsed.general.source_lang, "zh-TW");
    }

    #[test]
    fn legacy_self_hosted_migrates_to_cloudflare() {
        let raw = r#"
[engine]
active = "self_hosted"
self_hosted_endpoint = "https://example.workers.dev/"
self_hosted_secret = "s3cret"
"#;
        let parsed = parse_config(raw).unwrap();
        assert_eq!(parsed.engine.active, "cloudflare");
        assert_eq!(parsed.engine.actives, vec!["cloudflare".to_string()]);
        assert_eq!(
            parsed.engine.cloudflare_endpoint.as_deref(),
            Some("https://example.workers.dev/")
        );
        assert_eq!(parsed.engine.cloudflare_secret.as_deref(), Some("s3cret"));
    }

    #[test]
    fn actives_list_normalizes_and_syncs_active() {
        let raw = r#"
[engine]
actives = ["google_web", "microsoft_web", "google_web", "self_hosted"]
"#;
        let parsed = parse_config(raw).unwrap();
        assert_eq!(
            parsed.engine.actives,
            vec![
                "google_web".to_string(),
                "microsoft_web".to_string(),
                "cloudflare".to_string()
            ]
        );
        assert_eq!(parsed.engine.active, "google_web");
    }

    #[test]
    fn parses_named_engine_profile() {
        let raw = r#"
[engine]
active = "deepl"

[engines.deepl]
label = "DeepL"
method = "POST"
url = "https://api-free.deepl.com/v2/translate"
auth = "header"
auth_header = "Authorization"
auth_value = "DeepL-Auth-Key {{extra.api_key}}"
body_type = "form"
text_path = "translations.0.text"
error_path = "message"

[engines.deepl.body]
text = "{{text}}"
target_lang = "{{target_lang}}"
source_lang = "{{source_lang}}"

[engines.deepl.extra]
api_key = "secret"

[engines.deepl.lang_map]
zh-CN = "ZH"
en = "EN"
auto = ""
"#;
        let parsed = parse_config(raw).unwrap();
        assert_eq!(parsed.engine.active, "deepl");
        let profile = parsed.engines.get("deepl").expect("deepl profile");
        assert_eq!(profile.display_label("deepl"), "DeepL");
        assert_eq!(profile.auth, "header");
        assert_eq!(profile.body_type, "form");
        assert_eq!(profile.body.get("text").map(String::as_str), Some("{{text}}"));
        assert_eq!(profile.extra.get("api_key").map(String::as_str), Some("secret"));
        assert_eq!(profile.lang_map.get("zh-CN").map(String::as_str), Some("ZH"));
        assert!(is_builtin_engine("microsoft"));
        assert!(!is_builtin_engine("deepl"));
    }
}
