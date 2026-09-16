//! Translator trait and engine implementations.

#![allow(dead_code)]

pub trait Translator: Send + Sync {
    fn id(&self) -> &str;
}
