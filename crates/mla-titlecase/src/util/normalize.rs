//! String normalization helpers used by the library and CLI.

/// Returns a deterministic lowercase lookup key for lexicon operations.
#[must_use]
pub fn lookup_key(value: &str) -> String {
    value.trim().to_lowercase()
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
}
