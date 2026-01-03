use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub version: String,
    pub title: String,
    pub description: String,
    pub author: String,
    pub created_at: String,
    pub weeks: Vec<Week>,
    #[serde(default)]
    pub checkpoints: Vec<Checkpoint>,
    #[serde(default)]
    pub skills: Vec<Skill>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Week {
    pub id: String,
    pub title: String,
    pub description: String,
    pub days: Vec<Day>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Day {
    pub id: String,
    pub title: String,
    pub description: String,
    pub nodes: Vec<ContentNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentNode {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub title: String,
    pub description: String,
    pub difficulty: String,
    pub estimated_minutes: u32,
    pub xp_reward: u32,
    pub content_path: String,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub prerequisites: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: String,
    pub title: String,
    pub description: String,
    pub week: String,
    pub day: String,
    pub difficulty: String,
    pub estimated_hours: u32,
    pub xp_reward: u32,
    pub artifacts: Vec<String>,
    #[serde(default)]
    pub prerequisites: Vec<String>,
    #[serde(default)]
    pub rubrics: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quiz {
    pub id: String,
    pub title: String,
    pub questions: Vec<Question>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    pub id: String,
    pub question: String,
    #[serde(rename = "type")]
    pub question_type: String,
    pub options: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correct_answer: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correct_answers: Option<Vec<usize>>,
    pub explanation: String,
    #[serde(default)]
    pub skills: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Challenge {
    pub id: String,
    pub title: String,
    pub description: String,
    pub instructions: String,
    pub starter_code: String,
    pub test_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub solution: Option<String>,
    #[serde(default)]
    pub hints: Vec<String>,
    pub difficulty: String,
    #[serde(default)]
    pub skills: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_deserialization() {
        let json = r#"{
            "version": "1.0",
            "title": "Test Course",
            "description": "A test course",
            "author": "Test Author",
            "created_at": "2024-01-01",
            "weeks": [],
            "checkpoints": [],
            "skills": []
        }"#;

        let manifest: Manifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.version, "1.0");
        assert_eq!(manifest.title, "Test Course");
    }

    #[test]
    fn test_content_node_deserialization() {
        let json = r#"{
            "id": "test-node",
            "type": "lecture",
            "title": "Test Lecture",
            "description": "A test lecture",
            "difficulty": "easy",
            "estimated_minutes": 20,
            "xp_reward": 25,
            "content_path": "week1/day1/lecture.md",
            "skills": ["syntax"],
            "prerequisites": []
        }"#;

        let node: ContentNode = serde_json::from_str(json).unwrap();
        assert_eq!(node.id, "test-node");
        assert_eq!(node.node_type, "lecture");
    }

    #[test]
    fn test_quiz_deserialization() {
        let json = r#"{
            "id": "test-quiz",
            "title": "Test Quiz",
            "questions": [
                {
                    "id": "q1",
                    "question": "What is 2+2?",
                    "type": "multiple-choice",
                    "options": ["3", "4", "5"],
                    "correct_answer": 1,
                    "explanation": "2+2=4",
                    "skills": ["math"]
                }
            ]
        }"#;

        let quiz: Quiz = serde_json::from_str(json).unwrap();
        assert_eq!(quiz.id, "test-quiz");
        assert_eq!(quiz.questions.len(), 1);
        assert_eq!(quiz.questions[0].correct_answer, Some(1));
    }
}
