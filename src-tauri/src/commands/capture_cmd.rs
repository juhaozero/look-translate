use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

use crate::capture::{capture_payload, CapturePayload, CaptureState};
use crate::commands::{translate_cmd, window_cmd};
use crate::config::ConfigState;
use crate::ocr::{self, OcrRegionHint, OcrSessionState};
use crate::translate::TranslationState;

/// Clipboard selection capture for the translate hotkey.
/// Never falls back to OCR.
pub fn run_capture_and_show_popup(app: &AppHandle) {
    let payload = capture_payload();
    finish_capture(app, payload);
}

/// Dedicated OCR hotkey path: snapshot + region picker (never clipboard fallback).
pub fn run_ocr_and_show_popup(app: &AppHandle) {
    let _ = window_cmd::hide_popup(app);
    if let Err(err) = window_cmd::open_ocr_select(app) {
        eprintln!("[look-translate] open ocr-select failed: {err}");
        if let Some(state) = app.try_state::<OcrSessionState>() {
            state.clear();
        }
        let payload = CapturePayload::fail(err, "ocr");
        finish_capture(app, payload);
    }
}

fn finish_capture(app: &AppHandle, payload: CapturePayload) {
    if let Some(state) = app.try_state::<CaptureState>() {
        state.store(payload.clone());
    }

    // Failed capture must not keep the previous translation (popup remount reads it).
    if payload.empty || payload.error.is_some() {
        if let Some(state) = app.try_state::<TranslationState>() {
            state.clear();
        }
    }

    let _ = app.emit("capture-updated", &payload);
    let _ = window_cmd::open_popup(app);

    if !payload.empty && payload.error.is_none() {
        translate_cmd::start_translate_after_capture(app, payload.text);
    }
}

fn ocr_engine_is_tesseract(app: &AppHandle) -> bool {
    app.try_state::<ConfigState>()
        .and_then(|state| state.config.read().ok().map(|c| c.ocr.is_tesseract()))
        .unwrap_or(false)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OcrRegionInput {
    pub left: i32,
    pub top: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum OcrConfirmResult {
    /// System OCR finished; popup already opened.
    Done,
    /// Frontend should run Tesseract.js on this PNG data URL.
    Tesseract {
        /// Internally-tagged enums do not rename struct fields via `rename_all`.
        #[serde(rename = "imageDataUrl")]
        image_data_url: String,
    },
}

#[tauri::command]
pub fn get_ocr_region_hint(app: AppHandle) -> Result<OcrRegionHint, String> {
    app.try_state::<OcrSessionState>()
        .and_then(|state| state.hint())
        .ok_or_else(|| "OCR 选区会话不存在，请重新按 OCR 热键".into())
}

#[tauri::command]
pub fn cancel_ocr_select(app: AppHandle) -> Result<(), String> {
    if let Some(state) = app.try_state::<OcrSessionState>() {
        state.clear();
    }
    window_cmd::hide_ocr_select(&app)
}

/// Crop from the pre-overlay snapshot.
/// - `system`: Windows.Media.Ocr + popup
/// - `tesseract`: return PNG data URL for frontend WASM OCR
#[tauri::command]
pub fn confirm_ocr_region(
    app: AppHandle,
    region: OcrRegionInput,
) -> Result<OcrConfirmResult, String> {
    let session = app
        .try_state::<OcrSessionState>()
        .and_then(|state| state.take())
        .ok_or_else(|| "OCR  OCR 热键".to_string())?;

    // Keep the overlay visible while the webview runs Tesseract.js; hiding
    // early looks like a silent exit and hides load/recognition errors.
    if ocr_engine_is_tesseract(&app) {
        let (pixels, w, h) = ocr::prepare_region_image(
            &session,
            region.left,
            region.top,
            region.width,
            region.height,
        )?;
        let image_data_url = ocr::bgra_to_png_data_url(&pixels, w, h)?;
        return Ok(OcrConfirmResult::Tesseract { image_data_url });
    }

    let _ = window_cmd::hide_ocr_select(&app);
    let payload = ocr::capture_ocr_payload_from_session(
        &session,
        region.left,
        region.top,
        region.width,
        region.height,
    );
    finish_capture(&app, payload);
    Ok(OcrConfirmResult::Done)
}

/// Submit text recognized by Tesseract.js (or any frontend OCR).
#[tauri::command]
pub fn submit_ocr_text(app: AppHandle, text: String) -> Result<(), String> {
    let _ = window_cmd::hide_ocr_select(&app);
    let payload = match crate::capture::normalize_captured_text(&text) {
        Ok(t) => CapturePayload::ok(t, "ocr"),
        Err(_) => CapturePayload::fail("OCR 未识别到文字（可调整选区后重试）", "ocr"),
    };
    finish_capture(&app, payload);
    Ok(())
}

#[tauri::command]
pub fn get_last_capture(app: AppHandle) -> Option<CapturePayload> {
    app.try_state::<CaptureState>()
        .and_then(|state| state.latest())
}
