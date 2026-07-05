/// Words MLA lowercases mid-title: articles, coordinating conjunctions, and
/// prepositions of any length. Subordinating conjunctions (`if`, `that`,
/// `than`, `once`, `because`) are capitalized in MLA and deliberately absent.
/// Words that double as prepositions and subordinating conjunctions (`after`,
/// `before`, `since`, `till`, `until`) are kept here as prepositions, the more
/// common reading in titles.
const SMALL_WORDS: &[&str] = &[
    "a",
    "about",
    "above",
    "across",
    "after",
    "against",
    "along",
    "amid",
    "among",
    "an",
    "and",
    "around",
    "as",
    "at",
    "before",
    "behind",
    "below",
    "beneath",
    "beside",
    "besides",
    "between",
    "beyond",
    "but",
    "by",
    "despite",
    "down",
    "during",
    "except",
    "for",
    "from",
    "in",
    "inside",
    "into",
    "like",
    "near",
    "nor",
    "of",
    "off",
    "on",
    "onto",
    "or",
    "outside",
    "over",
    "past",
    "per",
    "since",
    "so",
    "the",
    "through",
    "throughout",
    "till",
    "to",
    "toward",
    "towards",
    "under",
    "underneath",
    "until",
    "unto",
    "up",
    "upon",
    "via",
    "with",
    "within",
    "without",
    "yet",
];

pub(crate) fn contains(word: &str) -> bool {
    // SMALL_WORDS is kept sorted (asserted by `list_is_sorted`), so a binary
    // search beats a linear scan on this hot path.
    SMALL_WORDS.binary_search(&word).is_ok()
}

#[cfg(test)]
mod tests {
    use super::contains;

    #[test]
    fn includes_articles_conjunctions_and_prepositions() {
        assert!(contains("and"));
        assert!(contains("the"));
        assert!(contains("among"));
        assert!(contains("throughout"));
        assert!(!contains("wind"));
    }

    #[test]
    fn includes_dual_role_prepositions() {
        // Documented choice: preposition/conjunction words stay lowercased.
        assert!(contains("after"));
        assert!(contains("before"));
        assert!(contains("since"));
        assert!(contains("till"));
        assert!(contains("until"));
    }

    #[test]
    fn excludes_subordinating_conjunctions() {
        assert!(!contains("if"));
        assert!(!contains("that"));
        assert!(!contains("than"));
        assert!(!contains("once"));
        assert!(!contains("because"));
    }

    #[test]
    fn list_is_sorted() {
        assert!(
            super::SMALL_WORDS.windows(2).all(|pair| pair[0] < pair[1]),
            "SMALL_WORDS must stay sorted for binary_search"
        );
    }
}
