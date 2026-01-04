use glp_core::{
    badges::{get_all_badge_definitions, check_badge_unlocks, calculate_badge_progress, UserStats},
    db::repos::{BadgeRepository, UserRepository, ProgressRepository, MasteryRepository, QuizRepository},
    models::{BadgeDefinition, BadgeProgress},
};
use serde::{Deserialize, Serialize};
use tauri::State;
use crate::state::AppState;

/// Badge with user progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BadgeWithProgress {
    pub definition: BadgeDefinition,
    pub progress: f64,
    pub current_value: f64,
    pub is_earned: bool,
    pub earned_at: Option<String>,
}

/// Get all badges with user progress
#[tauri::command]
pub fn get_all_badges(state: State<AppState>) -> Result<Vec<BadgeWithProgress>, String> {
    let user_id = state.get_current_user_id();

    state.db.with_connection(|conn| {
        // Get user stats
        let stats = build_user_stats(conn, &user_id)?;
        
        // Get all badge progress for user
        let badge_progress = BadgeRepository::get_all_for_user(conn, &user_id)?;

        // Build combined list
        let definitions = get_all_badge_definitions();
        let mut badges_with_progress = Vec::new();

        for def in definitions {
            let progress_record = badge_progress.iter().find(|p| p.badge_id == def.id);
            let progress_pct = calculate_badge_progress(&def, &stats);
            let current_value = stats.get_value_for_category(&def.category);

            badges_with_progress.push(BadgeWithProgress {
                is_earned: progress_record.map(|p| p.is_earned()).unwrap_or(false),
                earned_at: progress_record.and_then(|p| p.earned_at.map(|d| d.to_rfc3339())),
                progress: progress_pct,
                current_value,
                definition: def,
            });
        }

        Ok(badges_with_progress)
    }).map_err(|e| e.to_string())
}

/// Get only earned badges
#[tauri::command]
pub fn get_earned_badges(state: State<AppState>) -> Result<Vec<BadgeWithProgress>, String> {
    let all_badges = get_all_badges(state)?;
    Ok(all_badges.into_iter().filter(|b| b.is_earned).collect())
}

/// Check for newly unlocked badges and return them
#[tauri::command]
pub fn check_and_unlock_badges(state: State<AppState>) -> Result<Vec<BadgeDefinition>, String> {
    let user_id = state.get_current_user_id();

    state.db.with_connection(|conn| {
        // Get user stats
        let stats = build_user_stats(conn, &user_id)?;
        
        // Get current badge progress
        let current_progress = BadgeRepository::get_all_for_user(conn, &user_id)?;

        // Check for new unlocks
        let newly_unlocked_ids = check_badge_unlocks(&stats, &current_progress);
        
        // Update database for newly unlocked badges
        let mut newly_unlocked = Vec::new();
        for badge_id in &newly_unlocked_ids {
            if let Some(def) = get_all_badge_definitions().into_iter().find(|d| d.id == *badge_id) {
                // Create or update badge progress with earned status
                let mut progress = BadgeProgress::new(user_id.clone(), badge_id.clone());
                progress.update_progress(def.threshold, def.threshold);
                
                BadgeRepository::create_or_update(conn, &progress)?;
                
                newly_unlocked.push(def);
            }
        }

        Ok(newly_unlocked)
    }).map_err(|e| e.to_string())
}

/// Update badge progress for a specific badge
#[tauri::command]
pub fn update_badge_progress(
    state: State<AppState>,
    badge_id: String,
) -> Result<BadgeWithProgress, String> {
    let user_id = state.get_current_user_id();

    state.db.with_connection(|conn| {
        let stats = build_user_stats(conn, &user_id)?;
        
        let def = get_all_badge_definitions()
            .into_iter()
            .find(|d| d.id == badge_id)
            .ok_or_else(|| glp_core::DbError::NotFound(format!("Badge not found: {}", badge_id)))?;

        let current_value = stats.get_value_for_category(&def.category);
        let progress_pct = calculate_badge_progress(&def, &stats);

        // Get or create badge progress
        let mut badge_progress = BadgeRepository::get(conn, &user_id, &badge_id)?
            .unwrap_or_else(|| BadgeProgress::new(user_id.clone(), badge_id.clone()));

        badge_progress.update_progress(current_value, def.threshold);
        BadgeRepository::create_or_update(conn, &badge_progress)?;

        Ok(BadgeWithProgress {
            is_earned: badge_progress.is_earned(),
            earned_at: badge_progress.earned_at.map(|d| d.to_rfc3339()),
            progress: progress_pct,
            current_value,
            definition: def,
        })
    }).map_err(|e| e.to_string())
}

/// Helper function to build UserStats from database
fn build_user_stats(
    conn: &rusqlite::Connection,
    user_id: &str,
) -> Result<UserStats, glp_core::DbError> {
    // Get user data
    let user = UserRepository::get_by_id(conn, user_id)?
        .unwrap_or_else(|| glp_core::models::User::new(user_id.to_string()));

    // Get progress data
    let all_progress = ProgressRepository::get_all_for_user(conn, user_id)?;
    let completed_lectures = all_progress
        .iter()
        .filter(|p| p.status == glp_core::models::NodeStatus::Completed && p.node_id.contains("lecture"))
        .count() as u32;
    let total_completions = all_progress
        .iter()
        .filter(|p| p.status == glp_core::models::NodeStatus::Completed)
        .count() as u32;

    // Get quiz data
    let quiz_attempts = QuizRepository::get_all_for_user(conn, user_id)?;
    let completed_quizzes = quiz_attempts.len() as u32;
    let perfect_quiz_count = quiz_attempts
        .iter()
        .filter(|q| q.score_percentage >= 100)
        .count() as u32;

    // Get mastery data
    let masteries = MasteryRepository::get_all_for_user(conn, user_id)?;
    let max_mastery = masteries.iter().map(|m| m.score).fold(0.0_f64, f64::max);

    Ok(UserStats {
        streak_days: user.current_streak as u32,
        level: user.current_level as u32,
        total_xp: user.total_xp,
        completed_lectures,
        completed_quizzes,
        completed_challenges: 0, // TODO: Track challenges
        total_completions,
        perfect_quiz_count,
        max_mastery_score: max_mastery,
    })
}
