use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use crate::db::error::DbResult;
use crate::models::MasteryScore;

pub struct MasteryRepository;

impl MasteryRepository {
    pub fn create_or_update(conn: &Connection, mastery: &MasteryScore) -> DbResult<()> {
        conn.execute(
            "INSERT INTO mastery_scores (user_id, skill_id, score, last_updated_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(user_id, skill_id) DO UPDATE SET
                score = excluded.score,
                last_updated_at = excluded.last_updated_at",
            params![
                mastery.user_id,
                mastery.skill_id,
                mastery.score,
                mastery.last_updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn get(conn: &Connection, user_id: &str, skill_id: &str) -> DbResult<Option<MasteryScore>> {
        let mut stmt = conn.prepare(
            "SELECT user_id, skill_id, score, last_updated_at
             FROM mastery_scores WHERE user_id = ?1 AND skill_id = ?2"
        )?;

        let mastery = stmt.query_row(params![user_id, skill_id], |row| {
            Ok(MasteryScore {
                user_id: row.get(0)?,
                skill_id: row.get(1)?,
                score: row.get(2)?,
                last_updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(3, rusqlite::types::Type::Text, Box::new(e)))?
                    .with_timezone(&Utc),
            })
        }).optional()?;

        Ok(mastery)
    }

    pub fn get_all_for_user(conn: &Connection, user_id: &str) -> DbResult<Vec<MasteryScore>> {
        let mut stmt = conn.prepare(
            "SELECT user_id, skill_id, score, last_updated_at
             FROM mastery_scores WHERE user_id = ?1"
        )?;

        let mastery_iter = stmt.query_map(params![user_id], |row| {
            Ok(MasteryScore {
                user_id: row.get(0)?,
                skill_id: row.get(1)?,
                score: row.get(2)?,
                last_updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(3, rusqlite::types::Type::Text, Box::new(e)))?
                    .with_timezone(&Utc),
            })
        })?;

        let mut results = Vec::new();
        for mastery in mastery_iter {
            results.push(mastery?);
        }
        Ok(results)
    }

    pub fn update_score(conn: &Connection, user_id: &str, skill_id: &str, new_score: f64) -> DbResult<()> {
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE mastery_scores SET score = ?1, last_updated_at = ?2
             WHERE user_id = ?3 AND skill_id = ?4",
            params![new_score, now, user_id, skill_id],
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
    fn test_create_and_get_mastery() {
        let db = setup_db();
        let conn = db.connection();

        let mut mastery = MasteryScore::new("test-user".to_string(), "ownership".to_string());
        mastery.score = 0.75;
        MasteryRepository::create_or_update(conn, &mastery).unwrap();

        let retrieved = MasteryRepository::get(conn, "test-user", "ownership").unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert!((retrieved.score - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_get_all_for_user() {
        let db = setup_db();
        let conn = db.connection();

        let mastery1 = MasteryScore::new("test-user".to_string(), "ownership".to_string());
        let mastery2 = MasteryScore::new("test-user".to_string(), "lifetimes".to_string());
        MasteryRepository::create_or_update(conn, &mastery1).unwrap();
        MasteryRepository::create_or_update(conn, &mastery2).unwrap();

        let all = MasteryRepository::get_all_for_user(conn, "test-user").unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_update_score() {
        let db = setup_db();
        let conn = db.connection();

        let mastery = MasteryScore::new("test-user".to_string(), "ownership".to_string());
        MasteryRepository::create_or_update(conn, &mastery).unwrap();

        MasteryRepository::update_score(conn, "test-user", "ownership", 0.9).unwrap();

        let updated = MasteryRepository::get(conn, "test-user", "ownership").unwrap().unwrap();
        assert!((updated.score - 0.9).abs() < 0.01);
    }
}
