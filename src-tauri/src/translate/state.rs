use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;

use crate::translate::TranslationPayload;

#[derive(Debug, Default)]
pub struct TranslationState {
    pub last: RwLock<Option<TranslationPayload>>,
    generation: AtomicU64,
}

impl TranslationState {
    pub fn begin(&self) -> u64 {
        self.generation.fetch_add(1, Ordering::SeqCst) + 1
    }

    pub fn is_current(&self, generation: u64) -> bool {
        self.generation.load(Ordering::SeqCst) == generation
    }

    pub fn store(&self, payload: TranslationPayload) {
        if let Ok(mut guard) = self.last.write() {
            *guard = Some(payload);
        }
    }

    pub fn clear(&self) {
        if let Ok(mut guard) = self.last.write() {
            *guard = None;
        }
    }

    pub fn latest(&self) -> Option<TranslationPayload> {
        self.last.read().ok().and_then(|g| g.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newer_session_invalidates_previous_translation() {
        let state = TranslationState::default();
        let first = state.begin();
        assert!(state.is_current(first));
        let second = state.begin();
        assert!(!state.is_current(first));
        assert!(state.is_current(second));
    }
}
