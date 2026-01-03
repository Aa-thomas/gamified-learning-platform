use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use crate::db::error::DbResult;
use crate::models::SessionHistory;

pub struct SessionRepository;

impl SessionRepository {
    pub fn create(conn: &Connection, session: &SessionHistory) -> DbResult<()> {
        conn.execute(
            "INSERT INTO session_history (id, user_id, started_at, ended_at, total_xp_earned, items_completed)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                session.id,
                session.user_id,
                session.started_at.to_rfc3339(),
                session.ended_at.map(|d| d.to_rfc3339()),
                session.total_xp_earned,
                session.items_completed,
            ],
        )?;
        Ok(())
    }

    pub fn get_by_id(conn: &Connection, session_id: &str) -> DbResult<Option<SessionHistory>> {
        let mut stmt = conn.prepare(
            "SELECT id, user_id, started_at, ended_at, total_xp_earned, items_completed
             FROM session_history WHERE id = ?1"
        )?;

        let session = stmt.query_row(params![session_id], |row| {
            Ok(SessionHistory {
                id: row.get(0)?,
                user_id: row.get(1)?,
                started_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, Box::new(e)))?
                    .with_timezone(&Utc),
                ended_at: row.get::<_, Option<String>>(3)?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                total_xp_earned: row.get(4)?,
                items_completed: row.get(5)?,
            })
        }).optional()?;

        Ok(session)
    }

    pub fn update(conn: &Connection, session: &SessionHistory) -> DbResult<()> {
        conn.execute(
            "UPDATE session_history SET ended_at = ?1, total_xp_earned = ?2, items_completed = ?3
             WHERE id = ?4",
            params![
                session.ended_at.map(|d| d.to_rfc3339()),
                session.total_xp_earned,
                session.items_completed,
                session.id,
            ],
        )?;
        Ok(())
    }

    pub fn get_active_session(conn: &Connection, user_id: &str) -> DbResult<Option<SessionHistory>> {
        let mut stmt = conn.prepare(
            "SELECT id, user_id, started_at, ended_at, total_xp_earned, items_completed
             FROM session_history WHERE user_id = ?1 AND ended_at IS NULL
             ORDER BY started_at DESC LIMIT 1"
        )?;

        let session = stmt.query_row(params![user_id], |row| {
            Ok(SessionHistory {
                id: row.get(0)?,
                user_id: row.get(1)?,
                started_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, Box::new(e)))?
                    .with_timezone(&Utc),
                ended_at: row.get::<_, Option<String>>(3)?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                total_xp_earned: row.get(4)?,
                items_completed: row.get(5)?,
            })
        }).optional()?;

        Ok(session)
    }

    pub fn get_recent(conn: &Connection, user_id: &str, limit: i32) -> DbResult<Vec<SessionHistory>> {
        let mut stmt = conn.prepare(
            "SELECT id, user_id, started_at, ended_at, total_xp_earned, items_completed
             FROM session_history WHERE user_id = ?1 ORDER BY started_at DESC LIMIT ?2"
        )?;

        let session_iter = stmt.query_map(params![user_id, limit], |row| {
            Ok(SessionHistory {
                id: row.get(0)?,
                user_id: row.get(1)?,
                started_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, Box::new(e)))?
                    .with_timezone(&Utc),
                ended_at: row.get::<_, Option<String>>(3)?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                total_xp_earned: row.get(4)?,
                items_completed: row.get(5)?,
            })
        })?;

        let mut results = Vec::new();
        for session in session_iter {
            results.push(session?);
        }
        Ok(results)
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
        let user = User::new("test-user".to_string());
        UserRepository::create(db.connection(), &user).unwrap();
        db
    }

    #[test]
    fn test_create_and_get_session() {
        let db = setup_db();
        let conn = db.connection();

        let session = SessionHistory::new("test-user".to_string());
        SessionRepository::create(conn, &session).unwrap();

        let retrieved = SessionRepository::get_by_id(conn, &session.id).unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert!(retrieved.is_active());
    }

    #[test]
    fn test_get_active_session() {
        let db = setup_db();
        let conn = db.connection();

        let session = SessionHistory::new("test-user".to_string());
        SessionRepository::create(conn, &session).unwrap();

        let active = SessionRepository::get_active_session(conn, "test-user").unwrap();
        assert!(active.is_some());
        assert_eq!(active.unwrap().id, session.id);
    }

    #[test]
    fn test_update_session() {
        let db = setup_db();
        let conn = db.connection();

        let mut session = SessionHistory::new("test-user".to_string());
        SessionRepository::create(conn, &session).unwrap();

        session.add_completion(100);
        session.end_session();
        SessionRepository::update(conn, &session).unwrap();

        let updated = SessionRepository::get_by_id(conn, &session.id).unwrap().unwrap();
        assert!(!updated.is_active());
        assert_eq!(updated.total_xp_earned, 100);
        assert_eq!(updated.items_completed, 1);
    }
}
