use crate::error::{ContentError, ContentResult};
use crate::manifest::{Challenge, Manifest, Quiz};
use std::fs;
use std::path::PathBuf;

pub struct ContentLoader {
    content_dir: PathBuf,
    manifest: Manifest,
}

impl ContentLoader {
    pub fn new(content_dir: PathBuf) -> ContentResult<Self> {
        let manifest_path = content_dir.join("manifest.json");

        if !manifest_path.exists() {
            return Err(ContentError::NotFound(format!(
                "Manifest not found at {:?}",
                manifest_path
            )));
        }

        let manifest_json = fs::read_to_string(&manifest_path)?;
        let manifest: Manifest = serde_json::from_str(&manifest_json)?;

        Ok(Self {
            content_dir,
            manifest,
        })
    }

    pub fn get_manifest(&self) -> &Manifest {
        &self.manifest
    }

    pub fn content_dir(&self) -> &PathBuf {
        &self.content_dir
    }

    pub fn load_lecture(&self, content_path: &str) -> ContentResult<String> {
        let path = self.content_dir.join(content_path);

        if !path.exists() {
            return Err(ContentError::NotFound(format!(
                "Lecture not found at {:?}",
                path
            )));
        }

        let content = fs::read_to_string(&path)?;
        Ok(content)
    }

    pub fn load_quiz(&self, content_path: &str) -> ContentResult<Quiz> {
        let path = self.content_dir.join(content_path);

        if !path.exists() {
            return Err(ContentError::NotFound(format!(
                "Quiz not found at {:?}",
                path
            )));
        }

        let quiz_json = fs::read_to_string(&path)?;
        let quiz: Quiz = serde_json::from_str(&quiz_json)?;
        Ok(quiz)
    }

    pub fn load_challenge(&self, content_path: &str) -> ContentResult<Challenge> {
        let path = self.content_dir.join(content_path);

        if !path.exists() {
            return Err(ContentError::NotFound(format!(
                "Challenge not found at {:?}",
                path
            )));
        }

        let challenge_json = fs::read_to_string(&path)?;
        let challenge: Challenge = serde_json::from_str(&challenge_json)?;
        Ok(challenge)
    }

    /// Get all node IDs in the manifest
    pub fn get_all_node_ids(&self) -> Vec<String> {
        self.manifest
            .weeks
            .iter()
            .flat_map(|w| &w.days)
            .flat_map(|d| &d.nodes)
            .map(|n| n.id.clone())
            .collect()
    }

    /// Get node by ID
    pub fn get_node_by_id(&self, node_id: &str) -> Option<&crate::manifest::ContentNode> {
        self.manifest
            .weeks
            .iter()
            .flat_map(|w| &w.days)
            .flat_map(|d| &d.nodes)
            .find(|n| n.id == node_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn create_test_content() -> PathBuf {
        let dir = tempdir().unwrap();
        let content_dir = dir.path().to_path_buf();
        std::mem::forget(dir); // Keep the temp directory

        // Create manifest
        let manifest = r#"{
            "version": "1.0",
            "title": "Test Course",
            "description": "A test course",
            "author": "Test Author",
            "created_at": "2024-01-01",
            "weeks": [
                {
                    "id": "week1",
                    "title": "Week 1",
                    "description": "First week",
                    "days": [
                        {
                            "id": "week1-day1",
                            "title": "Day 1",
                            "description": "First day",
                            "nodes": [
                                {
                                    "id": "week1-day1-lecture",
                                    "type": "lecture",
                                    "title": "Test Lecture",
                                    "description": "A test lecture",
                                    "difficulty": "easy",
                                    "estimated_minutes": 20,
                                    "xp_reward": 25,
                                    "content_path": "week1/day1/lecture.md",
                                    "skills": ["syntax"],
                                    "prerequisites": []
                                }
                            ]
                        }
                    ]
                }
            ],
            "checkpoints": [],
            "skills": [
                {
                    "id": "syntax",
                    "name": "Rust Syntax",
                    "description": "Basic Rust syntax"
                }
            ]
        }"#;

        fs::write(content_dir.join("manifest.json"), manifest).unwrap();

        // Create content directories
        fs::create_dir_all(content_dir.join("week1/day1")).unwrap();

        // Create lecture
        fs::write(
            content_dir.join("week1/day1/lecture.md"),
            "# Test Lecture\n\nThis is a test lecture.",
        )
        .unwrap();

        content_dir
    }

    #[test]
    fn test_load_manifest() {
        let content_dir = create_test_content();
        let loader = ContentLoader::new(content_dir).unwrap();

        assert_eq!(loader.get_manifest().title, "Test Course");
        assert_eq!(loader.get_manifest().weeks.len(), 1);
    }

    #[test]
    fn test_load_lecture() {
        let content_dir = create_test_content();
        let loader = ContentLoader::new(content_dir).unwrap();

        let lecture = loader.load_lecture("week1/day1/lecture.md").unwrap();
        assert!(lecture.contains("Test Lecture"));
    }

    #[test]
    fn test_get_all_node_ids() {
        let content_dir = create_test_content();
        let loader = ContentLoader::new(content_dir).unwrap();

        let node_ids = loader.get_all_node_ids();
        assert_eq!(node_ids.len(), 1);
        assert_eq!(node_ids[0], "week1-day1-lecture");
    }

    #[test]
    fn test_get_node_by_id() {
        let content_dir = create_test_content();
        let loader = ContentLoader::new(content_dir).unwrap();

        let node = loader.get_node_by_id("week1-day1-lecture");
        assert!(node.is_some());
        assert_eq!(node.unwrap().title, "Test Lecture");

        let missing = loader.get_node_by_id("nonexistent");
        assert!(missing.is_none());
    }
}
