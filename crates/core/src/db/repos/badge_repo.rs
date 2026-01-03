use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use crate::db::error::DbResult;
use crate::models::BadgeProgress;

pub struct BadgeRepository;

impl BadgeRepository {
    pub fn create_or_update(conn: &Connection, badge: &BadgeProgress) -> DbResult<()> {
        conn.execute(
            "INSERT INTO badge_progress (user_id, badge_id, current_value, earned_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(user_id, badge_id) DO UPDATE SET
                current_value = excluded.current_value,
                earned_at = COALESCE(badge_progress.earned_at, excluded.earned_at)",
            params![
                badge.user_id,
                badge.badge_id,
                badge.current_value,
                badge.earned_at.map(|d| d.to_rfc3339()),
            ],
        )?;
        Ok(())
    }

    pub fn get(conn: &Connection, user_id: &str, badge_id: &str) -> DbResult<Option<BadgeProgress>> {
        let mut stmt = conn.prepare(
            "SELECT user_id, badge_id, current_value, earned_at
             FROM badge_progress WHERE user_id = ?1 AND badge_id = ?2"
        )?;

        let badge = stmt.query_row(params![user_id, badge_id], |row| {
            Ok(BadgeProgress {
                user_id: row.get(0)?,
                badge_id: row.get(1)?,
                current_value: row.get(2)?,
                earned_at: row.get::<_, Option<String>>(3)?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
            })
        }).optional()?;

        Ok(badge)
    }

    pub fn get_all_for_user(conn: &Connection, user_id: &str) -> DbResult<Vec<BadgeProgress>> {
        let mut stmt = conn.prepare(
            "SELECT user_id, badge_id, current_value, earned_at
             FROM badge_progress WHERE user_id = ?1"
        )?;

        let badge_iter = stmt.query_map(params![user_id], |row| {
            Ok(BadgeProgress {
                user_id: row.get(0)?,
                badge_id: row.get(1)?,
                current_value: row.get(2)?,
                earned_at: row.get::<_, Option<String>>(3)?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
            })
        })?;

        let mut results = Vec::new();
        for badge in badge_iter {
            results.push(badge?);
        }
        Ok(results)
    }

    pub fn get_earned(conn: &Connection, user_id: &str) -> DbResult<Vec<BadgeProgress>> {
        let mut stmt = conn.prepare(
            "SELECT user_id, badge_id, current_value, earned_at
             FROM badge_progress WHERE user_id = ?1 AND earned_at IS NOT NULL"
        )?;

        let badge_iter = stmt.query_map(params![user_id], |row| {
            Ok(BadgeProgress {
                user_id: row.get(0)?,
                badge_id: row.get(1)?,
                current_value: row.get(2)?,
                earned_at: row.get::<_, Option<String>>(3)?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
            })
        })?;

        let mut results = Vec::new();
        for badge in badge_iter {
            results.push(badge?);
        }
        Ok(results)
    }

    pub fn mark_earned(conn: &Connection, user_id: &str, badge_id: &str) -> DbResult<()> {
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE badge_progress SET earned_at = ?1 WHERE user_id = ?2 AND badge_id = ?3 AND earned_at IS NULL",
            params![now, user_id, badge_id],
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
        let user = User::new("test-user".to_string());
        UserRepository::create(db.connection(), &user).unwrap();
        db
    }

    #[test]
    fn test_create_and_get_badge() {
        let db = setup_db();
        let conn = db.connection();

        let badge = BadgeProgress::new("test-user".to_string(), "week_warrior".to_string());
        BadgeRepository::create_or_update(conn, &badge).unwrap();

        let retrieved = BadgeRepository::get(conn, "test-user", "week_warrior").unwrap();
        assert!(retrieved.is_some());
        assert!(!retrieved.unwrap().is_earned());
    }

    #[test]
    fn test_mark_earned() {
        let db = setup_db();
        let conn = db.connection();

        let badge = BadgeProgress::new("test-user".to_string(), "week_warrior".to_string());
        BadgeRepository::create_or_update(conn, &badge).unwrap();

        BadgeRepository::mark_earned(conn, "test-user", "week_warrior").unwrap();

        let updated = BadgeRepository::get(conn, "test-user", "week_warrior").unwrap().unwrap();
        assert!(updated.is_earned());
    }

    #[test]
    fn test_get_earned() {
        let db = setup_db();
        let conn = db.connection();

        let mut badge1 = BadgeProgress::new("test-user".to_string(), "badge1".to_string());
        badge1.earned_at = Some(Utc::now());
        let badge2 = BadgeProgress::new("test-user".to_string(), "badge2".to_string());

        BadgeRepository::create_or_update(conn, &badge1).unwrap();
        BadgeRepository::create_or_update(conn, &badge2).unwrap();

        let earned = BadgeRepository::get_earned(conn, "test-user").unwrap();
        assert_eq!(earned.len(), 1);
        assert_eq!(earned[0].badge_id, "badge1");
    }
}
