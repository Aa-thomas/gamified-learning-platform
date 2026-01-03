use rusqlite::Connection;
use crate::db::error::{DbError, DbResult};

pub const CURRENT_VERSION: i32 = 1;

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

        // Future migrations go here
        // if version < 2 {
        //     migrate_to_v2(conn)?;
        // }

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
