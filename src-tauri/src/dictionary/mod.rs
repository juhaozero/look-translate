//! Offline dictionary providers (MDict in MVP).

#![allow(dead_code)]

pub trait DictionaryProvider: Send + Sync {
    fn lookup(&self, word: &str) -> Result<Option<String>, String>;
}
