//! Docker-based code verification using bollard
//!
//! Provides a safe, sandboxed environment for executing student code.

use bollard::container::{
    Config, CreateContainerOptions, LogOutput, LogsOptions, RemoveContainerOptions,
    StartContainerOptions, WaitContainerOptions,
};
use bollard::models::{HostConfig, Mount, MountTypeEnum};
use bollard::Docker;
use futures::StreamExt;
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;
use tokio::time::timeout;
use uuid::Uuid;

use crate::error::RunnerError;
use crate::parser::parse_cargo_output;
use crate::types::{DockerConfig, RuntimeError, VerificationResult};

/// Docker-based code runner
pub struct DockerRunner {
    docker: Docker,
    config: DockerConfig,
}

impl DockerRunner {
    /// Create a new Docker runner with default configuration
    pub async fn new() -> Result<Self, RunnerError> {
        Self::with_config(DockerConfig::default()).await
    }

    /// Create a new Docker runner with custom configuration
    pub async fn with_config(config: DockerConfig) -> Result<Self, RunnerError> {
        let docker = Docker::connect_with_local_defaults()
            .map_err(|_| RunnerError::DockerNotAvailable)?;

        // Verify Docker is running
        docker.ping().await.map_err(|_| RunnerError::DockerNotAvailable)?;

        Ok(Self { docker, config })
    }

