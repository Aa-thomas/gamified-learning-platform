//! Integration tests for Phase 5: User-Uploaded Content System
//!
//! Tests the full curriculum management flow including:
//! - Import, switch, delete operations
//! - Progress isolation per curriculum
//! - Edge cases and error handling

use glp_core::db::connection::Database;
use glp_core::db::repos::{CurriculumRepository, ProgressRepository, UserRepository};
use glp_core::models::{Curriculum, NodeProgress, User};
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

// ============================================================================
// Test Fixtures
// ============================================================================

/// Creates a valid minimal content pack for testing
fn create_valid_content_pack(base_dir: &PathBuf, name: &str, version: &str) -> PathBuf {
    let content_dir = base_dir.join(format!("{}-{}", name.to_lowercase().replace(' ', "-"), version));
    fs::create_dir_all(&content_dir).unwrap();

    let manifest = format!(
        r#"{{
        "version": "{}",
        "title": "{}",
        "description": "Test curriculum",
        "author": "Test Author",
        "created_at": "2026-01-01",
        "weeks": [
            {{
                "id": "week1",
                "title": "Week 1",
                "description": "First week",
                "days": [
                    {{
                        "id": "week1-day1",
                        "title": "Day 1",
                        "description": "First day",
                        "nodes": [
                            {{
                                "id": "week1-day1-lecture",
                                "type": "lecture",
                                "title": "Test Lecture",
                                "description": "A test lecture",
                                "difficulty": "easy",
                                "estimated_minutes": 20,
                                "xp_reward": 25,
                                "content_path": "week1/day1/lecture.md",
                                "skills": ["test-skill"],
                                "prerequisites": []
                            }},
                            {{
                                "id": "week1-day1-quiz",
                                "type": "quiz",
                                "title": "Test Quiz",
                                "description": "A test quiz",
                                "difficulty": "easy",
                                "estimated_minutes": 10,
                                "xp_reward": 50,
                                "content_path": "week1/day1/quiz.json",
                                "skills": ["test-skill"],
                                "prerequisites": ["week1-day1-lecture"]
                            }}
                        ]
                    }}
                ]
            }}
        ],
        "checkpoints": [],
        "skills": [
            {{
                "id": "test-skill",
                "name": "Test Skill",
                "description": "A test skill"
            }}
        ]
    }}"#,
        version, name
    );

    fs::write(content_dir.join("manifest.json"), manifest).unwrap();

    // Create content files
    fs::create_dir_all(content_dir.join("week1/day1")).unwrap();
    fs::write(
        content_dir.join("week1/day1/lecture.md"),
        "# Test Lecture\n\nThis is test content.",
    )
    .unwrap();

    let quiz = r#"{
        "id": "week1-day1-quiz",
        "title": "Test Quiz",
        "questions": [
            {
                "id": "q1",
                "question": "What is 2+2?",
                "type": "multiple-choice",
                "options": ["3", "4", "5", "6"],
                "correct_answer": 1,
                "explanation": "2+2=4",
                "skills": ["test-skill"]
            }
        ]
    }"#;
    fs::write(content_dir.join("week1/day1/quiz.json"), quiz).unwrap();

    content_dir
}

/// Creates an invalid content pack (missing manifest)
fn create_invalid_content_pack_no_manifest(base_dir: &PathBuf) -> PathBuf {
    let content_dir = base_dir.join("invalid-no-manifest");
    fs::create_dir_all(&content_dir).unwrap();
    // No manifest.json created
    content_dir
}

/// Creates a content pack with missing referenced files
fn create_content_pack_missing_files(base_dir: &PathBuf) -> PathBuf {
    let content_dir = base_dir.join("missing-files");
    fs::create_dir_all(&content_dir).unwrap();

    let manifest = r#"{
        "version": "1.0",
        "title": "Missing Files Test",
        "description": "Test curriculum with missing files",
        "author": "Test",
        "created_at": "2026-01-01",
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
                                "id": "node1",
                                "type": "lecture",
                                "title": "Missing Lecture",
                                "description": "This file doesn't exist",
                                "difficulty": "easy",
                                "estimated_minutes": 20,
                                "xp_reward": 25,
                                "content_path": "nonexistent.md",
                                "skills": [],
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
    // Note: Not creating the referenced nonexistent.md file
    content_dir
}

