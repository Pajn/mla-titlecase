use crate::token::{Token, TokenKind};

pub(crate) fn is_significant_word(token: Token<'_>) -> bool {
    token.kind == TokenKind::Word
}

pub(crate) fn is_opening_punctuation(token: Token<'_>) -> bool {
    token.kind == TokenKind::Punctuation && matches!(token.text, "\"" | "'" | "(" | "[" | "{" | "<")
}

pub(crate) fn is_hyphen(token: Token<'_>) -> bool {
    token.kind == TokenKind::Hyphen
}
