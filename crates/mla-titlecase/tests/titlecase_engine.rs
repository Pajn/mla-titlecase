//! Integration tests for the MLA title-casing engine.

use mla_titlecase::{
    titlecase_into, titlecase_mla, titlecase_with_options, AllCapsPolicy, ExternalLexicons,
    HyphenStyle, LocaleProfile, NameParticlePolicy, SmallWordPolicy, TitleCaseOptions,
    UnknownWordCasing,
};

#[test]
fn titlecase_into_matches_allocating_api_and_reuses_buffer() {
    let options = TitleCaseOptions::default();
    let titles = [
        "the wind in the willows",
        "preface: the return of sherlock holmes",
        "a by-product of war",
    ];

    // A reused buffer is cleared each call, so it holds only the latest result
    // and matches the allocating API exactly.
    let mut buffer = String::from("stale contents");
    for title in titles {
        titlecase_into(&mut buffer, title, &options);
        assert_eq!(buffer, titlecase_with_options(title, &options));
    }
}

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

    // An em dash separates clauses; it is not a compound hyphen even under
    // CapitalizeBoth.
    assert_eq!(
        titlecase_with_options("well-known\u{2014}a memoir of sorts", &options),
        "Well-Known\u{2014}a Memoir of Sorts"
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
fn protected_spellings_survive_small_word_lowering() {
    // "via" is a built-in small word, but the protected spelling wins.
    let options = TitleCaseOptions::with_protected_words(&["VIA"]);
    assert_eq!(titlecase_with_options("traveling via rail", &options), "Traveling VIA Rail");

    // An AlwaysLowercase word-set entry does not override a protected spelling.
    let mut lexicons = ExternalLexicons::default();
    lexicons.add_protected_spellings([("github", "GitHub")]);
    lexicons.add_word_set(["github"]);
    let options = TitleCaseOptions {
        external_lexicons: Some(&lexicons),
        small_word_policy: SmallWordPolicy::AlwaysLowercase,
        ..TitleCaseOptions::default()
    };
    assert_eq!(titlecase_with_options("learning github daily", &options), "Learning GitHub Daily");
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

    // A particle next to a small word is not inside a personal-name run.
    assert_eq!(
        titlecase_with_options("riding the van to victory", &options),
        "Riding the Van to Victory"
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
fn capitalizes_adverbial_particles() {
    // A particle before punctuation or a conjunction cannot be a preposition.
    assert_eq!(titlecase_mla("come up and see me"), "Come Up and See Me");
    assert_eq!(titlecase_mla("wake up, little susie"), "Wake Up, Little Susie");
    // A particle after a phrasal-verb head is an adverb even with an object.
    assert_eq!(titlecase_mla("turn off the lights"), "Turn Off the Lights");
    assert_eq!(titlecase_mla("burning down the house"), "Burning Down the House");
    assert_eq!(titlecase_mla("runnin' down a dream"), "Runnin' Down a Dream");
    // An object pronoun may separate the verb from its particle.
    assert_eq!(titlecase_mla("wake me up before you go-go"), "Wake Me Up before You Go-Go");
}

#[test]
fn keeps_prepositional_uses_lowercase() {
    assert_eq!(titlecase_mla("walking down the street"), "Walking down the Street");
    assert_eq!(titlecase_mla("livin' on a prayer"), "Livin' on a Prayer");
    assert_eq!(titlecase_mla("the wind in the willows"), "The Wind in the Willows");

    let options =
        TitleCaseOptions { capitalize_phrasal_particles: false, ..TitleCaseOptions::default() };
    assert_eq!(titlecase_with_options("turn off the lights", &options), "Turn off the Lights");
}

#[test]
fn keeps_contraction_endings_lowercase() {
    assert_eq!(titlecase_mla("don't look up"), "Don't Look Up");
    assert_eq!(titlecase_mla("it's a wonderful life"), "It's a Wonderful Life");
    assert_eq!(titlecase_mla("the man who wasn't there"), "The Man Who Wasn't There");
    assert_eq!(titlecase_mla("o'neill's journey"), "O'Neill's Journey");
}

#[test]
fn keeps_contracted_and_lowercase() {
    assert_eq!(titlecase_mla("rock 'n' roll forever"), "Rock 'n' Roll Forever");
    assert_eq!(titlecase_mla("fish 'n' chips"), "Fish 'n' Chips");
    // A standalone "n" that is not the 'n' contraction is capitalized normally.
    assert_eq!(titlecase_mla("plan n for later"), "Plan N for Later");
}

#[test]
fn lowers_name_particles_even_when_small_words_are_not() {
    // With NeverLowercase small words keep their capitals, but the name-particle
    // heuristic still lowers particles inside a likely personal name.
    let options = TitleCaseOptions {
        small_word_policy: SmallWordPolicy::NeverLowercase,
        name_particle_policy: NameParticlePolicy::Heuristic,
        ..TitleCaseOptions::default()
    };
    assert_eq!(
        titlecase_with_options("ludwig van beethoven in concert", &options),
        "Ludwig van Beethoven In Concert"
    );
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
    // Known abbreviations survive the all-caps recasing.
    assert_eq!(titlecase_mla("IBM AND NASA HISTORY"), "IBM and NASA History");
}

#[test]
fn preserves_all_caps_input_under_preserve_policy() {
    let options = TitleCaseOptions {
        all_caps_policy: AllCapsPolicy::Preserve,
        ..TitleCaseOptions::default()
    };
    // Intentional stylization is returned verbatim: no recasing, no small-word
    // lowering.
    assert_eq!(titlecase_with_options("STAY WITH ME", &options), "STAY WITH ME");
    assert_eq!(titlecase_with_options("MONTERO", &options), "MONTERO");
    // Mixed-case input is not shouting, so the policy leaves it to normal rules.
    assert_eq!(
        titlecase_with_options("the wind in the willows", &options),
        "The Wind in the Willows"
    );
}

#[test]
fn recases_known_words_and_preserves_unknown_acronyms() {
    let mut dictionary = ExternalLexicons::default();
    dictionary.add_word_set(["history", "years", "handbook"]);
    let options = TitleCaseOptions {
        all_caps_policy: AllCapsPolicy::NormalizeKnownWords {
            unknown: UnknownWordCasing::Preserve,
        },
        external_lexicons: Some(&dictionary),
        ..TitleCaseOptions::default()
    };

    // NASA is restored by the built-in abbreviation table; "history" is a
    // dictionary word, so it recases. Neither exercises the dictionary gate on
    // its own.
    assert_eq!(titlecase_with_options("NASA HISTORY", &options), "NASA History");
    // The NormalizeKnownWords + Preserve behavior itself: unknown names, absent
    // from both the dictionary and the abbreviation table, stay all-caps unless
    // a canonical map restores them.
    assert_eq!(titlecase_with_options("SHERLOCK HOLMES", &options), "SHERLOCK HOLMES");

    // With no dictionary loaded, the policy falls back to full normalization.
    let no_dictionary = TitleCaseOptions {
        all_caps_policy: AllCapsPolicy::NormalizeKnownWords {
            unknown: UnknownWordCasing::Preserve,
        },
        ..TitleCaseOptions::default()
    };
    assert_eq!(
        titlecase_with_options("THE WIND IN THE WILLOWS", &no_dictionary),
        "The Wind in the Willows"
    );
}

#[test]
fn title_cases_unknown_words_when_requested() {
    let mut dictionary = ExternalLexicons::default();
    dictionary.add_word_set(["history", "years", "handbook"]);
    let options = TitleCaseOptions {
        all_caps_policy: AllCapsPolicy::NormalizeKnownWords {
            unknown: UnknownWordCasing::TitleCase,
        },
        external_lexicons: Some(&dictionary),
        ..TitleCaseOptions::default()
    };

    // Unknown words are now title-cased instead of preserved as acronyms.
    assert_eq!(titlecase_with_options("SHERLOCK HOLMES", &options), "Sherlock Holmes");
    // Known abbreviations are still restored, and dictionary words still recase.
    assert_eq!(titlecase_with_options("NASA HISTORY", &options), "NASA History");
    // With unknowns title-cased, the result matches plain `Normalize`.
    assert_eq!(
        titlecase_with_options("SHERLOCK HOLMES", &options),
        titlecase_mla("SHERLOCK HOLMES")
    );
}

#[test]
fn preserves_short_unknown_words_as_acronyms() {
    let mut dictionary = ExternalLexicons::default();
    dictionary.add_word_set(["report", "and", "the"]);
    let options = TitleCaseOptions {
        all_caps_policy: AllCapsPolicy::NormalizeKnownWords {
            unknown: UnknownWordCasing::PreserveShortAcronyms { max_acronym_len: 4 },
        },
        external_lexicons: Some(&dictionary),
        ..TitleCaseOptions::default()
    };

    // Short unknown words (<= 4 letters) are preserved as likely acronyms;
    // longer unknown words are title-cased as likely names.
    assert_eq!(
        titlecase_with_options("THE NSA AND SHERLOCK REPORT", &options),
        "The NSA and Sherlock Report"
    );
    // The threshold is inclusive: a 4-letter unknown (not on the built-in
    // abbreviation list) is still preserved.
    assert_eq!(titlecase_with_options("GAAP REPORT", &options), "GAAP Report");
}

#[test]
fn capitalizes_first_element_of_hyphenated_compound() {
    // The first element is capitalized even when it is a small word; interior
    // small words still lowercase.
    assert_eq!(titlecase_mla("a by-product of war"), "A By-Product of War");
    assert_eq!(titlecase_mla("the in-between"), "The In-Between");
    assert_eq!(titlecase_mla("life in the on-season"), "Life in the On-Season");
    // The established interior-lowering behavior is unchanged.
    assert_eq!(titlecase_mla("state-of-the-art design"), "State-of-the-Art Design");
}

#[test]
fn lowercases_english_small_words_under_non_english_locale() {
    let options = TitleCaseOptions::with_locale(LocaleProfile::Turkish);
    // "in" is an English small word matched by the built-in list; it must
    // lowercase to ASCII "in", not Turkish dotless "ın".
    let cased = titlecase_with_options("ISTANBUL IN WINTER", &options);
    assert!(cased.contains(" in "), "expected ASCII 'in', got {cased}");
}

#[test]
fn capitalizes_after_colon_through_curly_quotes() {
    assert_eq!(
        titlecase_mla("title: \u{201C}the sequel\u{201D}"),
        "Title: \u{201C}The Sequel\u{201D}"
    );
}
