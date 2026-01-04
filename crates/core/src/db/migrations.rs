use rusqlite::Connection;
use crate::db::error::{DbError, DbResult};

pub const CURRENT_VERSION: i32 = 2;

pub fn run_migrations(conn: &Connection) -> DbResult<()> {
    // Get current version
    let version: i32 = conn
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap_or(0);

    if version < CURRENT_VERSION {
        println!("Running migrations from v{} to v{}", version, CURRENT_VERSION);

        // Run each migration in order
        if version < 1 {
            migrate_to_v1(conn)?;
        }

        if version < 2 {
            migrate_to_v2(conn)?;
        }

        // Update version
        conn.pragma_update(None, "user_version", CURRENT_VERSION)?;
        println!("Database now at version {}", CURRENT_VERSION);
    }

    Ok(())
}

fn migrate_to_v1(conn: &Connection) -> DbResult<()> {
    println!("  Running migration to v1 (initial schema)");

    // Read schema.sql and execute it
    let schema_sql = include_str!("schema.sql");
    conn.execute_batch(schema_sql)
        .map_err(|e| DbError::Migration(format!("Failed to apply schema: {}", e)))?;

    Ok(())
}

fn migrate_to_v2(conn: &Connection) -> DbResult<()> {
    println!("  Running migration to v2 (curricula support)");

    // Create curricula table
    conn.execute_batch(
        r#"
        -- Curricula table for tracking imported content packs
        CREATE TABLE IF NOT EXISTS curricula (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            version TEXT NOT NULL,
            description TEXT,
            author TEXT,
            imported_at TEXT NOT NULL DEFAULT (datetime('now')),
            content_path TEXT NOT NULL,
            is_active INTEGER NOT NULL DEFAULT 0,
            CHECK (is_active IN (0, 1))
        );

        CREATE INDEX IF NOT EXISTS idx_curricula_active ON curricula(is_active);

        -- Add curriculum_id to node_progress
        ALTER TABLE node_progress ADD COLUMN curriculum_id TEXT REFERENCES curricula(id);
        CREATE INDEX IF NOT EXISTS idx_node_progress_curriculum ON node_progress(curriculum_id);

        -- Add curriculum_id to quiz_attempts
        ALTER TABLE quiz_attempts ADD COLUMN curriculum_id TEXT REFERENCES curricula(id);
        CREATE INDEX IF NOT EXISTS idx_quiz_curriculum ON quiz_attempts(curriculum_id);

        -- Add curriculum_id to challenge_attempts
        ALTER TABLE challenge_attempts ADD COLUMN curriculum_id TEXT REFERENCES curricula(id);
        CREATE INDEX IF NOT EXISTS idx_challenge_curriculum ON challenge_attempts(curriculum_id);

        -- Add curriculum_id to mastery_scores
        ALTER TABLE mastery_scores ADD COLUMN curriculum_id TEXT REFERENCES curricula(id);
        CREATE INDEX IF NOT EXISTS idx_mastery_curriculum ON mastery_scores(curriculum_id);

        -- Add curriculum_id to badge_progress
        ALTER TABLE badge_progress ADD COLUMN curriculum_id TEXT REFERENCES curricula(id);
        CREATE INDEX IF NOT EXISTS idx_badge_curriculum ON badge_progress(curriculum_id);

        -- Add curriculum_id to review_items
        ALTER TABLE review_items ADD COLUMN curriculum_id TEXT REFERENCES curricula(id);
        CREATE INDEX IF NOT EXISTS idx_review_curriculum ON review_items(curriculum_id);
        "#,
    )
    .map_err(|e| DbError::Migration(format!("Failed to add curricula support: {}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_migrations_run_successfully() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = Connection::open(&db_path).unwrap();

        // Run migrations
        let result = run_migrations(&conn);
        assert!(result.is_ok(), "Migrations failed: {:?}", result);

        // Check version was updated
        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, CURRENT_VERSION);
    }

    #[test]
    fn test_migrations_are_idempotent() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = Connection::open(&db_path).unwrap();

        // Run migrations twice
        run_migrations(&conn).unwrap();
        let result = run_migrations(&conn);
        assert!(result.is_ok(), "Second migration run failed: {:?}", result);
    }
}
