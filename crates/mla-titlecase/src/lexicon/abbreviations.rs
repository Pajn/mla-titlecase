// Every key must be a lowercase initialism that is never an ordinary English
// word: this table applies to all input, not just all-caps titles, so a key
// like `us` or `led` would corrupt normal titles ("Between Us" -> "Between US").
// The `keys_are_normalized_and_never_small_words` test guards the small-word
// half of that invariant mechanically; the common-word half is curation.
const ABBREVIATIONS: &[(&str, &str)] = &[
    ("api", "API"),
    ("ascii", "ASCII"),
    ("atm", "ATM"),
    ("bbc", "BBC"),
    ("ceo", "CEO"),
    ("cfo", "CFO"),
    ("cia", "CIA"),
    ("cnn", "CNN"),
    ("cpu", "CPU"),
    ("css", "CSS"),
    ("cto", "CTO"),
    ("diy", "DIY"),
    ("dna", "DNA"),
    ("dns", "DNS"),
    ("dvd", "DVD"),
    ("eu", "EU"),
    ("faq", "FAQ"),
    ("fbi", "FBI"),
    ("fifa", "FIFA"),
    ("ftp", "FTP"),
    ("gdp", "GDP"),
    ("gif", "GIF"),
    ("gps", "GPS"),
    ("gpu", "GPU"),
    ("hiv", "HIV"),
    ("html", "HTML"),
    ("http", "HTTP"),
    ("https", "HTTPS"),
    ("ibm", "IBM"),
    ("ip", "IP"),
    ("jpeg", "JPEG"),
    ("json", "JSON"),
    ("mla", "MLA"),
    ("mlb", "MLB"),
    ("nasa", "NASA"),
    ("nato", "NATO"),
    ("nba", "NBA"),
    ("nfl", "NFL"),
    ("nhl", "NHL"),
    ("pdf", "PDF"),
    ("png", "PNG"),
    ("rss", "RSS"),
    ("sdk", "SDK"),
    ("sql", "SQL"),
    ("svg", "SVG"),
    ("tv", "TV"),
    ("uk", "UK"),
    ("url", "URL"),
    ("usa", "USA"),
    ("usb", "USB"),
    ("ux", "UX"),
    ("xml", "XML"),
];

pub(crate) fn canonical(word: &str) -> Option<&'static str> {
    // ABBREVIATIONS is sorted and unique (asserted by `keys_are_unique_and_sorted`),
    // so a binary search beats a linear scan on this per-word lookup.
    ABBREVIATIONS
        .binary_search_by(|(key, _)| (*key).cmp(word))
        .ok()
        .map(|index| ABBREVIATIONS[index].1)
}

#[cfg(test)]
mod tests {
    use super::canonical;

    #[test]
    fn returns_known_abbreviations() {
        assert_eq!(canonical("mla"), Some("MLA"));
        assert_eq!(canonical("rust"), None);
    }

    #[test]
    fn keys_are_normalized_and_never_small_words() {
        for (key, value) in super::ABBREVIATIONS {
            assert_eq!(*key, key.to_lowercase(), "abbreviation key must be lowercase: {key}");
            assert!(
                !super::super::small_words::contains(key),
                "abbreviation key collides with a small word: {key}"
            );
            assert!(!value.is_empty());
        }
    }

    #[test]
    fn keys_are_unique_and_sorted() {
        let keys: Vec<&str> = super::ABBREVIATIONS.iter().map(|(key, _)| *key).collect();
        let mut sorted = keys.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(keys, sorted, "abbreviation keys must be unique and sorted");
    }
}
