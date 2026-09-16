use tauri::{AppHandle, Emitter, Manager};

use crate::capture::{capture_payload, CapturePayload, CaptureState};
use crate::commands::{translate_cmd, window_cmd};

/// Capture selection, store/emit result, show popup, then translate when possible.
pub fn run_capture_and_show_popup(app: &AppHandle) {
    let payload = capture_payload();
    if let Some(state) = app.try_state::<CaptureState>() {
        state.store(payload.clone());
    }
    let _ = app.emit("capture-updated", &payload);
    let _ = window_cmd::open_popup(app);

    if !payload.empty && payload.error.is_none() {
        translate_cmd::start_translate_after_capture(app, payload.text);
    }
}

#[tauri::command]
pub fn get_last_capture(app: AppHandle) -> Option<CapturePayload> {
    app.try_state::<CaptureState>()
        .and_then(|state| state.latest())
}
