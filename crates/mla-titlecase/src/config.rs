//! Configuration types for the MLA title-casing engine.

use crate::lexicon::ExternalLexicons;

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

/// Locale hook for opt-in future casing extensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocaleProfile {
    /// Default English-centric MLA behavior.
    English,
}

/// Options for [`crate::titlecase_with_options`].
#[derive(Debug, Clone)]
pub struct TitleCaseOptions<'a> {
    /// Preserve mixed-case input such as `iPhone` when the word is not forced lowercase.
    pub preserve_existing_caps: bool,
    /// Capitalize the first significant word after a colon.
    pub capitalize_after_colon: bool,
    /// Lowercase MLA small words when they appear internally.
    pub lowercase_small_words: bool,
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
}
