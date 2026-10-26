use thiserror::Error;

#[derive(Debug, Error)]
pub enum CuError {
    #[error("windows api error: {0}")]
    Windows(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("image error: {0}")]
    Image(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("invalid argument: {0}")]
    InvalidArg(String),
    #[error("clipboard error: {0}")]
    Clipboard(String),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("utf8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, CuError>;

impl From<windows::core::Error> for CuError {
    fn from(e: windows::core::Error) -> Self {
        CuError::Windows(e.to_string())
    }
}
