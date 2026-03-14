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

    /// The payload kind cannot be encoded in the requested format.
    #[error("unsupported payload kind {kind} for {format} format")]
    UnsupportedPayload {
        /// The payload kind name.
        kind: &'static str,
        /// The format name.
        format: &'static str,
    },

    /// JSON serialization or parsing failed.
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    /// FST serialization or parsing failed.
    #[error(transparent)]
    Fst(#[from] fst::Error),

    /// UTF-8 decoding failed.
    #[error(transparent)]
    Utf8(#[from] std::string::FromUtf8Error),

    /// UTF-8 decoding from a borrowed slice failed.
    #[error(transparent)]
    StrUtf8(#[from] std::str::Utf8Error),

    /// An I/O failure occurred.
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
