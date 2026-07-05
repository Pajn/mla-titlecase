//! String normalization helpers used by the library and CLI.

use std::borrow::Cow;

/// Returns a deterministic lowercase lookup key for lexicon operations.
///
/// The dotted capital `İ` (U+0130) is mapped to ASCII `i` before lowercasing.
/// Rust's default `to_lowercase` would otherwise yield `i` followed by a
/// combining dot (U+0307), so a Turkish-cased word like `İSTANBUL` would never
/// match an `istanbul` lexicon entry.
#[must_use]
pub fn lookup_key(value: &str) -> String {
    normalized_key(value).into_owned()
}

/// Borrowing variant of [`lookup_key`] for hot paths. Returns the trimmed input
/// borrowed (no allocation) only when every byte is already clean lowercase
/// ASCII — the common case for tokenized words. Any uppercase ASCII or any
/// non-ASCII byte takes the owned path and allocates, even when normalization
/// leaves the text unchanged (for example an already-lowercase `café`).
#[must_use]
pub(crate) fn normalized_key(value: &str) -> Cow<'_, str> {
    let trimmed = value.trim();
    if trimmed.bytes().all(|byte| byte.is_ascii() && !byte.is_ascii_uppercase()) {
        return Cow::Borrowed(trimmed);
    }

    let mut result = String::with_capacity(trimmed.len());
    for ch in trimmed.chars() {
        if ch == '\u{0130}' {
            result.push('i');
        } else {
            result.extend(ch.to_lowercase());
        }
    }
    Cow::Owned(result)
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

    #[test]
    fn normalized_key_borrows_only_clean_ascii() {
        use super::normalized_key;
        use std::borrow::Cow;

        // Clean lowercase ASCII borrows without allocating.
        assert!(matches!(normalized_key("wind"), Cow::Borrowed("wind")));
        // Uppercase ASCII takes the owned path.
        assert!(matches!(normalized_key("Wind"), Cow::Owned(_)));
        assert_eq!(normalized_key("Wind"), "wind");
        // Non-ASCII takes the owned path even when normalization changes nothing.
        assert!(matches!(normalized_key("café"), Cow::Owned(ref value) if value == "café"));
    }
}
