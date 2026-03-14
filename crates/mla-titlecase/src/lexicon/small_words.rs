const SMALL_WORDS: &[&str] = &[
    "a", "an", "and", "as", "at", "but", "by", "down", "for", "from", "if", "in", "into", "like",
    "near", "nor", "of", "off", "on", "once", "onto", "or", "over", "past", "per", "so", "than",
    "that", "the", "to", "up", "upon", "via", "with", "yet",
];

pub(crate) fn contains(word: &str) -> bool {
    SMALL_WORDS.contains(&word)
}

#[cfg(test)]
mod tests {
    use super::contains;

    #[test]
    fn includes_common_small_words() {
        assert!(contains("and"));
        assert!(contains("the"));
        assert!(!contains("wind"));
    }
}
