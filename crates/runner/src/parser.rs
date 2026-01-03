//! Parser for cargo test JSON output
//!
//! Parses the JSON output from `cargo test --message-format=json`
//! to extract test results, compile errors, and other information.

use serde::Deserialize;
use crate::types::{VerificationResult, CompileError, RuntimeError, ResourceLimit};

/// Parse cargo test output and return a VerificationResult
pub fn parse_cargo_output(output: &str, stderr: &str, duration_ms: u64) -> VerificationResult {
    let mut tests_passed = 0u32;
    let mut tests_failed = 0u32;
    let mut compile_error: Option<CompileError> = None;
    let mut build_success = true;
    let mut stdout_lines = Vec::new();

    // Parse each line of JSON output
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() || !line.starts_with('{') {
            // Collect non-JSON output for stdout
            if !line.is_empty() {
                stdout_lines.push(line.to_string());
            }
            continue;
        }

        // Try to parse as different cargo message types
        if let Ok(msg) = serde_json::from_str::<CargoMessage>(line) {
            match msg {
                CargoMessage::CompilerMessage { message } => {
                    if message.level == "error" {
                        let error = CompileError {
                            message: message.message.clone(),
                            line: message.spans.first().and_then(|s| s.line_start),
                            column: message.spans.first().and_then(|s| s.column_start),
                            file: message.spans.first().and_then(|s| s.file_name.clone()),
                        };
                        compile_error = Some(error);
                    }
                }
                CargoMessage::BuildFinished { success } => {
                    build_success = success;
                }
                CargoMessage::Test { event, .. } => {
                    match event.as_str() {
                        "ok" => tests_passed += 1,
                        "failed" => tests_failed += 1,
                        _ => {}
                    }
                }
                CargoMessage::Suite { event, passed, failed, .. } => {
                    match event.as_str() {
                        "started" => {
                            // test_count is in a separate field
                        }
                        "ok" | "failed" => {
                            if let Some(p) = passed {
                                tests_passed = p;
                            }
                            if let Some(f) = failed {
                                tests_failed = f;
                            }
                        }
                        _ => {}
                    }
                }
                CargoMessage::Unknown => {}
            }
        }
    }

    // Check for special error conditions in stderr
    let runtime_error = detect_runtime_error(stderr);
    let resource_limit = detect_resource_limit(stderr);

    // Calculate total tests
    let tests_total = tests_passed + tests_failed;

    // Handle compile error case
    if let Some(error) = compile_error {
        return VerificationResult::compile_error(error)
            .with_output(stdout_lines.join("\n"), stderr.to_string());
    }

    // Handle runtime error case
    if let Some(error) = runtime_error {
        let mut result = VerificationResult::runtime_error(error, duration_ms)
            .with_output(stdout_lines.join("\n"), stderr.to_string());
        result.resource_limit_hit = resource_limit;
        return result;
    }

    // Build success/failure result
    let success = build_success && tests_failed == 0 && tests_passed > 0;

    let mut result = if success {
        VerificationResult::success(tests_passed, tests_total, duration_ms)
    } else {
        VerificationResult::failure(tests_passed, tests_failed, tests_total, duration_ms)
    };

    result.stdout = stdout_lines.join("\n");
    result.stderr = stderr.to_string();
    result.resource_limit_hit = resource_limit;

    result
}

/// Detect runtime errors from stderr content
fn detect_runtime_error(stderr: &str) -> Option<RuntimeError> {
    // Check for panic
    if stderr.contains("panicked at") {
        // Try to extract panic message
        let message = extract_panic_message(stderr);
        return Some(RuntimeError::Panic { message });
    }

    // Check for timeout indicators
    if stderr.contains("timeout") || stderr.contains("SIGKILL") {
        return Some(RuntimeError::Timeout);
    }

    // Check for OOM
    if stderr.contains("out of memory") || stderr.contains("memory allocation") || stderr.contains("Cannot allocate memory") {
        return Some(RuntimeError::OutOfMemory);
    }

    None
}

/// Detect resource limit violations from stderr
fn detect_resource_limit(stderr: &str) -> Option<ResourceLimit> {
    if stderr.contains("OOMKilled") || stderr.contains("out of memory") || stderr.contains("Cannot allocate memory") {
        return Some(ResourceLimit::Memory);
    }

    if stderr.contains("pids limit") || stderr.contains("fork: Resource temporarily unavailable") {
        return Some(ResourceLimit::ProcessCount);
    }

    None
}

/// Extract panic message from stderr
fn extract_panic_message(stderr: &str) -> String {
    for line in stderr.lines() {
        if line.contains("panicked at") {
            // Format: thread 'main' panicked at 'message', src/main.rs:10:5
            if let Some(start) = line.find("panicked at") {
                let after_panicked = &line[start + 12..];
                // Try to extract the message in quotes
                if let Some(quote_start) = after_panicked.find('\'') {
                    let rest = &after_panicked[quote_start + 1..];
                    if let Some(quote_end) = rest.find('\'') {
                        return rest[..quote_end].to_string();
                    }
                }
                return after_panicked.trim().to_string();
            }
        }
    }
    "Unknown panic".to_string()
}

/// Cargo JSON message types
#[derive(Debug, Deserialize)]
#[serde(tag = "reason")]
enum CargoMessage {
    #[serde(rename = "compiler-message")]
    CompilerMessage { message: CompilerDiagnostic },

