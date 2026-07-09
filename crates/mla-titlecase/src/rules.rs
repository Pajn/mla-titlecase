use crate::analysis::{CasingRule, CasingSpan, Confidence, TitleCaseAnalysis};
use crate::casing::{lowercase_word, push_styled, StyleOutcome};
use crate::config::{
    AllCapsPolicy, HyphenStyle, LocaleProfile, NameParticlePolicy, SmallWordPolicy,
    TitleCaseOptions, UnknownWordCasing,
};
use crate::context::{
    first_significant_word, followed_by_hyphen, follows_subtitle_boundary, is_contracted_and,
    last_significant_word, likely_adverbial_particle, likely_name_particle_context,
    part_of_hyphenated_compound, preceded_by_hyphen, precedes_subtitle_boundary,
};
use crate::lexicon::{
    abbreviation_spelling, built_in_protected_spelling, is_name_particle_for_locale, is_small_word,
};
use crate::token::Token;
use crate::util::normalize::{lookup_key, normalized_key};
use crate::util::unicode::{append_uppercase, push_lowercased};

pub(crate) fn apply(input: &str, tokens: &[Token<'_>], options: &TitleCaseOptions<'_>) -> String {
    let mut output = String::new();
    let mut spans = Vec::new();
    run::<false>(input, &mut output, &mut spans, tokens, options);
    output
}

/// Appends the title-cased rendering of `tokens` into `output`. Callers that
/// process many titles can reuse a single buffer to avoid an allocation per
/// call.
pub(crate) fn apply_into(
    output: &mut String,
    input: &str,
    tokens: &[Token<'_>],
    options: &TitleCaseOptions<'_>,
) {
    let mut spans = Vec::new();
    run::<false>(input, output, &mut spans, tokens, options);
}

/// Title-cases `tokens` and records, for every word token, the rule and
/// confidence behind its casing. Shares the [`run`] driver (and [`emit_word`])
/// with the plain path so the two cannot disagree on output or branching.
pub(crate) fn analyze(
    input: &str,
    tokens: &[Token<'_>],
    options: &TitleCaseOptions<'_>,
) -> TitleCaseAnalysis {
    let mut output = String::new();
    let mut spans = Vec::new();
    run::<true>(input, &mut output, &mut spans, tokens, options);
    let confidence =
        spans.iter().fold(Confidence::Solid, |worst, span| worst.most_concerning(span.confidence));
    TitleCaseAnalysis { output, confidence, spans }
}

/// Shared token-walk driver for the plain and analysis paths. `RECORD` selects
/// whether per-word spans are recorded; with `RECORD == false` the span
/// bookkeeping compiles out, leaving the plain hot path unchanged.
fn run<const RECORD: bool>(
    input: &str,
    output: &mut String,
    spans: &mut Vec<CasingSpan>,
    tokens: &[Token<'_>],
    options: &TitleCaseOptions<'_>,
) {
    let first = first_significant_word(tokens);
    let last = last_significant_word(tokens);
    let shouting = input_is_all_caps(tokens);

    output.reserve(tokens.iter().map(|token| token.text.len()).sum());

    // `Preserve` treats all-caps input as intentional stylization and emits it
    // verbatim, skipping small-word lowering and recasing alike. Emitting the
    // input unchanged means no word changed, so there are no spans to record.
    if shouting && options.all_caps_policy == AllCapsPolicy::Preserve {
        for token in tokens {
            output.push_str(token.text);
        }
        return;
    }

    let base = input.as_ptr() as usize;
    let mut index = 0_usize;
    while index < tokens.len() {
        let token = &tokens[index];
        if !token.is_word() {
            output.push_str(token.text);
            index += 1;
            continue;
        }

        let out_start = output.len();

        if let Some((end_index, canonical_phrase)) = options
            .external_lexicons
            .and_then(|lexicons| lexicons.multiword_spelling(tokens, index))
            // Protected spellings are never recased, by anything, so a phrase
            // covering one falls back to the per-word cascade.
            .filter(|&(end_index, _)| !phrase_contains_protected(tokens, index, end_index, options))
        {
            // The canonical phrase is emitted verbatim except at the start of
            // the title or a subtitle, where MLA's first-word rule outranks it:
            // "de la soul is dead" opens with "De la Soul".
            let capitalize_first = first == Some(index)
                || (options.capitalize_after_subtitle_boundary
                    && follows_subtitle_boundary(tokens, index));
            if capitalize_first {
                push_phrase_capitalized(output, canonical_phrase, options.locale);
            } else {
                output.push_str(canonical_phrase);
            }
            if RECORD {
                let source_start = token.text.as_ptr() as usize - base;
                let end_token = &tokens[end_index];
                let source_end = end_token.text.as_ptr() as usize - base + end_token.text.len();
                record_span(
                    spans,
                    input,
                    output,
                    source_start..source_end,
                    out_start,
                    CasingRule::MultiwordLexicon,
                );
            }
            index = end_index + 1;
            continue;
        }

        let rule = emit_word::<RECORD>(output, tokens, index, first, last, shouting, options);
        if RECORD {
            let source_start = token.text.as_ptr() as usize - base;
            record_span(
                spans,
                input,
                output,
                source_start..source_start + token.text.len(),
                out_start,
                rule,
            );
        }
        index += 1;
    }
}

/// True when any word covered by a multiword match has a protected spelling.
fn phrase_contains_protected(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    options: &TitleCaseOptions<'_>,
) -> bool {
    tokens[start..=end].iter().any(|token| {
        token.is_word()
            && protected_spelling(token.text, &normalized_key(token.text), options).is_some()
    })
}

/// Appends a canonical phrase with its first letter capitalized, for phrases
/// that start a title or subtitle segment.
fn push_phrase_capitalized(output: &mut String, phrase: &str, locale: LocaleProfile) {
    let mut chars = phrase.chars();
    if let Some(first) = chars.next() {
        append_uppercase(output, first, locale);
        output.push_str(chars.as_str());
    }
}

/// Records one word's span, refining the rule's base confidence for dual-role
/// prepositions and flagging whether the casing actually changed.
fn record_span(
    spans: &mut Vec<CasingSpan>,
    input: &str,
    output: &str,
    source: core::ops::Range<usize>,
    out_start: usize,
    rule: CasingRule,
) {
    let source_text = &input[source.clone()];
    let output_text = &output[out_start..];
    let changed = source_text != output_text;
    let confidence = refine_confidence(rule, source_text);
    spans.push(CasingSpan { source, output: out_start..output.len(), rule, confidence, changed });
}

/// Dual-role prepositions (`after`, `before`, `since`, `till`, `until`) are
/// lowercased on the assumption they act as prepositions, which is a documented
/// judgment call rather than a certainty.
fn refine_confidence(rule: CasingRule, source_text: &str) -> Confidence {
    if rule == CasingRule::SmallWord
        && matches!(
            normalized_key(source_text).as_ref(),
            "after" | "before" | "since" | "till" | "until"
        )
    {
        return Confidence::Heuristic;
    }
    rule.confidence()
}

/// Writes one word into `output` and returns the rule that decided its casing.
/// The single source of truth for the per-word cascade, used by both the plain
/// and analysis paths.
///
/// `RECORD` is a compile-time flag: the plain path instantiates it `false`, so
/// the rule-attribution work (notably [`rule_for_style`]) is dead-code
/// eliminated and the returned value is a placeholder the caller ignores.
fn emit_word<const RECORD: bool>(
    output: &mut String,
    tokens: &[Token<'_>],
    index: usize,
    first: Option<usize>,
    last: Option<usize>,
    shouting: bool,
    options: &TitleCaseOptions<'_>,
) -> CasingRule {
    let token = &tokens[index];
    let key = normalized_key(token.text);
    let is_first = first == Some(index);
    let is_last = last == Some(index);
    // MLA capitalizes the first and last words of both the title and the
    // subtitle, so a boundary (colon, question mark, exclamation point)
    // capitalizes on both sides.
    let at_subtitle_boundary = options.capitalize_after_subtitle_boundary
        && (follows_subtitle_boundary(tokens, index) || precedes_subtitle_boundary(tokens, index));
    let hyphenated = part_of_hyphenated_compound(tokens, index);
    // The first element of a hyphenated compound is always capitalized, even
    // under MlaLike ("A By-Product of War"); only interior elements follow the
    // small-word rule ("State-of-the-Art").
    let starts_hyphenated_compound =
        followed_by_hyphen(tokens, index) && !preceded_by_hyphen(tokens, index);
    let capitalize_hyphen = starts_hyphenated_compound
        || (hyphenated && matches!(options.hyphen_style, HyphenStyle::CapitalizeBoth));
    let should_capitalize = is_first || is_last || at_subtitle_boundary || capitalize_hyphen;

    // Protected spellings always win, including over small-word lowering: a
    // spelling the caller asked to protect is never recased.
    if let Some(protected) = protected_spelling(token.text, &key, options) {
        output.push_str(protected);
        return CasingRule::ProtectedSpelling;
    }

    // Words lowered here matched Latin-script function-word lists, so they
    // lowercase the English way. Locale casing (e.g. Turkish dotless `ı`)
    // applies only to genuine title words, keeping "IN" -> "in" rather than "ın".
    if !should_capitalize && is_contracted_and(tokens, index, &key) {
        push_lowercased(output, token.text, LocaleProfile::English);
        return CasingRule::ContractedAnd;
    }
    if is_lowerable_name_particle(&key, should_capitalize, tokens, index, options) {
        push_lowercased(output, token.text, LocaleProfile::English);
        return CasingRule::NameParticle;
    }
    if should_force_lowercase(&key, should_capitalize, tokens, index, options) {
        push_lowercased(output, token.text, LocaleProfile::English);
        return CasingRule::SmallWord;
    }

    if let Some(abbreviation) = abbreviation_spelling(&key) {
        output.push_str(abbreviation);
        return CasingRule::Abbreviation;
    }

    if let Some(canonical) =
        options.external_lexicons.and_then(|lexicons| lexicons.canonical_spelling(token.text))
    {
        output.push_str(canonical);
        return CasingRule::CanonicalLexicon;
    }

    if shouting && normalize_all_caps_word(&key, options) {
        let lowered = lowercase_word(token.text, options.locale);
        // The rule is known to be `AllCapsNormalized`, so the styling outcome is
        // not needed here.
        let _ = push_styled(output, &lowered, true, should_capitalize, options);
        return CasingRule::AllCapsNormalized;
    }

    let outcome = push_styled(output, token.text, true, should_capitalize, options);
    if RECORD {
        rule_for_style(
            outcome,
            shouting,
            is_first,
            is_last,
            at_subtitle_boundary,
            capitalize_hyphen,
            tokens,
            index,
            &key,
            options,
        )
    } else {
        CasingRule::Capitalized
    }
}

/// Maps the branch [`push_styled`] took, plus the reason a word was forced to
/// capitalize, to a [`CasingRule`].
#[allow(clippy::too_many_arguments)]
fn rule_for_style(
    outcome: StyleOutcome,
    shouting: bool,
    is_first: bool,
    is_last: bool,
    at_subtitle_boundary: bool,
    capitalize_hyphen: bool,
    tokens: &[Token<'_>],
    index: usize,
    key: &str,
    options: &TitleCaseOptions<'_>,
) -> CasingRule {
    match outcome {
        StyleOutcome::AcronymPreserved if shouting => CasingRule::AllCapsPreservedAcronym,
        StyleOutcome::AcronymPreserved => CasingRule::AcronymPreserved,
        StyleOutcome::DottedAbbreviation => CasingRule::DottedAbbreviation,
        StyleOutcome::MixedCasePreserved => CasingRule::MixedCasePreserved,
        // Reached only with `capitalize = true`, so treat as capitalization.
        StyleOutcome::Lowercased | StyleOutcome::Capitalized => {
            if is_first {
                CasingRule::FirstWord
            } else if is_last {
                CasingRule::LastWord
            } else if at_subtitle_boundary {
                CasingRule::SubtitleBoundary
            } else if capitalize_hyphen {
                CasingRule::HyphenatedCompound
            } else if options.capitalize_phrasal_particles
                && likely_adverbial_particle(tokens, index, key)
            {
                CasingRule::AdverbialParticle
            } else {
                CasingRule::Capitalized
            }
        }
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
