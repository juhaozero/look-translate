//! Portable config under install-dir `data/`.

#![allow(dead_code)]

pub struct AppConfig;

impl AppConfig {
    pub fn default_path_hint() -> &'static str {
        "data/config.toml"
    }
}
