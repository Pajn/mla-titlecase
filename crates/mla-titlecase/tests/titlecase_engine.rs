//! Integration tests for the MLA title-casing engine.

use mla_titlecase::{
    titlecase_mla, titlecase_with_options, ExternalLexicons, HyphenStyle, LocaleProfile,
    NameParticlePolicy, SmallWordPolicy, TitleCaseOptions,
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
fn capitalizes_last_word_before_colon() {
    assert_eq!(
        titlecase_mla("what dreams are made of: a study"),
        "What Dreams Are Made Of: A Study"
    );
    assert_eq!(
        titlecase_mla("the world we live in: essays on modern life"),
        "The World We Live In: Essays on Modern Life"
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
    lexicons.add_multiword_map([("new york city", "New York City")]);
    lexicons.add_protected_spellings([("copilot", "Copilot")]);
    lexicons.add_word_set(["amidst"]);

    let options = TitleCaseOptions {
        external_lexicons: Some(&lexicons),
        small_word_policy: SmallWordPolicy::AlwaysLowercase,
        ..TitleCaseOptions::default()
    };

    assert_eq!(
        titlecase_with_options("copilot amidst postgres updates", &options),
        "Copilot amidst Postgres Updates"
    );
    assert_eq!(titlecase_with_options("new york city stories", &options), "New York City Stories");
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
    assert_eq!(titlecase_mla("meet me at 9 a.m. sharp"), "Meet Me at 9 a.m. Sharp");
}

#[test]
fn capitalizes_subordinating_conjunctions() {
    assert_eq!(titlecase_mla("what if that happens"), "What If That Happens");
    assert_eq!(titlecase_mla("stronger than pride"), "Stronger Than Pride");
    assert_eq!(titlecase_mla("love me once again"), "Love Me Once Again");
}

#[test]
fn lowercases_prepositions_of_any_length() {
    assert_eq!(titlecase_mla("dancing among the stars"), "Dancing among the Stars");
    assert_eq!(titlecase_mla("the war between us"), "The War between Us");
    assert_eq!(titlecase_mla("a river runs through it"), "A River Runs through It");
    assert_eq!(titlecase_mla("life without borders"), "Life without Borders");
}

#[test]
fn keeps_contraction_endings_lowercase() {
    assert_eq!(titlecase_mla("don't look up"), "Don't Look Up");
    assert_eq!(titlecase_mla("it's a wonderful life"), "It's a Wonderful Life");
    assert_eq!(titlecase_mla("the man who wasn't there"), "The Man Who Wasn't There");
    assert_eq!(titlecase_mla("o'neill's journey"), "O'Neill's Journey");
}

#[test]
fn keeps_ordinal_suffixes_lowercase() {
    assert_eq!(titlecase_mla("miracle on 34th street"), "Miracle on 34th Street");
    assert_eq!(titlecase_mla("42nd street"), "42nd Street");
}

#[test]
fn recases_all_caps_input() {
    assert_eq!(titlecase_mla("THE WIND IN THE WILLOWS"), "The Wind in the Willows");
    assert_eq!(titlecase_mla("MLA HANDBOOK"), "MLA Handbook");
    // A lone all-caps word is still treated as an acronym.
    assert_eq!(titlecase_mla("NASA"), "NASA");
    assert_eq!(titlecase_mla("the NASA years"), "The NASA Years");
}

#[test]
fn capitalizes_after_colon_through_curly_quotes() {
    assert_eq!(
        titlecase_mla("title: \u{201C}the sequel\u{201D}"),
        "Title: \u{201C}The Sequel\u{201D}"
    );
}
