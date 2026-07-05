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

/// Controls how fully all-caps ("shouting") input is handled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum AllCapsPolicy {
    /// Recase all-caps input to title case, restoring known abbreviations and
    /// protected spellings. This is the default MLA behavior.
    Normalize,
    /// Treat all-caps input as intentional stylization and leave every word as
    /// written (`MONTERO`, `STAY`). Useful for music and brand metadata.
    Preserve,
    /// Recase all-caps input using a loaded dictionary word-set: words the
    /// dictionary recognizes are always title-cased, while unrecognized words
    /// are handled per [`UnknownWordCasing`]. With SCOWL loaded and unknowns
    /// preserved, a recognized word recases while an unknown name stays as
    /// written (`SHERLOCK HISTORY` -> `SHERLOCK History`). Behaves like
    /// [`AllCapsPolicy::Normalize`] when no word-set lexicon is loaded, so it
    /// expects a comprehensive membership source such as SCOWL.
    NormalizeKnownWords {
        /// How to cast all-caps words the dictionary does not recognize.
        unknown: UnknownWordCasing,
    },
}

/// How [`AllCapsPolicy::NormalizeKnownWords`] casts all-caps words that the
/// loaded dictionary does not recognize.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum UnknownWordCasing {
    /// Keep every unrecognized word as written, assuming it is an acronym
    /// (`SHERLOCK` stays `SHERLOCK`). An external canonical map or protected
    /// spelling can still restore a genuine name's casing.
    Preserve,
    /// Title-case every unrecognized word, assuming it is an ordinary word or
    /// name (`SHERLOCK` becomes `Sherlock`). With this setting the dictionary
    /// gate has no visible effect on output, so it matches
    /// [`AllCapsPolicy::Normalize`].
    TitleCase,
    /// Preserve short unrecognized words as acronyms but title-case longer ones,
    /// on the heuristic that acronyms are short (`IBM`, `NASA`) while names and
    /// ordinary words are longer (`SHERLOCK` -> `Sherlock`). A word is preserved
    /// when its letter count is at most `max_acronym_len`; a value around 4 or 5
    /// is a reasonable starting point. Necessarily imperfect: short names
    /// (`LEE`) are preserved and long acronyms (`SCOTUS`) are title-cased, so
    /// pin the exceptions with a canonical map or protected spellings.
    PreserveShortAcronyms {
        /// Maximum letter count preserved as an acronym; longer words title-case.
        max_acronym_len: usize,
    },
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
    /// Selects how fully all-caps input is handled.
    pub all_caps_policy: AllCapsPolicy,
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
            all_caps_policy: AllCapsPolicy::Normalize,
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
