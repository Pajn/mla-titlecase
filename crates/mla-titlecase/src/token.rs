#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TokenKind {
    Word,
    Whitespace,
    Hyphen,
    Slash,
    Colon,
    Punctuation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Token<'a> {
    pub(crate) kind: TokenKind,
    pub(crate) text: &'a str,
}

impl<'a> Token<'a> {
    pub(crate) const fn new(kind: TokenKind, text: &'a str) -> Self {
        Self { kind, text }
    }

    pub(crate) const fn is_word(self) -> bool {
        matches!(self.kind, TokenKind::Word)
    }
}
