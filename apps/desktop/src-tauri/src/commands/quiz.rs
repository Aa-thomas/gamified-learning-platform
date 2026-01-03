use crate::state::AppState;
use glp_core::db::repos::{MasteryRepository, ProgressRepository, UserRepository};
use glp_core::gamification::{
    calculate_level, calculate_quiz_xp, get_retake_multiplier, update_mastery, Difficulty,
};
use glp_core::models::quiz::Quiz;
use glp_core::models::NodeProgress;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;

#[derive(Serialize)]
pub struct QuizResult {
    pub score: i32,
    pub total: i32,
    pub score_percentage: f64,
    pub passed: bool,
    pub xp_earned: i32,
    pub attempt_number: i32,
    pub mastery_updates: HashMap<String, f64>,
    pub feedback: Vec<QuestionFeedback>,
}

#[derive(Serialize)]
pub struct QuestionFeedback {
    pub question_id: String,
    pub user_answer: Option<String>,
    pub correct_answer: String,
    pub is_correct: bool,
    pub explanation: String,
}

#[derive(Deserialize)]
pub struct SubmitQuizRequest {
    pub quiz_id: String,
    pub answers: HashMap<String, String>,
    pub time_spent_ms: i64,
}

pub fn grade_quiz(quiz: &Quiz, answers: &HashMap<String, String>) -> (i32, usize, usize) {
    let mut score = 0;
    let mut correct_count = 0;
    let total = quiz.questions.len();

    for question in &quiz.questions {
        let user_answer = answers.get(&question.id);
        let is_correct = user_answer.map(|ans| ans == &question.correct_answer).unwrap_or(false);

        if is_correct {
            score += question.points;
            correct_count += 1;
        }
    }

    (score, correct_count, total)
}

pub fn generate_feedback(quiz: &Quiz, answers: &HashMap<String, String>) -> Vec<QuestionFeedback> {
    quiz.questions
        .iter()
        .map(|question| {
            let user_answer = answers.get(&question.id).cloned();
            let is_correct = user_answer.as_ref().map(|ans| ans == &question.correct_answer).unwrap_or(false);

            QuestionFeedback {
                question_id: question.id.clone(),
                user_answer,
                correct_answer: question.correct_answer.clone(),
                is_correct,
                explanation: question.explanation.clone(),
            }
        })
        .collect()
}

#[tauri::command]
pub fn submit_quiz(
    state: State<AppState>,
    request: SubmitQuizRequest,
) -> Result<QuizResult, String> {
    let user_id = state
        .current_user_id
        .lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or_else(|| "No user logged in".to_string())?;

    state
        .db
        .with_connection(|conn| {
            // Load quiz from content system
            let quiz = load_quiz_from_content(&request.quiz_id)?;

            // Get attempt count
            let progress = ProgressRepository::get(conn, &user_id, &request.quiz_id)?;
            let attempt_number = progress.as_ref().map(|p| p.attempts + 1).unwrap_or(1);

            // Grade quiz
            let (score, correct_count, total) = grade_quiz(&quiz, &request.answers);
            let total_points: i32 = quiz.questions.iter().map(|q| q.points).sum();
            let score_percentage = (score as f64 / total_points as f64) * 100.0;

            // Parse difficulty
            let difficulty = match quiz.difficulty.as_str() {
                "Easy" => Difficulty::Easy,
                "Medium" => Difficulty::Medium,
                "Hard" => Difficulty::Hard,
                "VeryHard" => Difficulty::VeryHard,
                _ => Difficulty::Easy,
            };

            // Get user's current streak
            let user = UserRepository::get_by_id(conn, &user_id)?
                .ok_or_else(|| glp_core::db::error::DbError::NotFound("User not found".to_string()))?;

            // Calculate XP with retake penalty
            let base_xp = calculate_quiz_xp(difficulty, score_percentage, user.current_streak as u32);
            let retake_multiplier = get_retake_multiplier(attempt_number as usize);
            let xp_earned = (base_xp as f64 * retake_multiplier) as i32;

            // Update mastery for all skills
            let mut mastery_updates = HashMap::new();
            for skill_id in &quiz.skills {
                let current_mastery = MasteryRepository::get(conn, &user_id, skill_id)?
                    .map(|m| m.score)
                    .unwrap_or(0.0);

                let performance_multiplier = get_mastery_retake_multiplier(attempt_number as usize);
                let effective_performance = (score_percentage / 100.0) * performance_multiplier;
                let new_mastery = update_mastery(current_mastery, effective_performance);

                // Save to DB
                let mut mastery_score = glp_core::models::MasteryScore::new(user_id.clone(), skill_id.clone());
                mastery_score.score = new_mastery;
                MasteryRepository::create_or_update(conn, &mastery_score)?;
                mastery_updates.insert(skill_id.clone(), new_mastery);
            }

            // Update progress
            let mut progress = progress.unwrap_or_else(|| NodeProgress::new(user_id.clone(), request.quiz_id.clone()));
            progress.add_time((request.time_spent_ms / 60000) as i32);
            progress.attempts = attempt_number;
            
            let passed = score_percentage >= quiz.passing_score as f64;
            if passed {
                progress.complete();
            } else {
                progress.fail();
            }
            ProgressRepository::create_or_update(conn, &progress)?;

            // Award XP and update level
            UserRepository::update_xp(conn, &user_id, xp_earned)?;
            let new_total_xp = user.total_xp + xp_earned;
            let new_level = calculate_level(new_total_xp);
            UserRepository::update_level(conn, &user_id, new_level as i32)?;

            // Generate feedback
            let feedback = generate_feedback(&quiz, &request.answers);

            Ok(QuizResult {
                score,
                total: total_points,
                score_percentage,
                passed,
                xp_earned,
                attempt_number,
                mastery_updates,
                feedback,
            })
        })
        .map_err(|e| e.to_string())
}

fn load_quiz_from_content(quiz_id: &str) -> Result<Quiz, glp_core::db::error::DbError> {
    // TODO: Load from content system
    // For now, return a dummy quiz for compilation
    Ok(Quiz {
        id: quiz_id.to_string(),
        title: "Sample Quiz".to_string(),
        description: "A sample quiz".to_string(),
        difficulty: "Easy".to_string(),
        skills: vec!["rust".to_string()],
        passing_score: 70,
        time_limit_seconds: None,
        questions: vec![],
    })
}

fn get_mastery_retake_multiplier(attempt_number: usize) -> f64 {
    glp_core::gamification::get_mastery_retake_multiplier(attempt_number)
}
