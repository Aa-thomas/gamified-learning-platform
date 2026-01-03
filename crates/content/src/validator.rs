use crate::error::{ContentError, ContentResult};
use crate::loader::ContentLoader;
use crate::manifest::Manifest;
use std::collections::HashSet;

pub struct ContentValidator;

impl ContentValidator {
    /// Validate manifest structure and references
    pub fn validate_manifest(manifest: &Manifest) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Collect all node IDs
        let all_node_ids: HashSet<String> = manifest
            .weeks
            .iter()
            .flat_map(|w| &w.days)
            .flat_map(|d| &d.nodes)
            .map(|n| n.id.clone())
            .collect();

        // Validate prerequisite references
        for week in &manifest.weeks {
            for day in &week.days {
                for node in &day.nodes {
                    for prereq in &node.prerequisites {
                        if !all_node_ids.contains(prereq) {
                            errors.push(format!(
                                "Node '{}' has invalid prerequisite '{}'",
                                node.id, prereq
                            ));
                        }
                    }
                }
            }
        }

        // Validate skill references
        let all_skill_ids: HashSet<String> =
            manifest.skills.iter().map(|s| s.id.clone()).collect();

        for week in &manifest.weeks {
            for day in &week.days {
                for node in &day.nodes {
                    for skill in &node.skills {
                        if !all_skill_ids.contains(skill) {
                            errors.push(format!(
                                "Node '{}' references unknown skill '{}'",
                                node.id, skill
                            ));
                        }
                    }
                }
            }
        }

