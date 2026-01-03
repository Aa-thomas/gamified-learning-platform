//! LLM-based artifact grading
//!
//! This crate provides functionality to grade student artifacts
//! (DESIGN.md, README.md, etc.) using GPT-4 with caching.

pub mod error;
pub mod cache;
pub mod rubrics;
pub mod llm;
pub mod types;

pub use error::GraderError;
pub use cache::GradeCache;
pub use rubrics::Rubric;
pub use llm::LLMGrader;
pub use types::{GradeResult, CategoryScore};
