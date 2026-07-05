//! MLA-style title casing for Rust.
//!
//! The library keeps a built-in MLA rule engine as the authoritative source of
//! small-word behavior. Optional external lexicons can add canonical spellings,
//! protected words, and word-set heuristics without changing the default MLA
//! semantics.
//!
//! # Examples
//!
//! ```
//! use mla_titlecase::titlecase_mla;
//!
//! assert_eq!(titlecase_mla("the wind in the willows"), "The Wind in the Willows");
//! ```

pub mod analysis;
pub mod config;
pub mod error;
pub mod fst_store;
pub mod json_store;
pub mod lexicon;
pub mod plugin;
pub mod util;

mod casing;
mod classify;
mod context;
mod locale;
mod rules;
mod titlecase;
mod token;
mod tokenizer;

pub use analysis::{CasingRule, CasingSpan, Confidence, TitleCaseAnalysis};
pub use config::{
    AllCapsPolicy, HyphenStyle, LocaleProfile, NameParticlePolicy, SmallWordPolicy,
    TitleCaseOptions, UnknownWordCasing,
};
pub use error::{Error, Result};
pub use lexicon::ExternalLexicons;
pub use plugin::{
    LexiconPlugin, MapEntry, PluginMetadata, PluginPayload, PluginPayloadKind, RankedEntry,
    PLUGIN_SCHEMA_VERSION,
};

/// Converts `input` to MLA-style title case using the default options.
#[must_use]
pub fn titlecase_mla(input: &str) -> String {
    titlecase::titlecase_mla(input)
}

/// Converts `input` to MLA-style title case using explicit options.
#[must_use]
pub fn titlecase_with_options(input: &str, options: &TitleCaseOptions<'_>) -> String {
    titlecase::titlecase_with_options(input, options)
}

/// Converts `input` to MLA-style title case, writing the result into `out`.
///
/// `out` is cleared first, so its previous contents are replaced. Reusing one
/// buffer across many calls avoids allocating a fresh `String` per title, which
/// matters when processing large batches. Only the output buffer is reused:
/// each call still allocates a `Vec` of tokens internally, so this reclaims the
/// result allocation, not every allocation (see the
/// `titlecase_batch_reused_buffer` benchmark).
///
/// # Examples
///
/// ```
/// use mla_titlecase::{titlecase_into, TitleCaseOptions};
///
/// let options = TitleCaseOptions::default();
/// let mut buffer = String::new();
/// for title in ["the wind in the willows", "love in the time of cholera"] {
///     titlecase_into(&mut buffer, title, &options);
///     // use `buffer` here
/// }
/// ```
pub fn titlecase_into(out: &mut String, input: &str, options: &TitleCaseOptions<'_>) {
    titlecase::titlecase_into(out, input, options);
}

/// Converts `input` to MLA-style title case with the default options and returns
/// a [`TitleCaseAnalysis`]: the cased string plus one span per casing decision,
/// in order. That is usually one span per word, but a multiword lexicon match
/// records a single span for the whole phrase, and all-caps input under
/// [`AllCapsPolicy::Preserve`] records none. Each span carries the rule and
/// confidence behind its casing; filter on [`CasingSpan::changed`] for only the
/// words the engine modified.
///
/// # Examples
///
/// ```
/// use mla_titlecase::{titlecase_analyze, CasingRule, Confidence};
///
/// let analysis = titlecase_analyze("turn off the lights");
/// assert_eq!(analysis.output, "Turn Off the Lights");
/// // The phrasal-verb particle is a heuristic, so the title is flagged.
/// assert_eq!(analysis.confidence, Confidence::Heuristic);
/// assert!(analysis.spans.iter().any(|span| span.rule == CasingRule::AdverbialParticle));
/// ```
#[must_use]
pub fn titlecase_analyze(input: &str) -> TitleCaseAnalysis {
    titlecase::titlecase_analyze(input, &TitleCaseOptions::default())
}

/// Converts `input` to MLA-style title case using explicit options and returns a
/// [`TitleCaseAnalysis`]. See [`titlecase_analyze`].
#[must_use]
pub fn titlecase_analyze_with_options(
    input: &str,
    options: &TitleCaseOptions<'_>,
) -> TitleCaseAnalysis {
    titlecase::titlecase_analyze(input, options)
}
