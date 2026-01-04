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
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
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
            // Badge commands
            commands::badge::get_all_badges,
            commands::badge::get_earned_badges,
            commands::badge::check_and_unlock_badges,
            commands::badge::update_badge_progress,
            // Review commands
            commands::review::get_due_reviews,
            commands::review::get_due_review_count,
            commands::review::get_all_reviews,
            commands::review::submit_review,
            commands::review::create_review_item,
            commands::review::apply_mastery_decay_on_startup,
            commands::review::get_low_mastery_skills,
            // Curriculum commands
            commands::curriculum::validate_curriculum,
            commands::curriculum::import_curriculum,
            commands::curriculum::list_curricula,
            commands::curriculum::get_active_curriculum,
            commands::curriculum::switch_curriculum,
            commands::curriculum::delete_curriculum,
            commands::curriculum::get_curriculum,
            // System commands
            commands::system::check_system_status,
            commands::system::check_docker_status,
            commands::system::save_api_key,
            commands::system::get_api_key_status,
            commands::system::export_user_data,
            commands::system::import_user_data,
            commands::system::reset_all_progress,
            commands::system::is_first_launch,
            commands::system::complete_onboarding,
            commands::system::is_onboarding_complete,
            // Update commands
            commands::update::check_for_update,
            commands::update::download_and_install_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
