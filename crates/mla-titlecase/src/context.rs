use crate::classify::{
    is_apostrophe, is_closing_punctuation, is_hyphen, is_opening_punctuation, is_significant_word,
    is_subtitle_boundary,
};
use crate::lexicon::{is_adverbial_particle, is_phrasal_verb_pair, is_small_word};
use crate::token::{Token, TokenKind};
use crate::util::normalize::normalized_key;

pub(crate) fn first_significant_word(tokens: &[Token<'_>]) -> Option<usize> {
    tokens.iter().position(|token| is_significant_word(*token))
}

pub(crate) fn last_significant_word(tokens: &[Token<'_>]) -> Option<usize> {
    tokens.iter().rposition(|token| is_significant_word(*token))
}

pub(crate) fn follows_subtitle_boundary(tokens: &[Token<'_>], index: usize) -> bool {
    let mut cursor = index;
    while cursor > 0 {
        cursor -= 1;
        let token = tokens[cursor];
        if token.is_word() {
            return false;
        }
        if is_subtitle_boundary(token) {
            return true;
        }
        if !token.text.chars().all(char::is_whitespace) && !is_opening_punctuation(token) {
            return false;
        }
    }
    false
}

pub(crate) fn precedes_subtitle_boundary(tokens: &[Token<'_>], index: usize) -> bool {
    for token in &tokens[index + 1..] {
        if token.is_word() {
            return false;
        }
        if is_subtitle_boundary(*token) {
            return true;
        }
        if !token.text.chars().all(char::is_whitespace) && !is_closing_punctuation(*token) {
            return false;
        }
    }
    false
}

pub(crate) fn part_of_hyphenated_compound(tokens: &[Token<'_>], index: usize) -> bool {
    preceded_by_hyphen(tokens, index) || followed_by_hyphen(tokens, index)
}

/// True for a lone `n` written as `'n'` with flanking apostrophes, standing in
/// for "and" as in "Rock 'n' Roll" or "Fish 'n' Chips".
pub(crate) fn is_contracted_and(tokens: &[Token<'_>], index: usize, key: &str) -> bool {
    key == "n"
        && tokens.get(index.wrapping_sub(1)).is_some_and(|token| is_apostrophe(*token))
        && tokens.get(index + 1).is_some_and(|token| is_apostrophe(*token))
}

pub(crate) fn preceded_by_hyphen(tokens: &[Token<'_>], index: usize) -> bool {
    tokens.get(index.wrapping_sub(1)).is_some_and(|token| is_hyphen(*token))
}

pub(crate) fn followed_by_hyphen(tokens: &[Token<'_>], index: usize) -> bool {
    tokens.get(index + 1).is_some_and(|token| is_hyphen(*token))
}

/// True when a small word is acting as an adverbial particle, which MLA
/// capitalizes, rather than as a preposition. Two signals qualify:
///
/// 1. Nothing that could serve as a preposition's complement follows — the
///    next token is punctuation, a dash, or a coordinating conjunction
///    ("Give Up, Move On", "Come Up and See Me").
/// 2. The word follows a known phrasal-verb head, directly or across one
///    object pronoun ("Turn Off the Lights", "Wake Me Up").
pub(crate) fn likely_adverbial_particle(tokens: &[Token<'_>], index: usize, key: &str) -> bool {
    if !is_adverbial_particle(key) || part_of_hyphenated_compound(tokens, index) {
        return false;
    }

    let mut cursor = index + 1;
    let following = loop {
        match tokens.get(cursor) {
            None => return true,
            Some(token) if token.kind == TokenKind::Whitespace => cursor += 1,
            Some(token) => break *token,
        }
    };

    if following.is_word() {
        // Coordinators cannot begin a preposition's complement. "so" and
        // "yet" are deliberately absent: they can open a noun phrase ("in so
        // many ways", "in yet another life").
        if matches!(normalized_key(following.text).as_ref(), "and" | "but" | "for" | "nor" | "or") {
            return true;
        }
    } else if following.kind == TokenKind::Slash {
        return false;
    } else if !is_opening_punctuation(following) {
        // Closing or clause punctuation: no complement can follow. An opening
        // quote or bracket may still introduce one, so it stays undecided.
        return true;
    }

    // Walk back over whitespace and closing punctuation (the trailing
    // apostrophe of "Runnin'") to the immediately preceding word.
    let mut cursor = index;
    let mut skipped_pronoun = false;
    while cursor > 0 {
        cursor -= 1;
        let token = tokens[cursor];
        if token.kind == TokenKind::Whitespace || is_closing_punctuation(token) {
            continue;
        }
        if token.is_word() {
            let word_key = normalized_key(token.text);
            // A phrasal verb's object pronoun may separate the verb from its
            // particle: "Wake Me Up", "Let You Down".
            if !skipped_pronoun
                && matches!(word_key.as_ref(), "me" | "you" | "him" | "her" | "it" | "us" | "them")
            {
                skipped_pronoun = true;
                continue;
            }
            return is_phrasal_verb_pair(&word_key, key);
        }
        break;
    }
    false
}

/// A particle sits inside a personal-name run when both neighbors look like
/// name words. Small words disqualify a neighbor, so `van` in "the van of
/// progress" keeps its regular capitalization while "Ludwig van Beethoven"
/// and chained particles like "Jan van der Heijden" stay lowered.
pub(crate) fn likely_name_particle_context(tokens: &[Token<'_>], index: usize) -> bool {
    looks_like_name_word(previous_word(tokens, index))
        && looks_like_name_word(next_word(tokens, index))
}

fn looks_like_name_word(word: Option<Token<'_>>) -> bool {
    word.is_some_and(|token| !is_small_word(&normalized_key(token.text)))
}

fn previous_word<'a>(tokens: &'a [Token<'a>], index: usize) -> Option<Token<'a>> {
    tokens[..index].iter().rev().copied().find(|token| token.is_word())
}

fn next_word<'a>(tokens: &'a [Token<'a>], index: usize) -> Option<Token<'a>> {
    tokens[index + 1..].iter().copied().find(|token| token.is_word())
}

#[cfg(test)]
mod tests {
    use super::{
        follows_subtitle_boundary, likely_name_particle_context, part_of_hyphenated_compound,
    };
    use crate::tokenizer::tokenize;

    #[test]
    fn detects_subtitle_boundary_context() {
        for input in ["Title: the sequel", "Title? the sequel", "Title! the sequel"] {
            let tokens = tokenize(input);
            let word_index = tokens.iter().position(|token| token.text == "the").unwrap();
            assert!(follows_subtitle_boundary(&tokens, word_index), "no boundary in {input:?}");
        }
        // An em dash separates clauses, not subtitles.
        let tokens = tokenize("Title\u{2014}the sequel");
        let word_index = tokens.iter().position(|token| token.text == "the").unwrap();
        assert!(!follows_subtitle_boundary(&tokens, word_index));
    }

    #[test]
    fn detects_word_preceding_subtitle_boundary() {
        for input in ["made of: a study", "made of? a study", "made of! a study"] {
            let tokens = tokenize(input);
            let word_index = tokens.iter().position(|token| token.text == "of").unwrap();
            assert!(
                super::precedes_subtitle_boundary(&tokens, word_index),
                "no boundary in {input:?}"
            );
            let word_index = tokens.iter().position(|token| token.text == "made").unwrap();
            assert!(!super::precedes_subtitle_boundary(&tokens, word_index));
        }
    }

    #[test]
    fn detects_hyphen_context() {
        let tokens = tokenize("state-of-the-art");
        let word_index = tokens.iter().position(|token| token.text == "of").unwrap();
        assert!(part_of_hyphenated_compound(&tokens, word_index));
    }

    #[test]
    fn detects_simple_name_particles() {
        let tokens = tokenize("Ludwig van Beethoven");
        let word_index = tokens.iter().position(|token| token.text == "van").unwrap();
        assert!(likely_name_particle_context(&tokens, word_index));
    }

    #[test]
    fn detects_adverbial_particles() {
        let particle_at = |input: &str, word: &str| {
            let tokens = tokenize(input);
            let index = tokens.iter().position(|token| token.text == word).unwrap();
            super::likely_adverbial_particle(&tokens, index, word)
        };

        // No complement can follow.
        assert!(particle_at("come up and see me", "up"));
        assert!(particle_at("give up, move on", "up"));
        assert!(particle_at("we're in for stormy weather", "in"));
        // "so" can open a noun phrase, so it is not a no-complement signal.
        assert!(!particle_at("lost in so many ways", "in"));
        // Phrasal-verb head precedes, object follows.
        assert!(particle_at("turn off the lights", "off"));
        // An object pronoun may sit between verb and particle.
        assert!(particle_at("wake me up before dawn", "up"));
        // Plain prepositional uses.
        assert!(!particle_at("walking down the street", "down"));
        assert!(!particle_at("the wind in the willows", "in"));
        // Hyphenated compounds keep their own rules.
        assert!(!particle_at("warm-up routine", "up"));
    }

    #[test]
    fn rejects_particles_next_to_small_words() {
        let tokens = tokenize("riding the van to victory");
        let word_index = tokens.iter().position(|token| token.text == "van").unwrap();
        assert!(!likely_name_particle_context(&tokens, word_index));
    }
}
