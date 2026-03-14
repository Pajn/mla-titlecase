use super::{abbreviations, particles, protected, small_words};

pub(crate) fn is_small_word(word: &str) -> bool {
    small_words::contains(word)
}

pub(crate) fn abbreviation_spelling(word: &str) -> Option<&'static str> {
    abbreviations::canonical(word)
}

pub(crate) fn built_in_protected_spelling(word: &str) -> Option<&'static str> {
    protected::canonical(word)
}

pub(crate) fn is_name_particle(word: &str) -> bool {
    particles::contains(word)
}
