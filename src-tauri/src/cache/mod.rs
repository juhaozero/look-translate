//! In-memory translation LRU cache (capacity 100).

use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use std::sync::Mutex;

use lru::LruCache;

use crate::translate::TranslationResult;

pub const DEFAULT_CAPACITY: usize = 100;

/// Cache key: engine + languages + source text.
#[derive(Debug, Clone, Eq)]
pub struct CacheKey {
    pub engine: String,
    pub source_lang: String,
    pub target_lang: String,
    pub text: String,
}

impl PartialEq for CacheKey {
    fn eq(&self, other: &Self) -> bool {
        self.engine == other.engine
            && self.source_lang == other.source_lang
            && self.target_lang == other.target_lang
            && self.text == other.text
    }
}

impl Hash for CacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.engine.hash(state);
        self.source_lang.hash(state);
        self.target_lang.hash(state);
        self.text.hash(state);
    }
}

impl CacheKey {
    pub fn new(
        engine: impl Into<String>,
        source_lang: impl Into<String>,
        target_lang: impl Into<String>,
        text: impl Into<String>,
    ) -> Self {
        Self {
            engine: engine.into(),
            source_lang: source_lang.into(),
            target_lang: target_lang.into(),
            text: text.into(),
        }
    }
}

pub struct TranslationCache {
    inner: LruCache<CacheKey, TranslationResult>,
}

impl TranslationCache {
    pub fn new(capacity: usize) -> Self {
        let capacity = NonZeroUsize::new(capacity.max(1)).unwrap_or(NonZeroUsize::MIN);
        Self {
            inner: LruCache::new(capacity),
        }
    }

    pub fn get(&mut self, key: &CacheKey) -> Option<TranslationResult> {
        self.inner.get(key).cloned()
    }

    pub fn put(&mut self, key: CacheKey, value: TranslationResult) {
        self.inner.put(key, value);
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }
}

impl Default for TranslationCache {
    fn default() -> Self {
        Self::new(DEFAULT_CAPACITY)
    }
}

/// Process-wide cache state managed by Tauri.
pub struct TranslationCacheState {
    pub cache: Mutex<TranslationCache>,
}

impl TranslationCacheState {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Mutex::new(TranslationCache::new(capacity)),
        }
    }

    pub fn get(&self, key: &CacheKey) -> Option<TranslationResult> {
        self.cache
            .lock()
            .ok()
            .and_then(|mut guard| guard.get(key))
    }

    pub fn put(&self, key: CacheKey, value: TranslationResult) {
        if let Ok(mut guard) = self.cache.lock() {
            guard.put(key, value);
        }
    }

    pub fn clear(&self) {
        if let Ok(mut guard) = self.cache.lock() {
            guard.clear();
        }
    }
}

impl Default for TranslationCacheState {
    fn default() -> Self {
        Self::new(DEFAULT_CAPACITY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_result(text: &str) -> TranslationResult {
        TranslationResult {
            engine: "microsoft".into(),
            text: text.into(),
            source_lang: "auto".into(),
            target_lang: "zh-CN".into(),
            detected_source_lang: Some("en".into()),
        }
    }

    #[test]
    fn get_put_roundtrip() {
        let mut cache = TranslationCache::new(2);
        let key = CacheKey::new("microsoft", "auto", "zh-CN", "hello");
        cache.put(key.clone(), sample_result("你好"));
        assert_eq!(cache.get(&key).unwrap().text, "你好");
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn evicts_least_recently_used() {
        let mut cache = TranslationCache::new(2);
        let k1 = CacheKey::new("microsoft", "auto", "zh-CN", "one");
        let k2 = CacheKey::new("microsoft", "auto", "zh-CN", "two");
        let k3 = CacheKey::new("microsoft", "auto", "zh-CN", "three");

        cache.put(k1.clone(), sample_result("1"));
        cache.put(k2.clone(), sample_result("2"));
        // Touch k1 so k2 becomes LRU.
        assert!(cache.get(&k1).is_some());
        cache.put(k3.clone(), sample_result("3"));

        assert!(cache.get(&k1).is_some());
        assert!(cache.get(&k2).is_none());
        assert!(cache.get(&k3).is_some());
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn different_langs_are_different_keys() {
        let mut cache = TranslationCache::new(10);
        let en = CacheKey::new("microsoft", "auto", "en", "你好");
        let zh = CacheKey::new("microsoft", "auto", "zh-CN", "你好");
        cache.put(en.clone(), sample_result("hello"));
        cache.put(zh.clone(), sample_result("你好"));
        assert_eq!(cache.get(&en).unwrap().text, "hello");
        assert_eq!(cache.get(&zh).unwrap().text, "你好");
    }
}
