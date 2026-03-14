//! Lightweight Unicode-related helpers.

/// Returns whether the string has at least one cased character.
#[must_use]
pub fn has_cased_letter(value: &str) -> bool {
    value.chars().any(|ch| ch.is_uppercase() || ch.is_lowercase())
}

#[cfg(test)]
mod tests {
    use super::has_cased_letter;

    #[test]
    fn detects_cased_letters() {
        assert!(has_cased_letter("Rust"));
        assert!(!has_cased_letter("123"));
    }
}
