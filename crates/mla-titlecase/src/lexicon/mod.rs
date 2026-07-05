//! Built-in and external lexicon support.

mod abbreviations;
mod builtins;
mod external;
mod particles;
mod phrasal;
mod protected;
mod small_words;

pub(crate) use builtins::{
    abbreviation_spelling, built_in_protected_spelling, is_name_particle_for_locale, is_small_word,
};
pub use external::ExternalLexicons;
pub(crate) use phrasal::{is_adverbial_particle, is_phrasal_verb_pair};
