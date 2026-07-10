//! Shared normalization helpers.

// `normalize` is shared with the CLI crate; `unicode` is library-internal.
pub mod normalize;
pub(crate) mod unicode;
