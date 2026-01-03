use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use crate::db::error::DbResult;
use crate::models::{NodeProgress, NodeStatus};

pub struct ProgressRepository;

impl ProgressRepository {
    pub fn create_or_update(conn: &Connection, progress: &NodeProgress) -> DbResult<()> {
        conn.execute(
            "INSERT INTO node_progress (user_id, node_id, status, attempts, time_spent_mins, first_started_at, completed_at, last_updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(user_id, node_id) DO UPDATE SET
                status = excluded.status,
                attempts = excluded.attempts,
                time_spent_mins = excluded.time_spent_mins,
                first_started_at = COALESCE(node_progress.first_started_at, excluded.first_started_at),
                completed_at = excluded.completed_at,
                last_updated_at = excluded.last_updated_at",
            params![
                progress.user_id,
                progress.node_id,
                progress.status.as_str(),
                progress.attempts,
                progress.time_spent_mins,
                progress.first_started_at.map(|d| d.to_rfc3339()),
                progress.completed_at.map(|d| d.to_rfc3339()),
                progress.last_updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn get(conn: &Connection, user_id: &str, node_id: &str) -> DbResult<Option<NodeProgress>> {
        let mut stmt = conn.prepare(
            "SELECT user_id, node_id, status, attempts, time_spent_mins, first_started_at, completed_at, last_updated_at
             FROM node_progress WHERE user_id = ?1 AND node_id = ?2"
        )?;

        let progress = stmt.query_row(params![user_id, node_id], |row| {
            Ok(NodeProgress {
                user_id: row.get(0)?,
                node_id: row.get(1)?,
                status: NodeStatus::from_str(&row.get::<_, String>(2)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e))))?,
                attempts: row.get(3)?,
                time_spent_mins: row.get(4)?,
                first_started_at: row.get::<_, Option<String>>(5)?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                completed_at: row.get::<_, Option<String>>(6)?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                last_updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(7, rusqlite::types::Type::Text, Box::new(e)))?
                    .with_timezone(&Utc),
            })
        }).optional()?;

        Ok(progress)
    }

    pub fn get_all_for_user(conn: &Connection, user_id: &str) -> DbResult<Vec<NodeProgress>> {
        let mut stmt = conn.prepare(
            "SELECT user_id, node_id, status, attempts, time_spent_mins, first_started_at, completed_at, last_updated_at
             FROM node_progress WHERE user_id = ?1"
        )?;

        let progress_iter = stmt.query_map(params![user_id], |row| {
            Ok(NodeProgress {
                user_id: row.get(0)?,
                node_id: row.get(1)?,
                status: NodeStatus::from_str(&row.get::<_, String>(2)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e))))?,
                attempts: row.get(3)?,
                time_spent_mins: row.get(4)?,
                first_started_at: row.get::<_, Option<String>>(5)?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                completed_at: row.get::<_, Option<String>>(6)?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                last_updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(7, rusqlite::types::Type::Text, Box::new(e)))?
                    .with_timezone(&Utc),
            })
        })?;

        let mut results = Vec::new();
        for progress in progress_iter {
            results.push(progress?);
        }
        Ok(results)
    }

    pub fn get_by_status(conn: &Connection, user_id: &str, status: &NodeStatus) -> DbResult<Vec<NodeProgress>> {
        let mut stmt = conn.prepare(
            "SELECT user_id, node_id, status, attempts, time_spent_mins, first_started_at, completed_at, last_updated_at
             FROM node_progress WHERE user_id = ?1 AND status = ?2"
        )?;

        let progress_iter = stmt.query_map(params![user_id, status.as_str()], |row| {
            Ok(NodeProgress {
                user_id: row.get(0)?,
                node_id: row.get(1)?,
                status: NodeStatus::from_str(&row.get::<_, String>(2)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e))))?,
                attempts: row.get(3)?,
                time_spent_mins: row.get(4)?,
                first_started_at: row.get::<_, Option<String>>(5)?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                completed_at: row.get::<_, Option<String>>(6)?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                last_updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(7, rusqlite::types::Type::Text, Box::new(e)))?
                    .with_timezone(&Utc),
            })
        })?;

        let mut results = Vec::new();
        for progress in progress_iter {
            results.push(progress?);
        }
        Ok(results)
    }

    pub fn mark_completed(conn: &Connection, user_id: &str, node_id: &str) -> DbResult<()> {
        let now = Utc::now().to_rfc3339();
        let rows = conn.execute(
            "UPDATE node_progress SET status = 'Completed', completed_at = ?1, last_updated_at = ?1
             WHERE user_id = ?2 AND node_id = ?3",
            params![now, user_id, node_id],
        )?;

        if rows == 0 {
            // Create new progress entry if it doesn't exist
            let mut progress = NodeProgress::new(user_id.to_string(), node_id.to_string());
            progress.complete();
            Self::create_or_update(conn, &progress)?;
        }
        Ok(())
    }

    pub fn increment_time(conn: &Connection, user_id: &str, node_id: &str, mins: i32) -> DbResult<()> {
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE node_progress SET time_spent_mins = time_spent_mins + ?1, last_updated_at = ?2
             WHERE user_id = ?3 AND node_id = ?4",
            params![mins, now, user_id, node_id],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::connection::Database;
    use crate::db::repos::UserRepository;
    use crate::models::User;

    fn setup_db() -> Database {
        let db = Database::new_in_memory().unwrap();
        // Create a test user
        let user = User::new("test-user".to_string());
        UserRepository::create(db.connection(), &user).unwrap();
        db
    }

    #[test]
    fn test_create_and_get_progress() {
        let db = setup_db();
        let conn = db.connection();

        let mut progress = NodeProgress::new("test-user".to_string(), "node1".to_string());
        progress.start();
        ProgressRepository::create_or_update(conn, &progress).unwrap();

        let retrieved = ProgressRepository::get(conn, "test-user", "node1").unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.status, NodeStatus::InProgress);
    }

    #[test]
    fn test_get_all_for_user() {
        let db = setup_db();
        let conn = db.connection();

        let progress1 = NodeProgress::new("test-user".to_string(), "node1".to_string());
        let progress2 = NodeProgress::new("test-user".to_string(), "node2".to_string());
        ProgressRepository::create_or_update(conn, &progress1).unwrap();
        ProgressRepository::create_or_update(conn, &progress2).unwrap();

        let all = ProgressRepository::get_all_for_user(conn, "test-user").unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_mark_completed() {
        let db = setup_db();
        let conn = db.connection();

        let progress = NodeProgress::new("test-user".to_string(), "node1".to_string());
        ProgressRepository::create_or_update(conn, &progress).unwrap();

        ProgressRepository::mark_completed(conn, "test-user", "node1").unwrap();

        let updated = ProgressRepository::get(conn, "test-user", "node1").unwrap().unwrap();
        assert_eq!(updated.status, NodeStatus::Completed);
        assert!(updated.completed_at.is_some());
    }

    #[test]
    fn test_get_by_status() {
        let db = setup_db();
        let conn = db.connection();

        let mut progress1 = NodeProgress::new("test-user".to_string(), "node1".to_string());
        progress1.complete();
        let progress2 = NodeProgress::new("test-user".to_string(), "node2".to_string());
        ProgressRepository::create_or_update(conn, &progress1).unwrap();
        ProgressRepository::create_or_update(conn, &progress2).unwrap();

        let completed = ProgressRepository::get_by_status(conn, "test-user", &NodeStatus::Completed).unwrap();
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0].node_id, "node1");
    }
}
