use tauri::{AppHandle, Emitter, Manager};

use crate::cache::{CacheKey, TranslationCacheState};
use crate::config::ConfigState;
use crate::translate::{
    request_from_config, translate_with_config, TranslationPayload, TranslationState,
};

#[tauri::command]
pub fn get_last_translation(app: AppHandle) -> Option<TranslationPayload> {
    app.try_state::<TranslationState>()
        .and_then(|state| state.latest())
}

#[tauri::command]
pub async fn translate_text(
    app: AppHandle,
    text: String,
    target_lang: Option<String>,
) -> Result<TranslationPayload, String> {
    run_translate(&app, text, target_lang).await
}

pub async fn run_translate(
    app: &AppHandle,
    text: String,
    target_lang: Option<String>,
) -> Result<TranslationPayload, String> {
    let config = {
        let state = app
            .try_state::<ConfigState>()
            .ok_or_else(|| "config state missing".to_string())?;
        let guard = state
            .config
            .read()
            .map_err(|_| "config lock poisoned".to_string())?;
        guard.clone()
    };

    let req = request_from_config(&config, text.clone(), target_lang);
    let cache_key = CacheKey::new(
        config.engine.active.clone(),
        req.source_lang.clone(),
        req.target_lang.clone(),
        req.text.clone(),
    );

    if let Some(cache) = app.try_state::<TranslationCacheState>() {
        if let Some(cached) = cache.get(&cache_key) {
            let payload = TranslationPayload::ok(&text, cached, true);
            store_and_emit(app, payload.clone());
            return Ok(payload);
        }
    }

    let loading = TranslationPayload::loading(&req.text, &req.source_lang, &req.target_lang);
    store_and_emit(app, loading);

    let payload = match translate_with_config(&config, req.clone()).await {
        Ok(result) => {
            if let Some(cache) = app.try_state::<TranslationCacheState>() {
                cache.put(cache_key, result.clone());
            }
            TranslationPayload::ok(&text, result, false)
        }
        Err(err) => TranslationPayload::fail(&text, &req.source_lang, &req.target_lang, err),
    };
    store_and_emit(app, payload.clone());
    Ok(payload)
}

pub fn start_translate_after_capture(app: &AppHandle, text: String) {
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let _ = run_translate(&app_handle, text, None).await;
    });
}

fn store_and_emit(app: &AppHandle, payload: TranslationPayload) {
    if let Some(state) = app.try_state::<TranslationState>() {
        state.store(payload.clone());
    }
    let _ = app.emit("translation-updated", &payload);
}
