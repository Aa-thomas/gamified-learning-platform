use crate::state::AppState;
use glp_core::db::repos::{ProgressRepository, SessionRepository, UserRepository};
use glp_core::gamification::{calculate_level, get_streak_multiplier};
use glp_core::models::SessionHistory;
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
pub struct SessionPlan {
    pub session_id: String,
    pub activities: Vec<PlannedActivity>,
    pub estimated_minutes: u32,
    pub total_xp_potential: i32,
}

#[derive(Serialize, Clone)]
pub struct PlannedActivity {
    pub node_id: String,
    pub node_type: String,
    pub title: String,
    pub difficulty: String,
    pub xp_reward: i32,
    pub estimated_minutes: u32,
}

#[derive(Serialize)]
pub struct SessionSummary {
    pub session_id: String,
    pub duration_minutes: u32,
    pub total_xp_earned: i32,
    pub activities_completed: Vec<CompletedActivitySummary>,
    pub level_before: u32,
    pub level_after: u32,
    pub leveled_up: bool,
    pub streak_days: i32,
    pub streak_multiplier: f64,
}

#[derive(Serialize)]
pub struct CompletedActivitySummary {
    pub title: String,
    pub xp_earned: i32,
}

#[tauri::command]
pub fn create_daily_session(
    state: State<AppState>,
    _target_minutes: u32,
) -> Result<SessionPlan, String> {
    let user_id = state
        .current_user_id
        .lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or_else(|| "No user logged in".to_string())?;

    state
        .db
        .with_connection(|conn| {
            // Get user's progress to find available content
            let all_progress = ProgressRepository::get_all_for_user(conn, &user_id)?;
            let _completed_ids: Vec<String> = all_progress
                .iter()
                .filter(|p| p.status == glp_core::models::NodeStatus::Completed)
                .map(|p| p.node_id.clone())
                .collect();

            // For now, create a simple session with mock activities
            // In production, this would query the content system
            let activities = vec![
                PlannedActivity {
                    node_id: "lecture-intro".to_string(),
                    node_type: "lecture".to_string(),
                    title: "Introduction to Rust".to_string(),
                    difficulty: "Easy".to_string(),
                    xp_reward: 25,
                    estimated_minutes: 10,
                },
                PlannedActivity {
                    node_id: "quiz-basics".to_string(),
                    node_type: "quiz".to_string(),
                    title: "Rust Basics Quiz".to_string(),
                    difficulty: "Easy".to_string(),
                    xp_reward: 50,
                    estimated_minutes: 10,
                },
            ];

            let total_xp: i32 = activities.iter().map(|a| a.xp_reward).sum();
            let total_minutes: u32 = activities.iter().map(|a| a.estimated_minutes).sum();

            // Create session in DB
            let session = SessionHistory::new(user_id.clone());
            SessionRepository::create(conn, &session)?;

            Ok(SessionPlan {
                session_id: session.id.clone(),
                activities,
                estimated_minutes: total_minutes,
                total_xp_potential: total_xp,
            })
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn start_session(
    state: State<AppState>,
    session_id: String,
) -> Result<(), String> {
    let _user_id = state
        .current_user_id
        .lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or_else(|| "No user logged in".to_string())?;

    state
        .db
        .with_connection(|conn| {
            let session = SessionRepository::get_by_id(conn, &session_id)?
                .ok_or_else(|| glp_core::db::error::DbError::NotFound("Session not found".to_string()))?;

            // Session is already started when created
            SessionRepository::update(conn, &session)?;
            Ok(())
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn complete_session(
    state: State<AppState>,
    session_id: String,
    xp_earned: i32,
) -> Result<SessionSummary, String> {
    let user_id = state
        .current_user_id
        .lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or_else(|| "No user logged in".to_string())?;

    state
        .db
        .with_connection(|conn| {
            // Get session
            let mut session = SessionRepository::get_by_id(conn, &session_id)?
                .ok_or_else(|| glp_core::db::error::DbError::NotFound("Session not found".to_string()))?;

            // Get user before update
            let user = UserRepository::get_by_id(conn, &user_id)?
                .ok_or_else(|| glp_core::db::error::DbError::NotFound("User not found".to_string()))?;
            let level_before = user.current_level;

            // Complete session
            session.add_completion(xp_earned);
            session.end_session();
            SessionRepository::update(conn, &session)?;

            // Update user XP
            UserRepository::update_xp(conn, &user_id, xp_earned)?;
            let new_total_xp = user.total_xp + xp_earned;
            let level_after = calculate_level(new_total_xp);
            UserRepository::update_level(conn, &user_id, level_after as i32)?;

            // Calculate duration
            let duration = session.duration_minutes() as u32;

            Ok(SessionSummary {
                session_id,
                duration_minutes: duration,
                total_xp_earned: xp_earned,
                activities_completed: vec![], // Would be populated from session activities
                level_before: level_before as u32,
                level_after,
                leveled_up: level_after > level_before as u32,
                streak_days: user.current_streak,
                streak_multiplier: get_streak_multiplier(user.current_streak as u32),
            })
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_interrupted_session(
    state: State<AppState>,
) -> Result<Option<SessionPlan>, String> {
    let user_id = state
        .current_user_id
        .lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or_else(|| "No user logged in".to_string())?;

    state
        .db
        .with_connection(|conn| {
            // Check for active sessions
            let session = SessionRepository::get_active_session(conn, &user_id)?;
            
            if let Some(session) = session {
                // Return the session plan
                Ok(Some(SessionPlan {
                    session_id: session.id.clone(),
                    activities: vec![], // Would be populated from session data
                    estimated_minutes: 0,
                    total_xp_potential: 0,
                }))
            } else {
                Ok(None)
            }
        })
        .map_err(|e| e.to_string())
}
