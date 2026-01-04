use crate::state::AppState;
use glp_core::db::repos::{
    BadgeRepository, MasteryRepository, ProgressRepository,
    QuizRepository, ReviewRepository, UserRepository,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tauri::State;

#[derive(Debug, Serialize)]
pub struct SystemStatus {
    pub docker_installed: bool,
    pub docker_running: bool,
    pub api_key_set: bool,
    pub database_ok: bool,
}

#[derive(Debug, Serialize)]
pub struct DockerStatus {
    pub installed: bool,
    pub running: bool,
    pub version: Option<String>,
}

/// Check system requirements
#[tauri::command]
pub fn check_system_status(state: State<AppState>) -> Result<SystemStatus, String> {
    let docker = check_docker_internal();

    // Check if API key is set
    let api_key_set = std::env::var("OPENAI_API_KEY").is_ok()
        || load_api_key_from_config().is_some();

    // Check database connection
    let database_ok = state
        .db
        .with_connection(|conn| {
            conn.execute("SELECT 1", [])?;
            Ok(())
        })
        .is_ok();

    Ok(SystemStatus {
        docker_installed: docker.installed,
        docker_running: docker.running,
        api_key_set,
        database_ok,
    })
}

/// Check Docker status
#[tauri::command]
pub fn check_docker_status() -> DockerStatus {
    check_docker_internal()
}

fn check_docker_internal() -> DockerStatus {
    // Check if Docker is installed
    let version_output = Command::new("docker").arg("--version").output();

    let (installed, version) = match version_output {
        Ok(output) if output.status.success() => {
            let version_str = String::from_utf8_lossy(&output.stdout);
            let version = version_str
                .trim()
                .strip_prefix("Docker version ")
                .map(|s| s.split(',').next().unwrap_or(s).to_string());
            (true, version)
        }
        _ => (false, None),
    };

    if !installed {
        return DockerStatus {
            installed: false,
            running: false,
            version: None,
        };
    }

    // Check if Docker daemon is running
    let info_output = Command::new("docker").arg("info").output();
    let running = info_output.map(|o| o.status.success()).unwrap_or(false);

    DockerStatus {
        installed,
        running,
        version,
    }
}

/// Save OpenAI API key
#[tauri::command]
pub fn save_api_key(api_key: String) -> Result<(), String> {
    let config_dir = get_config_dir()?;
    fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;

    let key_path = config_dir.join("api_key");

    // Simple obfuscation (not secure encryption, but better than plaintext)
    let obfuscated = obfuscate_key(&api_key);
    fs::write(&key_path, obfuscated).map_err(|e| e.to_string())?;

    // Also set as environment variable for current session
    std::env::set_var("OPENAI_API_KEY", &api_key);

    Ok(())
}

/// Load API key from config
#[tauri::command]
pub fn get_api_key_status() -> bool {
    std::env::var("OPENAI_API_KEY").is_ok() || load_api_key_from_config().is_some()
}

fn load_api_key_from_config() -> Option<String> {
    let config_dir = get_config_dir().ok()?;
    let key_path = config_dir.join("api_key");

    if !key_path.exists() {
        return None;
    }

    let obfuscated = fs::read_to_string(&key_path).ok()?;
    let key = deobfuscate_key(&obfuscated);

    // Set as environment variable
    std::env::set_var("OPENAI_API_KEY", &key);

    Some(key)
}

fn get_config_dir() -> Result<PathBuf, String> {
    dirs::config_dir()
        .map(|p| p.join("gamified-learning-platform"))
        .ok_or_else(|| "Could not find config directory".to_string())
}

// Simple XOR obfuscation (not secure, but prevents casual viewing)
fn obfuscate_key(key: &str) -> String {
    use base64::Engine;
    let xor_key = b"glp_secret_key_2024";
    let obfuscated: Vec<u8> = key
        .bytes()
        .enumerate()
        .map(|(i, b)| b ^ xor_key[i % xor_key.len()])
        .collect();
    base64::engine::general_purpose::STANDARD.encode(&obfuscated)
}

fn deobfuscate_key(obfuscated: &str) -> String {
    use base64::Engine;
    let xor_key = b"glp_secret_key_2024";
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(obfuscated)
        .unwrap_or_default();
    let deobfuscated: Vec<u8> = decoded
        .iter()
        .enumerate()
        .map(|(i, b)| b ^ xor_key[i % xor_key.len()])
        .collect();
    String::from_utf8(deobfuscated).unwrap_or_default()
}

/// Backup data structure
#[derive(Debug, Serialize, Deserialize)]
pub struct BackupData {
    pub version: String,
    pub exported_at: String,
    pub user: Option<serde_json::Value>,
    pub node_progress: Vec<serde_json::Value>,
    pub quiz_attempts: Vec<serde_json::Value>,
    pub mastery_scores: Vec<serde_json::Value>,
    pub badge_progress: Vec<serde_json::Value>,
    pub review_items: Vec<serde_json::Value>,
}

/// Export all user data to JSON file
#[tauri::command]
pub fn export_user_data(state: State<AppState>, path: String) -> Result<(), String> {
    // Get user ID
    let user_id_guard = state.current_user_id.lock().map_err(|e| e.to_string())?;
    let user_id = user_id_guard
        .as_ref()
        .ok_or_else(|| "No user logged in".to_string())?;
    let user_id = user_id.clone();
    drop(user_id_guard);

    // Collect all data using with_connection
    let user = state
        .db
        .with_connection(|conn| UserRepository::get_by_id(conn, &user_id))
        .map_err(|e| e.to_string())?
        .map(|u| serde_json::to_value(u).unwrap());

    let node_progress: Vec<serde_json::Value> = state
        .db
        .with_connection(|conn| ProgressRepository::get_all_for_user(conn, &user_id))
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|p| serde_json::to_value(p).unwrap())
        .collect();

    let quiz_attempts: Vec<serde_json::Value> = state
        .db
        .with_connection(|conn| QuizRepository::get_all_for_user(conn, &user_id))
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|a| serde_json::to_value(a).unwrap())
        .collect();

    let mastery_scores: Vec<serde_json::Value> = state
        .db
        .with_connection(|conn| MasteryRepository::get_all_for_user(conn, &user_id))
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|m| serde_json::to_value(m).unwrap())
        .collect();

    let badge_progress: Vec<serde_json::Value> = state
        .db
        .with_connection(|conn| BadgeRepository::get_all_for_user(conn, &user_id))
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|b| serde_json::to_value(b).unwrap())
        .collect();

    let review_items: Vec<serde_json::Value> = state
        .db
        .with_connection(|conn| ReviewRepository::get_all_for_user(conn, &user_id))
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|r| serde_json::to_value(r).unwrap())
        .collect();

    let backup = BackupData {
        version: "1.0".to_string(),
        exported_at: chrono::Utc::now().to_rfc3339(),
        user,
        node_progress,
        quiz_attempts,
        mastery_scores,
        badge_progress,
        review_items,
    };

    let json = serde_json::to_string_pretty(&backup).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())?;

    Ok(())
}

