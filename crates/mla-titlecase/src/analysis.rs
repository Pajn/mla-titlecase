//! Rich analysis output: per-word casing decisions and their confidence.
//!
//! [`crate::titlecase_analyze`] returns a [`TitleCaseAnalysis`] alongside the
//! cased string, recording which rule decided every word token and how much to
//! trust it (with a `changed` flag per span for callers that only want edits).

use core::ops::Range;

/// How much to trust a casing decision.
///
/// The tiers are ordered by concern for review: [`Confidence::Solid`] is safest,
/// [`Confidence::Heuristic`] most warrants a human look.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confidence {
    /// A structural MLA rule or an explicit lexicon/protected match. Not a guess.
    Solid,
    /// An ordinary word capitalized with no lexicon to confirm it is not a name
    /// or brand that should be spelled differently. Correct under MLA, but the
    /// open-world "a lexicon might disagree" case applies.
    Unverified,
    /// A heuristic that could genuinely be wrong: particle detection, all-caps
    /// word-versus-acronym classification, or a dual-role preposition.
    Heuristic,
}

impl Confidence {
    fn rank(self) -> u8 {
        match self {
            Confidence::Solid => 0,
            Confidence::Unverified => 1,
            Confidence::Heuristic => 2,
        }
    }

    /// Returns the more concerning (less confident) of two tiers.
    #[must_use]
    pub fn most_concerning(self, other: Self) -> Self {
        if other.rank() > self.rank() {
            other
        } else {
            self
        }
    }
}

/// The rule that decided a word's casing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CasingRule {
    /// Capitalized as the first significant word of the title.
    FirstWord,
    /// Capitalized as the last significant word of the title.
    LastWord,
    /// Capitalized as the first or last word across a subtitle boundary
    /// (colon, question mark, or exclamation point).
    SubtitleBoundary,
    /// Capitalized as part of a hyphenated compound.
    HyphenatedCompound,
    /// Lowercased as an MLA small word (article, conjunction, preposition).
    SmallWord,
    /// Capitalized as the adverbial particle of a phrasal verb (heuristic).
    AdverbialParticle,
    /// Lowercased as a name particle inside a likely personal name (heuristic).
    NameParticle,
    /// Lowercased as the `'n'` contraction of "and".
    ContractedAnd,
    /// Kept as a protected spelling (user, built-in, or external).
    ProtectedSpelling,
    /// Rendered from the built-in abbreviation list (e.g. `NASA`).
    Abbreviation,
    /// Rendered as a dotted initialism (e.g. `U.S.A.`).
    DottedAbbreviation,
    /// Rendered from an external canonical-spelling lexicon.
    CanonicalLexicon,
    /// Rendered from an external multiword-phrase lexicon.
    MultiwordLexicon,
    /// Recased from all-caps input to title case (heuristic).
    AllCapsNormalized,
    /// Left all-caps as a likely acronym under a dictionary-gated policy (heuristic).
    AllCapsPreservedAcronym,
    /// Left all-caps as a likely acronym among mixed-case words (heuristic).
    AcronymPreserved,
    /// Preserved because the input already had internal capitals (e.g. `iPhone`).
    MixedCasePreserved,
    /// Capitalized as an ordinary word with no lexicon confirmation.
    Capitalized,
}

impl CasingRule {
    /// The confidence for this rule, before any word-specific refinement.
    #[must_use]
    pub fn confidence(self) -> Confidence {
        match self {
            CasingRule::FirstWord
            | CasingRule::LastWord
            | CasingRule::SubtitleBoundary
            | CasingRule::HyphenatedCompound
            | CasingRule::SmallWord
            | CasingRule::ContractedAnd
            | CasingRule::ProtectedSpelling
            | CasingRule::Abbreviation
            | CasingRule::DottedAbbreviation
            | CasingRule::CanonicalLexicon
            | CasingRule::MultiwordLexicon
            | CasingRule::MixedCasePreserved => Confidence::Solid,

            CasingRule::AdverbialParticle
            | CasingRule::NameParticle
            | CasingRule::AllCapsNormalized
            | CasingRule::AllCapsPreservedAcronym
            | CasingRule::AcronymPreserved => Confidence::Heuristic,

            CasingRule::Capitalized => Confidence::Unverified,
        }
    }
}

/// One word's casing decision: the rule behind it, how much to trust it, and
/// whether it actually changed the input.
///
/// `source` and `output` are byte ranges into the original input and the
/// [`TitleCaseAnalysis::output`] string respectively.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CasingSpan {
    /// Byte range of the word in the original input.
    pub source: Range<usize>,
    /// Byte range of the word in the produced output.
    pub output: Range<usize>,
    /// The rule that decided this word's casing.
    pub rule: CasingRule,
    /// How much to trust the decision.
    pub confidence: Confidence,
    /// Whether the output text differs from the source text. `false` means the
    /// rule confirmed the input was already correct (still useful to know a
    /// heuristic was involved).
    pub changed: bool,
}

/// The result of [`crate::titlecase_analyze`]: the cased string, one span per
/// casing decision, and an overall confidence.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct TitleCaseAnalysis {
    /// The title-cased output.
    pub output: String,
    /// The least-confident tier across the recorded spans;
    /// [`Confidence::Solid`] when there are none.
    pub confidence: Confidence,
    /// One entry per casing decision, in order. Usually one span per word token,
    /// but a multiword lexicon match records a single span covering the whole
    /// phrase, and [`crate::AllCapsPolicy::Preserve`] on shouting input records
    /// none (the input is returned verbatim). Filter on [`CasingSpan::changed`]
    /// for just the edits.
    pub spans: Vec<CasingSpan>,
}