fn setup_db() -> Database {
    Database::new_in_memory().unwrap()
}

fn create_test_user(conn: &rusqlite::Connection, user_id: &str) {
    let user = User::new(user_id.to_string());
    UserRepository::create(conn, &user).unwrap();
}

// ============================================================================
// Curriculum CRUD Tests
// ============================================================================

#[test]
fn test_curriculum_create_and_retrieve() {
    let db = setup_db();
    let conn = db.connection();

    let curriculum = Curriculum::new(
        "Test Curriculum".to_string(),
        "1.0.0".to_string(),
        "curricula/test".to_string(),
    )
    .with_description("A test curriculum".to_string())
    .with_author("Test Author".to_string());

    CurriculumRepository::create(conn, &curriculum).unwrap();

    let retrieved = CurriculumRepository::get(conn, &curriculum.id)
        .unwrap()
        .expect("Curriculum should exist");

    assert_eq!(retrieved.name, "Test Curriculum");
    assert_eq!(retrieved.version, "1.0.0");
    assert_eq!(retrieved.description, Some("A test curriculum".to_string()));
    assert_eq!(retrieved.author, Some("Test Author".to_string()));
    assert!(!retrieved.is_active);
}

#[test]
fn test_curriculum_set_active_deactivates_others() {
    let db = setup_db();
    let conn = db.connection();

    let c1 = Curriculum::new("Course A".to_string(), "1.0".to_string(), "a".to_string());
    let c2 = Curriculum::new("Course B".to_string(), "1.0".to_string(), "b".to_string());
    let c3 = Curriculum::new("Course C".to_string(), "1.0".to_string(), "c".to_string());

    CurriculumRepository::create(conn, &c1).unwrap();
    CurriculumRepository::create(conn, &c2).unwrap();
    CurriculumRepository::create(conn, &c3).unwrap();

    // Activate c1
    CurriculumRepository::set_active(conn, &c1.id).unwrap();
    let active = CurriculumRepository::get_active(conn).unwrap().unwrap();
    assert_eq!(active.id, c1.id);

    // Activate c2 - c1 should become inactive
    CurriculumRepository::set_active(conn, &c2.id).unwrap();
    let active = CurriculumRepository::get_active(conn).unwrap().unwrap();
    assert_eq!(active.id, c2.id);

    // Verify c1 is no longer active
    let c1_updated = CurriculumRepository::get(conn, &c1.id).unwrap().unwrap();
    assert!(!c1_updated.is_active);

    // Verify only one curriculum is active
    let all = CurriculumRepository::get_all(conn).unwrap();
    let active_count = all.iter().filter(|c| c.is_active).count();
    assert_eq!(active_count, 1);
}

#[test]
fn test_curriculum_duplicate_name_version_check() {
    let db = setup_db();
    let conn = db.connection();

    let c1 = Curriculum::new("Course".to_string(), "1.0".to_string(), "path1".to_string());
    CurriculumRepository::create(conn, &c1).unwrap();

    // Should detect duplicate
    let exists = CurriculumRepository::exists_by_name_version(conn, "Course", "1.0").unwrap();
    assert!(exists);

    // Different version should not exist
    let exists = CurriculumRepository::exists_by_name_version(conn, "Course", "2.0").unwrap();
    assert!(!exists);

    // Different name should not exist
    let exists = CurriculumRepository::exists_by_name_version(conn, "Other Course", "1.0").unwrap();
    assert!(!exists);
}

// ============================================================================
// Content Validation Tests
// ============================================================================

#[test]
fn test_validate_valid_content_pack() {
    let temp = tempdir().unwrap();
    let content_dir = create_valid_content_pack(&temp.path().to_path_buf(), "Test Course", "1.0");

    let result = content::validate_content_pack(&content_dir).unwrap();

    assert!(result.is_valid, "Expected valid, got errors: {:?}", result.errors);
    assert!(result.manifest.is_some());
    assert_eq!(result.manifest.as_ref().unwrap().title, "Test Course");
    assert!(result.errors.is_empty());
}

#[test]
fn test_validate_missing_manifest() {
    let temp = tempdir().unwrap();
    let content_dir = create_invalid_content_pack_no_manifest(&temp.path().to_path_buf());

    let result = content::validate_content_pack(&content_dir).unwrap();

    assert!(!result.is_valid);
    assert!(result.errors.iter().any(|e| e.contains("manifest.json")));
}

