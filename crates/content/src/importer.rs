use crate::error::{ContentError, ContentResult};
use crate::manifest::Manifest;
use std::fs;
use std::path::{Path, PathBuf};

/// Result of validating a content pack
#[derive(Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub manifest: Option<Manifest>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn valid(manifest: Manifest) -> Self {
        Self {
            is_valid: true,
            manifest: Some(manifest),
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn invalid(errors: Vec<String>) -> Self {
        Self {
            is_valid: false,
            manifest: None,
            errors,
            warnings: Vec::new(),
        }
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
}

/// Validates a content pack at the given path
pub fn validate_content_pack(source_path: &Path) -> ContentResult<ValidationResult> {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Check source exists and is a directory
    if !source_path.exists() {
        return Ok(ValidationResult::invalid(vec![
            format!("Source path does not exist: {:?}", source_path)
        ]));
    }

    if !source_path.is_dir() {
        return Ok(ValidationResult::invalid(vec![
            format!("Source path is not a directory: {:?}", source_path)
        ]));
    }

    // Check manifest.json exists
    let manifest_path = source_path.join("manifest.json");
    if !manifest_path.exists() {
        return Ok(ValidationResult::invalid(vec![
            "Missing manifest.json in content pack".to_string()
        ]));
    }

    // Parse manifest
    let manifest_json = fs::read_to_string(&manifest_path)?;
    let manifest: Manifest = match serde_json::from_str(&manifest_json) {
        Ok(m) => m,
        Err(e) => {
            return Ok(ValidationResult::invalid(vec![
                format!("Invalid manifest.json: {}", e)
            ]));
        }
    };

    // Validate required manifest fields
    if manifest.title.is_empty() {
        errors.push("Manifest missing 'title' field".to_string());
    }
    if manifest.version.is_empty() {
        errors.push("Manifest missing 'version' field".to_string());
    }

    // Validate content files exist
    for week in &manifest.weeks {
        for day in &week.days {
            for node in &day.nodes {
                let content_file = source_path.join(&node.content_path);
                if !content_file.exists() {
                    errors.push(format!(
                        "Missing content file for node '{}': {}",
                        node.id, node.content_path
                    ));
                }
            }
        }
    }

    // Validate node types
    let valid_types = ["lecture", "quiz", "mini-challenge", "checkpoint"];
    for week in &manifest.weeks {
        for day in &week.days {
            for node in &day.nodes {
                if !valid_types.contains(&node.node_type.as_str()) {
                    warnings.push(format!(
                        "Node '{}' has non-standard type '{}'. Expected one of: {:?}",
                        node.id, node.node_type, valid_types
                    ));
                }
            }
        }
    }

    // Validate difficulties
    let valid_difficulties = ["easy", "medium", "hard", "very-hard"];
    for week in &manifest.weeks {
        for day in &week.days {
            for node in &day.nodes {
                if !valid_difficulties.contains(&node.difficulty.as_str()) {
                    warnings.push(format!(
                        "Node '{}' has non-standard difficulty '{}'. Expected one of: {:?}",
                        node.id, node.difficulty, valid_difficulties
                    ));
                }
            }
        }
    }

    // Check for duplicate node IDs
    let mut seen_ids = std::collections::HashSet::new();
    for week in &manifest.weeks {
        for day in &week.days {
            for node in &day.nodes {
                if !seen_ids.insert(node.id.clone()) {
                    errors.push(format!("Duplicate node ID: {}", node.id));
                }
            }
        }
    }

    // Validate prerequisites reference existing nodes
    let all_ids: std::collections::HashSet<_> = manifest.weeks.iter()
        .flat_map(|w| &w.days)
        .flat_map(|d| &d.nodes)
        .map(|n| n.id.clone())
        .collect();

    for week in &manifest.weeks {
        for day in &week.days {
            for node in &day.nodes {
                for prereq in &node.prerequisites {
                    if !all_ids.contains(prereq) {
                        errors.push(format!(
                            "Node '{}' has invalid prerequisite '{}' (not found)",
                            node.id, prereq
                        ));
                    }
                }
            }
        }
    }

    if errors.is_empty() {
        let mut result = ValidationResult::valid(manifest);
        result.warnings = warnings;
        Ok(result)
    } else {
        let mut result = ValidationResult::invalid(errors);
        result.warnings = warnings;
        Ok(result)
    }
}

/// Import a content pack to the app data directory
/// Returns the path to the imported content (relative to app data dir)
pub fn import_content_pack(
    source_path: &Path,
    app_data_dir: &Path,
    curriculum_id: &str,
) -> ContentResult<PathBuf> {
    // First validate
    let validation = validate_content_pack(source_path)?;
    if !validation.is_valid {
        return Err(ContentError::Validation(
            validation.errors.join("; ")
        ));
    }

    // Create destination directory
    let dest_dir = app_data_dir.join("curricula").join(curriculum_id);
    if dest_dir.exists() {
        // Remove existing content for this curriculum
        fs::remove_dir_all(&dest_dir)?;
    }
    fs::create_dir_all(&dest_dir)?;

    // Copy all content recursively
    copy_dir_all(source_path, &dest_dir)?;

    // Return the relative path
    Ok(PathBuf::from("curricula").join(curriculum_id))
}

/// Recursively copy a directory
fn copy_dir_all(src: &Path, dst: &Path) -> ContentResult<()> {
    fs::create_dir_all(dst)?;
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        
        if file_type.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    
    Ok(())
}

/// Delete an imported curriculum's content
pub fn delete_content_pack(app_data_dir: &Path, curriculum_id: &str) -> ContentResult<()> {
    let content_dir = app_data_dir.join("curricula").join(curriculum_id);
    if content_dir.exists() {
        fs::remove_dir_all(&content_dir)?;
    }
    Ok(())
}

/// Get statistics about a content pack
#[derive(Debug, Clone, serde::Serialize)]
pub struct ContentStats {
    pub total_weeks: usize,
    pub total_days: usize,
    pub total_nodes: usize,
    pub lectures: usize,
    pub quizzes: usize,
    pub challenges: usize,
    pub checkpoints: usize,
    pub total_xp: u32,
    pub total_estimated_minutes: u32,
}

pub fn get_content_stats(manifest: &Manifest) -> ContentStats {
    let mut stats = ContentStats {
        total_weeks: manifest.weeks.len(),
        total_days: 0,
        total_nodes: 0,
        lectures: 0,
        quizzes: 0,
        challenges: 0,
        checkpoints: manifest.checkpoints.len(),
        total_xp: 0,
        total_estimated_minutes: 0,
    };

    for week in &manifest.weeks {
        stats.total_days += week.days.len();
        for day in &week.days {
            stats.total_nodes += day.nodes.len();
            for node in &day.nodes {
                stats.total_xp += node.xp_reward;
                stats.total_estimated_minutes += node.estimated_minutes;
                
                match node.node_type.as_str() {
                    "lecture" => stats.lectures += 1,
                    "quiz" => stats.quizzes += 1,
                    "mini-challenge" => stats.challenges += 1,
                    "checkpoint" => stats.checkpoints += 1,
                    _ => {}
                }
            }
        }
    }

    // Add checkpoint XP
    for checkpoint in &manifest.checkpoints {
        stats.total_xp += checkpoint.xp_reward;
    }

    stats
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_valid_content_pack() -> PathBuf {
        let dir = tempdir().unwrap();
        let content_dir = dir.path().to_path_buf();
        std::mem::forget(dir);

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
            "skills": []
        }"#;

        fs::write(content_dir.join("manifest.json"), manifest).unwrap();
        fs::create_dir_all(content_dir.join("week1/day1")).unwrap();
        fs::write(
            content_dir.join("week1/day1/lecture.md"),
            "# Test Lecture\n\nContent here.",
        ).unwrap();

        content_dir
    }

    #[test]
    fn test_validate_valid_pack() {
        let content_dir = create_valid_content_pack();
        let result = validate_content_pack(&content_dir).unwrap();
        
        assert!(result.is_valid, "Expected valid, got errors: {:?}", result.errors);
        assert!(result.manifest.is_some());
        assert_eq!(result.manifest.unwrap().title, "Test Course");
    }

    #[test]
    fn test_validate_missing_manifest() {
        let dir = tempdir().unwrap();
        let result = validate_content_pack(dir.path()).unwrap();
        
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("manifest.json")));
    }

    #[test]
    fn test_validate_missing_content_file() {
        let dir = tempdir().unwrap();
        let content_dir = dir.path();

        let manifest = r#"{
            "version": "1.0",
            "title": "Test",
            "description": "Test",
            "author": "Test",
            "created_at": "2024-01-01",
            "weeks": [{
                "id": "week1",
                "title": "Week 1",
                "description": "Test",
                "days": [{
                    "id": "day1",
                    "title": "Day 1",
                    "description": "Test",
                    "nodes": [{
                        "id": "node1",
                        "type": "lecture",
                        "title": "Missing",
                        "description": "Test",
                        "difficulty": "easy",
                        "estimated_minutes": 10,
                        "xp_reward": 25,
                        "content_path": "missing.md"
                    }]
                }]
            }]
        }"#;

        fs::write(content_dir.join("manifest.json"), manifest).unwrap();
        
        let result = validate_content_pack(content_dir).unwrap();
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("missing.md")));
    }

    #[test]
    fn test_import_content_pack() {
        let source = create_valid_content_pack();
        let app_data = tempdir().unwrap();
        
        let rel_path = import_content_pack(&source, app_data.path(), "test-curriculum").unwrap();
        
        assert_eq!(rel_path, PathBuf::from("curricula/test-curriculum"));
        
        // Verify files were copied
        let dest = app_data.path().join("curricula/test-curriculum");
        assert!(dest.join("manifest.json").exists());
        assert!(dest.join("week1/day1/lecture.md").exists());
    }

    #[test]
    fn test_get_content_stats() {
        let content_dir = create_valid_content_pack();
        let manifest_json = fs::read_to_string(content_dir.join("manifest.json")).unwrap();
        let manifest: Manifest = serde_json::from_str(&manifest_json).unwrap();
        
        let stats = get_content_stats(&manifest);
        
        assert_eq!(stats.total_weeks, 1);
        assert_eq!(stats.total_days, 1);
        assert_eq!(stats.total_nodes, 1);
        assert_eq!(stats.lectures, 1);
        assert_eq!(stats.total_xp, 25);
        assert_eq!(stats.total_estimated_minutes, 20);
    }
}
