use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionHistory {
    pub id: String,
    pub user_id: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub total_xp_earned: i32,
    pub items_completed: i32,
}

impl SessionHistory {
    pub fn new(user_id: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            started_at: Utc::now(),
            ended_at: None,
            total_xp_earned: 0,
            items_completed: 0,
        }
    }

    pub fn end_session(&mut self) {
        self.ended_at = Some(Utc::now());
    }

    pub fn add_completion(&mut self, xp: i32) {
        self.total_xp_earned += xp;
        self.items_completed += 1;
    }

    pub fn duration_minutes(&self) -> i64 {
        let end = self.ended_at.unwrap_or_else(Utc::now);
        (end - self.started_at).num_minutes()
    }

    pub fn is_active(&self) -> bool {
        self.ended_at.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_lifecycle() {
        let mut session = SessionHistory::new("user1".to_string());
        assert!(session.is_active());
        assert_eq!(session.items_completed, 0);

        session.add_completion(50);
        session.add_completion(100);
        assert_eq!(session.items_completed, 2);
        assert_eq!(session.total_xp_earned, 150);

        session.end_session();
        assert!(!session.is_active());
    }
}
