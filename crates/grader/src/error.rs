//! Error types for the LLM grader

use thiserror::Error;

/// Errors that can occur during LLM-based grading
#[derive(Debug, Error)]
pub enum GraderError {
    #[error("OpenAI API error: {0}")]
    ApiError(String),

    #[error("Rate limit exceeded. Retry after {0}s")]
    RateLimit(u64),

    #[error("Request timeout after {0}s")]
    Timeout(u64),

    #[error("Failed to parse LLM response: {0}")]
    ParseError(String),

    #[error("Invalid artifact: {0}")]
    InvalidArtifact(String),

    #[error("Rubric not found: {0}")]
    RubricNotFound(String),

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
}

impl From<async_openai::error::OpenAIError> for GraderError {
    fn from(err: async_openai::error::OpenAIError) -> Self {
        match &err {
            async_openai::error::OpenAIError::ApiError(api_err) => {
                if api_err.message.contains("rate limit") {
                    // Try to extract retry time (default to 60s)
                    GraderError::RateLimit(60)
                } else {
                    GraderError::ApiError(api_err.message.clone())
                }
            }
            _ => GraderError::ApiError(err.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = GraderError::RateLimit(60);
        assert_eq!(err.to_string(), "Rate limit exceeded. Retry after 60s");

        let err = GraderError::ParseError("invalid JSON".to_string());
        assert_eq!(err.to_string(), "Failed to parse LLM response: invalid JSON");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let grader_err: GraderError = io_err.into();
        assert!(matches!(grader_err, GraderError::Io(_)));
    }
}
