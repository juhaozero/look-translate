//! Portable config under install-dir `data/`.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use serde::{Deserialize, Serialize};

pub const CONFIG_FILE_NAME: &str = "config.toml";
pub const DATA_DIR_NAME: &str = "data";

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
    pub ocr: OcrConfig,
    pub dictionary: DictionaryConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            engine: EngineConfig::default(),
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
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct EngineConfig {
    pub active: String,
    /// Microsoft Translator subscription key (stored locally only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub microsoft_api_key: Option<String>,
    /// Required for regional / multi-service Azure resources.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub microsoft_region: Option<String>,
    /// Google Cloud Translation API v2 key (stored locally only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub google_api_key: Option<String>,
    /// Self-hosted translate-api compatible endpoint (Cloudflare Worker URL, etc.).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub self_hosted_endpoint: Option<String>,
    /// Optional access secret for the self-hosted endpoint.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub self_hosted_secret: Option<String>,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            active: "microsoft".into(),
            microsoft_api_key: None,
            microsoft_region: None,
            google_api_key: None,
            self_hosted_endpoint: None,
            self_hosted_secret: None,
        }
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
}
