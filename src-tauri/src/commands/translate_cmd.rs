use std::sync::Arc;

use tauri::{AppHandle, Emitter, Manager};

use crate::cache::TranslationCacheState;
use crate::config::ConfigState;
use crate::dictionary::{lookup_short_word, DictionaryState};
use crate::translate::{
    list_engine_catalog, request_from_config, run_parallel, BuiltinEngineRunner, EngineCatalogItem,
    TranslationPayload, TranslationState,
};

#[tauri::command]
pub fn get_last_translation(app: AppHandle) -> Option<TranslationPayload> {
    app.try_state::<TranslationState>()
        .and_then(|state| state.latest())
}

#[tauri::command]
pub fn list_engines(app: AppHandle) -> Result<Vec<EngineCatalogItem>, String> {
    let state = app
        .try_state::<ConfigState>()
        .ok_or_else(|| "config state missing".to_string())?;
    let guard = state
        .config
        .read()
        .map_err(|_| "config lock poisoned".to_string())?;
    Ok(list_engine_catalog(&guard))
}

#[tauri::command]
pub async fn translate_text(
    app: AppHandle,
    text: String,
    target_lang: Option<String>,
) -> Result<TranslationPayload, String> {
    run_translate(&app, text, target_lang).await
}

#[tauri::command]
pub fn clear_translation_cache(app: AppHandle) -> Result<(), String> {
    let cache = app
        .try_state::<TranslationCacheState>()
        .ok_or_else(|| "translation cache state missing".to_string())?;
    cache.clear();
    Ok(())
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

    let req = request_from_config(&config, text, target_lang);

    let dict_entry = {
        let data_dir = app
            .try_state::<ConfigState>()
            .map(|state| state.paths.data_dir.clone());
        app.try_state::<DictionaryState>().and_then(|state| {
            let dir = data_dir?;
            lookup_short_word(&state, &config, &dir, &req.text)
        })
    };

    let cache = app.try_state::<TranslationCacheState>();
    let runner = Arc::new(BuiltinEngineRunner);

    let payload = run_parallel(
        &config,
        req,
        cache.as_deref(),
        dict_entry,
        runner,
        |payload| store_and_emit(app, payload),
    )
    .await;

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
