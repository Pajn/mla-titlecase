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

/// True for punctuation that separates a title from a subtitle. MLA
/// capitalizes the first and last words of both segments, so these mark a
/// capitalization boundary on each side. Periods are deliberately excluded:
/// they are far more often part of an abbreviation than a segment break, and
/// dashes separate clauses rather than subtitles.
pub(crate) fn is_subtitle_boundary(token: Token<'_>) -> bool {
    token.kind == TokenKind::Colon
        || (token.kind == TokenKind::Punctuation && matches!(token.text, "?" | "!"))
}

pub(crate) fn is_hyphen(token: Token<'_>) -> bool {
    token.kind == TokenKind::Hyphen
}

pub(crate) fn is_apostrophe(token: Token<'_>) -> bool {
    token.kind == TokenKind::Punctuation && matches!(token.text, "'" | "\u{2019}")
}
