//! Badge system for gamification
//!
//! This module provides badge definitions, tracking, and unlock logic.

pub mod definitions;
pub mod tracker;

pub use definitions::{get_all_badge_definitions, get_badge_by_id, get_badges_by_category};
pub use tracker::{check_badge_unlocks, check_single_badge, calculate_badge_progress, UserStats};
