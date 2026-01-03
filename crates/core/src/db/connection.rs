use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;
use crate::db::error::{DbError, DbResult};
use crate::db::migrations;

#[derive(Debug)]
pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(db_path: PathBuf) -> DbResult<Self> {
        let conn = Connection::open(&db_path)?;

        // Enable foreign keys
        conn.execute("PRAGMA foreign_keys = ON", [])?;

        // Enable WAL mode for better concurrency
        conn.pragma_update(None, "journal_mode", "WAL")?;

        // Run migrations
        migrations::run_migrations(&conn)?;

        Ok(Self { conn })
    }

    pub fn new_in_memory() -> DbResult<Self> {
        let conn = Connection::open_in_memory()?;

        // Enable foreign keys
        conn.execute("PRAGMA foreign_keys = ON", [])?;

        // Run migrations
        migrations::run_migrations(&conn)?;

        Ok(Self { conn })
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }
}

/// Thread-safe wrapper for Tauri state
pub struct AppDatabase {
    pub db: Mutex<Database>,
}

impl AppDatabase {
    pub fn new(db_path: PathBuf) -> DbResult<Self> {
        Ok(Self {
            db: Mutex::new(Database::new(db_path)?),
        })
    }

    pub fn new_in_memory() -> DbResult<Self> {
        Ok(Self {
            db: Mutex::new(Database::new_in_memory()?),
        })
    }

    pub fn with_connection<F, T>(&self, f: F) -> DbResult<T>
    where
        F: FnOnce(&Connection) -> DbResult<T>,
    {
        let db = self.db.lock().map_err(|e| DbError::InvalidData(e.to_string()))?;
        f(db.connection())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_create_database() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        
        let db = Database::new(db_path.clone());
        assert!(db.is_ok(), "Failed to create database: {:?}", db);
        assert!(db_path.exists());
    }

    #[test]
    fn test_create_in_memory_database() {
        let db = Database::new_in_memory();
        assert!(db.is_ok(), "Failed to create in-memory database: {:?}", db);
    }

    #[test]
    fn test_app_database_with_connection() {
        let app_db = AppDatabase::new_in_memory().unwrap();
        
        let result = app_db.with_connection(|conn| {
            // Test that we can query the database
            let count: i32 = conn.query_row(
                "SELECT COUNT(*) FROM users",
                [],
                |row| row.get(0),
            )?;
            Ok(count)
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }
}
