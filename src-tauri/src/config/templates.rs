//! Built-in Engine Profile templates users can insert from settings.

use std::collections::BTreeMap;

use super::EngineProfile;

/// Default id for the generic HTTP skeleton inserted from settings.
pub const CUSTOM_PROFILE_ID: &str = "custom";

/// Neutral REST skeleton — fill `url` / auth / paths for your vendor.
/// Concrete vendor samples (e.g. DeepL) live in `docs/engine-profiles.md`.
pub fn custom_http_profile() -> EngineProfile {
    let mut body = BTreeMap::new();
    body.insert("text".into(), "{{text}}".into());
    body.insert("source_lang".into(), "{{source_lang}}".into());
    body.insert("target_lang".into(), "{{target_lang}}".into());

    let mut extra = BTreeMap::new();
    extra.insert("api_key".into(), "your-api-key".into());

    let mut lang_map = BTreeMap::new();
    for (from, to) in [
        ("zh-CN", "zh-CN"),
        ("zh-TW", "zh-TW"),
        ("en", "en"),
        ("ja", "ja"),
        ("ko", "ko"),
        ("fr", "fr"),
        ("de", "de"),
        ("es", "es"),
        ("auto", "auto"),
    ] {
        lang_map.insert(from.into(), to.into());
    }

    EngineProfile {
        label: Some("自定义 HTTP".into()),
        method: "POST".into(),
        url: "https://example.com/v1/translate".into(),
        auth: "bearer".into(),
        token: Some("{{extra.api_key}}".into()),
        body_type: "json".into(),
        body,
        extra,
        lang_map,
        text_path: "data.text".into(),
        error_path: Some("error.message".into()),
        ..EngineProfile::default()
    }
}
