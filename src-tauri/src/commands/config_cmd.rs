use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::config::{
    custom_http_profile, load_from_path, save_to_path, AppConfig, ConfigState, CUSTOM_PROFILE_ID,
};
use crate::dictionary::{
    install_recommended_dictionary, recommended_mdx_path, DictionaryState,
    InstallRecommendedDictResult, RECOMMENDED_DICT_REL_PATH,
};
use crate::hotkey;
use crate::tray_state::TrayHotkeyToggle;
use tauri_plugin_autostart::ManagerExt;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppPaths {
    pub data_dir: String,
    pub config_path: String,
    pub log_dir: String,
    pub dicts_dir: String,
    pub recommended_dict_path: String,
    pub recommended_dict_relative: String,
    pub recommended_dict_present: bool,
}

#[tauri::command]
pub fn get_app_paths(state: State<'_, ConfigState>) -> AppPaths {
    let recommended = recommended_mdx_path(&state.paths.data_dir);
    let log_dir = state.paths.data_dir.join("logs");
    AppPaths {
        data_dir: state.paths.data_dir.to_string_lossy().into_owned(),
        config_path: state.paths.config_path.to_string_lossy().into_owned(),
        log_dir: log_dir.to_string_lossy().into_owned(),
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
    let saved = load_from_path(&state.paths.config_path)?;
    apply_saved_config(&app, &state, saved)
}

/// Re-read `data/config.toml` into memory (after external edits).
#[tauri::command]
pub fn reload_config(app: AppHandle, state: State<'_, ConfigState>) -> Result<AppConfig, String> {
    let loaded = load_from_path(&state.paths.config_path)?;
    apply_saved_config(&app, &state, loaded)
}

/// Open the portable config file with the OS default editor.
#[tauri::command]
pub fn open_config_file(state: State<'_, ConfigState>) -> Result<(), String> {
    let path = &state.paths.config_path;
    if !path.exists() {
        let guard = state
            .config
            .read()
            .map_err(|_| "config lock poisoned".to_string())?;
        save_to_path(path, &guard)?;
    }
    open_path_with_default_app(path)
}

/// Open (and create if needed) the portable logs folder under `data/logs`.
#[tauri::command]
pub fn open_log_dir(state: State<'_, ConfigState>) -> Result<(), String> {
    let log_dir = state.paths.data_dir.join("logs");
    std::fs::create_dir_all(&log_dir)
        .map_err(|e| format!("创建日志目录失败: {e}"))?;
    open_path_with_default_app(&log_dir)
}

/// Open the portable data directory in the file manager.
#[tauri::command]
pub fn open_data_dir(state: State<'_, ConfigState>) -> Result<(), String> {
    crate::config::ensure_data_dir(&state.paths)?;
    open_path_with_default_app(&state.paths.data_dir)
}

/// Open `docs/engine-profiles.md` when present next to the install / repo.
#[tauri::command]
pub fn open_engine_profiles_doc(state: State<'_, ConfigState>) -> Result<(), String> {
    let path = resolve_engine_profiles_doc(&state.paths.data_dir)
        .ok_or_else(|| {
            "未找到 docs/engine-profiles.md（开发时在仓库 docs/ 下；发布包可查看项目文档）".to_string()
        })?;
    open_path_with_default_app(&path)
}

/// Insert a generic HTTP Engine Profile if `engines.custom` is missing.
#[tauri::command]
pub fn ensure_custom_engine_profile(
    app: AppHandle,
    state: State<'_, ConfigState>,
) -> Result<EnsureProfileResult, String> {
    let mut config = state
        .config
        .read()
        .map_err(|_| "config lock poisoned".to_string())?
        .clone();

    if config.engines.contains_key(CUSTOM_PROFILE_ID) {
        return Ok(EnsureProfileResult {
            created: false,
            message: "已存在 [engines.custom]。请打开配置文件按 docs/engine-profiles.md 填写，或改名后再添加。".into(),
            config,
        });
    }

    config
        .engines
        .insert(CUSTOM_PROFILE_ID.into(), custom_http_profile());
    save_to_path(&state.paths.config_path, &config)?;
    let saved = load_from_path(&state.paths.config_path)?;
    let saved = apply_saved_config(&app, &state, saved)?;
    Ok(EnsureProfileResult {
        created: true,
        message: "已添加通用 HTTP 模板 [engines.custom]。请打开配置文件填写 url/鉴权/响应路径；厂商示例见文档 engine-profiles。".into(),
        config: saved,
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnsureProfileResult {
    pub created: bool,
    pub message: String,
    pub config: AppConfig,
}

fn apply_saved_config(
    app: &AppHandle,
    state: &State<'_, ConfigState>,
    saved: AppConfig,
) -> Result<AppConfig, String> {
    {
        let mut guard = state
            .config
            .write()
            .map_err(|_| "config lock poisoned".to_string())?;
        *guard = saved.clone();
    }
    hotkey::apply(app, &saved)?;
    sync_launch_at_startup(app, saved.general.launch_at_startup)?;
    if let Some(tray) = app.try_state::<TrayHotkeyToggle>() {
        let _ = tray.item.set_checked(saved.general.hotkey_enabled);
    }
    if let Some(dict) = app.try_state::<DictionaryState>() {
        dict.invalidate();
    }
    Ok(saved)
}

/// Keep OS login-item registration in sync with `general.launch_at_startup`.
pub fn sync_launch_at_startup(app: &AppHandle, enabled: bool) -> Result<(), String> {
    let manager = app.autolaunch();
    let currently = manager
        .is_enabled()
        .map_err(|e| format!("读取开机自启状态失败: {e}"))?;
    if enabled == currently {
        return Ok(());
    }
    if enabled {
        manager
            .enable()
            .map_err(|e| format!("启用开机自启失败: {e}"))?;
    } else {
        manager
            .disable()
            .map_err(|e| format!("关闭开机自启失败: {e}"))?;
    }
    Ok(())
}

/// Open an http(s) URL with the OS default browser.
#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
    let trimmed = url.trim();
    if !(trimmed.starts_with("https://") || trimmed.starts_with("http://")) {
        return Err("仅支持 http/https 链接".into());
    }
    open_url_with_default_browser(trimmed)
}

fn open_url_with_default_browser(url: &str) -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        // `cmd /C start "" <url>` opens the system default browser.
        std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| format!("打开浏览器失败: {e}"))?;
        return Ok(());
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(url)
            .spawn()
            .map_err(|e| format!("打开浏览器失败: {e}"))?;
        return Ok(());
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::process::Command::new("xdg-open")
            .arg(url)
            .spawn()
            .map_err(|e| format!("打开浏览器失败: {e}"))?;
        return Ok(());
    }
    #[allow(unreachable_code)]
    Err("当前平台不支持打开链接".into())
}

