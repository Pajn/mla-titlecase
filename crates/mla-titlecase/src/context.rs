use crate::classify::{
    is_closing_punctuation, is_hyphen, is_opening_punctuation, is_significant_word,
};
use crate::lexicon::is_small_word;
use crate::token::{Token, TokenKind};
use crate::util::normalize::lookup_key;

pub(crate) fn first_significant_word(tokens: &[Token<'_>]) -> Option<usize> {
    tokens.iter().position(|token| is_significant_word(*token))
}

pub(crate) fn last_significant_word(tokens: &[Token<'_>]) -> Option<usize> {
    tokens.iter().rposition(|token| is_significant_word(*token))
}

pub(crate) fn follows_colon(tokens: &[Token<'_>], index: usize) -> bool {
    let mut cursor = index;
    while cursor > 0 {
        cursor -= 1;
        let token = tokens[cursor];
        if token.is_word() {
            return false;
        }
        if token.kind == TokenKind::Colon {
            return true;
        }
        if !token.text.chars().all(char::is_whitespace) && !is_opening_punctuation(token) {
            return false;
        }
    }
    false
}

pub(crate) fn precedes_colon(tokens: &[Token<'_>], index: usize) -> bool {
    for token in &tokens[index + 1..] {
        if token.is_word() {
            return false;
        }
        if token.kind == TokenKind::Colon {
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

pub(crate) fn preceded_by_hyphen(tokens: &[Token<'_>], index: usize) -> bool {
    tokens.get(index.wrapping_sub(1)).is_some_and(|token| is_hyphen(*token))
}

pub(crate) fn followed_by_hyphen(tokens: &[Token<'_>], index: usize) -> bool {
    tokens.get(index + 1).is_some_and(|token| is_hyphen(*token))
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
    word.is_some_and(|token| !is_small_word(&lookup_key(token.text)))
}

fn previous_word<'a>(tokens: &'a [Token<'a>], index: usize) -> Option<Token<'a>> {
    tokens[..index].iter().rev().copied().find(|token| token.is_word())
}

fn next_word<'a>(tokens: &'a [Token<'a>], index: usize) -> Option<Token<'a>> {
    tokens[index + 1..].iter().copied().find(|token| token.is_word())
}

#[cfg(test)]
mod tests {
    use super::{follows_colon, likely_name_particle_context, part_of_hyphenated_compound};
    use crate::tokenizer::tokenize;

    #[test]
    fn detects_colon_context() {
        let tokens = tokenize("Title: the sequel");
        let word_index = tokens.iter().position(|token| token.text == "the").unwrap();
        assert!(follows_colon(&tokens, word_index));
    }

    #[test]
    fn detects_word_preceding_colon() {
        let tokens = tokenize("made of: a study");
        let word_index = tokens.iter().position(|token| token.text == "of").unwrap();
        assert!(super::precedes_colon(&tokens, word_index));
        let word_index = tokens.iter().position(|token| token.text == "made").unwrap();
        assert!(!super::precedes_colon(&tokens, word_index));
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
    fn rejects_particles_next_to_small_words() {
        let tokens = tokenize("riding the van to victory");
        let word_index = tokens.iter().position(|token| token.text == "van").unwrap();
        assert!(!likely_name_particle_context(&tokens, word_index));
    }
}
