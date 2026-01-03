use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use crate::db::error::DbResult;
use crate::models::QuizAttempt;

pub struct QuizRepository;

impl QuizRepository {
    pub fn create(conn: &Connection, attempt: &QuizAttempt) -> DbResult<()> {
        let answers_json = serde_json::to_string(&attempt.answers)
            .map_err(|e| crate::db::error::DbError::InvalidData(e.to_string()))?;

        conn.execute(
            "INSERT INTO quiz_attempts (id, user_id, quiz_id, node_id, answers_json, score_percentage, xp_earned, submitted_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                attempt.id,
                attempt.user_id,
                attempt.quiz_id,
                attempt.node_id,
                answers_json,
                attempt.score_percentage,
                attempt.xp_earned,
                attempt.submitted_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn get_by_id(conn: &Connection, attempt_id: &str) -> DbResult<Option<QuizAttempt>> {
        let mut stmt = conn.prepare(
            "SELECT id, user_id, quiz_id, node_id, answers_json, score_percentage, xp_earned, submitted_at
             FROM quiz_attempts WHERE id = ?1"
        )?;

        let attempt = stmt.query_row(params![attempt_id], |row| {
            let answers_json: String = row.get(4)?;
            let answers: Vec<String> = serde_json::from_str(&answers_json)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, Box::new(e)))?;

            Ok(QuizAttempt {
                id: row.get(0)?,
                user_id: row.get(1)?,
                quiz_id: row.get(2)?,
                node_id: row.get(3)?,
                answers,
                score_percentage: row.get(5)?,
                xp_earned: row.get(6)?,
                submitted_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(7, rusqlite::types::Type::Text, Box::new(e)))?
                    .with_timezone(&Utc),
            })
        }).optional()?;

        Ok(attempt)
    }

    pub fn get_for_quiz(conn: &Connection, user_id: &str, quiz_id: &str) -> DbResult<Vec<QuizAttempt>> {
        let mut stmt = conn.prepare(
            "SELECT id, user_id, quiz_id, node_id, answers_json, score_percentage, xp_earned, submitted_at
             FROM quiz_attempts WHERE user_id = ?1 AND quiz_id = ?2 ORDER BY submitted_at DESC"
        )?;

        let attempt_iter = stmt.query_map(params![user_id, quiz_id], |row| {
            let answers_json: String = row.get(4)?;
            let answers: Vec<String> = serde_json::from_str(&answers_json)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, Box::new(e)))?;

            Ok(QuizAttempt {
                id: row.get(0)?,
                user_id: row.get(1)?,
                quiz_id: row.get(2)?,
                node_id: row.get(3)?,
                answers,
                score_percentage: row.get(5)?,
                xp_earned: row.get(6)?,
                submitted_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(7, rusqlite::types::Type::Text, Box::new(e)))?
                    .with_timezone(&Utc),
            })
        })?;

        let mut results = Vec::new();
        for attempt in attempt_iter {
            results.push(attempt?);
        }
        Ok(results)
    }

    pub fn get_recent(conn: &Connection, user_id: &str, limit: i32) -> DbResult<Vec<QuizAttempt>> {
        let mut stmt = conn.prepare(
            "SELECT id, user_id, quiz_id, node_id, answers_json, score_percentage, xp_earned, submitted_at
             FROM quiz_attempts WHERE user_id = ?1 ORDER BY submitted_at DESC LIMIT ?2"
        )?;

        let attempt_iter = stmt.query_map(params![user_id, limit], |row| {
            let answers_json: String = row.get(4)?;
            let answers: Vec<String> = serde_json::from_str(&answers_json)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, Box::new(e)))?;

            Ok(QuizAttempt {
                id: row.get(0)?,
                user_id: row.get(1)?,
                quiz_id: row.get(2)?,
                node_id: row.get(3)?,
                answers,
                score_percentage: row.get(5)?,
                xp_earned: row.get(6)?,
                submitted_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(7, rusqlite::types::Type::Text, Box::new(e)))?
                    .with_timezone(&Utc),
            })
        })?;

        let mut results = Vec::new();
        for attempt in attempt_iter {
            results.push(attempt?);
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
    fn test_create_and_get_quiz_attempt() {
        let db = setup_db();
        let conn = db.connection();

        let attempt = QuizAttempt::new(
            "test-user".to_string(),
            "quiz1".to_string(),
            "node1".to_string(),
            vec!["a".to_string(), "b".to_string()],
            85,
            50,
        );

        QuizRepository::create(conn, &attempt).unwrap();

        let retrieved = QuizRepository::get_by_id(conn, &attempt.id).unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.score_percentage, 85);
        assert_eq!(retrieved.answers, vec!["a", "b"]);
    }

    #[test]
    fn test_get_for_quiz() {
        let db = setup_db();
        let conn = db.connection();

        let attempt1 = QuizAttempt::new(
            "test-user".to_string(),
            "quiz1".to_string(),
            "node1".to_string(),
            vec!["a".to_string()],
            70,
            40,
        );
        let attempt2 = QuizAttempt::new(
            "test-user".to_string(),
            "quiz1".to_string(),
            "node1".to_string(),
            vec!["b".to_string()],
            90,
            55,
        );

        QuizRepository::create(conn, &attempt1).unwrap();
        QuizRepository::create(conn, &attempt2).unwrap();

        let attempts = QuizRepository::get_for_quiz(conn, "test-user", "quiz1").unwrap();
        assert_eq!(attempts.len(), 2);
    }

    #[test]
    fn test_get_recent() {
        let db = setup_db();
        let conn = db.connection();

        for i in 0..5 {
            let attempt = QuizAttempt::new(
                "test-user".to_string(),
                format!("quiz{}", i),
                "node1".to_string(),
                vec!["a".to_string()],
                70 + i,
                40,
            );
            QuizRepository::create(conn, &attempt).unwrap();
        }

        let recent = QuizRepository::get_recent(conn, "test-user", 3).unwrap();
        assert_eq!(recent.len(), 3);
    }
}