    /// Check if Docker is available
    pub async fn check_available() -> Result<bool, RunnerError> {
        let docker = Docker::connect_with_local_defaults()
            .map_err(|_| RunnerError::DockerNotAvailable)?;

        match docker.ping().await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Check if the sandbox image exists
    pub async fn check_image_exists(&self) -> bool {
        self.docker
            .inspect_image(&self.config.image_name)
            .await
            .is_ok()
    }

    /// Run verification for a challenge
    pub async fn run_verification(
        &self,
        challenge_dir: &Path,
        student_code: &str,
    ) -> Result<VerificationResult, RunnerError> {
        let start = Instant::now();

        // Create a temporary directory for the challenge
        let temp_dir = tempfile::tempdir()?;
        let work_dir = temp_dir.path();

        // Copy challenge files and write student code
        self.prepare_challenge_dir(challenge_dir, work_dir, student_code)?;

        // Generate unique container name
        let container_name = format!("challenge-{}", Uuid::new_v4());

        // Create and run container
        let result = self
            .run_container(&container_name, work_dir, start)
            .await;

        // Cleanup container (best effort)
        let _ = self.cleanup_container(&container_name).await;

        result
    }

    /// Prepare the challenge directory with student code
    fn prepare_challenge_dir(
        &self,
        challenge_dir: &Path,
        work_dir: &Path,
        student_code: &str,
    ) -> Result<(), RunnerError> {
        // Copy challenge template files
        if challenge_dir.exists() {
            copy_dir_recursive(challenge_dir, work_dir)?;
        }

        // Write student code to src/lib.rs
        let src_dir = work_dir.join("src");
        std::fs::create_dir_all(&src_dir)?;
        std::fs::write(src_dir.join("lib.rs"), student_code)?;

        Ok(())
    }

    /// Run the container and collect results
    async fn run_container(
        &self,
        container_name: &str,
        work_dir: &Path,
        start: Instant,
    ) -> Result<VerificationResult, RunnerError> {
        // Container configuration
        let host_config = HostConfig {
            memory: Some(self.config.memory_limit as i64),
            nano_cpus: Some((self.config.cpu_limit * 1_000_000_000.0) as i64),
            network_mode: Some(self.config.network_mode.as_str().to_string()),
            pids_limit: Some(100), // Prevent fork bombs
            readonly_rootfs: Some(true),
            mounts: Some(vec![
                Mount {
                    target: Some("/challenge".to_string()),
                    source: Some(work_dir.to_string_lossy().to_string()),
                    typ: Some(MountTypeEnum::BIND),
                    read_only: Some(false), // Need write for cargo build
                    ..Default::default()
                },
            ]),
            ..Default::default()
        };

        let config = Config {
            image: Some(self.config.image_name.clone()),
            cmd: Some(vec![
                "cargo".to_string(),
                "test".to_string(),
                "--message-format=json".to_string(),
            ]),
            working_dir: Some("/challenge".to_string()),
            host_config: Some(host_config),
            labels: Some({
                let mut labels = HashMap::new();
                labels.insert("app".to_string(), "gamified-rust-challenge".to_string());
                labels
            }),
            ..Default::default()
        };

        // Create container
        let create_opts = CreateContainerOptions {
            name: container_name,
            platform: None,
        };

        self.docker
            .create_container(Some(create_opts), config)
            .await
            .map_err(|e| RunnerError::ContainerCreationFailed(e.to_string()))?;

        // Start container
        self.docker
            .start_container(container_name, None::<StartContainerOptions<String>>)
            .await
            .map_err(|e| RunnerError::ExecutionFailed(e.to_string()))?;

        // Wait for container with timeout
        let wait_result = timeout(self.config.timeout, self.wait_for_container(container_name)).await;

        let duration_ms = start.elapsed().as_millis() as u64;

        match wait_result {
            Ok(Ok((stdout, stderr, exit_code))) => {
                // Parse the output
                let mut result = parse_cargo_output(&stdout, &stderr, duration_ms);
                
                // Check for OOM kill (exit code 137)
                if exit_code == 137 {
                    result.runtime_error = Some(RuntimeError::OutOfMemory);
                    result.success = false;
                }

                Ok(result)
            }
            Ok(Err(e)) => Err(e),
            Err(_) => {
                // Timeout - kill container
                let _ = self.docker.kill_container(container_name, None::<bollard::container::KillContainerOptions<String>>).await;
                
                Ok(VerificationResult::runtime_error(
                    RuntimeError::Timeout,
                    duration_ms,
                ))
            }
        }
    }

    /// Wait for container to finish and collect output
    async fn wait_for_container(
        &self,
        container_name: &str,
    ) -> Result<(String, String, i64), RunnerError> {
        // Wait for container to exit
        let mut wait_stream = self.docker.wait_container(
            container_name,
            Some(WaitContainerOptions {
                condition: "not-running",
            }),
        );

        let exit_code = match wait_stream.next().await {
            Some(Ok(response)) => response.status_code,
            Some(Err(e)) => return Err(RunnerError::ExecutionFailed(e.to_string())),
            None => return Err(RunnerError::ExecutionFailed("Container disappeared".to_string())),
        };

        // Collect logs
        let log_opts = LogsOptions::<String> {
            stdout: true,
            stderr: true,
            ..Default::default()
        };

        let mut logs = self.docker.logs(container_name, Some(log_opts));
        let mut stdout = String::new();
        let mut stderr = String::new();

        while let Some(log_result) = logs.next().await {
            match log_result {
                Ok(LogOutput::StdOut { message }) => {
                    stdout.push_str(&String::from_utf8_lossy(&message));
                }
                Ok(LogOutput::StdErr { message }) => {
                    stderr.push_str(&String::from_utf8_lossy(&message));
                }
                _ => {}
            }
        }

        Ok((stdout, stderr, exit_code))
    }

    /// Cleanup a container
    async fn cleanup_container(&self, container_name: &str) -> Result<(), RunnerError> {
        let opts = RemoveContainerOptions {
            force: true,
            ..Default::default()
        };

        self.docker
            .remove_container(container_name, Some(opts))
            .await
            .map_err(|e| RunnerError::CleanupFailed(e.to_string()))?;

        Ok(())
    }

    /// Cleanup all orphaned challenge containers
    pub async fn cleanup_orphaned_containers(&self) -> Result<usize, RunnerError> {
        use bollard::container::ListContainersOptions;

        let filters: HashMap<String, Vec<String>> = {
            let mut f = HashMap::new();
            f.insert("label".to_string(), vec!["app=gamified-rust-challenge".to_string()]);
            f
        };

        let opts = ListContainersOptions {
            all: true,
            filters,
            ..Default::default()
        };

        let containers = self.docker.list_containers(Some(opts)).await
            .map_err(|e| RunnerError::Docker(e.to_string()))?;

        let mut cleaned = 0;
        for container in containers {
            if let Some(id) = container.id {
                // Check if container is stale (created > 1 hour ago)
                if let Some(created) = container.created {
                    let now = chrono::Utc::now().timestamp();
                    if now - created > 3600 {
                        let _ = self.cleanup_container(&id).await;
                        cleaned += 1;
                    }
                }
            }
        }

        Ok(cleaned)
    }
}

/// Recursively copy a directory
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    if !dst.exists() {
        std::fs::create_dir_all(dst)?;
    }

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());

        if path.is_dir() {
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            std::fs::copy(&path, &dest_path)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_docker_available_check() {
        // This test will pass if Docker is available, skip otherwise
        match DockerRunner::check_available().await {
            Ok(available) => {
                println!("Docker available: {}", available);
            }
            Err(e) => {
                println!("Docker check failed: {}", e);
            }
        }
    }

    #[test]
    fn test_copy_dir_recursive() {
        let temp_src = tempfile::tempdir().unwrap();
        let temp_dst = tempfile::tempdir().unwrap();

        // Create some files in source
        std::fs::write(temp_src.path().join("test.txt"), "hello").unwrap();
        std::fs::create_dir(temp_src.path().join("subdir")).unwrap();
        std::fs::write(temp_src.path().join("subdir/nested.txt"), "world").unwrap();

        // Copy
        copy_dir_recursive(temp_src.path(), temp_dst.path()).unwrap();

        // Verify
        assert!(temp_dst.path().join("test.txt").exists());
        assert!(temp_dst.path().join("subdir/nested.txt").exists());
        assert_eq!(
            std::fs::read_to_string(temp_dst.path().join("test.txt")).unwrap(),
            "hello"
        );
    }
}
