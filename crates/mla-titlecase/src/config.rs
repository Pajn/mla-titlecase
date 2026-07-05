//! Configuration types for the MLA title-casing engine.

use crate::lexicon::ExternalLexicons;
use crate::locale;

/// Controls how the engine decides whether a word should stay lowercase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmallWordPolicy {
    /// Use the built-in curated MLA small-word list.
    Mla,
    /// Lowercase built-in small words and any matching external word-set entry.
    AlwaysLowercase,
    /// Never lowercase a word just because it is on the small-word list.
    NeverLowercase,
}

/// Controls how hyphenated compounds are cased.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HyphenStyle {
    /// Apply the MLA rules to each hyphen-separated segment.
    MlaLike,
    /// Capitalize every word segment in a hyphenated compound.
    CapitalizeBoth,
}

/// Controls whether known name particles are lowered in likely personal names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameParticlePolicy {
    /// Keep the default MLA behavior and ignore particle heuristics.
    Disabled,
    /// Lowercase configured particles inside likely personal-name runs.
    Heuristic,
}

/// Locale/profile hook for opt-in CLDR-inspired casing extensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocaleProfile {
    /// Default English-centric MLA behavior.
    English,
    /// Dutch locale/profile behavior such as `IJ` digraph casing and common particles.
    Dutch,
    /// French locale/profile behavior and common particles.
    French,
    /// German locale/profile behavior and common particles.
    German,
    /// Italian locale/profile behavior and common particles.
    Italian,
    /// Spanish locale/profile behavior and common particles.
    Spanish,
    /// Turkish locale/profile behavior for dotted/dotless `i`.
    Turkish,
}

/// Options for [`crate::titlecase_with_options`].
#[derive(Debug, Clone)]
pub struct TitleCaseOptions<'a> {
    /// Preserve mixed-case input such as `iPhone` when the word is not forced lowercase.
    pub preserve_existing_caps: bool,
    /// Treat colons as subtitle boundaries: capitalize the first significant
    /// word after a colon and the last one before it.
    pub capitalize_after_colon: bool,
    /// Lowercase MLA small words when they appear internally.
    pub lowercase_small_words: bool,
    /// Capitalize small words acting as adverbial particles rather than
    /// prepositions (`Give Up`, `Turn Off the Lights`).
    pub capitalize_phrasal_particles: bool,
    /// Selects the small-word policy to apply.
    pub small_word_policy: SmallWordPolicy,
    /// Selects how hyphenated compounds are handled.
    pub hyphen_style: HyphenStyle,
    /// Additional user-supplied protected spellings matched case-insensitively.
    pub protected_words: &'a [&'a str],
    /// Optional externally loaded lexicons.
    pub external_lexicons: Option<&'a ExternalLexicons>,
    /// Controls optional name-particle handling.
    pub name_particle_policy: NameParticlePolicy,
    /// Locale hook for future opt-in casing changes.
    pub locale: LocaleProfile,
}

impl<'a> Default for TitleCaseOptions<'a> {
    fn default() -> Self {
        Self {
            preserve_existing_caps: true,
            capitalize_after_colon: true,
            lowercase_small_words: true,
            capitalize_phrasal_particles: true,
            small_word_policy: SmallWordPolicy::Mla,
            hyphen_style: HyphenStyle::MlaLike,
            protected_words: &[],
            external_lexicons: None,
            name_particle_policy: NameParticlePolicy::Disabled,
            locale: LocaleProfile::English,
        }
    }
}

impl<'a> TitleCaseOptions<'a> {
    /// Returns a copy of the default options with a custom protected-word slice.
    #[must_use]
    pub fn with_protected_words(protected_words: &'a [&'a str]) -> Self {
        Self { protected_words, ..Self::default() }
    }

    /// Returns a copy of the default options with external lexicons enabled.
    #[must_use]
    pub fn with_external_lexicons(external_lexicons: &'a ExternalLexicons) -> Self {
        Self { external_lexicons: Some(external_lexicons), ..Self::default() }
    }

    /// Returns default options configured for a specific locale/profile.
    #[must_use]
    pub fn with_locale(locale: LocaleProfile) -> Self {
        Self {
            locale,
            name_particle_policy: locale.default_name_particle_policy(),
            ..Self::default()
        }
    }
}

impl LocaleProfile {
    /// Resolves a profile from a BCP-47-like language tag, defaulting to English.
    #[must_use]
    pub fn from_bcp47(tag: &str) -> Self {
        locale::resolve_locale_profile(tag)
    }

    /// Returns the normalized primary language tag for this locale/profile.
    #[must_use]
    pub const fn bcp47_tag(self) -> &'static str {
        locale::locale_tag(self)
    }

    pub(crate) const fn default_name_particle_policy(self) -> NameParticlePolicy {
        locale::default_name_particle_policy(self)
    }
}

#[cfg(test)]
mod tests {
    use super::{LocaleProfile, NameParticlePolicy, TitleCaseOptions};

    #[test]
    fn builds_locale_specific_defaults() {
        let options = TitleCaseOptions::with_locale(LocaleProfile::Dutch);
        assert_eq!(options.locale, LocaleProfile::Dutch);
        assert_eq!(options.name_particle_policy, NameParticlePolicy::Heuristic);
    }

    #[test]
    fn resolves_profiles_from_bcp47_tags() {
        assert_eq!(LocaleProfile::from_bcp47("fr-FR"), LocaleProfile::French);
        assert_eq!(LocaleProfile::from_bcp47("tr"), LocaleProfile::Turkish);
        assert_eq!(LocaleProfile::from_bcp47("en-US"), LocaleProfile::English);
    }
}
