use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quiz {
    pub id: String,
    pub title: String,
    pub description: String,
    pub difficulty: String,
    pub skills: Vec<String>,
    pub passing_score: i32,
    pub time_limit_seconds: Option<i32>,
    pub questions: Vec<Question>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    pub id: String,
    pub question_type: String,
    pub prompt: String,
    pub code_snippet: Option<String>,
    pub options: Vec<QuestionOption>,
    pub correct_answer: String,
    pub explanation: String,
    pub points: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionOption {
    pub id: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizAttempt {
    pub id: String,
    pub user_id: String,
    pub quiz_id: String,
    pub node_id: String,
    pub answers: Vec<String>,
    pub score_percentage: i32,
    pub xp_earned: i32,
    pub submitted_at: DateTime<Utc>,
}

impl QuizAttempt {
    pub fn new(
        user_id: String,
        quiz_id: String,
        node_id: String,
        answers: Vec<String>,
        score_percentage: i32,
        xp_earned: i32,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            quiz_id,
            node_id,
            answers,
            score_percentage,
            xp_earned,
            submitted_at: Utc::now(),
        }
    }

    pub fn passed(&self) -> bool {
        self.score_percentage >= 70
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quiz_attempt_creation() {
        let attempt = QuizAttempt::new(
            "user1".to_string(),
            "quiz1".to_string(),
            "node1".to_string(),
            vec!["a".to_string(), "b".to_string()],
            85,
            50,
        );
        
        assert!(!attempt.id.is_empty());
        assert_eq!(attempt.score_percentage, 85);
        assert!(attempt.passed());
    }

    #[test]
    fn test_quiz_attempt_failed() {
        let attempt = QuizAttempt::new(
            "user1".to_string(),
            "quiz1".to_string(),
            "node1".to_string(),
            vec!["a".to_string()],
            60,
            25,
        );
        
        assert!(!attempt.passed());
    }
}
