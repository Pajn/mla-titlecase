use crate::token::{Token, TokenKind};

pub(crate) fn is_significant_word(token: Token<'_>) -> bool {
    token.kind == TokenKind::Word
}

pub(crate) fn is_opening_punctuation(token: Token<'_>) -> bool {
    token.kind == TokenKind::Punctuation
        && matches!(
            token.text,
            "\"" | "'" | "(" | "[" | "{" | "<" | "\u{2018}" | "\u{201C}" | "«" | "‹" | "¿" | "¡"
        )
}

pub(crate) fn is_closing_punctuation(token: Token<'_>) -> bool {
    token.kind == TokenKind::Punctuation
        && matches!(
            token.text,
            "\"" | "'" | ")" | "]" | "}" | ">" | "\u{2019}" | "\u{201D}" | "»" | "›"
        )
}

pub(crate) fn is_hyphen(token: Token<'_>) -> bool {
    token.kind == TokenKind::Hyphen
}

pub(crate) fn is_apostrophe(token: Token<'_>) -> bool {
    token.kind == TokenKind::Punctuation && matches!(token.text, "'" | "\u{2019}")
}
