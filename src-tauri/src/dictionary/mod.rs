//! Offline dictionary providers (MDict in MVP).

mod html;
mod mdict;
mod short_word;

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::Serialize;

use crate::config::AppConfig;

pub use short_word::is_short_word;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DictEntry {
    pub text: String,
    pub source: String,
    pub path: String,
}

pub trait DictionaryProvider: Send + Sync {
    fn lookup(&self, word: &str) -> Result<Option<DictEntry>, String>;
}

/// Resolve config path to an `.mdx` file (file path or directory containing .mdx).
pub fn resolve_mdx_path(raw: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(raw.trim());
    if !path.exists() {
        return Err(format!("词典路径不存在: {}", path.display()));
    }
    if path.is_file() {
        if path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("mdx"))
        {
            return Ok(path);
        }
        return Err(format!("不是 .mdx 文件: {}", path.display()));
    }
    if path.is_dir() {
        let mut files: Vec<PathBuf> = std::fs::read_dir(&path)
            .map_err(|e| format!("读取词典目录失败: {e}"))?
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|p| {
                p.extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|e| e.eq_ignore_ascii_case("mdx"))
            })
            .collect();
        files.sort();
        return files.into_iter().next().ok_or_else(|| {
            format!("目录中未找到 .mdx 文件: {}", path.display())
        });
    }
    Err(format!("无效的词典路径: {}", path.display()))
}

pub fn display_name(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("dictionary")
        .to_string()
}

struct OpenDict {
    path: PathBuf,
    provider: mdict::MdictProvider,
}

/// Keeps the first configured MDX open for repeated lookups.
pub struct DictionaryState {
    open: Mutex<Option<OpenDict>>,
}

impl Default for DictionaryState {
    fn default() -> Self {
        Self {
            open: Mutex::new(None),
        }
    }
}

impl DictionaryState {
    pub fn invalidate(&self) {
        if let Ok(mut guard) = self.open.lock() {
            *guard = None;
        }
    }

    pub fn lookup_for_config(
        &self,
        config: &AppConfig,
        word: &str,
    ) -> Result<Option<DictEntry>, String> {
        if !config.dictionary.enabled {
            return Ok(None);
        }
        if !is_short_word(word) {
            return Ok(None);
        }
        let Some(raw_path) = config.dictionary.paths.first() else {
            return Ok(None);
        };
        if raw_path.trim().is_empty() {
            return Ok(None);
        }

        let mdx_path = resolve_mdx_path(raw_path)?;
        let mut guard = self
            .open
            .lock()
            .map_err(|_| "dictionary lock poisoned".to_string())?;

        let needs_open = guard
            .as_ref()
            .map(|d| d.path != mdx_path)
            .unwrap_or(true);
        if needs_open {
            let provider = mdict::MdictProvider::open(&mdx_path)?;
            *guard = Some(OpenDict {
                path: mdx_path.clone(),
                provider,
            });
        }

        let open = guard
            .as_ref()
            .ok_or_else(|| "dictionary not loaded".to_string())?;
        open.provider.lookup(word)
    }
}

/// Sync helper for translate pipeline (may open MDX on first use).
pub fn lookup_short_word(
    state: &DictionaryState,
    config: &AppConfig,
    word: &str,
) -> Option<DictEntry> {
    match state.lookup_for_config(config, word) {
        Ok(entry) => entry,
        Err(err) => {
            eprintln!("[look-translate] dictionary lookup failed: {err}");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn resolve_rejects_missing() {
        assert!(resolve_mdx_path("C:/definitely-missing-dict-xyz.mdx").is_err());
    }

    #[test]
    fn resolve_directory_picks_mdx() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("look-translate-dict-{nanos}"));
        fs::create_dir_all(&dir).unwrap();
        let mdx = dir.join("demo.mdx");
        fs::write(&mdx, b"not-a-real-mdx").unwrap();
        let resolved = resolve_mdx_path(dir.to_str().unwrap()).unwrap();
        assert_eq!(resolved, mdx);
        let _ = fs::remove_dir_all(&dir);
    }
}
