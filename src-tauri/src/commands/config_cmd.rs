use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::config::{save_to_path, AppConfig, ConfigState};
use crate::dictionary::{
    install_recommended_dictionary, recommended_mdx_path, DictionaryState,
    InstallRecommendedDictResult, RECOMMENDED_DICT_REL_PATH,
};
use crate::hotkey;
use crate::tray_state::TrayHotkeyToggle;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppPaths {
    pub data_dir: String,
    pub config_path: String,
    pub dicts_dir: String,
    pub recommended_dict_path: String,
    pub recommended_dict_relative: String,
    pub recommended_dict_present: bool,
}

#[tauri::command]
pub fn get_app_paths(state: State<'_, ConfigState>) -> AppPaths {
    let recommended = recommended_mdx_path(&state.paths.data_dir);
    AppPaths {
        data_dir: state.paths.data_dir.to_string_lossy().into_owned(),
        config_path: state.paths.config_path.to_string_lossy().into_owned(),
        dicts_dir: state
            .paths
            .data_dir
            .join("dicts")
            .to_string_lossy()
            .into_owned(),
        recommended_dict_path: recommended.to_string_lossy().into_owned(),
        recommended_dict_relative: RECOMMENDED_DICT_REL_PATH.to_string(),
        recommended_dict_present: recommended.is_file(),
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
    if !config.general.hotkey_ocr.trim().is_empty() {
        hotkey::validate_shortcut(&config.general.hotkey_ocr)?;
    }
    if config
        .general
        .hotkey_translate
        .trim()
        .eq_ignore_ascii_case(config.general.hotkey_ocr.trim())
    {
        return Err("划词热键与 OCR 热键不能相同".into());
    }
    // serialize_config normalizes legacy zh-Hans / zh-Hant on write.
    save_to_path(&state.paths.config_path, &config)?;
    let saved = crate::config::load_from_path(&state.paths.config_path)?;
    {
        let mut guard = state
            .config
            .write()
            .map_err(|_| "config lock poisoned".to_string())?;
        *guard = saved.clone();
    }
    hotkey::apply(&app, &saved)?;
    if let Some(tray) = app.try_state::<TrayHotkeyToggle>() {
        let _ = tray.item.set_checked(saved.general.hotkey_enabled);
    }
    if let Some(dict) = app.try_state::<DictionaryState>() {
        dict.invalidate();
    }
    Ok(saved)
}

#[tauri::command]
pub async fn install_recommended_dict(
    app: AppHandle,
) -> Result<InstallRecommendedDictResult, String> {
    let (paths, config, follow_proxy) = {
        let state = app
            .try_state::<ConfigState>()
            .ok_or_else(|| "config state missing".to_string())?;
        let guard = state
            .config
            .read()
            .map_err(|_| "config lock poisoned".to_string())?;
        (
            state.paths.clone(),
            guard.clone(),
            guard.general.follow_system_proxy,
        )
    };

    let result =
        install_recommended_dictionary(&paths, config, follow_proxy).await?;

    {
        let state = app
            .try_state::<ConfigState>()
            .ok_or_else(|| "config state missing".to_string())?;
        let mut guard = state
            .config
            .write()
            .map_err(|_| "config lock poisoned".to_string())?;
        *guard = result.config.clone();
    }
    if let Some(dict) = app.try_state::<DictionaryState>() {
        dict.invalidate();
    }

    Ok(result)
}
