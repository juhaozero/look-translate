//! Canonical language codes are Google-style (`zh-CN`, `zh-TW`, …).

/// Detect other Latin languages before applying script defaults to short selections.
/// Very short words remain ambiguous, so the popup always allows overrides.
pub fn automatic_target(text: &str) -> Option<&'static str> {
    if let Some(info) = whatlang::detect(text) {
        // Statistical detection needs enough letters; short English selections
        // such as "How are you" otherwise often look like another Latin language.
        if (info.is_reliable()
            || (info.confidence() >= 0.2
                && text.chars().filter(|c| c.is_alphabetic()).count() >= 20))
            && info.lang() != whatlang::Lang::Eng
            && info.lang() != whatlang::Lang::Cmn
        {
            return None;
        }
    }
    let mut han = false;
    let mut latin = false;
    for c in text.chars().filter(|c| c.is_alphabetic()) {
        match c {
            '\u{3400}'..='\u{4dbf}'
            | '\u{4e00}'..='\u{9fff}'
            | '\u{f900}'..='\u{faff}'
            | '\u{20000}'..='\u{323af}' => han = true,
            'a'..='z' | 'A'..='Z' => latin = true,
            _ => return None,
        }
    }
    if han {
        Some("en")
    } else if latin {
        Some("zh-CN")
    } else {
        None
    }
}

/// Normalize a language tag to the app's Google-style codes.
pub fn normalize_lang_code(code: &str) -> String {
    let trimmed = code.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.eq_ignore_ascii_case("auto") {
        return "auto".into();
    }
    match trimmed {
        "zh-Hans" | "zh-CN" | "zh" => "zh-CN".into(),
        "zh-Hant" | "zh-TW" => "zh-TW".into(),
        other => other.to_string(),
    }
}

/// Map app (Google-style) codes to Microsoft Translator codes.
pub fn to_microsoft_lang(code: &str) -> String {
    match normalize_lang_code(code).as_str() {
        "auto" => "auto".into(),
        "zh-CN" => "zh-Hans".into(),
        "zh-TW" => "zh-Hant".into(),
        other => other.to_string(),
    }
}

/// Map Microsoft codes back to app (Google-style) codes.
pub fn from_microsoft_lang(code: &str) -> String {
    match code.trim() {
        "zh-Hans" => "zh-CN".into(),
        "zh-Hant" => "zh-TW".into(),
        other => other.to_string(),
    }
}

/// Map app codes to Baidu Translate API codes.
pub fn to_baidu_lang(code: &str) -> String {
    match normalize_lang_code(code).as_str() {
        "auto" => "auto".into(),
        "zh-CN" => "zh".into(),
        "zh-TW" => "cht".into(),
        "ja" => "jp".into(),
        "ko" => "kor".into(),
        "fr" => "fra".into(),
        "es" => "spa".into(),
        other => other.to_string(),
    }
}

/// Map Baidu codes back to app (Google-style) codes.
pub fn from_baidu_lang(code: &str) -> String {
    match code.trim() {
        "zh" => "zh-CN".into(),
        "cht" => "zh-TW".into(),
        "jp" => "ja".into(),
        "kor" => "ko".into(),
        "fra" => "fr".into(),
        "spa" => "es".into(),
        other => other.to_string(),
    }
}

/// Map app codes to Youdao Translate API codes.
pub fn to_youdao_lang(code: &str) -> String {
    match normalize_lang_code(code).as_str() {
        "auto" => "auto".into(),
        "zh-CN" => "zh-CHS".into(),
        "zh-TW" => "zh-CHT".into(),
        other => other.to_string(),
    }
}

/// Map Youdao codes back to app (Google-style) codes.
pub fn from_youdao_lang(code: &str) -> String {
    match code.trim() {
        "zh-CHS" | "zh-CN" => "zh-CN".into(),
        "zh-CHT" | "zh-TW" => "zh-TW".into(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn automatic_direction_handles_scripts_and_empty_text() {
        assert_eq!(automatic_target("Hello, world! 123"), Some("zh-CN"));
        for text in [
            "This is a test",
            "How are you",
            "good morning",
            "Translate this text into Chinese.",
        ] {
            assert_eq!(
                automatic_target(text),
                Some("zh-CN"),
                "{text}: {:?}",
                whatlang::detect(text)
            );
        }
        assert_eq!(automatic_target("你好，世界！"), Some("en"));
        assert_eq!(automatic_target("繁體中文與 API"), Some("en"));
        for text in [
            "日本語です",
            "한국어",
            "Привет",
            "français",
            "Bonjour, je voudrais traduire cette phrase en anglais.",
            "123 !",
            "",
        ] {
            assert_eq!(automatic_target(text), None, "{text}");
        }
    }

    #[test]
    fn normalizes_legacy_chinese() {
        assert_eq!(normalize_lang_code("zh-Hans"), "zh-CN");
        assert_eq!(normalize_lang_code("zh-Hant"), "zh-TW");
        assert_eq!(normalize_lang_code("zh-CN"), "zh-CN");
        assert_eq!(normalize_lang_code("auto"), "auto");
    }

    #[test]
    fn microsoft_roundtrip() {
        assert_eq!(to_microsoft_lang("zh-CN"), "zh-Hans");
        assert_eq!(to_microsoft_lang("zh-TW"), "zh-Hant");
        assert_eq!(from_microsoft_lang("zh-Hans"), "zh-CN");
        assert_eq!(from_microsoft_lang("zh-Hant"), "zh-TW");
    }

    #[test]
    fn baidu_roundtrip() {
        assert_eq!(to_baidu_lang("zh-CN"), "zh");
        assert_eq!(to_baidu_lang("zh-TW"), "cht");
        assert_eq!(to_baidu_lang("ja"), "jp");
        assert_eq!(from_baidu_lang("cht"), "zh-TW");
        assert_eq!(from_baidu_lang("jp"), "ja");
    }

    #[test]
    fn youdao_roundtrip() {
        assert_eq!(to_youdao_lang("zh-CN"), "zh-CHS");
        assert_eq!(to_youdao_lang("zh-TW"), "zh-CHT");
        assert_eq!(from_youdao_lang("zh-CHS"), "zh-CN");
        assert_eq!(from_youdao_lang("zh-CHT"), "zh-TW");
    }
}
