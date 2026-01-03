use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasteryScore {
    pub user_id: String,
    pub skill_id: String,
    pub score: f64,
    pub last_updated_at: DateTime<Utc>,
}

impl MasteryScore {
    const LEARNING_RATE: f64 = 0.25;
    const DECAY_RATE: f64 = 0.05;
    const GRACE_PERIOD_DAYS: i64 = 3;
    const MINIMUM_SCORE: f64 = 0.30;

    pub fn new(user_id: String, skill_id: String) -> Self {
        Self {
            user_id,
            skill_id,
            score: 0.0,
            last_updated_at: Utc::now(),
        }
    }

    /// Update mastery score based on performance (0.0 to 1.0)
    /// Uses exponential moving average: new = old + learning_rate × (performance - old)
    pub fn update_with_performance(&mut self, performance: f64) {
        let performance = performance.clamp(0.0, 1.0);
        self.score = self.score + Self::LEARNING_RATE * (performance - self.score);
        self.score = self.score.clamp(0.0, 1.0);
        self.last_updated_at = Utc::now();
    }

    /// Apply decay based on days since last activity
    /// Formula: score = score × e^(-decay_rate × days_inactive)
    pub fn apply_decay(&mut self, days_inactive: i64) {
        if days_inactive <= Self::GRACE_PERIOD_DAYS {
            return;
        }

        let decay_days = days_inactive - Self::GRACE_PERIOD_DAYS;
        let decay_factor = (-Self::DECAY_RATE * decay_days as f64).exp();
        self.score = (self.score * decay_factor).max(Self::MINIMUM_SCORE);
    }

    /// Get mastery level description
    pub fn level_description(&self) -> &'static str {
        match self.score {
            s if s >= 0.9 => "Master",
            s if s >= 0.7 => "Proficient",
            s if s >= 0.5 => "Competent",
            s if s >= 0.3 => "Familiar",
            _ => "Novice",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_mastery() {
        let mastery = MasteryScore::new("user1".to_string(), "ownership".to_string());
        assert_eq!(mastery.score, 0.0);
        assert_eq!(mastery.skill_id, "ownership");
    }

    #[test]
    fn test_update_with_performance() {
        let mut mastery = MasteryScore::new("user1".to_string(), "ownership".to_string());
        
        // Perfect performance should increase score
        mastery.update_with_performance(1.0);
        assert!(mastery.score > 0.0);
        assert!(mastery.score <= 0.25); // First update with learning rate 0.25
    }

    #[test]
    fn test_decay_within_grace_period() {
        let mut mastery = MasteryScore::new("user1".to_string(), "ownership".to_string());
        mastery.score = 0.8;
        
        mastery.apply_decay(2); // Within grace period
        assert_eq!(mastery.score, 0.8); // No decay
    }

    #[test]
    fn test_decay_after_grace_period() {
        let mut mastery = MasteryScore::new("user1".to_string(), "ownership".to_string());
        mastery.score = 0.8;
        
        mastery.apply_decay(10); // After grace period
        assert!(mastery.score < 0.8);
        assert!(mastery.score >= MasteryScore::MINIMUM_SCORE);
    }

    #[test]
    fn test_level_description() {
        let mut mastery = MasteryScore::new("user1".to_string(), "ownership".to_string());
        
        mastery.score = 0.95;
        assert_eq!(mastery.level_description(), "Master");
        
        mastery.score = 0.1;
        assert_eq!(mastery.level_description(), "Novice");
    }
}
