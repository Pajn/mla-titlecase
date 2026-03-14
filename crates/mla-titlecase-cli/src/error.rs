use std::path::PathBuf;

pub(crate) type Result<T> = std::result::Result<T, CliError>;

#[derive(Debug, thiserror::Error)]
pub(crate) enum CliError {
    #[error(transparent)]
    Library(#[from] mla_titlecase::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error("source {0} requires --acknowledge-cc-by-sa")]
    MissingAcknowledgement(&'static str),

    #[error("unsupported source input at {path}: {message}")]
    UnsupportedInput { path: PathBuf, message: String },

    #[error("unsupported file format for {0}")]
    UnsupportedFormat(PathBuf),
}
