//! Canonical language codes are Google-style (`zh-CN`, `zh-TW`, …).

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
