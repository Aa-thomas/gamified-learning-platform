//! Error types for the Docker runner

use thiserror::Error;

/// Errors that can occur during Docker-based code verification
#[derive(Debug, Error)]
pub enum RunnerError {
    #[error("Docker is not installed or not running")]
    DockerNotAvailable,

    #[error("Docker image not found: {0}")]
    ImageNotFound(String),

    #[error("Failed to create container: {0}")]
    ContainerCreationFailed(String),

    #[error("Container execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Failed to cleanup container: {0}")]
    CleanupFailed(String),

    #[error("Timeout exceeded: {0}s")]
    Timeout(u64),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Docker API error: {0}")]
    Docker(String),

    #[error("Failed to parse output: {0}")]
    ParseError(String),
}

impl From<bollard::errors::Error> for RunnerError {
    fn from(err: bollard::errors::Error) -> Self {
        match &err {
            bollard::errors::Error::DockerResponseServerError { status_code, message } => {
                if *status_code == 404 {
                    RunnerError::ImageNotFound(message.clone())
                } else {
                    RunnerError::Docker(message.clone())
                }
            }
            _ => RunnerError::Docker(err.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = RunnerError::DockerNotAvailable;
        assert_eq!(err.to_string(), "Docker is not installed or not running");

        let err = RunnerError::Timeout(30);
        assert_eq!(err.to_string(), "Timeout exceeded: 30s");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let runner_err: RunnerError = io_err.into();
        assert!(matches!(runner_err, RunnerError::Io(_)));
    }
}
