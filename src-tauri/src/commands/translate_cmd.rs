use std::collections::HashMap;

use futures::stream::{FuturesUnordered, StreamExt};
use tauri::{AppHandle, Emitter, Manager};

use crate::cache::{CacheKey, TranslationCacheState};
use crate::config::ConfigState;
use crate::dictionary::{lookup_short_word, DictEntry, DictionaryState};
use crate::translate::{
    request_from_config, translate_with_engine, EngineTranslationResult, TranslationPayload,
    TranslationState,
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

    let req = request_from_config(&config, text.clone(), target_lang);
    let actives = config.engine.resolved_actives();

    let dict_entry = {
        let data_dir = app
            .try_state::<ConfigState>()
            .map(|state| state.paths.data_dir.clone());
        app.try_state::<DictionaryState>().and_then(|state| {
            let dir = data_dir?;
            lookup_short_word(&state, &config, &dir, &req.text)
        })
    };

    let loading = TranslationPayload::loading(
        &req.text,
        &req.source_lang,
        &req.target_lang,
        &actives,
    )
    .with_dictionary(dict_entry.clone());
    store_and_emit(app, loading);

    let mut by_engine: HashMap<String, EngineTranslationResult> = HashMap::new();
    let mut pending: Vec<String> = Vec::new();

    for engine_id in &actives {
        let key = CacheKey::new(
            engine_id.clone(),
            req.source_lang.clone(),
            req.target_lang.clone(),
            req.text.clone(),
        );
        if let Some(cache) = app.try_state::<TranslationCacheState>() {
            if let Some(cached) = cache.get(&key) {
                by_engine.insert(
                    engine_id.clone(),
                    EngineTranslationResult::ok(cached, true),
                );
                continue;
            }
        }
        pending.push(engine_id.clone());
    }

    // Show cache hits immediately while others stay in loading.
    if !by_engine.is_empty() {
        emit_progress(
            app,
            &text,
            &req.source_lang,
            &req.target_lang,
            &actives,
            &by_engine,
            dict_entry.clone(),
        );
    }

    if !pending.is_empty() {
        let mut tasks = FuturesUnordered::new();
        for engine_id in pending {
            let config = config.clone();
            let req = req.clone();
            tasks.push(async move {
                let outcome = translate_with_engine(&config, &engine_id, req).await;
                (engine_id, outcome)
            });
        }

        while let Some((engine_id, outcome)) = tasks.next().await {
            match outcome {
                Ok(result) => {
                    if let Some(cache) = app.try_state::<TranslationCacheState>() {
                        let key = CacheKey::new(
                            engine_id.clone(),
                            req.source_lang.clone(),
                            req.target_lang.clone(),
                            req.text.clone(),
                        );
                        cache.put(key, result.clone());
                    }
                    by_engine.insert(engine_id, EngineTranslationResult::ok(result, false));
                }
                Err(err) => {
                    by_engine.insert(
                        engine_id.clone(),
                        EngineTranslationResult::fail(engine_id, err),
                    );
                }
            }
            emit_progress(
                app,
                &text,
                &req.source_lang,
                &req.target_lang,
                &actives,
                &by_engine,
                dict_entry.clone(),
            );
        }
    }

    let payload = build_progress_payload(
        &text,
        &req.source_lang,
        &req.target_lang,
        &actives,
        &by_engine,
        dict_entry,
    );
    store_and_emit(app, payload.clone());
    Ok(payload)
}

pub fn start_translate_after_capture(app: &AppHandle, text: String) {
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let _ = run_translate(&app_handle, text, None).await;
    });
}

fn build_progress_payload(
    text: &str,
    source_lang: &str,
    target_lang: &str,
    actives: &[String],
    by_engine: &HashMap<String, EngineTranslationResult>,
    dict_entry: Option<DictEntry>,
) -> TranslationPayload {
    let results: Vec<EngineTranslationResult> = actives
        .iter()
        .map(|id| {
            by_engine
                .get(id)
                .cloned()
                .unwrap_or_else(|| EngineTranslationResult::loading(id.clone()))
        })
        .collect();
    TranslationPayload::from_engine_results(text, source_lang, target_lang, results)
        .with_dictionary(dict_entry)
}

fn emit_progress(
    app: &AppHandle,
    text: &str,
    source_lang: &str,
    target_lang: &str,
    actives: &[String],
    by_engine: &HashMap<String, EngineTranslationResult>,
    dict_entry: Option<DictEntry>,
) {
    let payload = build_progress_payload(
        text,
        source_lang,
        target_lang,
        actives,
        by_engine,
        dict_entry,
    );
    store_and_emit(app, payload);
}

fn store_and_emit(app: &AppHandle, payload: TranslationPayload) {
    if let Some(state) = app.try_state::<TranslationState>() {
        state.store(payload.clone());
    }
    let _ = app.emit("translation-updated", &payload);
}
