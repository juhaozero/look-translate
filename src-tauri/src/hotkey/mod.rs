//! Global hotkey registration.
//!
//! - Translate hotkey → clipboard selection capture
//! - OCR hotkey → screenshot OCR (separate path, never a clipboard fallback)

use std::str::FromStr;

use serde::Serialize;
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use crate::commands::capture_cmd;
use crate::config::{AppConfig, ConfigState};

#[allow(dead_code)]
pub const DEFAULT_TRANSLATE_HOTKEY: &str = "Ctrl+Shift+D";
#[allow(dead_code)]
pub const DEFAULT_OCR_HOTKEY: &str = "Ctrl+Shift+S";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyStatus {
    pub enabled: bool,
    pub translate: String,
    pub ocr: String,
    pub translate_registered: bool,
    pub ocr_registered: bool,
    /// Backward-compatible alias of `translate_registered`.
    pub registered: bool,
}

/// Validate a hotkey string such as `Ctrl+Shift+D`.
pub fn validate_shortcut(raw: &str) -> Result<Shortcut, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("热键不能为空".into());
    }
    Shortcut::from_str(trimmed).map_err(|e| format!("无效热键 `{trimmed}`: {e}"))
}

/// Unregister all shortcuts, then register translate + OCR hotkeys when enabled.
pub fn apply(app: &AppHandle, config: &AppConfig) -> Result<(), String> {
    let gs = app.global_shortcut();
    gs.unregister_all()
        .map_err(|e| format!("unregister hotkeys: {e}"))?;

    if !config.general.hotkey_enabled {
        return Ok(());
    }

    let translate_raw = config.general.hotkey_translate.trim().to_string();
    let ocr_raw = config.general.hotkey_ocr.trim().to_string();

    if !ocr_raw.is_empty() && translate_raw.eq_ignore_ascii_case(&ocr_raw) {
        return Err("划词热键与 OCR 热键不能相同".into());
    }

    let translate = validate_shortcut(&translate_raw)?;
    gs.on_shortcut(translate, move |app, _shortcut, event| {
        // Use Released so Ctrl/Shift from the hotkey are already up before Ctrl+C.
        if event.state == ShortcutState::Released {
            capture_cmd::run_capture_and_show_popup(app);
        }
    })
    .map_err(|e| format!("注册划词热键 `{translate_raw}` 失败（可能被占用）: {e}"))?;

    if !ocr_raw.is_empty() {
        let ocr = validate_shortcut(&ocr_raw)?;
        gs.on_shortcut(ocr, move |app, _shortcut, event| {
            if event.state == ShortcutState::Released {
                capture_cmd::run_ocr_and_show_popup(app);
            }
        })
        .map_err(|e| format!("注册 OCR 热键 `{ocr_raw}` 失败（可能被占用）: {e}"))?;
    }

    Ok(())
}

pub fn apply_from_state(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<ConfigState>();
    let config = state
        .config
        .read()
        .map_err(|_| "config lock poisoned".to_string())?
        .clone();
    apply(app, &config)
}

pub fn status(app: &AppHandle) -> Result<HotkeyStatus, String> {
    let state = app.state::<ConfigState>();
    let config = state
        .config
        .read()
        .map_err(|_| "config lock poisoned".to_string())?;
    let enabled = config.general.hotkey_enabled;
    let translate = config.general.hotkey_translate.clone();
    let ocr = config.general.hotkey_ocr.clone();

    let translate_registered = if enabled {
        validate_shortcut(&translate)
            .ok()
            .map(|s| app.global_shortcut().is_registered(s))
            .unwrap_or(false)
    } else {
        false
    };
    let ocr_registered = if enabled && !ocr.trim().is_empty() {
        validate_shortcut(&ocr)
            .ok()
            .map(|s| app.global_shortcut().is_registered(s))
            .unwrap_or(false)
    } else {
        false
    };

    Ok(HotkeyStatus {
        enabled,
        translate,
        ocr,
        translate_registered,
        ocr_registered,
        registered: translate_registered,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_default_hotkeys() {
        assert!(validate_shortcut(DEFAULT_TRANSLATE_HOTKEY).is_ok());
        assert!(validate_shortcut(DEFAULT_OCR_HOTKEY).is_ok());
        assert!(validate_shortcut("ctrl+shift+d").is_ok());
    }

    #[test]
    fn rejects_empty_and_invalid() {
        assert!(validate_shortcut("").is_err());
        assert!(validate_shortcut("NotAKey").is_err());
        assert!(validate_shortcut("Ctrl+Shift+D+A").is_err());
    }
}
