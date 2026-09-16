use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::config::{save_to_path, AppConfig, ConfigState};
use crate::hotkey;
use crate::tray_state::TrayHotkeyToggle;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppPaths {
    pub data_dir: String,
    pub config_path: String,
}

#[tauri::command]
pub fn get_app_paths(state: State<'_, ConfigState>) -> AppPaths {
    AppPaths {
        data_dir: state.paths.data_dir.to_string_lossy().into_owned(),
        config_path: state.paths.config_path.to_string_lossy().into_owned(),
    }
}

#[tauri::command]
pub fn get_config(state: State<'_, ConfigState>) -> Result<AppConfig, String> {
    state
        .config
        .read()
        .map(|guard| guard.clone())
        .map_err(|_| "config lock poisoned".into())
}

#[tauri::command]
pub fn save_config(
    app: AppHandle,
    state: State<'_, ConfigState>,
    config: AppConfig,
) -> Result<AppConfig, String> {
    hotkey::validate_shortcut(&config.general.hotkey_translate)?;
    save_to_path(&state.paths.config_path, &config)?;
    {
        let mut guard = state
            .config
            .write()
            .map_err(|_| "config lock poisoned".to_string())?;
        *guard = config.clone();
    }
    hotkey::apply(&app, &config)?;
    if let Some(tray) = app.try_state::<TrayHotkeyToggle>() {
        let _ = tray.item.set_checked(config.general.hotkey_enabled);
    }
    Ok(config)
}
