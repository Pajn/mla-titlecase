//! Built-in and external lexicon support.

mod abbreviations;
mod builtins;
mod external;
mod particles;
mod protected;
mod small_words;

pub(crate) use builtins::{
    abbreviation_spelling, built_in_protected_spelling, is_name_particle, is_small_word,
};
pub use external::ExternalLexicons;
