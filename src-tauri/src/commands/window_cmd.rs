use std::sync::Mutex;
use std::time::{Duration, Instant};

use arboard::Clipboard;
use enigo::{Enigo, Mouse, Settings};
use tauri::{AppHandle, Manager, PhysicalPosition, Position, WebviewWindow};

/// Tracks when the popup was last shown to ignore spurious early blur events.
#[derive(Default)]
pub struct PopupUiState {
    pub last_shown: Mutex<Option<Instant>>,
}

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

    if let Err(err) = position_near_cursor(app, &window) {
        eprintln!("[look-translate] position popup failed: {err}");
    }

    if let Some(state) = app.try_state::<PopupUiState>() {
        if let Ok(mut guard) = state.last_shown.lock() {
            *guard = Some(Instant::now());
        }
    }

    show_window(&window)
}

pub fn hide_popup(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("popup")
        .ok_or_else(|| "popup window not found".to_string())?;
    window.hide().map_err(|e| e.to_string())
}

/// Hide popup on blur, unless it was just shown (avoids flicker on open).
pub fn maybe_hide_popup_on_blur(app: &AppHandle) {
    if let Some(state) = app.try_state::<PopupUiState>() {
        if let Ok(guard) = state.last_shown.lock() {
            if let Some(shown_at) = *guard {
                if shown_at.elapsed() < Duration::from_millis(250) {
                    return;
                }
            }
        }
    }
    let _ = hide_popup(app);
}

fn cursor_position() -> Result<(i32, i32), String> {
    let enigo = Enigo::new(&Settings::default()).map_err(|e| format!("enigo: {e}"))?;
    enigo
        .location()
        .map_err(|e| format!("read cursor position: {e}"))
}

fn position_near_cursor(app: &AppHandle, window: &WebviewWindow) -> Result<(), String> {
    let (cx, cy) = cursor_position()?;
    let outer = window.outer_size().map_err(|e| e.to_string())?;
    let width = outer.width as i32;
    let height = outer.height as i32;

    let mut x = cx + 12;
    let mut y = cy + 18;

    if let Ok(monitors) = app.available_monitors() {
        if let Some(monitor) = monitors.into_iter().find(|m| {
            let pos = m.position();
            let size = m.size();
            cx >= pos.x
                && cy >= pos.y
                && cx < pos.x + size.width as i32
                && cy < pos.y + size.height as i32
        }) {
            let pos = monitor.position();
            let size = monitor.size();
            let left = pos.x;
            let top = pos.y;
            let right = pos.x + size.width as i32;
            let bottom = pos.y + size.height as i32;
            x = x.clamp(left, (right - width).max(left));
            y = y.clamp(top, (bottom - height).max(top));
        }
    }

    window
        .set_position(Position::Physical(PhysicalPosition { x, y }))
        .map_err(|e| e.to_string())
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
    hide_popup(&app)
}

#[tauri::command]
pub fn copy_text(text: String) -> Result<(), String> {
    let mut clipboard =
        Clipboard::new().map_err(|e| format!("打开剪贴板失败: {e}"))?;
    clipboard
        .set_text(text)
        .map_err(|e| format!("复制失败: {e}"))
}
