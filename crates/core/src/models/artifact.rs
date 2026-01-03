use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ArtifactType {
    Readme,
    Design,
    Bench,
    Runbook,
    Invariants,
}

impl ArtifactType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ArtifactType::Readme => "README",
            ArtifactType::Design => "DESIGN",
            ArtifactType::Bench => "BENCH",
            ArtifactType::Runbook => "RUNBOOK",
            ArtifactType::Invariants => "INVARIANTS",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "README" => Ok(ArtifactType::Readme),
            "DESIGN" => Ok(ArtifactType::Design),
            "BENCH" => Ok(ArtifactType::Bench),
            "RUNBOOK" => Ok(ArtifactType::Runbook),
            "INVARIANTS" => Ok(ArtifactType::Invariants),
            _ => Err(format!("Invalid artifact type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactSubmission {
    pub id: String,
    pub user_id: String,
    pub checkpoint_id: String,
    pub artifact_type: ArtifactType,
    pub content_hash: String,
    pub grade_percentage: Option<i32>,
    pub reasoning_json: Option<String>,
    pub xp_earned: i32,
    pub submitted_at: DateTime<Utc>,
    pub graded_at: Option<DateTime<Utc>>,
}

impl ArtifactSubmission {
    pub fn new(
        user_id: String,
        checkpoint_id: String,
        artifact_type: ArtifactType,
        content: &str,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            checkpoint_id,
            artifact_type,
            content_hash: Self::hash_content(content),
            grade_percentage: None,
            reasoning_json: None,
            xp_earned: 0,
            submitted_at: Utc::now(),
            graded_at: None,
        }
    }

    pub fn hash_content(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn set_grade(&mut self, grade: i32, reasoning: String, xp: i32) {
        self.grade_percentage = Some(grade);
        self.reasoning_json = Some(reasoning);
        self.xp_earned = xp;
        self.graded_at = Some(Utc::now());
    }

    pub fn is_graded(&self) -> bool {
        self.grade_percentage.is_some()
    }

    pub fn passed(&self) -> bool {
        self.grade_percentage.map(|g| g >= 70).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_artifact_type_conversion() {
        assert_eq!(ArtifactType::Readme.as_str(), "README");
        assert_eq!(ArtifactType::from_str("DESIGN").unwrap(), ArtifactType::Design);
    }

    #[test]
    fn test_artifact_submission_lifecycle() {
        let mut submission = ArtifactSubmission::new(
            "user1".to_string(),
            "checkpoint1".to_string(),
            ArtifactType::Readme,
            "# My Project\n\nThis is a README.",
        );

        assert!(!submission.is_graded());
        assert!(!submission.passed());

        submission.set_grade(85, r#"{"clarity": 90}"#.to_string(), 200);

        assert!(submission.is_graded());
        assert!(submission.passed());
        assert_eq!(submission.xp_earned, 200);
    }
}
