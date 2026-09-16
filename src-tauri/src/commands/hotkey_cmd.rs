use tauri::{AppHandle, Manager};

use crate::config::{save_to_path, ConfigState};
use crate::hotkey::{self, HotkeyStatus};
use crate::tray_state::TrayHotkeyToggle;

#[tauri::command]
pub fn get_hotkey_status(app: AppHandle) -> Result<HotkeyStatus, String> {
    hotkey::status(&app)
}

/// Toggle hotkey from tray; persists config and re-registers shortcuts.
pub fn toggle_hotkey_enabled(app: &AppHandle) -> Result<bool, String> {
    let state = app.state::<ConfigState>();
    let snapshot = {
        let mut config = state
            .config
            .write()
            .map_err(|_| "config lock poisoned".to_string())?;
        config.general.hotkey_enabled = !config.general.hotkey_enabled;
        config.clone()
    };

    save_to_path(&state.paths.config_path, &snapshot)?;
    hotkey::apply(app, &snapshot)?;

    if let Some(tray) = app.try_state::<TrayHotkeyToggle>() {
        let _ = tray.item.set_checked(snapshot.general.hotkey_enabled);
    }

    Ok(snapshot.general.hotkey_enabled)
}
