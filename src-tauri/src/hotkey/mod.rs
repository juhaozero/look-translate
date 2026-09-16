//! Global hotkey registration (Ctrl+Shift+D by default).

use std::str::FromStr;

use serde::Serialize;
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use crate::commands::window_cmd;
use crate::config::{AppConfig, ConfigState};

#[allow(dead_code)]
pub const DEFAULT_TRANSLATE_HOTKEY: &str = "Ctrl+Shift+D";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyStatus {
    pub enabled: bool,
    pub translate: String,
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

/// Unregister all shortcuts, then register translate hotkey when enabled.
pub fn apply(app: &AppHandle, config: &AppConfig) -> Result<(), String> {
    let gs = app.global_shortcut();
    gs.unregister_all()
        .map_err(|e| format!("unregister hotkeys: {e}"))?;

    if !config.general.hotkey_enabled {
        return Ok(());
    }

    let shortcut = validate_shortcut(&config.general.hotkey_translate)?;
    let shortcut_str = config.general.hotkey_translate.trim().to_string();

    gs.on_shortcut(shortcut, move |app, _shortcut, event| {
        if event.state == ShortcutState::Pressed {
            // Phase 1 step 3: show popup. Capture/translate pipeline comes next.
            let _ = window_cmd::open_popup(app);
        }
    })
    .map_err(|e| format!("注册热键 `{shortcut_str}` 失败（可能被占用）: {e}"))?;

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
    let registered = if enabled {
        validate_shortcut(&translate)
            .ok()
            .map(|s| app.global_shortcut().is_registered(s))
            .unwrap_or(false)
    } else {
        false
    };

    Ok(HotkeyStatus {
        enabled,
        translate,
        registered,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_default_hotkey() {
        assert!(validate_shortcut(DEFAULT_TRANSLATE_HOTKEY).is_ok());
        assert!(validate_shortcut("ctrl+shift+d").is_ok());
    }

    #[test]
    fn rejects_empty_and_invalid() {
        assert!(validate_shortcut("").is_err());
        assert!(validate_shortcut("NotAKey").is_err());
        assert!(validate_shortcut("Ctrl+Shift+D+A").is_err());
    }
}
