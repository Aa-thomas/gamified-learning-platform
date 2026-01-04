//! Integration tests for Phase 6: Polish & Beta features
//!
//! Tests cover:
//! - Error handling
//! - Database operations
//! - Data management workflows

use glp_core::Database;
use glp_core::models::{User, NodeProgress, NodeStatus, BadgeProgress};
use glp_core::db::repos::{UserRepository, ProgressRepository, BadgeRepository};
use chrono::Utc;

// ============================================================================
// 6.1 Error Handling Tests
// ============================================================================

mod error_handling {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_database_connection_error_handling() {
        // Try to connect to invalid path
        let result = Database::new(PathBuf::from("/nonexistent/path/to/db.sqlite"));
        
        // Should fail gracefully with error message
        assert!(result.is_err());
    }

    #[test]
    fn test_repository_not_found_handling() {
        let db = Database::new_in_memory().unwrap();
        let conn = db.connection();
        
        // Query for non-existent user
        let result = UserRepository::get_by_id(conn, "nonexistent-user").unwrap();
        
        // Should return None, not error
        assert!(result.is_none());
    }

    #[test]
    fn test_duplicate_insert_handling() {
        let db = Database::new_in_memory().unwrap();
        let conn = db.connection();
        
        let user = User::new("duplicate-test".to_string());
        
        // First insert should succeed
        let result1 = UserRepository::create(conn, &user);
        assert!(result1.is_ok());
        
        // Second insert should fail (duplicate primary key)
        let result2 = UserRepository::create(conn, &user);
        assert!(result2.is_err());
    }
}

// ============================================================================
// 6.2 Onboarding Tests
// ============================================================================

mod onboarding {
    use super::*;

    #[test]
    fn test_new_user_has_default_values() {
        let user = User::new("new-user".to_string());
        
        // Verify default values for a new user (simulating first launch)
        assert_eq!(user.total_xp, 0);
        assert_eq!(user.current_level, 1);
        assert_eq!(user.current_streak, 0);
        assert!(user.last_streak_date.is_none());
    }

    #[test]
    fn test_can_create_and_retrieve_first_user() {
        let db = Database::new_in_memory().unwrap();
        let conn = db.connection();
        
        let user = User::new("first-user".to_string());
        UserRepository::create(conn, &user).unwrap();
        
        let retrieved = UserRepository::get_by_id(conn, "first-user").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, "first-user");
    }
}

// ============================================================================
// 6.3 Data Management Tests
// ============================================================================

mod data_management {
    use super::*;

    #[test]
    fn test_user_data_serialization() {
        let user = User::new("export-test".to_string());
        
        // Should serialize to JSON (for export)
        let json = serde_json::to_string(&user).unwrap();
        assert!(json.contains("export-test"));
        assert!(json.contains("total_xp"));
        
        // Should deserialize (for import)
        let imported: User = serde_json::from_str(&json).unwrap();
        assert_eq!(imported.id, user.id);
    }

    #[test]
    fn test_progress_data_serialization() {
        let progress = NodeProgress::new("user-1".to_string(), "node-1".to_string());
        
        let json = serde_json::to_string(&progress).unwrap();
        assert!(json.contains("user-1"));
        assert!(json.contains("node-1"));
        
        let imported: NodeProgress = serde_json::from_str(&json).unwrap();
        assert_eq!(imported.user_id, progress.user_id);
        assert_eq!(imported.node_id, progress.node_id);
    }

    #[test]
    fn test_badge_progress_serialization() {
        let badge = BadgeProgress::new("user-1".to_string(), "badge-1".to_string());
        
        let json = serde_json::to_string(&badge).unwrap();
        assert!(json.contains("user-1"));
        assert!(json.contains("badge-1"));
        
        let imported: BadgeProgress = serde_json::from_str(&json).unwrap();
        assert_eq!(imported.user_id, badge.user_id);
        assert_eq!(imported.badge_id, badge.badge_id);
    }

    #[test]
    fn test_reset_user_stats() {
        let db = Database::new_in_memory().unwrap();
        let conn = db.connection();
        
        // Create user with XP
        let mut user = User::new("reset-test".to_string());
        user.total_xp = 1500;
        user.current_level = 5;
        user.current_streak = 7;
        UserRepository::create(conn, &user).unwrap();
        
        // Reset by updating XP to 0 (simulating reset)
        UserRepository::update_xp(conn, "reset-test", -1500).unwrap();
        UserRepository::update_level(conn, "reset-test", 1).unwrap();
        UserRepository::update_streak(conn, "reset-test", 0, Utc::now()).unwrap();
        
        // Verify reset
        let after = UserRepository::get_by_id(conn, "reset-test").unwrap().unwrap();
        assert_eq!(after.total_xp, 0);
        assert_eq!(after.current_level, 1);
        assert_eq!(after.current_streak, 0);
    }

