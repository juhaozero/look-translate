use tauri::{AppHandle, Manager, WebviewWindow};

fn show_window(window: &WebviewWindow) -> Result<(), String> {
    window.show().map_err(|e| e.to_string())?;
    window.set_focus().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn open_settings(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("settings")
        .ok_or_else(|| "settings window not found".to_string())?;
    show_window(&window)
}

pub fn open_popup(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("popup")
        .ok_or_else(|| "popup window not found".to_string())?;
    show_window(&window)
}

#[tauri::command]
pub fn show_settings_window(app: AppHandle) -> Result<(), String> {
    open_settings(&app)
}

#[tauri::command]
pub fn show_popup_window(app: AppHandle) -> Result<(), String> {
    open_popup(&app)
}

#[tauri::command]
pub fn hide_popup_window(app: AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("popup")
        .ok_or_else(|| "popup window not found".to_string())?;
    window.hide().map_err(|e| e.to_string())
}
