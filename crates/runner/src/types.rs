//! Core types for Docker-based code verification

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for the Docker runner
#[derive(Debug, Clone)]
pub struct DockerConfig {
    /// Docker image name to use for running challenges
    pub image_name: String,
    /// Memory limit in bytes
    pub memory_limit: u64,
    /// CPU limit (number of cores)
    pub cpu_limit: f64,
    /// Maximum execution time
    pub timeout: Duration,
    /// Network mode for the container
    pub network_mode: NetworkMode,
    /// Number of pre-warmed containers to keep in pool
    pub pre_warm_pool_size: usize,
}

impl Default for DockerConfig {
    fn default() -> Self {
        Self {
            image_name: "gamified-rust-sandbox:latest".to_string(),
            memory_limit: 256 * 1024 * 1024, // 256MB
            cpu_limit: 1.0,
            timeout: Duration::from_secs(30),
            network_mode: NetworkMode::None,
            pre_warm_pool_size: 2,
        }
    }
}

/// Network mode for Docker containers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkMode {
    /// No network access (most secure)
    None,
    /// Bridge network (for future HTTP whitelist)
    Bridge,
}

impl NetworkMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            NetworkMode::None => "none",
            NetworkMode::Bridge => "bridge",
        }
    }
}

/// Result of running a challenge verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// Whether all tests passed
    pub success: bool,
    /// Standard output from the test run
    pub stdout: String,
    /// Standard error from the test run
    pub stderr: String,
    /// Duration of the test run in milliseconds
    pub duration_ms: u64,
    /// Number of tests that passed
    pub tests_passed: u32,
    /// Number of tests that failed
    pub tests_failed: u32,
    /// Total number of tests
    pub tests_total: u32,
    /// Compile error if any
    pub compile_error: Option<CompileError>,
    /// Runtime error if any
    pub runtime_error: Option<RuntimeError>,
    /// Resource limit that was hit, if any
    pub resource_limit_hit: Option<ResourceLimit>,
}

impl VerificationResult {
    /// Create a successful result
    pub fn success(tests_passed: u32, tests_total: u32, duration_ms: u64) -> Self {
        Self {
            success: true,
            stdout: String::new(),
            stderr: String::new(),
            duration_ms,
            tests_passed,
            tests_failed: 0,
            tests_total,
            compile_error: None,
            runtime_error: None,
            resource_limit_hit: None,
        }
    }

    /// Create a failed result
    pub fn failure(tests_passed: u32, tests_failed: u32, tests_total: u32, duration_ms: u64) -> Self {
        Self {
            success: false,
            stdout: String::new(),
            stderr: String::new(),
            duration_ms,
            tests_passed,
            tests_failed,
            tests_total,
            compile_error: None,
            runtime_error: None,
            resource_limit_hit: None,
        }
    }

    /// Create a compile error result
    pub fn compile_error(error: CompileError) -> Self {
        Self {
            success: false,
            stdout: String::new(),
            stderr: String::new(),
            duration_ms: 0,
            tests_passed: 0,
            tests_failed: 0,
            tests_total: 0,
            compile_error: Some(error),
            runtime_error: None,
            resource_limit_hit: None,
        }
    }

    /// Create a runtime error result
    pub fn runtime_error(error: RuntimeError, duration_ms: u64) -> Self {
        Self {
            success: false,
            stdout: String::new(),
            stderr: String::new(),
            duration_ms,
            tests_passed: 0,
            tests_failed: 0,
            tests_total: 0,
            compile_error: None,
            runtime_error: Some(error),
            resource_limit_hit: None,
        }
    }

    /// Add output to the result
    pub fn with_output(mut self, stdout: String, stderr: String) -> Self {
        self.stdout = stdout;
        self.stderr = stderr;
        self
    }
}

/// Compile error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileError {
    /// Error message
    pub message: String,
    /// Line number where the error occurred
    pub line: Option<u32>,
    /// Column number where the error occurred
    pub column: Option<u32>,
    /// File where the error occurred
    pub file: Option<String>,
}

impl CompileError {
    pub fn new(message: String) -> Self {
        Self {
            message,
            line: None,
            column: None,
            file: None,
        }
    }

    pub fn with_location(mut self, line: u32, column: u32, file: String) -> Self {
        self.line = Some(line);
        self.column = Some(column);
        self.file = Some(file);
        self
    }
}

/// Runtime error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuntimeError {
    /// Code timed out
    Timeout,
    /// Code panicked
    Panic { message: String },
    /// Code ran out of memory
    OutOfMemory,
    /// Unknown runtime error
    Unknown { stderr: String },
}

/// Resource limits that can be hit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceLimit {
    /// Memory limit exceeded
    Memory,
    /// CPU limit exceeded
    Cpu,
    /// Disk space limit exceeded
    DiskSpace,
    /// Process count limit exceeded (fork bomb protection)
    ProcessCount,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docker_config_default() {
        let config = DockerConfig::default();
        assert_eq!(config.memory_limit, 256 * 1024 * 1024);
        assert_eq!(config.cpu_limit, 1.0);
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.network_mode, NetworkMode::None);
    }

    #[test]
    fn test_verification_result_success() {
        let result = VerificationResult::success(5, 5, 1000);
        assert!(result.success);
        assert_eq!(result.tests_passed, 5);
        assert_eq!(result.tests_failed, 0);
        assert!(result.compile_error.is_none());
    }

    #[test]
    fn test_verification_result_failure() {
        let result = VerificationResult::failure(3, 2, 5, 1000);
        assert!(!result.success);
        assert_eq!(result.tests_passed, 3);
        assert_eq!(result.tests_failed, 2);
    }

    #[test]
    fn test_compile_error_with_location() {
        let error = CompileError::new("expected `;`".to_string())
            .with_location(10, 5, "src/lib.rs".to_string());
        assert_eq!(error.line, Some(10));
        assert_eq!(error.column, Some(5));
        assert_eq!(error.file, Some("src/lib.rs".to_string()));
    }

    #[test]
    fn test_network_mode_as_str() {
        assert_eq!(NetworkMode::None.as_str(), "none");
        assert_eq!(NetworkMode::Bridge.as_str(), "bridge");
    }
}
