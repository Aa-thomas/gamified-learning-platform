use glp_core::{
    db::repos::{ReviewRepository, MasteryRepository},
    models::ReviewItem,
    spaced_repetition::{apply_mastery_decay, score_to_quality},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::State;
use crate::state::AppState;

/// Review item for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewItemResponse {
    pub quiz_id: String,
    pub due_date: String,
    pub ease_factor: f64,
    pub interval_days: i32,
    pub repetitions: i32,
    pub last_reviewed_at: Option<String>,
}

impl From<ReviewItem> for ReviewItemResponse {
    fn from(item: ReviewItem) -> Self {
        Self {
            quiz_id: item.quiz_id,
            due_date: item.due_date.to_rfc3339(),
            ease_factor: item.ease_factor,
            interval_days: item.interval_days,
            repetitions: item.repetitions,
            last_reviewed_at: item.last_reviewed_at.map(|d| d.to_rfc3339()),
        }
    }
}

/// Get all due reviews for the user
#[tauri::command]
pub fn get_due_reviews(state: State<AppState>) -> Result<Vec<ReviewItemResponse>, String> {
    let user_id = state.get_current_user_id();

    state.db.with_connection(|conn| {
        let due_reviews = ReviewRepository::get_due_reviews(conn, &user_id)?;
        Ok(due_reviews.into_iter().map(ReviewItemResponse::from).collect())
    }).map_err(|e| e.to_string())
}

/// Get count of due reviews
#[tauri::command]
pub fn get_due_review_count(state: State<AppState>) -> Result<i32, String> {
    let user_id = state.get_current_user_id();

    state.db.with_connection(|conn| {
        ReviewRepository::count_due_reviews(conn, &user_id)
    }).map_err(|e| e.to_string())
}

/// Get all reviews for the user (due and upcoming)
#[tauri::command]
pub fn get_all_reviews(state: State<AppState>) -> Result<Vec<ReviewItemResponse>, String> {
    let user_id = state.get_current_user_id();

    state.db.with_connection(|conn| {
        let reviews = ReviewRepository::get_all_for_user(conn, &user_id)?;
        Ok(reviews.into_iter().map(ReviewItemResponse::from).collect())
    }).map_err(|e| e.to_string())
}

/// Submit a review result
#[tauri::command]
pub fn submit_review(
    state: State<AppState>,
    quiz_id: String,
    score_percentage: f64,
) -> Result<ReviewItemResponse, String> {
    let user_id = state.get_current_user_id();

    state.db.with_connection(|conn| {
        // Get existing review item
        let mut review = ReviewRepository::get(conn, &user_id, &quiz_id)?
            .ok_or_else(|| glp_core::DbError::NotFound(format!("Review item not found: {}", quiz_id)))?;

        // Convert score to quality and update
        let quality = score_to_quality(score_percentage);
        review.update_after_review(quality as i32);

        // Save updated review
        ReviewRepository::create_or_update(conn, &review)?;

        Ok(ReviewItemResponse::from(review))
    }).map_err(|e| e.to_string())
}

/// Create a review item for a quiz (called after completing a quiz)
#[tauri::command]
pub fn create_review_item(
    state: State<AppState>,
    quiz_id: String,
) -> Result<ReviewItemResponse, String> {
    let user_id = state.get_current_user_id();

    state.db.with_connection(|conn| {
        // Check if already exists
        if let Some(existing) = ReviewRepository::get(conn, &user_id, &quiz_id)? {
            return Ok(ReviewItemResponse::from(existing));
        }

        // Create new review item
        let review = ReviewItem::new(user_id.clone(), quiz_id);
        ReviewRepository::create_or_update(conn, &review)?;

        Ok(ReviewItemResponse::from(review))
    }).map_err(|e| e.to_string())
}

/// Apply mastery decay on app startup
#[tauri::command]
pub fn apply_mastery_decay_on_startup(state: State<AppState>) -> Result<i32, String> {
    let user_id = state.get_current_user_id();

    state.db.with_connection(|conn| {
        // Get all masteries
        let mut masteries = MasteryRepository::get_all_for_user(conn, &user_id)?;

        // Apply decay
        let decayed_count = apply_mastery_decay(&mut masteries, Utc::now());

        // Update database with decayed scores
        for mastery in &masteries {
            MasteryRepository::create_or_update(conn, mastery)?;
        }

        Ok(decayed_count as i32)
    }).map_err(|e| e.to_string())
}

/// Get mastery scores that need attention (below threshold)
#[tauri::command]
pub fn get_low_mastery_skills(
    state: State<AppState>,
    threshold: f64,
) -> Result<Vec<MasterySkillResponse>, String> {
    let user_id = state.get_current_user_id();

    state.db.with_connection(|conn| {
        let masteries = MasteryRepository::get_all_for_user(conn, &user_id)?;

        let low_skills: Vec<MasterySkillResponse> = masteries
            .into_iter()
            .filter(|m| m.score < threshold)
            .map(|m| {
                let level = m.level_description().to_string();
                MasterySkillResponse {
                    skill_id: m.skill_id,
                    score: m.score,
                    level,
                    last_updated: m.last_updated_at.to_rfc3339(),
                }
            })
            .collect();

        Ok(low_skills)
    }).map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterySkillResponse {
    pub skill_id: String,
    pub score: f64,
    pub level: String,
    pub last_updated: String,
}