/// Import user data from JSON file
#[tauri::command]
pub fn import_user_data(state: State<AppState>, path: String) -> Result<(), String> {
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let backup: BackupData = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    // Import user if present
    if let Some(user_value) = backup.user {
        let user: glp_core::models::User =
            serde_json::from_value(user_value).map_err(|e| e.to_string())?;

        // Check if user exists, create if not
        let exists = state
            .db
            .with_connection(|conn| UserRepository::get_by_id(conn, &user.id))
            .map_err(|e| e.to_string())?
            .is_some();

        if !exists {
            state
                .db
                .with_connection(|conn| UserRepository::create(conn, &user))
                .map_err(|e| e.to_string())?;
        }

        // Set as current user
        *state.current_user_id.lock().map_err(|e| e.to_string())? = Some(user.id.clone());
    }

    // Import progress
    for progress_value in backup.node_progress {
        let progress: glp_core::models::NodeProgress =
            serde_json::from_value(progress_value).map_err(|e| e.to_string())?;
        state
            .db
            .with_connection(|conn| ProgressRepository::create_or_update(conn, &progress))
            .map_err(|e| e.to_string())?;
    }

    // Import mastery scores
    for mastery_value in backup.mastery_scores {
        let mastery: glp_core::models::MasteryScore =
            serde_json::from_value(mastery_value).map_err(|e| e.to_string())?;
        state
            .db
            .with_connection(|conn| MasteryRepository::create_or_update(conn, &mastery))
            .map_err(|e| e.to_string())?;
    }

    // Import badge progress
    for badge_value in backup.badge_progress {
        let badge: glp_core::models::BadgeProgress =
            serde_json::from_value(badge_value).map_err(|e| e.to_string())?;
        state
            .db
            .with_connection(|conn| BadgeRepository::create_or_update(conn, &badge))
            .map_err(|e| e.to_string())?;
    }

    // Import review items
    for review_value in backup.review_items {
        let review: glp_core::models::ReviewItem =
            serde_json::from_value(review_value).map_err(|e| e.to_string())?;
        state
            .db
            .with_connection(|conn| ReviewRepository::create_or_update(conn, &review))
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Reset all user progress
#[tauri::command]
pub fn reset_all_progress(state: State<AppState>) -> Result<(), String> {
    let user_id_guard = state.current_user_id.lock().map_err(|e| e.to_string())?;
    let user_id = user_id_guard
        .as_ref()
        .ok_or_else(|| "No user logged in".to_string())?;
    let user_id = user_id.clone();
    drop(user_id_guard);

    // Delete all progress data
    state
        .db
        .with_connection(|conn| {
            conn.execute("DELETE FROM node_progress WHERE user_id = ?1", [&user_id])?;
            conn.execute("DELETE FROM quiz_attempts WHERE user_id = ?1", [&user_id])?;
            conn.execute("DELETE FROM challenge_attempts WHERE user_id = ?1", [&user_id])?;
            conn.execute("DELETE FROM mastery_scores WHERE user_id = ?1", [&user_id])?;
            conn.execute("DELETE FROM badge_progress WHERE user_id = ?1", [&user_id])?;
            conn.execute("DELETE FROM review_items WHERE user_id = ?1", [&user_id])?;
            conn.execute(
                "UPDATE users SET total_xp = 0, current_level = 1, current_streak = 0 WHERE id = ?1",
                [&user_id],
            )?;
            Ok(())
        })
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Check if this is first launch (no user exists)
#[tauri::command]
pub fn is_first_launch(state: State<AppState>) -> Result<bool, String> {
    state
        .db
        .with_connection(|conn| {
            let count: i64 = conn
                .query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))?;
            Ok(count == 0)
        })
        .map_err(|e| e.to_string())
}

/// Mark onboarding as complete
#[tauri::command]
pub fn complete_onboarding(state: State<AppState>) -> Result<(), String> {
    let config_dir = get_config_dir()?;
    fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;

    let flag_path = config_dir.join("onboarding_complete");
    fs::write(&flag_path, "true").map_err(|e| e.to_string())?;

    Ok(())
}

/// Check if onboarding is complete
#[tauri::command]
pub fn is_onboarding_complete() -> bool {
    get_config_dir()
        .map(|d| d.join("onboarding_complete").exists())
        .unwrap_or(false)
}
