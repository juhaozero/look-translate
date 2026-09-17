//! MDict (.mdx) provider via mdict-rs.

use std::path::Path;

use mdict_rs::MdxFile;

use super::html::html_to_plain;
use super::{display_name, DictEntry, DictionaryProvider};

pub struct MdictProvider {
    path: std::path::PathBuf,
    file: MdxFile,
}

impl MdictProvider {
    pub fn open(path: &Path) -> Result<Self, String> {
        let file = MdxFile::open(path).map_err(|e| format!("打开 MDX 失败 ({}): {e}", path.display()))?;
        Ok(Self {
            path: path.to_path_buf(),
            file,
        })
    }

    fn lookup_raw(&self, key: &str) -> Result<Option<String>, String> {
        match self.file.lookup(key) {
            Ok(Some(record)) => Ok(Some(html_to_plain(&record.text))),
            Ok(None) => Ok(None),
            Err(err) => Err(format!("查词失败 (`{key}`): {err}")),
        }
    }
}

impl DictionaryProvider for MdictProvider {
    fn lookup(&self, word: &str) -> Result<Option<DictEntry>, String> {
        let trimmed = word.trim();
        if trimmed.is_empty() {
            return Ok(None);
        }

        let candidates = lookup_candidates(trimmed);
        for key in candidates {
            if let Some(text) = self.lookup_raw(&key)? {
                if text.is_empty() {
                    continue;
                }
                return Ok(Some(DictEntry {
                    text,
                    source: display_name(&self.path),
                    path: self.path.to_string_lossy().into_owned(),
                }));
            }
        }
        Ok(None)
    }
}

fn lookup_candidates(word: &str) -> Vec<String> {
    let mut keys = Vec::new();
    keys.push(word.to_string());
    let lower = word.to_lowercase();
    if lower != word {
        keys.push(lower);
    }
    let stripped: String = word
        .trim_matches(|c: char| !c.is_alphanumeric() && c != '\'' && c != '-')
        .to_string();
    if !stripped.is_empty() && stripped != word {
        keys.push(stripped.clone());
        let stripped_lower = stripped.to_lowercase();
        if stripped_lower != stripped {
            keys.push(stripped_lower);
        }
    }
    keys.dedup();
    keys
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidates_include_lower_and_stripped() {
        let keys = lookup_candidates("Hello,");
        assert!(keys.iter().any(|k| k == "Hello,"));
        assert!(keys.iter().any(|k| k == "hello,"));
        assert!(keys.iter().any(|k| k == "Hello"));
        assert!(keys.iter().any(|k| k == "hello"));
    }
}