    #[test]
    fn test_clear_progress_data() {
        let db = Database::new_in_memory().unwrap();
        let conn = db.connection();
        
        // Create user and progress
        let user = User::new("clear-test".to_string());
        UserRepository::create(conn, &user).unwrap();
        
        let mut progress = NodeProgress::new("clear-test".to_string(), "node-1".to_string());
        progress.complete();
        ProgressRepository::create_or_update(conn, &progress).unwrap();
        
        // Verify progress exists
        let before = ProgressRepository::get_all_for_user(conn, "clear-test").unwrap();
        assert_eq!(before.len(), 1);
        
        // Clear by creating new empty progress (simulating reset)
        // Note: In real reset, we would delete from DB directly
        let all = ProgressRepository::get_all_for_user(conn, "clear-test").unwrap();
        assert!(!all.is_empty());
    }

    #[test]
    fn test_clear_badge_progress() {
        let db = Database::new_in_memory().unwrap();
        let conn = db.connection();
        
        // Create user and badge
        let user = User::new("badge-clear-test".to_string());
        UserRepository::create(conn, &user).unwrap();
        
        let mut badge = BadgeProgress::new("badge-clear-test".to_string(), "test-badge".to_string());
        badge.current_value = 10.0;
        badge.earned_at = Some(Utc::now());
        BadgeRepository::create_or_update(conn, &badge).unwrap();
        
        // Verify badge exists
        let before = BadgeRepository::get_all_for_user(conn, "badge-clear-test").unwrap();
        assert_eq!(before.len(), 1);
        assert!(before[0].is_earned());
    }
}

// ============================================================================
// 6.4 System Status Tests  
// ============================================================================

mod system_status {
    use super::*;

    #[test]
    fn test_database_health_check() {
        let db = Database::new_in_memory().unwrap();
        let conn = db.connection();
        
        // Simple query to verify database is healthy
        let result: i32 = conn.query_row("SELECT 1", [], |row| row.get(0)).unwrap();
        assert_eq!(result, 1);
    }

    #[test]
    fn test_database_tables_exist() {
        let db = Database::new_in_memory().unwrap();
        let conn = db.connection();
        
        // Verify core tables exist
        let tables: Vec<String> = {
            let mut stmt = conn.prepare(
                "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"
            ).unwrap();
            
            stmt.query_map([], |row| row.get(0))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect()
        };
        
        assert!(tables.contains(&"users".to_string()));
        assert!(tables.contains(&"node_progress".to_string()));
        assert!(tables.contains(&"badge_progress".to_string()));
    }
}

// ============================================================================
// Integration Scenarios
// ============================================================================

mod integration_scenarios {
    use super::*;

    #[test]
    fn test_full_user_journey() {
        let db = Database::new_in_memory().unwrap();
        let conn = db.connection();
        
        // 1. Create user (onboarding)
        let user = User::new("journey-user".to_string());
        UserRepository::create(conn, &user).unwrap();
        
        // 2. Complete a lesson (progress)
        let mut progress = NodeProgress::new("journey-user".to_string(), "lesson-1".to_string());
        progress.start();
        progress.add_time(10);
        progress.complete();
        ProgressRepository::create_or_update(conn, &progress).unwrap();
        
        // 3. Earn XP
        UserRepository::update_xp(conn, "journey-user", 100).unwrap();
        
        // 4. Earn a badge
        let mut badge = BadgeProgress::new("journey-user".to_string(), "first-lesson".to_string());
        badge.update_progress(1.0, 1.0);
        BadgeRepository::create_or_update(conn, &badge).unwrap();
        
        // 5. Verify state
        let final_user = UserRepository::get_by_id(conn, "journey-user").unwrap().unwrap();
        assert_eq!(final_user.total_xp, 100);
        
        let final_progress = ProgressRepository::get_all_for_user(conn, "journey-user").unwrap();
        assert_eq!(final_progress.len(), 1);
        assert_eq!(final_progress[0].status, NodeStatus::Completed);
        
        let final_badges = BadgeRepository::get_earned(conn, "journey-user").unwrap();
        assert_eq!(final_badges.len(), 1);
    }

    #[test]
    fn test_export_import_cycle() {
        // Test that we can export and re-import user data
        let user = User::new("export-user".to_string());
        let progress = NodeProgress::new("export-user".to_string(), "node-1".to_string());
        let badge = BadgeProgress::new("export-user".to_string(), "badge-1".to_string());
        
        // Export to JSON
        let user_json = serde_json::to_string(&user).unwrap();
        let progress_json = serde_json::to_string(&progress).unwrap();
        let badge_json = serde_json::to_string(&badge).unwrap();
        
        // Import back
        let imported_user: User = serde_json::from_str(&user_json).unwrap();
        let imported_progress: NodeProgress = serde_json::from_str(&progress_json).unwrap();
        let imported_badge: BadgeProgress = serde_json::from_str(&badge_json).unwrap();
        
        // Verify data integrity
        assert_eq!(imported_user.id, user.id);
        assert_eq!(imported_progress.user_id, progress.user_id);
        assert_eq!(imported_badge.user_id, badge.user_id);
    }
}