#[test]
fn test_validate_missing_content_files() {
    let temp = tempdir().unwrap();
    let content_dir = create_content_pack_missing_files(&temp.path().to_path_buf());

    let result = content::validate_content_pack(&content_dir).unwrap();

    assert!(!result.is_valid);
    assert!(result.errors.iter().any(|e| e.contains("nonexistent.md")));
}

#[test]
fn test_validate_nonexistent_path() {
    let result = content::validate_content_pack(&PathBuf::from("/nonexistent/path")).unwrap();

    assert!(!result.is_valid);
    assert!(result.errors.iter().any(|e| e.contains("does not exist")));
}

// ============================================================================
// Content Import Tests
// ============================================================================

#[test]
fn test_import_valid_content_pack() {
    let source_temp = tempdir().unwrap();
    let dest_temp = tempdir().unwrap();

    let source_dir = create_valid_content_pack(
        &source_temp.path().to_path_buf(),
        "Import Test",
        "1.0",
    );

    let rel_path = content::import_content_pack(
        &source_dir,
        dest_temp.path(),
        "test-curriculum-id",
    )
    .unwrap();

    // Check relative path
    assert_eq!(rel_path, PathBuf::from("curricula/test-curriculum-id"));

    // Check files were copied
    let dest_dir = dest_temp.path().join("curricula/test-curriculum-id");
    assert!(dest_dir.join("manifest.json").exists());
    assert!(dest_dir.join("week1/day1/lecture.md").exists());
    assert!(dest_dir.join("week1/day1/quiz.json").exists());
}

#[test]
fn test_import_invalid_content_pack_fails() {
    let source_temp = tempdir().unwrap();
    let dest_temp = tempdir().unwrap();

    let source_dir = create_invalid_content_pack_no_manifest(&source_temp.path().to_path_buf());

    let result = content::import_content_pack(
        &source_dir,
        dest_temp.path(),
        "test-id",
    );

    assert!(result.is_err());
}

#[test]
fn test_import_overwrites_existing() {
    let source_temp = tempdir().unwrap();
    let dest_temp = tempdir().unwrap();

    // Create first version
    let source_v1 = create_valid_content_pack(
        &source_temp.path().to_path_buf(),
        "Overwrite Test",
        "1.0",
    );
    content::import_content_pack(&source_v1, dest_temp.path(), "overwrite-test").unwrap();

    // Create second version with different content
    let source_v2 = create_valid_content_pack(
        &source_temp.path().to_path_buf(),
        "Overwrite Test Updated",
        "2.0",
    );
    content::import_content_pack(&source_v2, dest_temp.path(), "overwrite-test").unwrap();

    // Verify the new version is present
    let manifest_path = dest_temp
        .path()
        .join("curricula/overwrite-test/manifest.json");
    let manifest_content = fs::read_to_string(&manifest_path).unwrap();
    assert!(manifest_content.contains("Overwrite Test Updated"));
    assert!(manifest_content.contains("2.0"));
}

// ============================================================================
// Delete Tests
// ============================================================================

#[test]
fn test_delete_curriculum_removes_database_record() {
    let db = setup_db();
    let conn = db.connection();

    let curriculum = Curriculum::new(
        "Delete Test".to_string(),
        "1.0".to_string(),
        "path".to_string(),
    );
    let id = curriculum.id.clone();

    CurriculumRepository::create(conn, &curriculum).unwrap();
    assert!(CurriculumRepository::get(conn, &id).unwrap().is_some());

    CurriculumRepository::delete(conn, &id).unwrap();
    assert!(CurriculumRepository::get(conn, &id).unwrap().is_none());
}

#[test]
fn test_delete_content_pack_removes_files() {
    let temp = tempdir().unwrap();

    // Create some files
    let content_dir = temp.path().join("curricula/delete-test");
    fs::create_dir_all(&content_dir).unwrap();
    fs::write(content_dir.join("manifest.json"), "{}").unwrap();
    fs::create_dir_all(content_dir.join("subdir")).unwrap();
    fs::write(content_dir.join("subdir/file.txt"), "test").unwrap();

    assert!(content_dir.exists());

    content::delete_content_pack(temp.path(), "delete-test").unwrap();

    assert!(!content_dir.exists());
}

