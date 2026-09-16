use std::sync::RwLock;

use crate::translate::TranslationPayload;

#[derive(Debug, Default)]
pub struct TranslationState {
    pub last: RwLock<Option<TranslationPayload>>,
}

impl TranslationState {
    pub fn store(&self, payload: TranslationPayload) {
        if let Ok(mut guard) = self.last.write() {
            *guard = Some(payload);
        }
    }

    pub fn latest(&self) -> Option<TranslationPayload> {
        self.last.read().ok().and_then(|g| g.clone())
    }
}
