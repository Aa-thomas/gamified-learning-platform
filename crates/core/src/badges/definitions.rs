//! Badge definitions for the gamification system
//!
//! This module defines all available badges and their unlock criteria.

use crate::models::{BadgeCategory, BadgeDefinition};

/// Returns all badge definitions for the platform
pub fn get_all_badge_definitions() -> Vec<BadgeDefinition> {
    vec![
        // Streak badges
        BadgeDefinition {
            id: "week_warrior".to_string(),
            name: "Week Warrior".to_string(),
            description: "Maintain a 7-day learning streak".to_string(),
            icon: "🔥".to_string(),
            threshold: 7.0,
            category: BadgeCategory::Streak,
        },
        BadgeDefinition {
            id: "streak_master".to_string(),
            name: "Streak Master".to_string(),
            description: "Maintain a 30-day learning streak".to_string(),
            icon: "⚡".to_string(),
            threshold: 30.0,
            category: BadgeCategory::Streak,
        },
        BadgeDefinition {
            id: "unstoppable".to_string(),
            name: "Unstoppable".to_string(),
            description: "Maintain a 100-day learning streak".to_string(),
            icon: "💫".to_string(),
            threshold: 100.0,
            category: BadgeCategory::Streak,
        },
        // Level badges
        BadgeDefinition {
            id: "rising_star".to_string(),
            name: "Rising Star".to_string(),
            description: "Reach level 5".to_string(),
            icon: "⭐".to_string(),
            threshold: 5.0,
            category: BadgeCategory::Level,
        },
        BadgeDefinition {
            id: "apprentice".to_string(),
            name: "Apprentice".to_string(),
            description: "Reach level 10".to_string(),
            icon: "🌟".to_string(),
            threshold: 10.0,
            category: BadgeCategory::Level,
        },
        BadgeDefinition {
            id: "journeyman".to_string(),
            name: "Journeyman".to_string(),
            description: "Reach level 20".to_string(),
            icon: "✨".to_string(),
            threshold: 20.0,
            category: BadgeCategory::Level,
        },
        // XP badges
        BadgeDefinition {
            id: "xp_hunter".to_string(),
            name: "XP Hunter".to_string(),
            description: "Earn 1,000 total XP".to_string(),
            icon: "💎".to_string(),
            threshold: 1000.0,
            category: BadgeCategory::Xp,
        },
        BadgeDefinition {
            id: "xp_collector".to_string(),
            name: "XP Collector".to_string(),
            description: "Earn 5,000 total XP".to_string(),
            icon: "💰".to_string(),
            threshold: 5000.0,
            category: BadgeCategory::Xp,
        },
        BadgeDefinition {
            id: "xp_legend".to_string(),
            name: "XP Legend".to_string(),
            description: "Earn 10,000 total XP".to_string(),
            icon: "👑".to_string(),
            threshold: 10000.0,
            category: BadgeCategory::Xp,
        },
        // Completion badges
        BadgeDefinition {
            id: "first_steps".to_string(),
            name: "First Steps".to_string(),
            description: "Complete your first lecture".to_string(),
            icon: "👣".to_string(),
            threshold: 1.0,
            category: BadgeCategory::Completion,
        },
        BadgeDefinition {
            id: "quiz_whiz".to_string(),
            name: "Quiz Whiz".to_string(),
            description: "Complete 10 quizzes".to_string(),
            icon: "📝".to_string(),
            threshold: 10.0,
            category: BadgeCategory::Completion,
        },
        BadgeDefinition {
            id: "completionist".to_string(),
            name: "Completionist".to_string(),
            description: "Complete 50 learning activities".to_string(),
            icon: "🏆".to_string(),
            threshold: 50.0,
            category: BadgeCategory::Completion,
        },
        BadgeDefinition {
            id: "perfect_score".to_string(),
            name: "Perfect Score".to_string(),
            description: "Get 100% on any quiz".to_string(),
            icon: "💯".to_string(),
            threshold: 1.0,
            category: BadgeCategory::Completion,
        },
        // Mastery badges
        BadgeDefinition {
            id: "skill_seeker".to_string(),
            name: "Skill Seeker".to_string(),
            description: "Reach 50% mastery in any skill".to_string(),
            icon: "🎯".to_string(),
            threshold: 0.5,
            category: BadgeCategory::Mastery,
        },
        BadgeDefinition {
            id: "skill_master".to_string(),
            name: "Skill Master".to_string(),
            description: "Reach 90% mastery in any skill".to_string(),
            icon: "🏅".to_string(),
            threshold: 0.9,
            category: BadgeCategory::Mastery,
        },
    ]
}

/// Get a badge definition by ID
pub fn get_badge_by_id(badge_id: &str) -> Option<BadgeDefinition> {
    get_all_badge_definitions()
        .into_iter()
        .find(|b| b.id == badge_id)
}

/// Get all badge definitions for a specific category
pub fn get_badges_by_category(category: BadgeCategory) -> Vec<BadgeDefinition> {
    get_all_badge_definitions()
        .into_iter()
        .filter(|b| b.category == category)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_badge_definitions_load() {
        let badges = get_all_badge_definitions();
        assert!(badges.len() >= 10, "Should have at least 10 badges");
        assert!(badges.len() <= 15, "Should have at most 15 badges");
    }

    #[test]
    fn test_all_badges_have_required_fields() {
        for badge in get_all_badge_definitions() {
            assert!(!badge.id.is_empty(), "Badge ID should not be empty");
            assert!(!badge.name.is_empty(), "Badge name should not be empty");
            assert!(!badge.description.is_empty(), "Badge description should not be empty");
            assert!(!badge.icon.is_empty(), "Badge icon should not be empty");
            assert!(badge.threshold > 0.0, "Badge threshold should be positive");
        }
    }

    #[test]
    fn test_get_badge_by_id() {
        let badge = get_badge_by_id("week_warrior");
        assert!(badge.is_some());
        assert_eq!(badge.unwrap().name, "Week Warrior");

        let missing = get_badge_by_id("nonexistent");
        assert!(missing.is_none());
    }

    #[test]
    fn test_get_badges_by_category() {
        let streak_badges = get_badges_by_category(BadgeCategory::Streak);
        assert_eq!(streak_badges.len(), 3);

        let level_badges = get_badges_by_category(BadgeCategory::Level);
        assert_eq!(level_badges.len(), 3);
    }

    #[test]
    fn test_unique_badge_ids() {
        let badges = get_all_badge_definitions();
        let mut ids: Vec<&str> = badges.iter().map(|b| b.id.as_str()).collect();
        let original_len = ids.len();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), original_len, "Badge IDs must be unique");
    }
}
