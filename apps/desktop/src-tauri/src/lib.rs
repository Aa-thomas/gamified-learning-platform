mod commands;
mod state;

use state::AppState;
use std::path::PathBuf;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Determine content path (relative to executable in dev, or bundled in prod)
    let content_path = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("content");

    // Initialize app state
    let app_state = AppState::new(content_path).expect("Failed to initialize app state");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            // User commands
            commands::user::get_user_data,
            commands::user::create_user,
            commands::user::update_user_xp,
            // Progress commands
            commands::progress::get_node_progress,
            commands::progress::get_all_progress,
            commands::progress::mark_node_complete,
            commands::progress::start_node,
            // Content commands
            commands::content::get_content_tree,
            commands::content::get_node_by_id,
            commands::content::load_lecture,
            commands::content::load_quiz,
            // Lecture commands
            commands::lecture::start_lecture,
            commands::lecture::update_lecture_time,
            commands::lecture::complete_lecture,
            // Quiz commands
            commands::quiz::submit_quiz,
            // Session commands
            commands::session::create_daily_session,
            commands::session::start_session,
            commands::session::complete_session,
            commands::session::get_interrupted_session,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
