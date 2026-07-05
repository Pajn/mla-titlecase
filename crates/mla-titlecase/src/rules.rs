use crate::casing::{lowercase_word, push_styled};
use crate::config::{
    AllCapsPolicy, HyphenStyle, LocaleProfile, NameParticlePolicy, SmallWordPolicy,
    TitleCaseOptions, UnknownWordCasing,
};
use crate::context::{
    first_significant_word, followed_by_hyphen, follows_colon, is_contracted_and,
    last_significant_word, likely_adverbial_particle, likely_name_particle_context,
    part_of_hyphenated_compound, preceded_by_hyphen, precedes_colon,
};
use crate::lexicon::{
    abbreviation_spelling, built_in_protected_spelling, is_name_particle_for_locale, is_small_word,
};
use crate::token::Token;
use crate::util::normalize::{lookup_key, normalized_key};
use crate::util::unicode::push_lowercased;

pub(crate) fn apply(tokens: &[Token<'_>], options: &TitleCaseOptions<'_>) -> String {
    let mut output = String::new();
    apply_into(&mut output, tokens, options);
    output
}

/// Appends the title-cased rendering of `tokens` into `output`. Callers that
/// process many titles can reuse a single buffer to avoid an allocation per
/// call.
pub(crate) fn apply_into(
    output: &mut String,
    tokens: &[Token<'_>],
    options: &TitleCaseOptions<'_>,
) {
    let first = first_significant_word(tokens);
    let last = last_significant_word(tokens);
    let shouting = input_is_all_caps(tokens);

    output.reserve(tokens.iter().map(|token| token.text.len()).sum());

    // `Preserve` treats all-caps input as intentional stylization and emits it
    // verbatim, skipping small-word lowering and recasing alike.
    if shouting && options.all_caps_policy == AllCapsPolicy::Preserve {
        for token in tokens {
            output.push_str(token.text);
        }
        return;
    }

    let mut index = 0_usize;
    while index < tokens.len() {
        let token = &tokens[index];
        if !token.is_word() {
            output.push_str(token.text);
            index += 1;
            continue;
        }

        if let Some((end_index, canonical_phrase)) = options
            .external_lexicons
            .and_then(|lexicons| lexicons.multiword_spelling(tokens, index))
        {
            output.push_str(canonical_phrase);
            index = end_index + 1;
            continue;
        }

        let key = normalized_key(token.text);
        let is_first = first == Some(index);
        let is_last = last == Some(index);
        // MLA capitalizes the first and last words of both the title and the
        // subtitle, so a colon boundary capitalizes on both sides.
        let at_colon_boundary = options.capitalize_after_colon
            && (follows_colon(tokens, index) || precedes_colon(tokens, index));
        let hyphenated = part_of_hyphenated_compound(tokens, index);
        // The first element of a hyphenated compound is always capitalized, even
        // under MlaLike ("A By-Product of War"); only interior elements follow
        // the small-word rule ("State-of-the-Art").
        let starts_hyphenated_compound =
            followed_by_hyphen(tokens, index) && !preceded_by_hyphen(tokens, index);
        let capitalize_hyphen = starts_hyphenated_compound
            || (hyphenated && matches!(options.hyphen_style, HyphenStyle::CapitalizeBoth));
        let should_capitalize = is_first || is_last || at_colon_boundary || capitalize_hyphen;

        // Protected spellings always win, including over small-word lowering:
        // a spelling the caller asked to protect is never recased.
        if let Some(protected) = protected_spelling(token.text, &key, options) {
            output.push_str(protected);
            index += 1;
            continue;
        }

        // Words lowered here matched Latin-script function-word lists (small
        // words, name particles, the "'n'" contraction), so they lowercase the
        // English way. Locale casing (e.g. Turkish dotless `ı`) applies only to
        // genuine title words, keeping "IN" -> "in" rather than "ın".
        if (!should_capitalize && is_contracted_and(tokens, index, &key))
            || is_lowerable_name_particle(&key, should_capitalize, tokens, index, options)
            || should_force_lowercase(&key, should_capitalize, tokens, index, options)
        {
            push_lowercased(output, token.text, LocaleProfile::English);
            index += 1;
            continue;
        }

        if let Some(abbreviation) = abbreviation_spelling(&key) {
            output.push_str(abbreviation);
            index += 1;
            continue;
        }

        if let Some(canonical) =
            options.external_lexicons.and_then(|lexicons| lexicons.canonical_spelling(token.text))
        {
            output.push_str(canonical);
            index += 1;
            continue;
        }

        if shouting && normalize_all_caps_word(&key, options) {
            let lowered = lowercase_word(token.text, options.locale);
            push_styled(output, &lowered, true, options);
        } else {
            push_styled(output, token.text, true, options);
        }
        index += 1;
    }
}

/// Decides whether an individual word of shouting input should be recased.
/// Only reached when the whole input was detected as all-caps and the policy is
/// not `Preserve` (handled by an early return).
fn normalize_all_caps_word(key: &str, options: &TitleCaseOptions<'_>) -> bool {
    match options.all_caps_policy {
        AllCapsPolicy::Normalize => true,
        AllCapsPolicy::NormalizeKnownWords { unknown } => {
            match options.external_lexicons.and_then(|lexicons| lexicons.dictionary_contains(key)) {
                // No dictionary loaded: recase everything, matching `Normalize`.
                None => true,
                // A recognized word is always title-cased.
                Some(true) => true,
                // An unrecognized word is handled per the caller's choice.
                Some(false) => match unknown {
                    UnknownWordCasing::Preserve => false,
                    UnknownWordCasing::TitleCase => true,
                    // Short words are likely acronyms and preserved; longer ones
                    // are likely names or words and title-cased.
                    UnknownWordCasing::PreserveShortAcronyms { max_acronym_len } => {
                        letter_count(key) > max_acronym_len
                    }
                },
            }
        }
        // Preserve short-circuits before the word loop.
        AllCapsPolicy::Preserve => false,
    }
}

/// Counts the alphabetic characters in a word, used by the short-acronym
/// heuristic so digits and punctuation do not inflate the length.
fn letter_count(word: &str) -> usize {
    word.chars().filter(|ch| ch.is_alphabetic()).count()
}

/// Detects shouting input such as `THE WIND IN THE WILLOWS`, where acronym and
/// mixed-case preservation would otherwise leave every word untouched. A single
/// all-caps word is more likely a genuine acronym, so it stays preserved.
fn input_is_all_caps(tokens: &[Token<'_>]) -> bool {
    let mut cased_words = 0_usize;
    for token in tokens {
        if !token.is_word() {
            continue;
        }
        let mut has_cased_letter = false;
        for ch in token.text.chars() {
            if ch.is_lowercase() {
                return false;
            }
            if ch.is_uppercase() {
                has_cased_letter = true;
            }
        }
        if has_cased_letter {
            cased_words += 1;
        }
    }
    cased_words > 1
}

fn protected_spelling<'a>(
    original: &str,
    key: &str,
    options: &'a TitleCaseOptions<'a>,
) -> Option<&'a str> {
    options
        .protected_words
        .iter()
        .copied()
        .find(|candidate| lookup_key(candidate) == key)
        .or_else(|| built_in_protected_spelling(key))
        .or_else(|| {
            options.external_lexicons.and_then(|lexicons| lexicons.protected_spelling(original))
        })
}

/// True when the name-particle heuristic should lower this word. Independent of
/// the small-word policy, so a particle such as `van` still lowers inside a
/// likely personal name even when general small-word lowering is off.
fn is_lowerable_name_particle(
    key: &str,
    should_capitalize: bool,
    tokens: &[Token<'_>],
    index: usize,
    options: &TitleCaseOptions<'_>,
) -> bool {
    !should_capitalize
        && options.name_particle_policy == NameParticlePolicy::Heuristic
        && is_name_particle_for_locale(key, options.locale)
        && likely_name_particle_context(tokens, index)
}

fn should_force_lowercase(
    key: &str,
    should_capitalize: bool,
    tokens: &[Token<'_>],
    index: usize,
    options: &TitleCaseOptions<'_>,
) -> bool {
    if should_capitalize {
        return false;
    }

    // MLA capitalizes adverbs, so a small word acting as an adverbial
    // particle ("Turn Off the Lights") escapes lowering.
    if options.capitalize_phrasal_particles && likely_adverbial_particle(tokens, index, key) {
        return false;
    }

    match options.small_word_policy {
        SmallWordPolicy::Mla => is_small_word(key),
        SmallWordPolicy::AlwaysLowercase => {
            is_small_word(key)
                || options.external_lexicons.is_some_and(|lexicons| lexicons.contains_word(key))
        }
        SmallWordPolicy::NeverLowercase => false,
    }
}
