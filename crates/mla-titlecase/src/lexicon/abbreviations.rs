const ABBREVIATIONS: &[(&str, &str)] = &[
    ("api", "API"),
    ("cpu", "CPU"),
    ("dna", "DNA"),
    ("eu", "EU"),
    ("html", "HTML"),
    ("mla", "MLA"),
    ("nasa", "NASA"),
    ("sql", "SQL"),
    ("uk", "UK"),
    ("usa", "USA"),
    ("ux", "UX"),
];

pub(crate) fn canonical(word: &str) -> Option<&'static str> {
    ABBREVIATIONS.iter().find(|(key, _)| *key == word).map(|(_, value)| *value)
}

#[cfg(test)]
mod tests {
    use super::canonical;

    #[test]
    fn returns_known_abbreviations() {
        assert_eq!(canonical("mla"), Some("MLA"));
        assert_eq!(canonical("rust"), None);
    }
}
