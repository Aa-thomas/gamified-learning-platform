use crate::state::AppState;
use glp_core::db::repos::{ProgressRepository, UserRepository};
use glp_core::gamification::{calculate_lecture_xp, calculate_level, Difficulty};
use glp_core::models::NodeProgress;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Serialize)]
pub struct LectureData {
    pub id: String,
    pub title: String,
    pub content: String,
    pub difficulty: String,
    pub xp_reward: i32,
}

#[derive(Serialize)]
pub struct CompletionResult {
    pub xp_earned: i32,
    pub new_total_xp: i32,
    pub new_level: u32,
    pub unlocked_nodes: Vec<String>,
}

#[tauri::command]
pub fn start_lecture(
    state: State<AppState>,
    lecture_id: String,
) -> Result<(), String> {
    let user_id = state
        .current_user_id
        .lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or_else(|| "No user logged in".to_string())?;

    state
        .db
        .with_connection(|conn| {
            let mut progress = NodeProgress::new(user_id.clone(), lecture_id.clone());
            progress.start();
            ProgressRepository::create_or_update(conn, &progress)?;
            Ok(())
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_lecture_time(
    state: State<AppState>,
    lecture_id: String,
    time_spent_ms: i64,
) -> Result<(), String> {
    let user_id = state
        .current_user_id
        .lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or_else(|| "No user logged in".to_string())?;

    state
        .db
        .with_connection(|conn| {
            let mut progress = ProgressRepository::get(conn, &user_id, &lecture_id)?
                .ok_or_else(|| glp_core::db::error::DbError::NotFound("Progress not found".to_string()))?;

            progress.add_time((time_spent_ms / 60000) as i32);
            ProgressRepository::create_or_update(conn, &progress)?;
            Ok(())
        })
        .map_err(|e| e.to_string())
}

#[derive(Deserialize)]
pub struct CompleteLectureRequest {
    pub lecture_id: String,
    pub time_spent_ms: i64,
    pub difficulty: String,
}

#[tauri::command]
pub fn complete_lecture(
    state: State<AppState>,
    request: CompleteLectureRequest,
) -> Result<CompletionResult, String> {
    let user_id = state
        .current_user_id
        .lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or_else(|| "No user logged in".to_string())?;

    state
        .db
        .with_connection(|conn| {
            // Parse difficulty
            let difficulty = match request.difficulty.as_str() {
                "Easy" => Difficulty::Easy,
                "Medium" => Difficulty::Medium,
                "Hard" => Difficulty::Hard,
                "VeryHard" => Difficulty::VeryHard,
                _ => Difficulty::Easy,
            };

            // Get user's current streak
            let user = UserRepository::get_by_id(conn, &user_id)?
                .ok_or_else(|| glp_core::db::error::DbError::NotFound("User not found".to_string()))?;

            // Calculate XP
            let xp_earned = calculate_lecture_xp(difficulty, user.current_streak as u32);

            // Update progress
            let mut progress = ProgressRepository::get(conn, &user_id, &request.lecture_id)?
                .unwrap_or_else(|| NodeProgress::new(user_id.clone(), request.lecture_id.clone()));

            progress.add_time((request.time_spent_ms / 60000) as i32);
            progress.complete();
            ProgressRepository::create_or_update(conn, &progress)?;

            // Award XP and update level
            UserRepository::update_xp(conn, &user_id, xp_earned)?;
            let new_total_xp = user.total_xp + xp_earned;
            let new_level = calculate_level(new_total_xp);
            UserRepository::update_level(conn, &user_id, new_level as i32)?;

            Ok(CompletionResult {
                xp_earned,
                new_total_xp,
                new_level,
                unlocked_nodes: vec![], // TODO: Implement unlock logic
            })
        })
        .map_err(|e| e.to_string())
}
