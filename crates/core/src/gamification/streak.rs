use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const GRACE_PERIOD_DAYS: i64 = 5;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreakInfo {
    pub current_streak: u32,
    pub is_grace_period: bool,
    pub grace_days_remaining: u32,
    pub last_activity: DateTime<Utc>,
}

/// Calculate streak information based on last activity date
pub fn calculate_streak_info(
    last_activity: DateTime<Utc>,
    current_streak: u32,
) -> StreakInfo {
    let now = Utc::now();
    let days_since = (now - last_activity).num_days();

    let (new_streak, is_grace, grace_remaining) = match days_since {
        0 => {
            // Same day - no change
            (current_streak, false, 0)
        }
        1 => {
            // Next day - increment
            (current_streak + 1, false, 0)
        }
        d if d > 1 && d <= GRACE_PERIOD_DAYS => {
            // Within grace period - maintain but warn
            (current_streak, true, (GRACE_PERIOD_DAYS - d) as u32)
        }
        _ => {
            // Beyond grace period - reset
            (1, false, 0)
        }
    };

    StreakInfo {
        current_streak: new_streak,
        is_grace_period: is_grace,
        grace_days_remaining: grace_remaining,
        last_activity,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn now() -> DateTime<Utc> {
        Utc::now()
    }

    fn yesterday() -> DateTime<Utc> {
        now() - Duration::days(1)
    }

    fn days_ago(days: i64) -> DateTime<Utc> {
        now() - Duration::days(days)
    }

    #[test]
    fn test_streak_same_day() {
        let info = calculate_streak_info(now(), 10);
        assert_eq!(info.current_streak, 10);
        assert!(!info.is_grace_period);
        assert_eq!(info.grace_days_remaining, 0);
    }

    #[test]
    fn test_streak_next_day() {
        let info = calculate_streak_info(yesterday(), 10);
        assert_eq!(info.current_streak, 11);
        assert!(!info.is_grace_period);
    }

    #[test]
    fn test_streak_grace_period() {
        let info = calculate_streak_info(days_ago(3), 10);
        assert_eq!(info.current_streak, 10);
        assert!(info.is_grace_period);
        assert_eq!(info.grace_days_remaining, 2); // 5 - 3 = 2
    }

    #[test]
    fn test_streak_grace_period_last_day() {
        let info = calculate_streak_info(days_ago(5), 10);
        assert_eq!(info.current_streak, 10);
        assert!(info.is_grace_period);
        assert_eq!(info.grace_days_remaining, 0); // 5 - 5 = 0
    }

    #[test]
    fn test_streak_reset() {
        let info = calculate_streak_info(days_ago(6), 10);
        assert_eq!(info.current_streak, 1);
        assert!(!info.is_grace_period);
        assert_eq!(info.grace_days_remaining, 0);
    }

    #[test]
    fn test_first_activity() {
        let info = calculate_streak_info(days_ago(1), 0);
        assert_eq!(info.current_streak, 1);
    }
}
