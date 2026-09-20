//! Parallel progressive translation orchestration (domain layer).
//!
//! Keeps cache lookup, multi-engine fan-out, and progressive payloads out of
//! the Tauri command adapter.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use futures::stream::{FuturesUnordered, StreamExt};

use crate::cache::{CacheKey, TranslationCacheState};
use crate::config::AppConfig;
use crate::dictionary::DictEntry;

use super::{
    translate_with_engine, EngineTranslationResult, TranslationPayload, TranslationRequest,
    TranslationResult,
};

/// Seam for running one engine; production uses [`BuiltinEngineRunner`].
#[async_trait]
pub trait EngineRunner: Send + Sync {
    async fn translate_engine(
        &self,
        config: &AppConfig,
        engine_id: &str,
        req: TranslationRequest,
    ) -> Result<TranslationResult, String>;
}

/// Dispatches to builtin / config-driven engines via [`translate_with_engine`].
pub struct BuiltinEngineRunner;

#[async_trait]
impl EngineRunner for BuiltinEngineRunner {
    async fn translate_engine(
        &self,
        config: &AppConfig,
        engine_id: &str,
        req: TranslationRequest,
    ) -> Result<TranslationResult, String> {
        translate_with_engine(config, engine_id, req).await
    }
}

/// Run all `engine.actives` in parallel with cache-first progressive updates.
///
/// `on_progress` is invoked for: initial loading, cache-hit snapshot (if any),
/// each completed engine, and the final payload.
pub async fn run_parallel(
    config: &AppConfig,
    req: TranslationRequest,
    cache: Option<&TranslationCacheState>,
    dict_entry: Option<DictEntry>,
    runner: Arc<dyn EngineRunner>,
    mut on_progress: impl FnMut(TranslationPayload),
) -> TranslationPayload {
    let actives = config.engine.resolved_actives();

    let loading = TranslationPayload::loading(
        &req.text,
        &req.source_lang,
        &req.target_lang,
        &actives,
    )
    .with_dictionary(dict_entry.clone());
    on_progress(loading);

    let mut by_engine: HashMap<String, EngineTranslationResult> = HashMap::new();
    let mut pending: Vec<String> = Vec::new();

    for engine_id in &actives {
        let key = CacheKey::new(
            engine_id.clone(),
            req.source_lang.clone(),
            req.target_lang.clone(),
            req.text.clone(),
        );
        if let Some(cache) = cache {
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
        on_progress(build_progress_payload(
            &req.text,
            &req.source_lang,
            &req.target_lang,
            &actives,
            &by_engine,
            dict_entry.clone(),
        ));
    }

    if !pending.is_empty() {
        let mut tasks = FuturesUnordered::new();
        for engine_id in pending {
            let config = config.clone();
            let req = req.clone();
            let runner = Arc::clone(&runner);
            tasks.push(async move {
                let outcome = runner
                    .translate_engine(&config, &engine_id, req)
                    .await;
                (engine_id, outcome)
            });
        }

        while let Some((engine_id, outcome)) = tasks.next().await {
            match outcome {
                Ok(result) => {
                    if let Some(cache) = cache {
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
            on_progress(build_progress_payload(
                &req.text,
                &req.source_lang,
                &req.target_lang,
                &actives,
                &by_engine,
                dict_entry.clone(),
            ));
        }
    }

    let payload = build_progress_payload(
        &req.text,
        &req.source_lang,
        &req.target_lang,
        &actives,
        &by_engine,
        dict_entry,
    );
    on_progress(payload.clone());
    payload
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use std::sync::Mutex;

    struct FakeRunner {
        map: HashMap<String, Result<TranslationResult, String>>,
    }

    #[async_trait]
    impl EngineRunner for FakeRunner {
        async fn translate_engine(
            &self,
            _config: &AppConfig,
            engine_id: &str,
            _req: TranslationRequest,
        ) -> Result<TranslationResult, String> {
            self.map
                .get(engine_id)
                .cloned()
                .unwrap_or_else(|| Err(format!("unknown fake engine `{engine_id}`")))
        }
    }

    fn sample_ok(engine: &str, text: &str) -> TranslationResult {
        TranslationResult {
            engine: engine.into(),
            text: text.into(),
            source_lang: "auto".into(),
            target_lang: "zh-CN".into(),
            detected_source_lang: Some("en".into()),
        }
    }

    fn config_with_actives(actives: &[&str]) -> AppConfig {
        let mut config = AppConfig::default();
        config.engine.actives = actives.iter().map(|s| (*s).to_string()).collect();
        if let Some(first) = actives.first() {
            config.engine.active = (*first).to_string();
        }
        config
    }

    fn req(text: &str) -> TranslationRequest {
        TranslationRequest {
            text: text.into(),
            source_lang: "auto".into(),
            target_lang: "zh-CN".into(),
        }
    }

    #[test]
    fn cache_hits_emit_before_network() {
        tauri::async_runtime::block_on(async {
            let config = config_with_actives(&["fast", "slow"]);
            let cache = TranslationCacheState::new(10);
            cache.put(
                CacheKey::new("fast", "auto", "zh-CN", "hi"),
                sample_ok("fast", "快"),
            );

            let runner: Arc<dyn EngineRunner> = Arc::new(FakeRunner {
                map: HashMap::from([("slow".into(), Ok(sample_ok("slow", "慢")))]),
            });

            let progress = Arc::new(Mutex::new(Vec::new()));
            let progress_cb = progress.clone();
            let final_payload = run_parallel(
                &config,
                req("hi"),
                Some(&cache),
                None,
                runner,
                |p| progress_cb.lock().unwrap().push(p),
            )
            .await;

            let snaps = progress.lock().unwrap();
            assert!(snaps.len() >= 3);
            assert_eq!(snaps[0].status, "loading");
            let cache_snap = snaps
                .iter()
                .find(|p| {
                    p.results.iter().any(|r| r.engine == "fast" && r.cached)
                        && p.results
                            .iter()
                            .any(|r| r.engine == "slow" && r.status == "loading")
                })
                .expect("cache progressive snapshot");
            assert_eq!(cache_snap.status, "loading");
            assert_eq!(final_payload.status, "ok");
            assert_eq!(final_payload.results.len(), 2);
            assert!(final_payload.results.iter().all(|r| r.status == "ok"));
        });
    }

    #[test]
    fn partial_failure_keeps_ok_overall() {
        tauri::async_runtime::block_on(async {
            let config = config_with_actives(&["ok_eng", "bad_eng"]);
            let runner: Arc<dyn EngineRunner> = Arc::new(FakeRunner {
                map: HashMap::from([
                    ("ok_eng".into(), Ok(sample_ok("ok_eng", "好"))),
                    ("bad_eng".into(), Err("boom".into())),
                ]),
            });

            let final_payload =
                run_parallel(&config, req("hi"), None, None, runner, |_| {}).await;

            assert_eq!(final_payload.status, "ok");
            assert_eq!(final_payload.translated_text.as_deref(), Some("好"));
            let bad = final_payload
                .results
                .iter()
                .find(|r| r.engine == "bad_eng")
                .unwrap();
            assert_eq!(bad.status, "error");
        });
    }

    #[test]
    fn all_failed_status_error() {
        tauri::async_runtime::block_on(async {
            let config = config_with_actives(&["a", "b"]);
            let runner: Arc<dyn EngineRunner> = Arc::new(FakeRunner {
                map: HashMap::from([
                    ("a".into(), Err("a fail".into())),
                    ("b".into(), Err("b fail".into())),
                ]),
            });

            let final_payload =
                run_parallel(&config, req("hi"), None, None, runner, |_| {}).await;

            assert_eq!(final_payload.status, "error");
            assert!(final_payload.translated_text.is_none());
        });
    }

    #[test]
    fn successful_results_are_written_to_cache() {
        tauri::async_runtime::block_on(async {
            let config = config_with_actives(&["only"]);
            let cache = TranslationCacheState::new(10);
            let runner: Arc<dyn EngineRunner> = Arc::new(FakeRunner {
                map: HashMap::from([("only".into(), Ok(sample_ok("only", "只")))]),
            });

            let _ = run_parallel(&config, req("hi"), Some(&cache), None, runner, |_| {}).await;

            let hit = cache.get(&CacheKey::new("only", "auto", "zh-CN", "hi"));
            assert_eq!(hit.unwrap().text, "只");
        });
    }
}
