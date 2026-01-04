//! Badge tracker - checks unlock criteria for all badges
//!
//! This module provides functionality to check which badges a user has earned
//! based on their current stats.

use crate::models::{BadgeCategory, BadgeDefinition, BadgeProgress};
use super::definitions::get_all_badge_definitions;

/// User stats used for badge evaluation
#[derive(Debug, Clone, Default)]
pub struct UserStats {
    pub streak_days: u32,
    pub level: u32,
    pub total_xp: i32,
    pub completed_lectures: u32,
    pub completed_quizzes: u32,
    pub completed_challenges: u32,
    pub total_completions: u32,
    pub perfect_quiz_count: u32,
    pub max_mastery_score: f64,
}

impl UserStats {
    /// Get the value for a specific badge category
    pub fn get_value_for_category(&self, category: &BadgeCategory) -> f64 {
        match category {
            BadgeCategory::Streak => self.streak_days as f64,
            BadgeCategory::Level => self.level as f64,
            BadgeCategory::Xp => self.total_xp as f64,
            BadgeCategory::Completion => self.total_completions as f64,
            BadgeCategory::Mastery => self.max_mastery_score,
        }
    }
}

/// Check which badges should be unlocked based on user stats
/// Returns a list of badge IDs that are newly unlocked
pub fn check_badge_unlocks(
    stats: &UserStats,
    current_progress: &[BadgeProgress],
) -> Vec<String> {
    let definitions = get_all_badge_definitions();
    let mut newly_unlocked = Vec::new();

    for badge_def in definitions {
        // Skip if already earned
        if current_progress.iter().any(|p| p.badge_id == badge_def.id && p.is_earned()) {
            continue;
        }

        // Check if badge criteria is met
        if check_single_badge(&badge_def, stats) {
            newly_unlocked.push(badge_def.id);
        }
    }

    newly_unlocked
}

/// Check if a single badge's criteria is met
pub fn check_single_badge(badge: &BadgeDefinition, stats: &UserStats) -> bool {
    match badge.category {
        BadgeCategory::Streak => stats.streak_days as f64 >= badge.threshold,
        BadgeCategory::Level => stats.level as f64 >= badge.threshold,
        BadgeCategory::Xp => stats.total_xp as f64 >= badge.threshold,
        BadgeCategory::Completion => {
            // Special handling for specific completion badges
            match badge.id.as_str() {
                "first_steps" => stats.completed_lectures >= badge.threshold as u32,
                "quiz_whiz" => stats.completed_quizzes >= badge.threshold as u32,
                "perfect_score" => stats.perfect_quiz_count >= badge.threshold as u32,
                "completionist" => stats.total_completions >= badge.threshold as u32,
                _ => stats.total_completions as f64 >= badge.threshold,
            }
        }
        BadgeCategory::Mastery => stats.max_mastery_score >= badge.threshold,
    }
}

