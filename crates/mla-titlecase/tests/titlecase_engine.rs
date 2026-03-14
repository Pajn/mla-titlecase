//! Integration tests for the MLA title-casing engine.

use mla_titlecase::{
    titlecase_mla, titlecase_with_options, ExternalLexicons, HyphenStyle, NameParticlePolicy,
    LocaleProfile, SmallWordPolicy, TitleCaseOptions,
};

#[test]
fn titlecases_basic_examples() {
    assert_eq!(titlecase_mla("the wind in the willows"), "The Wind in the Willows");
    assert_eq!(titlecase_mla("love in the time of cholera"), "Love in the Time of Cholera");
}

#[test]
fn capitalizes_after_colon() {
    assert_eq!(
        titlecase_mla("preface: the return of sherlock holmes"),
        "Preface: The Return of Sherlock Holmes"
    );
}

#[test]
fn handles_hyphenated_compounds() {
    assert_eq!(titlecase_mla("state-of-the-art design"), "State-of-the-Art Design");

    let options = TitleCaseOptions {
        hyphen_style: HyphenStyle::CapitalizeBoth,
        ..TitleCaseOptions::default()
    };
    assert_eq!(
        titlecase_with_options("state-of-the-art design", &options),
        "State-Of-The-Art Design"
    );
}

#[test]
fn preserves_protected_words_and_mixed_case() {
    let options = TitleCaseOptions::with_protected_words(&["PostgreSQL"]);
    assert_eq!(
        titlecase_with_options("learning postgresql with github and iphone", &options),
        "Learning PostgreSQL with GitHub and iPhone"
    );
}

#[test]
fn supports_additive_external_lexicons() {
    let mut lexicons = ExternalLexicons::default();
    lexicons.add_canonical_map([("postgres", "Postgres")]);
    lexicons.add_protected_spellings([("copilot", "Copilot")]);
    lexicons.add_word_set(["amid"]);

    let options = TitleCaseOptions {
        external_lexicons: Some(&lexicons),
        small_word_policy: SmallWordPolicy::AlwaysLowercase,
        ..TitleCaseOptions::default()
    };

    assert_eq!(
        titlecase_with_options("copilot amid postgres updates", &options),
        "Copilot amid Postgres Updates"
    );
}

#[test]
fn supports_name_particle_heuristics() {
    let options = TitleCaseOptions {
        name_particle_policy: NameParticlePolicy::Heuristic,
        ..TitleCaseOptions::default()
    };

    assert_eq!(
        titlecase_with_options("ludwig van beethoven in concert", &options),
        "Ludwig van Beethoven in Concert"
    );
}

#[test]
fn supports_locale_profile_defaults() {
    let options = TitleCaseOptions::with_locale(LocaleProfile::Dutch);
    assert_eq!(
        titlecase_with_options("ijsselmeer and jan van der heijden", &options),
        "IJsselmeer and Jan van der Heijden"
    );
}

#[test]
fn keeps_default_english_behavior_stable() {
    assert_eq!(
        titlecase_with_options("the wind in the willows", &TitleCaseOptions::default()),
        "The Wind in the Willows"
    );
}

#[test]
fn preserves_acronyms_and_dotted_abbreviations() {
    assert_eq!(titlecase_mla("nasa and the u.s.a. mission"), "NASA and the U.S.A. Mission");
}