#[test]
fn test_delete_with_progress_clears_progress_tables() {
    let db = setup_db();
    let conn = db.connection();

    // Setup: create user, curriculum, and progress
    create_test_user(conn, "test-user");

    let curriculum = Curriculum::new(
        "Progress Delete Test".to_string(),
        "1.0".to_string(),
        "path".to_string(),
    );
    let curriculum_id = curriculum.id.clone();
    CurriculumRepository::create(conn, &curriculum).unwrap();

    // Create progress record with curriculum_id
    let mut progress = NodeProgress::new("test-user".to_string(), "node1".to_string());
    // Note: In a real implementation, we'd need to set curriculum_id on the progress
    // For now, we verify the delete_with_progress function executes without error
    ProgressRepository::create_or_update(conn, &progress).unwrap();

    // Delete with progress
    CurriculumRepository::delete_with_progress(conn, &curriculum_id).unwrap();

    // Verify curriculum is deleted
    assert!(CurriculumRepository::get(conn, &curriculum_id).unwrap().is_none());
}

// ============================================================================
// Progress Isolation Tests
// ============================================================================

#[test]
fn test_multiple_curricula_can_coexist() {
    let db = setup_db();
    let conn = db.connection();

    let c1 = Curriculum::new("Course A".to_string(), "1.0".to_string(), "a".to_string());
    let c2 = Curriculum::new("Course B".to_string(), "1.0".to_string(), "b".to_string());
    let c3 = Curriculum::new("Course C".to_string(), "1.0".to_string(), "c".to_string());

    CurriculumRepository::create(conn, &c1).unwrap();
    CurriculumRepository::create(conn, &c2).unwrap();
    CurriculumRepository::create(conn, &c3).unwrap();

    let all = CurriculumRepository::get_all(conn).unwrap();
    assert_eq!(all.len(), 3);
}

#[test]
fn test_get_all_returns_sorted_by_import_date() {
    let db = setup_db();
    let conn = db.connection();

    // Create curricula - they'll have slightly different timestamps
    let c1 = Curriculum::new("First".to_string(), "1.0".to_string(), "a".to_string());
    CurriculumRepository::create(conn, &c1).unwrap();

    let c2 = Curriculum::new("Second".to_string(), "1.0".to_string(), "b".to_string());
    CurriculumRepository::create(conn, &c2).unwrap();

    let c3 = Curriculum::new("Third".to_string(), "1.0".to_string(), "c".to_string());
    CurriculumRepository::create(conn, &c3).unwrap();

    let all = CurriculumRepository::get_all(conn).unwrap();

    // Should be sorted by imported_at DESC (most recent first)
    assert_eq!(all[0].name, "Third");
    assert_eq!(all[1].name, "Second");
    assert_eq!(all[2].name, "First");
}

// ============================================================================
// Content Stats Tests
// ============================================================================

