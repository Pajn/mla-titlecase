const PROTECTED: &[(&str, &str)] = &[
    ("ebay", "eBay"),
    ("github", "GitHub"),
    ("iphone", "iPhone"),
    ("ipad", "iPad"),
    ("macos", "macOS"),
    ("rust", "Rust"),
];

pub(crate) fn canonical(word: &str) -> Option<&'static str> {
    PROTECTED.iter().find(|(key, _)| *key == word).map(|(_, value)| *value)
}
