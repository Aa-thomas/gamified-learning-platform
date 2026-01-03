use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BadgeCategory {
    Streak,
    Level,
    Xp,
    Completion,
    Mastery,
}

impl BadgeCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            BadgeCategory::Streak => "Streak",
            BadgeCategory::Level => "Level",
            BadgeCategory::Xp => "Xp",
            BadgeCategory::Completion => "Completion",
            BadgeCategory::Mastery => "Mastery",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "Streak" => Ok(BadgeCategory::Streak),
            "Level" => Ok(BadgeCategory::Level),
            "Xp" => Ok(BadgeCategory::Xp),
            "Completion" => Ok(BadgeCategory::Completion),
            "Mastery" => Ok(BadgeCategory::Mastery),
            _ => Err(format!("Invalid badge category: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BadgeDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub threshold: f64,
    pub category: BadgeCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BadgeProgress {
    pub user_id: String,
    pub badge_id: String,
    pub current_value: f64,
    pub earned_at: Option<DateTime<Utc>>,
}

impl BadgeProgress {
    pub fn new(user_id: String, badge_id: String) -> Self {
        Self {
            user_id,
            badge_id,
            current_value: 0.0,
            earned_at: None,
        }
    }

    pub fn is_earned(&self) -> bool {
        self.earned_at.is_some()
    }

    pub fn update_progress(&mut self, value: f64, threshold: f64) {
        self.current_value = value;
        if value >= threshold && self.earned_at.is_none() {
            self.earned_at = Some(Utc::now());
        }
    }

    pub fn progress_percentage(&self, threshold: f64) -> f64 {
        if threshold == 0.0 {
            return 100.0;
        }
        ((self.current_value / threshold) * 100.0).min(100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_badge_category_conversion() {
        assert_eq!(BadgeCategory::Streak.as_str(), "Streak");
        assert_eq!(BadgeCategory::from_str("Level").unwrap(), BadgeCategory::Level);
    }

    #[test]
    fn test_badge_progress() {
        let mut progress = BadgeProgress::new("user1".to_string(), "week_warrior".to_string());
        assert!(!progress.is_earned());

        progress.update_progress(5.0, 7.0);
        assert!(!progress.is_earned());
        assert!((progress.progress_percentage(7.0) - 71.43).abs() < 0.1);

        progress.update_progress(7.0, 7.0);
        assert!(progress.is_earned());
    }
}
