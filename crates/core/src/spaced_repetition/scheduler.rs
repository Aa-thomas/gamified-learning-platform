//! Spaced repetition scheduler
//!
//! This module provides scheduling logic for review items using the SM-2 algorithm.

use chrono::{DateTime, Duration, Utc};
use crate::models::{MasteryScore, ReviewItem};

/// Quality of response for SM-2 algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewQuality {
    /// Complete failure - rating 0
    Blackout = 0,
    /// Wrong but recognized correct answer - rating 1  
    Wrong = 1,
    /// Wrong but easy to recall correct answer - rating 2
    Hard = 2,
    /// Correct with difficulty - rating 3
    Difficult = 3,
    /// Correct with some hesitation - rating 4
    Good = 4,
    /// Perfect response - rating 5
    Perfect = 5,
}

impl ReviewQuality {
    /// Check if this response means the review passed
    pub fn passed(&self) -> bool {
        *self as i32 >= 3
    }
    
    /// Convert from i32
    pub fn from_i32(value: i32) -> Self {
        match value {
            0 => ReviewQuality::Blackout,
            1 => ReviewQuality::Wrong,
            2 => ReviewQuality::Hard,
            3 => ReviewQuality::Difficult,
            4 => ReviewQuality::Good,
            _ => ReviewQuality::Perfect,
        }
    }
}

/// Schedule a quiz for review after completion
pub fn schedule_initial_review(user_id: &str, quiz_id: &str) -> ReviewItem {
    ReviewItem::new(user_id.to_string(), quiz_id.to_string())
}

/// Check if a review item is due now
pub fn is_due_now(item: &ReviewItem) -> bool {
    item.is_due()
}

/// Get all due reviews from a list
pub fn get_due_reviews(items: &[ReviewItem]) -> Vec<&ReviewItem> {
    items.iter().filter(|item| item.is_due()).collect()
}

/// Get the count of due reviews
pub fn count_due_reviews(items: &[ReviewItem]) -> usize {
    items.iter().filter(|item| item.is_due()).count()
}

/// Calculate the next review date based on current schedule
pub fn calculate_next_review_date(
    current_interval: i32,
    ease_factor: f64,
    repetitions: i32,
) -> DateTime<Utc> {
    let new_interval = if repetitions == 0 {
        1
    } else if repetitions == 1 {
        6
    } else {
        (current_interval as f64 * ease_factor).round() as i32
    };
    
    Utc::now() + Duration::days(new_interval as i64)
}

/// Convert quiz score percentage to review quality
pub fn score_to_quality(score_percentage: f64) -> ReviewQuality {
    match score_percentage {
        s if s >= 100.0 => ReviewQuality::Perfect,
        s if s >= 90.0 => ReviewQuality::Good,
        s if s >= 80.0 => ReviewQuality::Difficult,
        s if s >= 60.0 => ReviewQuality::Hard,
        s if s >= 40.0 => ReviewQuality::Wrong,
        _ => ReviewQuality::Blackout,
    }
}

/// Apply mastery decay to all stale skills
/// Returns the number of skills that were decayed
pub fn apply_mastery_decay(
    masteries: &mut [MasteryScore],
    current_time: DateTime<Utc>,
) -> usize {
    let mut decayed_count = 0;
    
    for mastery in masteries.iter_mut() {
        let days_since_update = (current_time - mastery.last_updated_at).num_days();
        
        if days_since_update > 3 {  // Beyond grace period
            let original_score = mastery.score;
            mastery.apply_decay(days_since_update);
            
            if (mastery.score - original_score).abs() > 0.001 {
                decayed_count += 1;
            }
        }
    }
    
    decayed_count
}

/// Get skills that need review (below threshold)
pub fn get_skills_needing_review(masteries: &[MasteryScore], threshold: f64) -> Vec<&MasteryScore> {
    masteries.iter().filter(|m| m.score < threshold).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_review_quality_passed() {
        assert!(!ReviewQuality::Blackout.passed());
        assert!(!ReviewQuality::Wrong.passed());
        assert!(!ReviewQuality::Hard.passed());
        assert!(ReviewQuality::Difficult.passed());
        assert!(ReviewQuality::Good.passed());
        assert!(ReviewQuality::Perfect.passed());
    }

    #[test]
    fn test_schedule_initial_review() {
        let review = schedule_initial_review("user1", "quiz1");
        assert_eq!(review.user_id, "user1");
        assert_eq!(review.quiz_id, "quiz1");
        assert_eq!(review.interval_days, 1);
        assert_eq!(review.repetitions, 0);
    }

    #[test]
    fn test_get_due_reviews() {
        let mut due_item = ReviewItem::new("user1".to_string(), "quiz1".to_string());
        due_item.due_date = Utc::now() - Duration::hours(1); // Past due
        
        let future_item = ReviewItem::new("user1".to_string(), "quiz2".to_string());
        // future_item.due_date is already tomorrow
        
        let items = vec![due_item, future_item];
        let due = get_due_reviews(&items);
        
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].quiz_id, "quiz1");
    }

    #[test]
    fn test_score_to_quality() {
        assert_eq!(score_to_quality(100.0), ReviewQuality::Perfect);
        assert_eq!(score_to_quality(95.0), ReviewQuality::Good);
        assert_eq!(score_to_quality(85.0), ReviewQuality::Difficult);
        assert_eq!(score_to_quality(70.0), ReviewQuality::Hard);
        assert_eq!(score_to_quality(50.0), ReviewQuality::Wrong);
        assert_eq!(score_to_quality(30.0), ReviewQuality::Blackout);
    }

    #[test]
    fn test_apply_mastery_decay() {
        let mut masteries = vec![
            MasteryScore {
                user_id: "user1".to_string(),
                skill_id: "skill1".to_string(),
                score: 0.8,
                last_updated_at: Utc::now() - Duration::days(10), // Stale
            },
            MasteryScore {
                user_id: "user1".to_string(),
                skill_id: "skill2".to_string(),
                score: 0.8,
                last_updated_at: Utc::now() - Duration::days(2), // Fresh
            },
        ];
        
        let decayed = apply_mastery_decay(&mut masteries, Utc::now());
        
        assert_eq!(decayed, 1);
        assert!(masteries[0].score < 0.8); // Should have decayed
        assert_eq!(masteries[1].score, 0.8); // Should not have decayed
    }

    #[test]
    fn test_calculate_next_review_date() {
        // First review
        let date1 = calculate_next_review_date(0, 2.5, 0);
        assert!((date1 - Utc::now()).num_hours() >= 23); // About 1 day
        
        // Second review
        let date2 = calculate_next_review_date(1, 2.5, 1);
        assert!((date2 - Utc::now()).num_days() >= 5); // About 6 days
    }
}
