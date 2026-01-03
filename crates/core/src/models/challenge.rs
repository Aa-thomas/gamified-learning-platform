use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeAttempt {
    pub id: String,
    pub user_id: String,
    pub challenge_id: String,
    pub node_id: String,
    pub code_hash: String,
    pub tests_passed: i32,
    pub tests_failed: i32,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub xp_earned: i32,
    pub submitted_at: DateTime<Utc>,
}

impl ChallengeAttempt {
    pub fn new(
        user_id: String,
        challenge_id: String,
        node_id: String,
        code: &str,
        tests_passed: i32,
        tests_failed: i32,
        stdout: Option<String>,
        stderr: Option<String>,
        xp_earned: i32,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            challenge_id,
            node_id,
            code_hash: Self::hash_code(code),
            tests_passed,
            tests_failed,
            stdout,
            stderr,
            xp_earned,
            submitted_at: Utc::now(),
        }
    }

    pub fn hash_code(code: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(code.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn passed(&self) -> bool {
        self.tests_failed == 0 && self.tests_passed > 0
    }

    pub fn pass_rate(&self) -> f64 {
        let total = self.tests_passed + self.tests_failed;
        if total == 0 {
            return 0.0;
        }
        self.tests_passed as f64 / total as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_challenge_attempt_creation() {
        let attempt = ChallengeAttempt::new(
            "user1".to_string(),
            "challenge1".to_string(),
            "node1".to_string(),
            "fn main() {}",
            5,
            0,
            Some("All tests passed".to_string()),
            None,
            100,
        );

        assert!(attempt.passed());
        assert_eq!(attempt.pass_rate(), 1.0);
    }

    #[test]
    fn test_code_hashing() {
        let hash1 = ChallengeAttempt::hash_code("fn main() {}");
        let hash2 = ChallengeAttempt::hash_code("fn main() {}");
        let hash3 = ChallengeAttempt::hash_code("fn main() { println!(); }");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
}
