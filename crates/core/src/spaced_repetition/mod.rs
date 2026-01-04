//! Spaced repetition system for the learning platform
//!
//! This module provides SM-2 based spaced repetition scheduling and mastery decay.

pub mod scheduler;

pub use scheduler::{
    ReviewQuality,
    schedule_initial_review,
    is_due_now,
    get_due_reviews,
    count_due_reviews,
    calculate_next_review_date,
    score_to_quality,
    apply_mastery_decay,
    get_skills_needing_review,
};
