use crate::casing::{lowercase_word, style_word};
use crate::config::{HyphenStyle, NameParticlePolicy, SmallWordPolicy, TitleCaseOptions};
use crate::context::{
    first_significant_word, follows_colon, last_significant_word, likely_name_particle_context,
    part_of_hyphenated_compound, precedes_colon,
};
use crate::lexicon::{
    abbreviation_spelling, built_in_protected_spelling, is_name_particle_for_locale, is_small_word,
};
use crate::token::Token;
use crate::util::normalize::lookup_key;

pub(crate) fn apply(tokens: &[Token<'_>], options: &TitleCaseOptions<'_>) -> String {
    let first = first_significant_word(tokens);
    let last = last_significant_word(tokens);
    let normalize_all_caps = input_is_all_caps(tokens);
    let mut output = String::with_capacity(tokens.iter().map(|token| token.text.len()).sum());

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

        let key = lookup_key(token.text);
        let is_first = first == Some(index);
        let is_last = last == Some(index);
        // MLA capitalizes the first and last words of both the title and the
        // subtitle, so a colon boundary capitalizes on both sides.
        let at_colon_boundary = options.capitalize_after_colon
            && (follows_colon(tokens, index) || precedes_colon(tokens, index));
        let hyphenated = part_of_hyphenated_compound(tokens, index);
        let capitalize_hyphen =
            hyphenated && matches!(options.hyphen_style, HyphenStyle::CapitalizeBoth);
        let should_capitalize = is_first || is_last || at_colon_boundary || capitalize_hyphen;

        if let Some(protected) = protected_spelling(token.text, &key, options) {
            if should_force_lowercase(&key, should_capitalize, tokens, index, options) {
                output.push_str(&lowercase_word(protected, options.locale));
            } else {
                output.push_str(protected);
            }
            index += 1;
            continue;
        }

        if should_force_lowercase(&key, should_capitalize, tokens, index, options) {
            output.push_str(&lowercase_word(token.text, options.locale));
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

        if options.name_particle_policy == NameParticlePolicy::Heuristic
            && is_name_particle_for_locale(&key, options.locale)
            && !should_capitalize
            && likely_name_particle_context(tokens, index)
        {
            output.push_str(&lowercase_word(token.text, options.locale));
            index += 1;
            continue;
        }

        if normalize_all_caps {
            output.push_str(&style_word(
                &lowercase_word(token.text, options.locale),
                true,
                options,
            ));
        } else {
            output.push_str(&style_word(token.text, true, options));
        }
        index += 1;
    }

    output
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

fn should_force_lowercase(
    key: &str,
    should_capitalize: bool,
    tokens: &[Token<'_>],
    index: usize,
    options: &TitleCaseOptions<'_>,
) -> bool {
    if should_capitalize || !options.lowercase_small_words {
        return false;
    }

    if options.name_particle_policy == NameParticlePolicy::Heuristic
        && is_name_particle_for_locale(key, options.locale)
        && likely_name_particle_context(tokens, index)
    {
        return true;
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
