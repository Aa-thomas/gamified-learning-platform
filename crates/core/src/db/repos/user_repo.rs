use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use crate::db::error::{DbError, DbResult};
use crate::models::User;

pub struct UserRepository;

impl UserRepository {
    pub fn create(conn: &Connection, user: &User) -> DbResult<()> {
        conn.execute(
            "INSERT INTO users (id, created_at, last_activity, total_xp, current_level, current_streak, last_streak_date)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                user.id,
                user.created_at.to_rfc3339(),
                user.last_activity.to_rfc3339(),
                user.total_xp,
                user.current_level,
                user.current_streak,
                user.last_streak_date.map(|d| d.to_rfc3339()),
            ],
        )?;
        Ok(())
    }

    pub fn get_by_id(conn: &Connection, user_id: &str) -> DbResult<Option<User>> {
        let mut stmt = conn.prepare(
            "SELECT id, created_at, last_activity, total_xp, current_level, current_streak, last_streak_date
             FROM users WHERE id = ?1"
        )?;

        let user = stmt.query_row(params![user_id], |row| {
            Ok(User {
                id: row.get(0)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(1)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(1, rusqlite::types::Type::Text, Box::new(e)))?
                    .with_timezone(&Utc),
                last_activity: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, Box::new(e)))?
                    .with_timezone(&Utc),
                total_xp: row.get(3)?,
                current_level: row.get(4)?,
                current_streak: row.get(5)?,
                last_streak_date: row.get::<_, Option<String>>(6)?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
            })
        }).optional()?;

        Ok(user)
    }

    pub fn update_xp(conn: &Connection, user_id: &str, xp_delta: i32) -> DbResult<()> {
        let rows = conn.execute(
            "UPDATE users SET total_xp = total_xp + ?1, last_activity = ?2 WHERE id = ?3",
            params![xp_delta, Utc::now().to_rfc3339(), user_id],
        )?;

        if rows == 0 {
            return Err(DbError::NotFound(format!("User not found: {}", user_id)));
        }
        Ok(())
    }

    pub fn update_level(conn: &Connection, user_id: &str, new_level: i32) -> DbResult<()> {
        let rows = conn.execute(
            "UPDATE users SET current_level = ?1, last_activity = ?2 WHERE id = ?3",
            params![new_level, Utc::now().to_rfc3339(), user_id],
        )?;

        if rows == 0 {
            return Err(DbError::NotFound(format!("User not found: {}", user_id)));
        }
        Ok(())
    }

    pub fn update_streak(conn: &Connection, user_id: &str, new_streak: i32, streak_date: DateTime<Utc>) -> DbResult<()> {
        let rows = conn.execute(
            "UPDATE users SET current_streak = ?1, last_streak_date = ?2, last_activity = ?3 WHERE id = ?4",
            params![new_streak, streak_date.to_rfc3339(), Utc::now().to_rfc3339(), user_id],
        )?;

        if rows == 0 {
            return Err(DbError::NotFound(format!("User not found: {}", user_id)));
        }
        Ok(())
    }

    pub fn delete(conn: &Connection, user_id: &str) -> DbResult<()> {
        let rows = conn.execute("DELETE FROM users WHERE id = ?1", params![user_id])?;

        if rows == 0 {
            return Err(DbError::NotFound(format!("User not found: {}", user_id)));
        }
        Ok(())
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
    fn test_create_and_get_user() {
        let db = setup_db();
        let conn = db.connection();

        let user = User::new("test-user".to_string());
        UserRepository::create(conn, &user).unwrap();

        let retrieved = UserRepository::get_by_id(conn, "test-user").unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, "test-user");
        assert_eq!(retrieved.total_xp, 0);
        assert_eq!(retrieved.current_level, 1);
    }

    #[test]
    fn test_get_nonexistent_user() {
        let db = setup_db();
        let conn = db.connection();

        let result = UserRepository::get_by_id(conn, "nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_update_xp() {
        let db = setup_db();
        let conn = db.connection();

        let user = User::new("test-user".to_string());
        UserRepository::create(conn, &user).unwrap();

        UserRepository::update_xp(conn, "test-user", 100).unwrap();

        let updated = UserRepository::get_by_id(conn, "test-user").unwrap().unwrap();
        assert_eq!(updated.total_xp, 100);

        // Add more XP
        UserRepository::update_xp(conn, "test-user", 50).unwrap();
        let updated = UserRepository::get_by_id(conn, "test-user").unwrap().unwrap();
        assert_eq!(updated.total_xp, 150);
    }

    #[test]
    fn test_update_level() {
        let db = setup_db();
        let conn = db.connection();

        let user = User::new("test-user".to_string());
        UserRepository::create(conn, &user).unwrap();

        UserRepository::update_level(conn, "test-user", 5).unwrap();

        let updated = UserRepository::get_by_id(conn, "test-user").unwrap().unwrap();
        assert_eq!(updated.current_level, 5);
    }

    #[test]
    fn test_update_streak() {
        let db = setup_db();
        let conn = db.connection();

        let user = User::new("test-user".to_string());
        UserRepository::create(conn, &user).unwrap();

        let streak_date = Utc::now();
        UserRepository::update_streak(conn, "test-user", 7, streak_date).unwrap();

        let updated = UserRepository::get_by_id(conn, "test-user").unwrap().unwrap();
        assert_eq!(updated.current_streak, 7);
        assert!(updated.last_streak_date.is_some());
    }

    #[test]
    fn test_delete_user() {
        let db = setup_db();
        let conn = db.connection();

        let user = User::new("test-user".to_string());
        UserRepository::create(conn, &user).unwrap();

        UserRepository::delete(conn, "test-user").unwrap();

        let result = UserRepository::get_by_id(conn, "test-user").unwrap();
        assert!(result.is_none());
    }
}
