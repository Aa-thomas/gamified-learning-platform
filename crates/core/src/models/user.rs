use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub total_xp: i32,
    pub current_level: i32,
    pub current_streak: i32,
    pub last_streak_date: Option<DateTime<Utc>>,
}

impl User {
    pub fn new(id: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            created_at: now,
            last_activity: now,
            total_xp: 0,
            current_level: 1,
            current_streak: 0,
            last_streak_date: None,
        }
    }

    /// Calculate XP required for next level using formula: 100 × N^1.5
    pub fn xp_for_next_level(&self) -> i32 {
        Self::xp_for_level(self.current_level + 1)
    }

    /// Calculate XP required to reach a specific level
    pub fn xp_for_level(level: i32) -> i32 {
        (100.0 * (level as f64).powf(1.5)) as i32
    }

    /// Calculate current XP progress percentage toward next level
    pub fn xp_progress_percentage(&self) -> f64 {
        let current_level_threshold = Self::xp_for_level(self.current_level);
        let next_level_threshold = self.xp_for_next_level();
        let xp_in_current_level = self.total_xp - current_level_threshold;
        let xp_needed_for_level = next_level_threshold - current_level_threshold;

        if xp_needed_for_level == 0 {
            return 100.0;
        }

        ((xp_in_current_level as f64 / xp_needed_for_level as f64) * 100.0).clamp(0.0, 100.0)
    }

    /// Check if user should level up and return the new level
    pub fn check_level_up(&self) -> Option<i32> {
        let mut level = self.current_level;
        while self.total_xp >= Self::xp_for_level(level + 1) {
            level += 1;
        }
        if level > self.current_level {
            Some(level)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_user() {
        let user = User::new("test-id".to_string());
        assert_eq!(user.id, "test-id");
        assert_eq!(user.total_xp, 0);
        assert_eq!(user.current_level, 1);
        assert_eq!(user.current_streak, 0);
    }

    #[test]
    fn test_xp_for_level() {
        assert_eq!(User::xp_for_level(1), 100);
        assert_eq!(User::xp_for_level(2), 282); // 100 * 2^1.5 ≈ 282
        assert_eq!(User::xp_for_level(5), 1118); // 100 * 5^1.5 ≈ 1118
    }

    #[test]
    fn test_check_level_up() {
        let mut user = User::new("test".to_string());
        user.total_xp = 300; // Should be level 2 (threshold is 282)
        assert_eq!(user.check_level_up(), Some(2));

        user.current_level = 2;
        user.total_xp = 200; // Not enough for level 3
        assert_eq!(user.check_level_up(), None);
    }
}
