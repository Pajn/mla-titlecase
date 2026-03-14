//! Error types for plugin and lexicon operations.

/// Result type used by fallible library operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Library error variants.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The requested operation used invalid data.
    #[error("invalid data: {0}")]
    InvalidData(String),

    /// The requested operation used an unsupported format version.
    #[error("unsupported format version {found}; expected {expected}")]
    UnsupportedVersion {
        /// The version encountered in the input.
        found: u32,
        /// The version supported by the library.
        expected: u32,
    },

    /// The input checksum did not match the expected value.
    #[error("checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch {
        /// The expected checksum.
        expected: String,
        /// The checksum actually computed.
        actual: String,
    },

    /// An I/O failure occurred.
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
