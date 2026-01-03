//! Core types for LLM grading

use serde::{Deserialize, Serialize};

/// Result of grading an artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradeResult {
    /// Total score (0-100)
    pub score: u32,
    /// Maximum possible score
    pub max_score: u32,
    /// Overall feedback
    pub overall_feedback: String,
    /// Scores for each category
    pub category_scores: Vec<CategoryScore>,
    /// Whether this result came from cache
    pub from_cache: bool,
    /// Latency in milliseconds (0 if from cache)
    pub latency_ms: u64,
}

impl GradeResult {
    /// Create a new grade result
    pub fn new(
        score: u32,
        overall_feedback: String,
        category_scores: Vec<CategoryScore>,
        latency_ms: u64,
    ) -> Self {
        Self {
            score,
            max_score: 100,
            overall_feedback,
            category_scores,
            from_cache: false,
            latency_ms,
        }
    }

    /// Mark this result as coming from cache
    pub fn from_cache(mut self) -> Self {
        self.from_cache = true;
        self.latency_ms = 0;
        self
    }

    /// Get the letter grade
    pub fn letter_grade(&self) -> &'static str {
        match self.score {
            90..=100 => "A",
            80..=89 => "B",
            70..=79 => "C",
            60..=69 => "D",
            _ => "F",
        }
    }

    /// Check if this is a passing grade (≥70)
    pub fn is_passing(&self) -> bool {
        self.score >= 70
    }
}

/// Score for a single category in the rubric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryScore {
    /// Category name
    pub category: String,
    /// Score achieved
    pub score: u32,
    /// Maximum score for this category
    pub max_score: u32,
    /// Specific feedback for this category
    pub feedback: String,
}

impl CategoryScore {
    /// Create a new category score
    pub fn new(category: String, score: u32, max_score: u32, feedback: String) -> Self {
        Self {
            category,
            score,
            max_score,
            feedback,
        }
    }

    /// Get the percentage for this category
    pub fn percentage(&self) -> f64 {
        if self.max_score == 0 {
            return 0.0;
        }
        (self.score as f64 / self.max_score as f64) * 100.0
    }
}

/// Configuration for the grader
#[derive(Debug, Clone)]
pub struct GraderConfig {
    /// OpenAI model to use
    pub model: String,
    /// Temperature for LLM (lower = more consistent)
    pub temperature: f32,
    /// Maximum tokens for response
    pub max_tokens: u16,
    /// Request timeout in seconds
    pub timeout_secs: u64,
    /// Daily grading limit per user
    pub daily_limit: u32,
    /// Whether to enable caching
    pub enable_cache: bool,
}

impl Default for GraderConfig {
    fn default() -> Self {
        Self {
            model: "gpt-4".to_string(),
            temperature: 0.3,
            max_tokens: 2000,
            timeout_secs: 30,
            daily_limit: 20,
            enable_cache: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grade_result_letter_grades() {
        assert_eq!(GradeResult::new(95, String::new(), vec![], 0).letter_grade(), "A");
        assert_eq!(GradeResult::new(85, String::new(), vec![], 0).letter_grade(), "B");
        assert_eq!(GradeResult::new(75, String::new(), vec![], 0).letter_grade(), "C");
        assert_eq!(GradeResult::new(65, String::new(), vec![], 0).letter_grade(), "D");
        assert_eq!(GradeResult::new(55, String::new(), vec![], 0).letter_grade(), "F");
    }

    #[test]
    fn test_grade_result_passing() {
        assert!(GradeResult::new(70, String::new(), vec![], 0).is_passing());
        assert!(GradeResult::new(100, String::new(), vec![], 0).is_passing());
        assert!(!GradeResult::new(69, String::new(), vec![], 0).is_passing());
    }

    #[test]
    fn test_category_score_percentage() {
        let score = CategoryScore::new("Test".to_string(), 20, 25, String::new());
        assert!((score.percentage() - 80.0).abs() < 0.001);

        let zero_max = CategoryScore::new("Test".to_string(), 0, 0, String::new());
        assert_eq!(zero_max.percentage(), 0.0);
    }

    #[test]
    fn test_from_cache() {
        let result = GradeResult::new(85, "Good".to_string(), vec![], 500);
        let cached = result.from_cache();
        
        assert!(cached.from_cache);
        assert_eq!(cached.latency_ms, 0);
        assert_eq!(cached.score, 85);
    }
}