/// Calculate badge progress as a percentage (0.0 to 1.0)
pub fn calculate_badge_progress(badge: &BadgeDefinition, stats: &UserStats) -> f64 {
    let current_value = match badge.category {
        BadgeCategory::Streak => stats.streak_days as f64,
        BadgeCategory::Level => stats.level as f64,
        BadgeCategory::Xp => stats.total_xp as f64,
        BadgeCategory::Completion => {
            match badge.id.as_str() {
                "first_steps" => stats.completed_lectures as f64,
                "quiz_whiz" => stats.completed_quizzes as f64,
                "perfect_score" => stats.perfect_quiz_count as f64,
                "completionist" => stats.total_completions as f64,
                _ => stats.total_completions as f64,
            }
        }
        BadgeCategory::Mastery => stats.max_mastery_score,
    };

    (current_value / badge.threshold).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streak_badge_unlock() {
        let stats = UserStats {
            streak_days: 7,
            ..Default::default()
        };
        let badge = BadgeDefinition {
            id: "week_warrior".to_string(),
            name: "Week Warrior".to_string(),
            description: "7-day streak".to_string(),
            icon: "🔥".to_string(),
            threshold: 7.0,
            category: BadgeCategory::Streak,
        };
        
        assert!(check_single_badge(&badge, &stats));
    }

    #[test]
    fn test_level_badge_unlock() {
        let stats = UserStats {
            level: 5,
            ..Default::default()
        };
        let badge = BadgeDefinition {
            id: "rising_star".to_string(),
            name: "Rising Star".to_string(),
            description: "Reach level 5".to_string(),
            icon: "⭐".to_string(),
            threshold: 5.0,
            category: BadgeCategory::Level,
        };
        
        assert!(check_single_badge(&badge, &stats));
    }

    #[test]
    fn test_xp_badge_unlock() {
        let stats = UserStats {
            total_xp: 1000,
            ..Default::default()
        };
        let badge = BadgeDefinition {
            id: "xp_hunter".to_string(),
            name: "XP Hunter".to_string(),
            description: "Earn 1000 XP".to_string(),
            icon: "💎".to_string(),
            threshold: 1000.0,
            category: BadgeCategory::Xp,
        };
        
        assert!(check_single_badge(&badge, &stats));
    }

    #[test]
    fn test_completion_badge_unlock() {
        let stats = UserStats {
            completed_lectures: 1,
            total_completions: 1,
            ..Default::default()
        };
        let badge = BadgeDefinition {
            id: "first_steps".to_string(),
            name: "First Steps".to_string(),
            description: "Complete first lecture".to_string(),
            icon: "👣".to_string(),
            threshold: 1.0,
            category: BadgeCategory::Completion,
        };
        
        assert!(check_single_badge(&badge, &stats));
    }

    #[test]
    fn test_mastery_badge_unlock() {
        let stats = UserStats {
            max_mastery_score: 0.9,
            ..Default::default()
        };
        let badge = BadgeDefinition {
            id: "skill_master".to_string(),
            name: "Skill Master".to_string(),
            description: "90% mastery".to_string(),
            icon: "🏅".to_string(),
            threshold: 0.9,
            category: BadgeCategory::Mastery,
        };
        
        assert!(check_single_badge(&badge, &stats));
    }

    #[test]
    fn test_no_duplicate_unlock() {
        let stats = UserStats {
            streak_days: 10,
            ..Default::default()
        };
        
        // Already earned badge
        let mut progress = BadgeProgress::new("user1".to_string(), "week_warrior".to_string());
        progress.update_progress(7.0, 7.0); // This marks it as earned
        
        let newly_unlocked = check_badge_unlocks(&stats, &[progress]);
        
        // week_warrior should not be in newly unlocked since it's already earned
        assert!(!newly_unlocked.contains(&"week_warrior".to_string()));
    }

    #[test]
    fn test_check_all_badges() {
        let stats = UserStats {
            streak_days: 7,
            level: 5,
            total_xp: 1000,
            completed_lectures: 1,
            total_completions: 1,
            ..Default::default()
        };
        
        let newly_unlocked = check_badge_unlocks(&stats, &[]);
        
        // Should unlock multiple badges
        assert!(newly_unlocked.contains(&"week_warrior".to_string()));
        assert!(newly_unlocked.contains(&"rising_star".to_string()));
        assert!(newly_unlocked.contains(&"xp_hunter".to_string()));
        assert!(newly_unlocked.contains(&"first_steps".to_string()));
    }

    #[test]
    fn test_calculate_badge_progress() {
        let stats = UserStats {
            streak_days: 3,
            ..Default::default()
        };
        let badge = BadgeDefinition {
            id: "week_warrior".to_string(),
            name: "Week Warrior".to_string(),
            description: "7-day streak".to_string(),
            icon: "🔥".to_string(),
            threshold: 7.0,
            category: BadgeCategory::Streak,
        };
        
        let progress = calculate_badge_progress(&badge, &stats);
        assert!((progress - (3.0 / 7.0)).abs() < 0.01);
    }
}