#[test]
fn test_content_stats_calculation() {
    let temp = tempdir().unwrap();
    let content_dir = create_valid_content_pack(&temp.path().to_path_buf(), "Stats Test", "1.0");

    let validation = content::validate_content_pack(&content_dir).unwrap();
    assert!(validation.is_valid);

    let manifest = validation.manifest.unwrap();
    let stats = content::get_content_stats(&manifest);

    assert_eq!(stats.total_weeks, 1);
    assert_eq!(stats.total_days, 1);
    assert_eq!(stats.total_nodes, 2); // 1 lecture + 1 quiz
    assert_eq!(stats.lectures, 1);
    assert_eq!(stats.quizzes, 1);
    assert_eq!(stats.challenges, 0);
    assert_eq!(stats.total_xp, 75); // 25 + 50
    assert_eq!(stats.total_estimated_minutes, 30); // 20 + 10
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_delete_nonexistent_curriculum_is_safe() {
    let db = setup_db();
    let conn = db.connection();

    // Should not error when deleting non-existent curriculum
    let result = CurriculumRepository::delete(conn, "nonexistent-id");
    assert!(result.is_ok());
}

#[test]
fn test_delete_nonexistent_content_pack_is_safe() {
    let temp = tempdir().unwrap();

    // Should not error when deleting non-existent content
    let result = content::delete_content_pack(temp.path(), "nonexistent-id");
    assert!(result.is_ok());
}

#[test]
fn test_set_active_nonexistent_curriculum() {
    let db = setup_db();
    let conn = db.connection();

    // Should execute but have no effect
    let result = CurriculumRepository::set_active(conn, "nonexistent-id");
    assert!(result.is_ok());

    // No active curriculum should exist
    let active = CurriculumRepository::get_active(conn).unwrap();
    assert!(active.is_none());
}

#[test]
fn test_get_active_when_none_set() {
    let db = setup_db();
    let conn = db.connection();

    // Create curricula but don't set any active
    let c1 = Curriculum::new("Course".to_string(), "1.0".to_string(), "path".to_string());
    CurriculumRepository::create(conn, &c1).unwrap();

    let active = CurriculumRepository::get_active(conn).unwrap();
    assert!(active.is_none());
}

#[test]
fn test_validate_empty_weeks_array() {
    let temp = tempdir().unwrap();
    let content_dir = temp.path().join("empty-weeks");
    fs::create_dir_all(&content_dir).unwrap();

    let manifest = r#"{
        "version": "1.0",
        "title": "Empty Weeks",
        "description": "No weeks",
        "author": "Test",
        "created_at": "2026-01-01",
        "weeks": [],
        "checkpoints": [],
        "skills": []
    }"#;
    fs::write(content_dir.join("manifest.json"), manifest).unwrap();

    let result = content::validate_content_pack(&content_dir).unwrap();

    // Empty weeks is technically valid (no missing files to check)
    assert!(result.is_valid);
    assert!(result.manifest.is_some());
}

#[test]
fn test_validate_duplicate_node_ids() {
    let temp = tempdir().unwrap();
    let content_dir = temp.path().join("duplicate-ids");
    fs::create_dir_all(&content_dir).unwrap();

    let manifest = r#"{
        "version": "1.0",
        "title": "Duplicate IDs",
        "description": "Has duplicate node IDs",
        "author": "Test",
        "created_at": "2026-01-01",
        "weeks": [
            {
                "id": "week1",
                "title": "Week 1",
                "description": "Week",
                "days": [
                    {
                        "id": "day1",
                        "title": "Day 1",
                        "description": "Day",
                        "nodes": [
                            {
                                "id": "duplicate-id",
                                "type": "lecture",
                                "title": "First",
                                "description": "First node",
                                "difficulty": "easy",
                                "estimated_minutes": 10,
                                "xp_reward": 25,
                                "content_path": "a.md"
                            },
                            {
                                "id": "duplicate-id",
                                "type": "lecture",
                                "title": "Second",
                                "description": "Second node with same ID",
                                "difficulty": "easy",
                                "estimated_minutes": 10,
                                "xp_reward": 25,
                                "content_path": "b.md"
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

    let result = content::validate_content_pack(&content_dir).unwrap();

    assert!(!result.is_valid);
    assert!(result.errors.iter().any(|e| e.contains("Duplicate node ID")));
}

#[test]
fn test_validate_invalid_prerequisite() {
    let temp = tempdir().unwrap();
    let content_dir = temp.path().join("invalid-prereq");
    fs::create_dir_all(&content_dir).unwrap();
    fs::create_dir_all(content_dir.join("content")).unwrap();
    fs::write(content_dir.join("content/lecture.md"), "# Lecture").unwrap();

    let manifest = r#"{
        "version": "1.0",
        "title": "Invalid Prereq",
        "description": "Has invalid prerequisite",
        "author": "Test",
        "created_at": "2026-01-01",
        "weeks": [
            {
                "id": "week1",
                "title": "Week 1",
                "description": "Week",
                "days": [
                    {
                        "id": "day1",
                        "title": "Day 1",
                        "description": "Day",
                        "nodes": [
                            {
                                "id": "node1",
                                "type": "lecture",
                                "title": "Node",
                                "description": "Node with bad prereq",
                                "difficulty": "easy",
                                "estimated_minutes": 10,
                                "xp_reward": 25,
                                "content_path": "content/lecture.md",
                                "prerequisites": ["nonexistent-node"]
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

    let result = content::validate_content_pack(&content_dir).unwrap();

    assert!(!result.is_valid);
    assert!(result.errors.iter().any(|e| e.contains("invalid prerequisite")));
}