fn open_path_with_default_app(path: &std::path::Path) -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        if path.is_dir() {
            std::process::Command::new("explorer")
                .arg(path)
                .creation_flags(CREATE_NO_WINDOW)
                .spawn()
                .map_err(|e| format!("打开目录失败: {e}"))?;
        } else {
            std::process::Command::new("cmd")
                .args(["/C", "start", "", &path.to_string_lossy()])
                .creation_flags(CREATE_NO_WINDOW)
                .spawn()
                .map_err(|e| format!("打开文件失败: {e}"))?;
        }
        return Ok(());
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("打开文件失败: {e}"))?;
        return Ok(());
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("打开文件失败: {e}"))?;
        return Ok(());
    }
    #[allow(unreachable_code)]
    Err("当前平台不支持打开该文件".into())
}

/// `data/` is next to the exe; docs may live at repo root or beside install dir.
fn resolve_engine_profiles_doc(data_dir: &std::path::Path) -> Option<std::path::PathBuf> {
    let candidates = [
        data_dir.join("../docs/engine-profiles.md"),
        data_dir.join("../../docs/engine-profiles.md"),
        data_dir.join("../../../docs/engine-profiles.md"),
        data_dir.join("docs/engine-profiles.md"),
    ];
    candidates.into_iter().find(|p| p.is_file()).map(|p| {
        std::fs::canonicalize(&p).unwrap_or(p)
    })
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
