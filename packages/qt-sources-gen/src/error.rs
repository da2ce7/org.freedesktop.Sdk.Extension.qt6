use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid version format (expected X.Y.Z): {0}")]
    InvalidVersion(String),

    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),

    #[error("HTTP {status} for {url}")]
    HttpStatus { url: String, status: reqwest::StatusCode },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("MD5 mismatch for {filename}: expected {expected}, got {actual}")]
    Md5Mismatch {
        filename: String,
        expected: String,
        actual: String,
    },

    #[error("SHA-512 mismatch for {filename}")]
    Sha512Mismatch { filename: String },

    #[error("{count} module(s) failed verification")]
    VerificationFailed { count: usize },
}
