use crate::state::AppState;
use glp_core::db::repos::UserRepository;
use glp_core::models::User;
use serde::Serialize;
use tauri::State;
use uuid::Uuid;

#[derive(Serialize)]
pub struct UserData {
    pub id: String,
    pub total_xp: i32,
    pub current_level: i32,
    pub current_streak: i32,
    pub xp_for_next_level: i32,
    pub xp_progress_percentage: f64,
}

impl From<User> for UserData {
    fn from(user: User) -> Self {
        Self {
            id: user.id.clone(),
            total_xp: user.total_xp,
            current_level: user.current_level,
            current_streak: user.current_streak,
            xp_for_next_level: user.xp_for_next_level(),
            xp_progress_percentage: user.xp_progress_percentage(),
        }
    }
}

#[tauri::command]
pub fn get_user_data(state: State<AppState>) -> Result<Option<UserData>, String> {
    let user_id = state.current_user_id.lock().map_err(|e| e.to_string())?;

    if let Some(ref uid) = *user_id {
        state
            .db
            .with_connection(|conn| {
                let user = UserRepository::get_by_id(conn, uid)?;
                Ok(user.map(UserData::from))
            })
            .map_err(|e| e.to_string())
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub fn create_user(state: State<AppState>) -> Result<UserData, String> {
    let user_id = Uuid::new_v4().to_string();
    let user = User::new(user_id.clone());

    state
        .db
        .with_connection(|conn| {
            UserRepository::create(conn, &user)?;
            Ok(())
        })
        .map_err(|e| e.to_string())?;

    // Set as current user
    *state.current_user_id.lock().map_err(|e| e.to_string())? = Some(user_id);

    Ok(user.into())
}

#[tauri::command]
pub fn update_user_xp(state: State<AppState>, xp_delta: i32) -> Result<UserData, String> {
    let user_id = state
        .current_user_id
        .lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or_else(|| "No user logged in".to_string())?;

    state
        .db
        .with_connection(|conn| {
            UserRepository::update_xp(conn, &user_id, xp_delta)?;

            // Check for level up
            let user = UserRepository::get_by_id(conn, &user_id)?
                .ok_or_else(|| glp_core::db::error::DbError::NotFound("User not found".to_string()))?;

            if let Some(new_level) = user.check_level_up() {
                UserRepository::update_level(conn, &user_id, new_level)?;
            }

            // Get updated user
            let updated_user = UserRepository::get_by_id(conn, &user_id)?
                .ok_or_else(|| glp_core::db::error::DbError::NotFound("User not found".to_string()))?;

            Ok(UserData::from(updated_user))
        })
        .map_err(|e| e.to_string())
}
