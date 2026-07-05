//! String normalization helpers used by the library and CLI.

/// Returns a deterministic lowercase lookup key for lexicon operations.
///
/// The dotted capital `İ` (U+0130) is mapped to ASCII `i` before lowercasing.
/// Rust's default `to_lowercase` would otherwise yield `i` followed by a
/// combining dot (U+0307), so a Turkish-cased word like `İSTANBUL` would never
/// match an `istanbul` lexicon entry.
#[must_use]
pub fn lookup_key(value: &str) -> String {
    value
        .trim()
        .chars()
        .map(|ch| if ch == '\u{0130}' { 'i' } else { ch })
        .collect::<String>()
        .to_lowercase()
}

/// Returns sorted unique strings after normalizing whitespace and lowercasing.
#[must_use]
pub fn normalized_unique_sorted<I, S>(values: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut items: Vec<String> =
        values.into_iter().map(|value| lookup_key(value.as_ref())).collect();
    items.sort();
    items.dedup();
    items
}

#[cfg(test)]
mod tests {
    use super::normalized_unique_sorted;

    #[test]
    fn normalizes_and_deduplicates() {
        assert_eq!(
            normalized_unique_sorted([" Beta ", "alpha", "ALPHA"]),
            vec!["alpha".to_string(), "beta".to_string()]
        );
    }

    #[test]
    fn maps_turkish_dotted_capital_i_to_ascii() {
        use super::lookup_key;
        // Without the pre-map this would be "i\u{0307}stanbul" and miss lookups.
        assert_eq!(lookup_key("\u{0130}STANBUL"), "istanbul");
        assert_eq!(lookup_key("istanbul"), "istanbul");
    }
}
