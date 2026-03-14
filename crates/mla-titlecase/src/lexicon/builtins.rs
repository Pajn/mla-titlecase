use super::{abbreviations, particles, protected, small_words};
use crate::config::LocaleProfile;

pub(crate) fn is_small_word(word: &str) -> bool {
    small_words::contains(word)
}

pub(crate) fn abbreviation_spelling(word: &str) -> Option<&'static str> {
    abbreviations::canonical(word)
}

pub(crate) fn built_in_protected_spelling(word: &str) -> Option<&'static str> {
    protected::canonical(word)
}

pub(crate) fn is_name_particle_for_locale(word: &str, locale: LocaleProfile) -> bool {
    particles::contains_for_locale(word, locale)
}
