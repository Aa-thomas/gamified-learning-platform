use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use crate::db::error::DbResult;
use crate::models::ReviewItem;

pub struct ReviewRepository;

impl ReviewRepository {
    pub fn create_or_update(conn: &Connection, review: &ReviewItem) -> DbResult<()> {
        conn.execute(
            "INSERT INTO review_items (user_id, quiz_id, due_date, ease_factor, interval_days, repetitions, last_reviewed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(user_id, quiz_id) DO UPDATE SET
                due_date = excluded.due_date,
                ease_factor = excluded.ease_factor,
                interval_days = excluded.interval_days,
                repetitions = excluded.repetitions,
                last_reviewed_at = excluded.last_reviewed_at",
            params![
                review.user_id,
                review.quiz_id,
                review.due_date.to_rfc3339(),
                review.ease_factor,
                review.interval_days,
                review.repetitions,
                review.last_reviewed_at.map(|d| d.to_rfc3339()),
            ],
        )?;
        Ok(())
    }

    pub fn get(conn: &Connection, user_id: &str, quiz_id: &str) -> DbResult<Option<ReviewItem>> {
        let mut stmt = conn.prepare(
            "SELECT user_id, quiz_id, due_date, ease_factor, interval_days, repetitions, last_reviewed_at
             FROM review_items WHERE user_id = ?1 AND quiz_id = ?2"
        )?;

        let review = stmt.query_row(params![user_id, quiz_id], |row| {
            Ok(ReviewItem {
                user_id: row.get(0)?,
                quiz_id: row.get(1)?,
                due_date: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, Box::new(e)))?
                    .with_timezone(&Utc),
                ease_factor: row.get(3)?,
                interval_days: row.get(4)?,
                repetitions: row.get(5)?,
                last_reviewed_at: row.get::<_, Option<String>>(6)?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
            })
        }).optional()?;

        Ok(review)
    }

    pub fn get_all_for_user(conn: &Connection, user_id: &str) -> DbResult<Vec<ReviewItem>> {
        let mut stmt = conn.prepare(
            "SELECT user_id, quiz_id, due_date, ease_factor, interval_days, repetitions, last_reviewed_at
             FROM review_items WHERE user_id = ?1"
        )?;

        let review_iter = stmt.query_map(params![user_id], |row| {
            Ok(ReviewItem {
                user_id: row.get(0)?,
                quiz_id: row.get(1)?,
                due_date: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, Box::new(e)))?
                    .with_timezone(&Utc),
                ease_factor: row.get(3)?,
                interval_days: row.get(4)?,
                repetitions: row.get(5)?,
                last_reviewed_at: row.get::<_, Option<String>>(6)?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
            })
        })?;

        let mut results = Vec::new();
        for review in review_iter {
            results.push(review?);
        }
        Ok(results)
    }

    pub fn get_due_reviews(conn: &Connection, user_id: &str) -> DbResult<Vec<ReviewItem>> {
        let now = Utc::now().to_rfc3339();
        let mut stmt = conn.prepare(
            "SELECT user_id, quiz_id, due_date, ease_factor, interval_days, repetitions, last_reviewed_at
             FROM review_items WHERE user_id = ?1 AND due_date <= ?2
             ORDER BY due_date ASC"
        )?;

        let review_iter = stmt.query_map(params![user_id, now], |row| {
            Ok(ReviewItem {
                user_id: row.get(0)?,
                quiz_id: row.get(1)?,
                due_date: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, Box::new(e)))?
                    .with_timezone(&Utc),
                ease_factor: row.get(3)?,
                interval_days: row.get(4)?,
                repetitions: row.get(5)?,
                last_reviewed_at: row.get::<_, Option<String>>(6)?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
            })
        })?;

        let mut results = Vec::new();
        for review in review_iter {
            results.push(review?);
        }
        Ok(results)
    }

    pub fn count_due_reviews(conn: &Connection, user_id: &str) -> DbResult<i32> {
        let now = Utc::now().to_rfc3339();
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM review_items WHERE user_id = ?1 AND due_date <= ?2",
            params![user_id, now],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    pub fn delete(conn: &Connection, user_id: &str, quiz_id: &str) -> DbResult<()> {
        conn.execute(
            "DELETE FROM review_items WHERE user_id = ?1 AND quiz_id = ?2",
            params![user_id, quiz_id],
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
    use chrono::Duration;

    fn setup_db() -> Database {
        let db = Database::new_in_memory().unwrap();
        let user = User::new("test-user".to_string());
        UserRepository::create(db.connection(), &user).unwrap();
        db
    }

    #[test]
    fn test_create_review_item() {
        let db = setup_db();
        let conn = db.connection();

        let review = ReviewItem::new("test-user".to_string(), "quiz1".to_string());
        ReviewRepository::create_or_update(conn, &review).unwrap();

        let retrieved = ReviewRepository::get(conn, "test-user", "quiz1").unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.quiz_id, "quiz1");
        assert_eq!(retrieved.repetitions, 0);
    }

    #[test]
    fn test_get_due_reviews() {
        let db = setup_db();
        let conn = db.connection();

        // Create a due review (past due date)
        let mut due_review = ReviewItem::new("test-user".to_string(), "quiz1".to_string());
        due_review.due_date = Utc::now() - Duration::hours(1);
        ReviewRepository::create_or_update(conn, &due_review).unwrap();

        // Create a future review
        let future_review = ReviewItem::new("test-user".to_string(), "quiz2".to_string());
        ReviewRepository::create_or_update(conn, &future_review).unwrap();

        let due = ReviewRepository::get_due_reviews(conn, "test-user").unwrap();
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].quiz_id, "quiz1");
    }

    #[test]
    fn test_count_due_reviews() {
        let db = setup_db();
        let conn = db.connection();

        // Create two due reviews
        let mut review1 = ReviewItem::new("test-user".to_string(), "quiz1".to_string());
        review1.due_date = Utc::now() - Duration::hours(1);
        let mut review2 = ReviewItem::new("test-user".to_string(), "quiz2".to_string());
        review2.due_date = Utc::now() - Duration::hours(2);

        ReviewRepository::create_or_update(conn, &review1).unwrap();
        ReviewRepository::create_or_update(conn, &review2).unwrap();

        let count = ReviewRepository::count_due_reviews(conn, "test-user").unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_update_review_schedule() {
        let db = setup_db();
        let conn = db.connection();

        let mut review = ReviewItem::new("test-user".to_string(), "quiz1".to_string());
        ReviewRepository::create_or_update(conn, &review).unwrap();

        // Update after review
        review.update_after_review(4); // Good
        ReviewRepository::create_or_update(conn, &review).unwrap();

        let updated = ReviewRepository::get(conn, "test-user", "quiz1").unwrap().unwrap();
        assert_eq!(updated.repetitions, 1);
        assert!(updated.last_reviewed_at.is_some());
    }
}
