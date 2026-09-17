//! Short-word rule for dictionary lookup.

/// ≤30 chars AND ≤3 whitespace tokens, and no newlines.
pub fn is_short_word(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return false;
    }
    if trimmed.contains('\n') || trimmed.contains('\r') {
        return false;
    }
    if trimmed.chars().count() > 30 {
        return false;
    }
    let tokens = trimmed.split_whitespace().filter(|t| !t.is_empty()).count();
    (1..=3).contains(&tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_words_and_short_phrases() {
        assert!(is_short_word("horizon"));
        assert!(is_short_word("look up"));
        assert!(is_short_word("machine learning"));
        assert!(is_short_word("  hello  "));
    }

    #[test]
    fn rejects_long_or_multiline() {
        assert!(!is_short_word(""));
        assert!(!is_short_word("one two three four"));
        assert!(!is_short_word("a\nb"));
        assert!(!is_short_word(&"x".repeat(31)));
        assert!(!is_short_word(
            "This is a full sentence that should not hit the dictionary."
        ));
    }
}
