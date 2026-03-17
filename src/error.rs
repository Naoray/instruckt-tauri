use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Base64 decode: {0}")]
    Base64(#[from] base64::DecodeError),

    #[error("Annotation not found: {0}")]
    NotFound(String),

    #[error("Mutex poisoned: {0}")]
    MutexPoisoned(String),

    #[error("{0}")]
    Other(String),
}

impl Serialize for Error {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
