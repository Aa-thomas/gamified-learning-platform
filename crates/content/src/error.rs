use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContentError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Content not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),
}

pub type ContentResult<T> = Result<T, ContentError>;
