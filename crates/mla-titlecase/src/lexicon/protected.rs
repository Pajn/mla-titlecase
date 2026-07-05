// Mixed-case spellings that neither `capitalize_word` nor the all-caps
// initialism table would produce (`iOS`, not `IOS`; `PhD`, not `PHD`).
const PROTECTED: &[(&str, &str)] = &[
    ("ebay", "eBay"),
    ("github", "GitHub"),
    ("ios", "iOS"),
    ("ipad", "iPad"),
    ("iphone", "iPhone"),
    ("macos", "macOS"),
    ("phd", "PhD"),
    ("rust", "Rust"),
];

pub(crate) fn canonical(word: &str) -> Option<&'static str> {
    PROTECTED.iter().find(|(key, _)| *key == word).map(|(_, value)| *value)
}

#[cfg(test)]
mod tests {
    use super::{canonical, PROTECTED};

    #[test]
    fn returns_known_protected_spellings() {
        assert_eq!(canonical("github"), Some("GitHub"));
        assert_eq!(canonical("ios"), Some("iOS"));
        assert_eq!(canonical("unknown"), None);
    }

    #[test]
    fn keys_are_lowercase_with_non_empty_values() {
        for (key, value) in PROTECTED {
            assert_eq!(*key, key.to_lowercase(), "protected key must be lowercase: {key}");
            assert!(!value.is_empty(), "protected value must be non-empty: {key}");
        }
    }

    #[test]
    fn keys_are_unique_and_sorted() {
        let keys: Vec<&str> = PROTECTED.iter().map(|(key, _)| *key).collect();
        let mut sorted = keys.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(keys, sorted, "protected keys must be unique and sorted");
    }
}
