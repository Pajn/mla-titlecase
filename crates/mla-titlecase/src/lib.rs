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
mod rules;
mod titlecase;
mod token;
mod tokenizer;

pub use config::{
    HyphenStyle, LocaleProfile, NameParticlePolicy, SmallWordPolicy, TitleCaseOptions,
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
