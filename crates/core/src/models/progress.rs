use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeStatus {
    NotStarted,
    InProgress,
    Completed,
    Failed,
}

impl NodeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeStatus::NotStarted => "NotStarted",
            NodeStatus::InProgress => "InProgress",
            NodeStatus::Completed => "Completed",
            NodeStatus::Failed => "Failed",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "NotStarted" => Ok(NodeStatus::NotStarted),
            "InProgress" => Ok(NodeStatus::InProgress),
            "Completed" => Ok(NodeStatus::Completed),
            "Failed" => Ok(NodeStatus::Failed),
            _ => Err(format!("Invalid node status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeProgress {
    pub user_id: String,
    pub node_id: String,
    pub status: NodeStatus,
    pub attempts: i32,
    pub time_spent_mins: i32,
    pub first_started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub last_updated_at: DateTime<Utc>,
}

impl NodeProgress {
    pub fn new(user_id: String, node_id: String) -> Self {
        Self {
            user_id,
            node_id,
            status: NodeStatus::NotStarted,
            attempts: 0,
            time_spent_mins: 0,
            first_started_at: None,
            completed_at: None,
            last_updated_at: Utc::now(),
        }
    }

    pub fn start(&mut self) {
        if self.first_started_at.is_none() {
            self.first_started_at = Some(Utc::now());
        }
        self.status = NodeStatus::InProgress;
        self.last_updated_at = Utc::now();
    }

    pub fn complete(&mut self) {
        self.status = NodeStatus::Completed;
        self.completed_at = Some(Utc::now());
        self.last_updated_at = Utc::now();
    }

    pub fn fail(&mut self) {
        self.status = NodeStatus::Failed;
        self.attempts += 1;
        self.last_updated_at = Utc::now();
    }

    pub fn add_time(&mut self, mins: i32) {
        self.time_spent_mins += mins;
        self.last_updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_status_conversion() {
        assert_eq!(NodeStatus::NotStarted.as_str(), "NotStarted");
        assert_eq!(NodeStatus::from_str("Completed").unwrap(), NodeStatus::Completed);
        assert!(NodeStatus::from_str("Invalid").is_err());
    }

    #[test]
    fn test_node_progress_lifecycle() {
        let mut progress = NodeProgress::new("user1".to_string(), "node1".to_string());
        assert_eq!(progress.status, NodeStatus::NotStarted);

        progress.start();
        assert_eq!(progress.status, NodeStatus::InProgress);
        assert!(progress.first_started_at.is_some());

        progress.complete();
        assert_eq!(progress.status, NodeStatus::Completed);
        assert!(progress.completed_at.is_some());
    }
}
