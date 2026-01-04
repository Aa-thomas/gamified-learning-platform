use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use crate::db::error::DbResult;
use crate::models::Curriculum;

pub struct CurriculumRepository;

impl CurriculumRepository {
    /// Create a new curriculum record
    pub fn create(conn: &Connection, curriculum: &Curriculum) -> DbResult<()> {
        conn.execute(
            "INSERT INTO curricula (id, name, version, description, author, imported_at, content_path, is_active)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                curriculum.id,
                curriculum.name,
                curriculum.version,
                curriculum.description,
                curriculum.author,
                curriculum.imported_at.to_rfc3339(),
                curriculum.content_path,
                curriculum.is_active as i32,
            ],
        )?;
        Ok(())
    }

    /// Get a curriculum by ID
    pub fn get(conn: &Connection, id: &str) -> DbResult<Option<Curriculum>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, version, description, author, imported_at, content_path, is_active
             FROM curricula WHERE id = ?1"
        )?;

        let curriculum = stmt.query_row(params![id], |row| {
            Ok(Curriculum {
                id: row.get(0)?,
                name: row.get(1)?,
                version: row.get(2)?,
                description: row.get(3)?,
                author: row.get(4)?,
                imported_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(5, rusqlite::types::Type::Text, Box::new(e)))?
                    .with_timezone(&Utc),
                content_path: row.get(6)?,
                is_active: row.get::<_, i32>(7)? != 0,
            })
        }).optional()?;

        Ok(curriculum)
    }

    /// Get all curricula
    pub fn get_all(conn: &Connection) -> DbResult<Vec<Curriculum>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, version, description, author, imported_at, content_path, is_active
             FROM curricula ORDER BY imported_at DESC"
        )?;

        let curricula_iter = stmt.query_map([], |row| {
            Ok(Curriculum {
                id: row.get(0)?,
                name: row.get(1)?,
                version: row.get(2)?,
                description: row.get(3)?,
                author: row.get(4)?,
                imported_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(5, rusqlite::types::Type::Text, Box::new(e)))?
                    .with_timezone(&Utc),
                content_path: row.get(6)?,
                is_active: row.get::<_, i32>(7)? != 0,
            })
        })?;

        let mut results = Vec::new();
        for curriculum in curricula_iter {
            results.push(curriculum?);
        }
        Ok(results)
    }

    /// Get the currently active curriculum
    pub fn get_active(conn: &Connection) -> DbResult<Option<Curriculum>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, version, description, author, imported_at, content_path, is_active
             FROM curricula WHERE is_active = 1 LIMIT 1"
        )?;

        let curriculum = stmt.query_row([], |row| {
            Ok(Curriculum {
                id: row.get(0)?,
                name: row.get(1)?,
                version: row.get(2)?,
                description: row.get(3)?,
                author: row.get(4)?,
                imported_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(5, rusqlite::types::Type::Text, Box::new(e)))?
                    .with_timezone(&Utc),
                content_path: row.get(6)?,
                is_active: row.get::<_, i32>(7)? != 0,
            })
        }).optional()?;

        Ok(curriculum)
    }

    /// Set a curriculum as active (deactivates all others)
    pub fn set_active(conn: &Connection, id: &str) -> DbResult<()> {
        // Deactivate all curricula
        conn.execute("UPDATE curricula SET is_active = 0", [])?;
        
        // Activate the specified one
        conn.execute(
            "UPDATE curricula SET is_active = 1 WHERE id = ?1",
            params![id],
        )?;
        
        Ok(())
    }

    /// Delete a curriculum by ID
    pub fn delete(conn: &Connection, id: &str) -> DbResult<()> {
        conn.execute("DELETE FROM curricula WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Delete a curriculum and all associated progress data
    pub fn delete_with_progress(conn: &Connection, id: &str) -> DbResult<()> {
        // Delete associated progress data first
        conn.execute("DELETE FROM node_progress WHERE curriculum_id = ?1", params![id])?;
        conn.execute("DELETE FROM quiz_attempts WHERE curriculum_id = ?1", params![id])?;
        conn.execute("DELETE FROM challenge_attempts WHERE curriculum_id = ?1", params![id])?;
        conn.execute("DELETE FROM mastery_scores WHERE curriculum_id = ?1", params![id])?;
        conn.execute("DELETE FROM badge_progress WHERE curriculum_id = ?1", params![id])?;
        conn.execute("DELETE FROM review_items WHERE curriculum_id = ?1", params![id])?;
        
        // Delete the curriculum itself
        conn.execute("DELETE FROM curricula WHERE id = ?1", params![id])?;
        
        Ok(())
    }

    /// Check if a curriculum with the given name and version already exists
    pub fn exists_by_name_version(conn: &Connection, name: &str, version: &str) -> DbResult<bool> {
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM curricula WHERE name = ?1 AND version = ?2",
            params![name, version],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::connection::Database;

    fn setup_db() -> Database {
        Database::new_in_memory().unwrap()
    }

    #[test]
    fn test_create_and_get_curriculum() {
        let db = setup_db();
        let conn = db.connection();

        let curriculum = Curriculum::new(
            "Test Course".to_string(),
            "1.0.0".to_string(),
            "curricula/test-course".to_string(),
        );

        CurriculumRepository::create(conn, &curriculum).unwrap();
        
        let retrieved = CurriculumRepository::get(conn, &curriculum.id).unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.name, "Test Course");
        assert_eq!(retrieved.version, "1.0.0");
        assert!(!retrieved.is_active);
    }

    #[test]
    fn test_get_all() {
        let db = setup_db();
        let conn = db.connection();

        let c1 = Curriculum::new("Course 1".to_string(), "1.0".to_string(), "c1".to_string());
        let c2 = Curriculum::new("Course 2".to_string(), "1.0".to_string(), "c2".to_string());

        CurriculumRepository::create(conn, &c1).unwrap();
        CurriculumRepository::create(conn, &c2).unwrap();

        let all = CurriculumRepository::get_all(conn).unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_set_active() {
        let db = setup_db();
        let conn = db.connection();

        let c1 = Curriculum::new("Course 1".to_string(), "1.0".to_string(), "c1".to_string());
        let c2 = Curriculum::new("Course 2".to_string(), "1.0".to_string(), "c2".to_string());

        CurriculumRepository::create(conn, &c1).unwrap();
        CurriculumRepository::create(conn, &c2).unwrap();

        // Set c1 as active
        CurriculumRepository::set_active(conn, &c1.id).unwrap();
        
        let active = CurriculumRepository::get_active(conn).unwrap();
        assert!(active.is_some());
        assert_eq!(active.unwrap().id, c1.id);

        // Switch to c2
        CurriculumRepository::set_active(conn, &c2.id).unwrap();
        
        let active = CurriculumRepository::get_active(conn).unwrap();
        assert!(active.is_some());
        assert_eq!(active.unwrap().id, c2.id);

        // Verify c1 is no longer active
        let c1_updated = CurriculumRepository::get(conn, &c1.id).unwrap().unwrap();
        assert!(!c1_updated.is_active);
    }

    #[test]
    fn test_delete() {
        let db = setup_db();
        let conn = db.connection();

        let curriculum = Curriculum::new(
            "Test Course".to_string(),
            "1.0.0".to_string(),
            "curricula/test".to_string(),
        );

        CurriculumRepository::create(conn, &curriculum).unwrap();
        CurriculumRepository::delete(conn, &curriculum.id).unwrap();

        let retrieved = CurriculumRepository::get(conn, &curriculum.id).unwrap();
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_exists_by_name_version() {
        let db = setup_db();
        let conn = db.connection();

        let curriculum = Curriculum::new(
            "Test Course".to_string(),
            "1.0.0".to_string(),
            "curricula/test".to_string(),
        );

        assert!(!CurriculumRepository::exists_by_name_version(conn, "Test Course", "1.0.0").unwrap());
        
        CurriculumRepository::create(conn, &curriculum).unwrap();
        
        assert!(CurriculumRepository::exists_by_name_version(conn, "Test Course", "1.0.0").unwrap());
        assert!(!CurriculumRepository::exists_by_name_version(conn, "Test Course", "2.0.0").unwrap());
    }
}
