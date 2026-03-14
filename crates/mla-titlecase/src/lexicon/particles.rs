use crate::config::LocaleProfile;

const DUTCH_PARTICLES: &[&str] = &["de", "den", "der", "ten", "ter", "van", "von"];
const FRENCH_PARTICLES: &[&str] = &["d", "de", "des", "du", "la", "le"];
const GERMAN_PARTICLES: &[&str] = &["am", "auf", "im", "von", "vom", "zu", "zur"];
const ITALIAN_PARTICLES: &[&str] = &["da", "de", "del", "della", "di", "la", "lo"];
const SPANISH_PARTICLES: &[&str] = &["de", "del", "la", "las", "los"];
const GENERIC_PARTICLES: &[&str] = &["da", "de", "del", "der", "di", "la", "van", "von"];

pub(crate) fn contains_for_locale(word: &str, locale: LocaleProfile) -> bool {
    let particles = match locale {
        LocaleProfile::Dutch => DUTCH_PARTICLES,
        LocaleProfile::French => FRENCH_PARTICLES,
        LocaleProfile::German => GERMAN_PARTICLES,
        LocaleProfile::Italian => ITALIAN_PARTICLES,
        LocaleProfile::Spanish => SPANISH_PARTICLES,
        LocaleProfile::English | LocaleProfile::Turkish => GENERIC_PARTICLES,
    };

    particles.contains(&word)
}

#[cfg(test)]
mod tests {
    use crate::config::LocaleProfile;

    use super::contains_for_locale;

    #[test]
    fn matches_locale_specific_particles() {
        assert!(contains_for_locale("van", LocaleProfile::Dutch));
        assert!(contains_for_locale("del", LocaleProfile::Spanish));
        assert!(!contains_for_locale("van", LocaleProfile::Spanish));
    }
}