        // Validate difficulty values
        let valid_difficulties = ["easy", "medium", "hard", "very-hard"];
        for week in &manifest.weeks {
            for day in &week.days {
                for node in &day.nodes {
                    if !valid_difficulties.contains(&node.difficulty.as_str()) {
                        errors.push(format!(
                            "Node '{}' has invalid difficulty '{}'",
                            node.id, node.difficulty
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
                        errors.push(format!(
                            "Node '{}' has invalid type '{}'",
                            node.id, node.node_type
                        ));
                    }
                }
            }
        }

        // Check for duplicate IDs
        let mut seen_ids = HashSet::new();
        for week in &manifest.weeks {
            for day in &week.days {
                for node in &day.nodes {
                    if !seen_ids.insert(node.id.clone()) {
                        errors.push(format!("Duplicate node ID: '{}'", node.id));
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validate that all content files exist
    pub fn validate_content_files(loader: &ContentLoader) -> ContentResult<Vec<String>> {
        let mut errors = Vec::new();
        let manifest = loader.get_manifest();

        for week in &manifest.weeks {
            for day in &week.days {
                for node in &day.nodes {
                    let path = loader.content_dir().join(&node.content_path);
                    if !path.exists() {
                        errors.push(format!("Missing content file: {}", node.content_path));
                    }
                }
            }
        }

        // Validate rubric files for checkpoints
        for checkpoint in &manifest.checkpoints {
            for (artifact_type, rubric_path) in &checkpoint.rubrics {
                let path = loader.content_dir().join(rubric_path);
                if !path.exists() {
                    errors.push(format!(
                        "Missing rubric for {}: {}",
                        artifact_type, rubric_path
                    ));
                }
            }
        }

        if errors.is_empty() {
            Ok(vec!["All content files validated successfully".to_string()])
        } else {
            Err(ContentError::Validation(format!(
                "Validation errors:\n{}",
                errors.join("\n")
            )))
        }
    }

    /// Check for circular dependencies in prerequisites
    pub fn check_circular_dependencies(manifest: &Manifest) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Build adjacency list
        let mut deps: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();

        for week in &manifest.weeks {
            for day in &week.days {
                for node in &day.nodes {
                    deps.insert(node.id.clone(), node.prerequisites.clone());
                }
            }
        }

        // DFS to detect cycles
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        fn has_cycle(
            node: &str,
            deps: &std::collections::HashMap<String, Vec<String>>,
            visited: &mut HashSet<String>,
            rec_stack: &mut HashSet<String>,
        ) -> bool {
            if rec_stack.contains(node) {
                return true;
            }
            if visited.contains(node) {
                return false;
            }

            visited.insert(node.to_string());
            rec_stack.insert(node.to_string());

            if let Some(prerequisites) = deps.get(node) {
                for prereq in prerequisites {
                    if has_cycle(prereq, deps, visited, rec_stack) {
                        return true;
                    }
                }
            }

            rec_stack.remove(node);
            false
        }

        for node_id in deps.keys() {
            if has_cycle(node_id, &deps, &mut visited, &mut rec_stack) {
                errors.push(format!("Circular dependency detected involving '{}'", node_id));
            }
            visited.clear();
            rec_stack.clear();
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{ContentNode, Day, Skill, Week};

    fn create_test_manifest() -> Manifest {
        Manifest {
            version: "1.0".to_string(),
            title: "Test".to_string(),
            description: "Test".to_string(),
            author: "Test".to_string(),
            created_at: "2024-01-01".to_string(),
            weeks: vec![Week {
                id: "week1".to_string(),
                title: "Week 1".to_string(),
                description: "Test".to_string(),
                days: vec![Day {
                    id: "day1".to_string(),
                    title: "Day 1".to_string(),
                    description: "Test".to_string(),
                    nodes: vec![
                        ContentNode {
                            id: "node1".to_string(),
                            node_type: "lecture".to_string(),
                            title: "Node 1".to_string(),
                            description: "Test".to_string(),
                            difficulty: "easy".to_string(),
                            estimated_minutes: 20,
                            xp_reward: 25,
                            content_path: "test.md".to_string(),
                            skills: vec!["syntax".to_string()],
                            prerequisites: vec![],
                        },
                        ContentNode {
                            id: "node2".to_string(),
                            node_type: "quiz".to_string(),
                            title: "Node 2".to_string(),
                            description: "Test".to_string(),
                            difficulty: "easy".to_string(),
                            estimated_minutes: 10,
                            xp_reward: 50,
                            content_path: "test.json".to_string(),
                            skills: vec!["syntax".to_string()],
                            prerequisites: vec!["node1".to_string()],
                        },
                    ],
                }],
            }],
            checkpoints: vec![],
            skills: vec![Skill {
                id: "syntax".to_string(),
                name: "Syntax".to_string(),
                description: "Test".to_string(),
            }],
        }
    }

    #[test]
    fn test_validate_valid_manifest() {
        let manifest = create_test_manifest();
        let result = ContentValidator::validate_manifest(&manifest);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_prerequisite() {
        let mut manifest = create_test_manifest();
        manifest.weeks[0].days[0].nodes[1].prerequisites = vec!["nonexistent".to_string()];

        let result = ContentValidator::validate_manifest(&manifest);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors[0].contains("invalid prerequisite"));
    }

    #[test]
    fn test_validate_invalid_skill() {
        let mut manifest = create_test_manifest();
        manifest.weeks[0].days[0].nodes[0].skills = vec!["unknown_skill".to_string()];

        let result = ContentValidator::validate_manifest(&manifest);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors[0].contains("unknown skill"));
    }

    #[test]
    fn test_validate_invalid_difficulty() {
        let mut manifest = create_test_manifest();
        manifest.weeks[0].days[0].nodes[0].difficulty = "super-hard".to_string();

        let result = ContentValidator::validate_manifest(&manifest);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors[0].contains("invalid difficulty"));
    }

    #[test]
    fn test_check_no_circular_dependencies() {
        let manifest = create_test_manifest();
        let result = ContentValidator::check_circular_dependencies(&manifest);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_circular_dependencies() {
        let mut manifest = create_test_manifest();
        // Create circular: node1 -> node2 -> node1
        manifest.weeks[0].days[0].nodes[0].prerequisites = vec!["node2".to_string()];

        let result = ContentValidator::check_circular_dependencies(&manifest);
        assert!(result.is_err());
    }
}
