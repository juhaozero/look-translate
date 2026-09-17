use tauri::{AppHandle, Emitter, Manager};

use crate::capture::{capture_payload, CapturePayload, CaptureState};
use crate::commands::{translate_cmd, window_cmd};
use crate::ocr;

/// Clipboard selection capture for the translate hotkey.
/// Never falls back to OCR.
pub fn run_capture_and_show_popup(app: &AppHandle) {
    let payload = capture_payload();
    finish_capture(app, payload);
}

/// Dedicated OCR hotkey path (screenshot near cursor).
/// Never falls back to clipboard selection capture.
pub fn run_ocr_and_show_popup(app: &AppHandle) {
    let payload = ocr::capture_ocr_payload();
    finish_capture(app, payload);
}

fn finish_capture(app: &AppHandle, payload: CapturePayload) {
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
