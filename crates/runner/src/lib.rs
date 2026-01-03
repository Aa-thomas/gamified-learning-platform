//! Docker-based code verification runner
//!
//! This crate provides functionality to safely execute student code
//! in isolated Docker containers for verification.

pub mod error;
pub mod parser;
pub mod types;
pub mod docker;
pub mod pool;

pub use error::RunnerError;
pub use types::{DockerConfig, VerificationResult, CompileError, RuntimeError, ResourceLimit};
pub use docker::DockerRunner;
pub use pool::ContainerPool;