    #[serde(rename = "build-finished")]
    BuildFinished { success: bool },

#[serde(rename = "test")]
    Test { 
        #[allow(dead_code)]
        name: String,
        event: String,
    },

#[serde(rename = "suite")]
    Suite { 
        event: String,
        #[serde(default)]
        passed: Option<u32>,
        #[serde(default)]
        failed: Option<u32>,
        #[serde(default)]
        #[allow(dead_code)]
        ignored: Option<u32>,
    },

    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
struct CompilerDiagnostic {
    message: String,
    level: String,
    #[serde(default)]
    spans: Vec<DiagnosticSpan>,
}

#[derive(Debug, Deserialize)]
struct DiagnosticSpan {
    file_name: Option<String>,
    line_start: Option<u32>,
    column_start: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_successful_tests() {
        let output = r#"{"reason":"suite","event":"started","test_count":3}
{"reason":"test","name":"test_add","event":"started"}
{"reason":"test","name":"test_add","event":"ok"}
{"reason":"test","name":"test_sub","event":"started"}
{"reason":"test","name":"test_sub","event":"ok"}
{"reason":"test","name":"test_mul","event":"started"}
{"reason":"test","name":"test_mul","event":"ok"}
{"reason":"suite","event":"ok","passed":3,"failed":0,"ignored":0}"#;

        let result = parse_cargo_output(output, "", 1000);
        
        assert!(result.success);
        assert_eq!(result.tests_passed, 3);
        assert_eq!(result.tests_failed, 0);
        assert!(result.compile_error.is_none());
    }

    #[test]
    fn test_parse_failed_tests() {
        let output = r#"{"reason":"suite","event":"started","test_count":3}
{"reason":"test","name":"test_add","event":"started"}
{"reason":"test","name":"test_add","event":"ok"}
{"reason":"test","name":"test_sub","event":"started"}
{"reason":"test","name":"test_sub","event":"failed"}
{"reason":"test","name":"test_mul","event":"started"}
{"reason":"test","name":"test_mul","event":"ok"}
{"reason":"suite","event":"failed","passed":2,"failed":1,"ignored":0}"#;

        let result = parse_cargo_output(output, "", 1000);
        
        assert!(!result.success);
        assert_eq!(result.tests_passed, 2);
        assert_eq!(result.tests_failed, 1);
    }

    #[test]
    fn test_parse_compile_error() {
        let output = r#"{"reason":"compiler-message","message":{"message":"expected `;`","level":"error","spans":[{"file_name":"src/lib.rs","line_start":10,"column_start":5}]}}"#;

        let result = parse_cargo_output(output, "", 0);
        
        assert!(!result.success);
        assert!(result.compile_error.is_some());
        let error = result.compile_error.unwrap();
        assert!(error.message.contains("expected"));
        assert_eq!(error.line, Some(10));
        assert_eq!(error.column, Some(5));
    }

    #[test]
    fn test_detect_panic() {
        let stderr = "thread 'main' panicked at 'assertion failed: x == 5', src/lib.rs:15:5";
        let result = parse_cargo_output("", stderr, 1000);
        
        assert!(!result.success);
        assert!(result.runtime_error.is_some());
        match result.runtime_error.unwrap() {
            RuntimeError::Panic { message } => {
                assert!(message.contains("assertion failed"));
            }
            _ => panic!("Expected Panic error"),
        }
    }

    #[test]
    fn test_detect_timeout() {
        let stderr = "Process killed due to timeout after 30s";
        let result = parse_cargo_output("", stderr, 30000);
        
        assert!(!result.success);
        assert!(result.runtime_error.is_some());
        assert!(matches!(result.runtime_error.unwrap(), RuntimeError::Timeout));
    }

    #[test]
    fn test_detect_oom() {
        let stderr = "Cannot allocate memory";
        let result = parse_cargo_output("", stderr, 1000);
        
        assert!(!result.success);
        assert!(result.runtime_error.is_some());
        assert!(matches!(result.runtime_error.unwrap(), RuntimeError::OutOfMemory));
        assert!(matches!(result.resource_limit_hit, Some(ResourceLimit::Memory)));
    }

    #[test]
    fn test_extract_panic_message() {
        let stderr = "thread 'main' panicked at 'index out of bounds: the len is 3 but the index is 5', src/lib.rs:10:5";
        let message = extract_panic_message(stderr);
        assert!(message.contains("index out of bounds"));
    }

    #[test]
    fn test_parse_mixed_output() {
        // Some non-JSON lines mixed with JSON
        let output = r#"Compiling foo v0.1.0
{"reason":"build-finished","success":true}
{"reason":"suite","event":"started","test_count":2}
Running unittests
{"reason":"test","name":"test_one","event":"ok"}
{"reason":"test","name":"test_two","event":"ok"}
{"reason":"suite","event":"ok","passed":2,"failed":0,"ignored":0}"#;

        let result = parse_cargo_output(output, "", 1000);
        
        assert!(result.success);
        assert_eq!(result.tests_passed, 2);
        // Non-JSON lines should be in stdout
        assert!(result.stdout.contains("Compiling foo"));
    }

    #[test]
    fn test_empty_output() {
        let result = parse_cargo_output("", "", 0);
        
        assert!(!result.success);
        assert_eq!(result.tests_passed, 0);
        assert_eq!(result.tests_failed, 0);
    }
}
