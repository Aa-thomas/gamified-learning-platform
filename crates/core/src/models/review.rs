use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Spaced repetition review item using SM-2 algorithm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewItem {
    pub user_id: String,
    pub quiz_id: String,
    pub due_date: DateTime<Utc>,
    pub ease_factor: f64,
    pub interval_days: i32,
    pub repetitions: i32,
    pub last_reviewed_at: Option<DateTime<Utc>>,
}

impl ReviewItem {
    const MIN_EASE_FACTOR: f64 = 1.3;
    const INITIAL_EASE_FACTOR: f64 = 2.5;

    pub fn new(user_id: String, quiz_id: String) -> Self {
        Self {
            user_id,
            quiz_id,
            due_date: Utc::now() + Duration::days(1),
            ease_factor: Self::INITIAL_EASE_FACTOR,
            interval_days: 1,
            repetitions: 0,
            last_reviewed_at: None,
        }
    }

    /// Update review item based on quality of response (0-5 scale)
    /// 0-2: Again (failed), 3: Hard, 4: Good, 5: Easy
    pub fn update_after_review(&mut self, quality: i32) {
        let quality = quality.clamp(0, 5);

        if quality < 3 {
            // Failed - reset
            self.repetitions = 0;
            self.interval_days = 1;
        } else {
            // Passed
            if self.repetitions == 0 {
                self.interval_days = 1;
            } else if self.repetitions == 1 {
                self.interval_days = 6;
            } else {
                self.interval_days = (self.interval_days as f64 * self.ease_factor).round() as i32;
            }
            self.repetitions += 1;
        }

        // Update ease factor
        self.ease_factor = self.ease_factor
            + (0.1 - (5 - quality) as f64 * (0.08 + (5 - quality) as f64 * 0.02));
        self.ease_factor = self.ease_factor.max(Self::MIN_EASE_FACTOR);

        // Set next due date
        self.due_date = Utc::now() + Duration::days(self.interval_days as i64);
        self.last_reviewed_at = Some(Utc::now());
    }

    pub fn is_due(&self) -> bool {
        Utc::now() >= self.due_date
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_review_item() {
        let item = ReviewItem::new("user1".to_string(), "quiz1".to_string());
        assert_eq!(item.interval_days, 1);
        assert_eq!(item.repetitions, 0);
        assert!((item.ease_factor - 2.5).abs() < 0.01);
    }

    #[test]
    fn test_update_after_good_review() {
        let mut item = ReviewItem::new("user1".to_string(), "quiz1".to_string());

        item.update_after_review(4); // Good
        assert_eq!(item.repetitions, 1);
        assert_eq!(item.interval_days, 1);

        item.update_after_review(4); // Good again
        assert_eq!(item.repetitions, 2);
        assert_eq!(item.interval_days, 6);
    }

    #[test]
    fn test_update_after_failed_review() {
        let mut item = ReviewItem::new("user1".to_string(), "quiz1".to_string());
        item.repetitions = 5;
        item.interval_days = 30;

        item.update_after_review(2); // Failed
        assert_eq!(item.repetitions, 0);
        assert_eq!(item.interval_days, 1);
    }
}
