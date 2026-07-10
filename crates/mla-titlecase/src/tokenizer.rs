use crate::token::{Token, TokenKind};

pub(crate) fn tokenize(input: &str) -> Vec<Token<'_>> {
    // Rough estimate of token count (words, whitespace runs, punctuation) to
    // avoid repeated reallocations while pushing on the bulk-processing path.
    let mut tokens = Vec::with_capacity(input.len() / 4 + 1);
    let mut index = 0;

    while index < input.len() {
        let slice = &input[index..];
        let ch = slice.chars().next().expect("index always at char boundary");
        let next_index = index + ch.len_utf8();

        if ch.is_whitespace() {
            let end = consume_while(input, index, char::is_whitespace);
            tokens.push(Token::new(TokenKind::Whitespace, &input[index..end]));
            index = end;
        } else if ch.is_alphanumeric() {
            let end = scan_word(input, index);
            tokens.push(Token::new(TokenKind::Word, &input[index..end]));
            index = end;
        } else {
            let kind = match ch {
                // Only true hyphens join compounds; figure/en/em dashes
                // (U+2012..U+2014) separate clauses and stay punctuation.
                '-' | '\u{2010}' | '\u{2011}' => TokenKind::Hyphen,
                '/' => TokenKind::Slash,
                ':' => TokenKind::Colon,
                _ => TokenKind::Punctuation,
            };
            tokens.push(Token::new(kind, &input[index..next_index]));
            index = next_index;
        }
    }

    tokens
}

fn consume_while<F>(input: &str, start: usize, predicate: F) -> usize
where
    F: Fn(char) -> bool,
{
    let mut end = start;
    for (offset, ch) in input[start..].char_indices() {
        if !predicate(ch) {
            return start + offset;
        }
        end = start + offset + ch.len_utf8();
    }
    end
}

fn scan_word(input: &str, start: usize) -> usize {
    let mut end = start;
    let mut chars = input[start..].char_indices().peekable();
    let mut saw_dot_pair = false;

    while let Some((offset, ch)) = chars.next() {
        let absolute = start + offset;
        let next = chars.peek().map(|(_, value)| *value);

        let consumes = ch.is_alphanumeric()
            || is_combining_mark(ch)
            || is_apostrophe_connector(ch, absolute, input)
            || (ch == '.' && next.is_some_and(char::is_alphanumeric))
            || (ch == '.' && saw_dot_pair);

        if !consumes {
            break;
        }

        if ch == '.' && next.is_some_and(char::is_alphanumeric) {
            saw_dot_pair = true;
        }

        end = absolute + ch.len_utf8();
    }

    end
}

/// True for combining marks, which continue the word of the base character
/// they modify. Keeps decomposed (NFD) input such as `e\u{301}tude` one word.
/// Covers the dedicated combining blocks; marks never start a word because
/// [`tokenize`] only enters [`scan_word`] on an alphanumeric character.
fn is_combining_mark(ch: char) -> bool {
    matches!(
        ch,
        '\u{0300}'..='\u{036F}' // Combining Diacritical Marks
        | '\u{0483}'..='\u{0489}' // Combining Cyrillic marks
        | '\u{1AB0}'..='\u{1AFF}' // Combining Diacritical Marks Extended
        | '\u{1DC0}'..='\u{1DFF}' // Combining Diacritical Marks Supplement
        | '\u{20D0}'..='\u{20FF}' // Combining Marks for Symbols
        | '\u{FE20}'..='\u{FE2F}' // Combining Half Marks
    )
}

fn is_apostrophe_connector(ch: char, index: usize, input: &str) -> bool {
    if !matches!(ch, '\'' | '\u{2019}') {
        return false;
    }

    let is_word_char = |ch: char| ch.is_alphanumeric() || is_combining_mark(ch);
    let before = input[..index].chars().next_back();
    let after = input[index + ch.len_utf8()..].chars().next();
    before.is_some_and(is_word_char) && after.is_some_and(char::is_alphanumeric)
}

#[cfg(test)]
mod tests {
    use super::tokenize;
    use crate::token::TokenKind;

    #[test]
    fn keeps_whitespace_and_punctuation() {
        let tokens = tokenize("hello,  world: again");
        let kinds: Vec<_> = tokens.iter().map(|token| token.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Word,
                TokenKind::Punctuation,
                TokenKind::Whitespace,
                TokenKind::Word,
                TokenKind::Colon,
                TokenKind::Whitespace,
                TokenKind::Word,
            ]
        );
    }

    #[test]
    fn keeps_hyphenated_words_split() {
        let tokens = tokenize("state-of-the-art");
        let texts: Vec<_> = tokens.iter().map(|token| token.text).collect();
        assert_eq!(texts, vec!["state", "-", "of", "-", "the", "-", "art"]);
    }

    #[test]
    fn distinguishes_hyphens_from_dashes() {
        let tokens = tokenize("well-known\u{2014}a memoir");
        let kinds: Vec<_> = tokens.iter().map(|token| (token.kind, token.text)).collect();
        assert_eq!(kinds[1], (TokenKind::Hyphen, "-"));
        assert_eq!(kinds[3], (TokenKind::Punctuation, "\u{2014}"));
    }

    #[test]
    fn keeps_apostrophes_inside_words() {
        let tokens = tokenize("o'neill and rock'n'roll");
        let texts: Vec<_> =
            tokens.iter().filter(|token| token.is_word()).map(|token| token.text).collect();
        assert_eq!(texts, vec!["o'neill", "and", "rock'n'roll"]);
    }

    #[test]
    fn keeps_combining_marks_inside_words() {
        // NFD input: the combining acute continues the word it modifies.
        let tokens = tokenize("e\u{301}tude and beyonce\u{301}'s hits");
        let texts: Vec<_> =
            tokens.iter().filter(|token| token.is_word()).map(|token| token.text).collect();
        assert_eq!(texts, vec!["e\u{301}tude", "and", "beyonce\u{301}'s", "hits"]);
    }

    #[test]
    fn keeps_dotted_abbreviations_together() {
        let tokens = tokenize("U.S.A. and e.g. examples");
        let texts: Vec<_> =
            tokens.iter().filter(|token| token.is_word()).map(|token| token.text).collect();
        assert_eq!(texts, vec!["U.S.A.", "and", "e.g.", "examples"]);
    }
}
