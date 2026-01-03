use crate::state::AppState;
use glp_core::db::repos::ProgressRepository;
use glp_core::models::{NodeProgress, NodeStatus};
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
pub struct ProgressData {
    pub node_id: String,
    pub status: String,
    pub attempts: i32,
    pub time_spent_mins: i32,
    pub completed: bool,
}

impl From<NodeProgress> for ProgressData {
    fn from(progress: NodeProgress) -> Self {
        Self {
            node_id: progress.node_id,
            status: progress.status.as_str().to_string(),
            attempts: progress.attempts,
            time_spent_mins: progress.time_spent_mins,
            completed: progress.status == NodeStatus::Completed,
        }
    }
}

#[tauri::command]
pub fn get_node_progress(state: State<AppState>, node_id: String) -> Result<Option<ProgressData>, String> {
    let user_id = state
        .current_user_id
        .lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or_else(|| "No user logged in".to_string())?;

    state
        .db
        .with_connection(|conn| {
            let progress = ProgressRepository::get(conn, &user_id, &node_id)?;
            Ok(progress.map(ProgressData::from))
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_all_progress(state: State<AppState>) -> Result<Vec<ProgressData>, String> {
    let user_id = state
        .current_user_id
        .lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or_else(|| "No user logged in".to_string())?;

    state
        .db
        .with_connection(|conn| {
            let progress_list = ProgressRepository::get_all_for_user(conn, &user_id)?;
            Ok(progress_list.into_iter().map(ProgressData::from).collect())
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn mark_node_complete(state: State<AppState>, node_id: String) -> Result<ProgressData, String> {
    let user_id = state
        .current_user_id
        .lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or_else(|| "No user logged in".to_string())?;

    state
        .db
        .with_connection(|conn| {
            ProgressRepository::mark_completed(conn, &user_id, &node_id)?;

            let progress = ProgressRepository::get(conn, &user_id, &node_id)?
                .ok_or_else(|| glp_core::db::error::DbError::NotFound("Progress not found".to_string()))?;

            Ok(ProgressData::from(progress))
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn start_node(state: State<AppState>, node_id: String) -> Result<ProgressData, String> {
    let user_id = state
        .current_user_id
        .lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or_else(|| "No user logged in".to_string())?;

    state
        .db
        .with_connection(|conn| {
            let mut progress = NodeProgress::new(user_id.clone(), node_id.clone());
            progress.start();
            ProgressRepository::create_or_update(conn, &progress)?;

            Ok(ProgressData::from(progress))
        })
        .map_err(|e| e.to_string())
}
