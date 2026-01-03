use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Result of running a challenge verification
#[derive(Debug)]
pub struct VerificationResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub duration: Duration,
    pub tests_passed: u32,
    pub tests_failed: u32,
    pub compile_error: bool,
    pub timeout: bool,
}

/// Docker-based challenge runner
pub struct DockerRunner {
    image_name: String,
    timeout_secs: u64,
    memory_limit: String,
    cpu_limit: String,
}

impl DockerRunner {
    pub fn new() -> Self {
        Self {
            image_name: "rust-sandbox".to_string(),
            timeout_secs: 30,
            memory_limit: "256m".to_string(),
            cpu_limit: "1.0".to_string(),
        }
    }

    /// Check if Docker is available on the system
    pub fn check_docker_available() -> Result<bool, String> {
        let output = Command::new("docker")
            .arg("--version")
            .output();

        match output {
            Ok(output) => Ok(output.status.success()),
            Err(e) => Err(format!("Docker not found: {}", e)),
        }
    }

    /// Build the Docker image
    pub fn build_image(&self, dockerfile_dir: &Path) -> Result<(), String> {
        let output = Command::new("docker")
            .arg("build")
            .arg("-t")
            .arg(&self.image_name)
            .arg(dockerfile_dir)
            .output()
            .map_err(|e| format!("Failed to build Docker image: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "Docker build failed:\n{}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(())
    }

    /// Run verification for a challenge
    pub fn run_verification(
        &self,
        challenge_dir: &Path,
        student_code: &str,
    ) -> Result<VerificationResult, String> {
        let start = Instant::now();

        // Create temporary directory for this run
        let temp_dir = self.create_temp_challenge(challenge_dir, student_code)?;

        // Generate unique container name
        let container_name = format!("challenge-{}", uuid::Uuid::new_v4());

        // Run Docker container with resource limits and timeout
        let result = self.run_docker_container(&temp_dir, &container_name)?;

        // Cleanup
        self.cleanup_container(&container_name);
        self.cleanup_temp_dir(&temp_dir);

        let duration = start.elapsed();

        Ok(VerificationResult {
            success: result.success,
            stdout: result.stdout,
            stderr: result.stderr,
            duration,
            tests_passed: result.tests_passed,
            tests_failed: result.tests_failed,
            compile_error: result.compile_error,
            timeout: result.timeout,
        })
    }

    /// Create temporary directory with challenge files and student code
    fn create_temp_challenge(
        &self,
        challenge_dir: &Path,
        student_code: &str,
    ) -> Result<PathBuf, String> {
        let temp_dir = std::env::temp_dir().join(format!("challenge-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir)
            .map_err(|e| format!("Failed to create temp directory: {}", e))?;

        // Copy challenge structure
        self.copy_dir_recursive(challenge_dir, &temp_dir)?;

        // Write student code
        let lib_path = temp_dir.join("src/lib.rs");
        fs::write(&lib_path, student_code)
            .map_err(|e| format!("Failed to write student code: {}", e))?;

        Ok(temp_dir)
    }

    /// Copy directory recursively
    fn copy_dir_recursive(&self, src: &Path, dst: &Path) -> Result<(), String> {
        fs::create_dir_all(dst)
            .map_err(|e| format!("Failed to create directory: {}", e))?;

        for entry in fs::read_dir(src)
            .map_err(|e| format!("Failed to read directory: {}", e))?
        {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let file_type = entry.file_type()
                .map_err(|e| format!("Failed to get file type: {}", e))?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if file_type.is_dir() {
                self.copy_dir_recursive(&src_path, &dst_path)?;
            } else {
                fs::copy(&src_path, &dst_path)
                    .map_err(|e| format!("Failed to copy file: {}", e))?;
            }
        }

        Ok(())
    }

    /// Run Docker container with the challenge
    fn run_docker_container(
        &self,
        challenge_dir: &Path,
        container_name: &str,
    ) -> Result<DockerResult, String> {
        let output = Command::new("docker")
            .arg("run")
            .arg("--rm")
            .arg("--name")
            .arg(container_name)
            .arg("--memory")
            .arg(&self.memory_limit)
            .arg("--cpus")
            .arg(&self.cpu_limit)
            .arg("--network")
            .arg("none")
            .arg("--stop-timeout")
            .arg(self.timeout_secs.to_string())
            .arg("-v")
            .arg(format!("{}:/challenge", challenge_dir.display()))
            .arg(&self.image_name)
            .arg("sh")
            .arg("-c")
            .arg("timeout 30s cargo test --message-format=json 2>&1 || true")
            .output()
            .map_err(|e| format!("Failed to run Docker container: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        // Parse test results from JSON output
        let (tests_passed, tests_failed, compile_error) = self.parse_test_output(&stdout);

        // Check if timeout occurred
        let timeout = stdout.contains("Killed") || stderr.contains("timeout");

        Ok(DockerResult {
            success: output.status.success() && tests_failed == 0 && !compile_error,
            stdout,
            stderr,
            tests_passed,
            tests_failed,
            compile_error,
            timeout,
        })
    }

    /// Parse cargo test JSON output to extract test results
    fn parse_test_output(&self, output: &str) -> (u32, u32, bool) {
        let mut passed = 0;
        let mut failed = 0;
        let mut compile_error = false;

        for line in output.lines() {
            // Check for compile errors
            if line.contains("error:") || line.contains("could not compile") {
                compile_error = true;
            }

            // Parse JSON test results
            if line.starts_with("{") {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                    if let Some(reason) = json.get("reason").and_then(|v| v.as_str()) {
                        if reason == "test" {
                            if let Some(event) = json.get("event").and_then(|v| v.as_str()) {
                                match event {
                                    "ok" => passed += 1,
                                    "failed" => failed += 1,
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }

        // Fallback: parse summary line if JSON parsing failed
        if passed == 0 && failed == 0 && !compile_error {
            if let Some(summary) = output.lines().find(|line| line.contains("test result:")) {
                // Example: "test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"
                if let Some(passed_str) = summary.split("passed").next().and_then(|s| s.split_whitespace().last()) {
                    passed = passed_str.parse().unwrap_or(0);
                }
                if let Some(failed_str) = summary.split("failed").next().and_then(|s| s.split_whitespace().last()) {
                    failed = failed_str.parse().unwrap_or(0);
                }
            }
        }

        (passed, failed, compile_error)
    }

    /// Cleanup container (in case it's still running)
    fn cleanup_container(&self, container_name: &str) {
        let _ = Command::new("docker")
            .arg("stop")
            .arg(container_name)
            .output();

        let _ = Command::new("docker")
            .arg("rm")
            .arg("-f")
            .arg(container_name)
            .output();
    }

    /// Cleanup temporary directory
    fn cleanup_temp_dir(&self, temp_dir: &Path) {
        let _ = fs::remove_dir_all(temp_dir);
    }
}

#[derive(Debug)]
struct DockerResult {
    success: bool,
    stdout: String,
    stderr: String,
    tests_passed: u32,
    tests_failed: u32,
    compile_error: bool,
    timeout: bool,
}

// Placeholder for uuid - would normally use the uuid crate
mod uuid {
    use std::time::{SystemTime, UNIX_EPOCH};

    pub struct Uuid(String);

    impl Uuid {
        pub fn new_v4() -> Self {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            Self(format!("{:x}", timestamp))
        }
    }

    impl std::fmt::Display for Uuid {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }
}

// Placeholder for serde_json - would normally use the serde_json crate
mod serde_json {
    use std::collections::HashMap;

    #[derive(Debug)]
    pub struct Value(HashMap<String, String>);

    impl Value {
        pub fn get(&self, key: &str) -> Option<&str> {
            self.0.get(key).map(|s| s.as_str())
        }

        pub fn as_str(&self) -> Option<&str> {
            None
        }
    }

    pub fn from_str<T>(s: &str) -> Result<Value, String> {
        Err("Not implemented".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docker_runner_creation() {
        let runner = DockerRunner::new();
        assert_eq!(runner.timeout_secs, 30);
        assert_eq!(runner.memory_limit, "256m");
    }
}
