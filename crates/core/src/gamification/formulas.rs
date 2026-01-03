use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    VeryHard,
}

// Base XP values per content type
pub const LECTURE_BASE_XP: i32 = 25;
pub const QUIZ_BASE_XP: i32 = 50;
pub const CHALLENGE_BASE_XP: i32 = 100;
pub const CHECKPOINT_BASE_XP: i32 = 200;

// Mastery learning rate
pub const LEARNING_RATE: f64 = 0.25;
pub const MASTERY_FLOOR: f64 = 0.30;

/// Get difficulty multiplier for XP calculation
pub fn get_difficulty_multiplier(difficulty: Difficulty) -> f64 {
    match difficulty {
        Difficulty::Easy => 1.0,
        Difficulty::Medium => 1.5,
        Difficulty::Hard => 2.0,
        Difficulty::VeryHard => 3.0,
    }
}

/// Get streak multiplier based on current streak days
pub fn get_streak_multiplier(streak_days: u32) -> f64 {
    match streak_days {
        0..=3 => 1.0,
        4..=7 => 1.1,
        8..=14 => 1.2,
        15..=30 => 1.3,
        _ => 1.5,
    }
}

/// Get accuracy multiplier based on performance percentage
pub fn get_accuracy_multiplier(accuracy_pct: f64) -> f64 {
    match accuracy_pct {
        a if a >= 100.0 => 1.5,
        a if a >= 90.0 => 1.3,
        a if a >= 80.0 => 1.1,
        a if a >= 70.0 => 1.0,
        a if a >= 60.0 => 0.8,
        _ => 0.5,
    }
}

/// Calculate XP for lecture completion
pub fn calculate_lecture_xp(difficulty: Difficulty, streak_days: u32) -> i32 {
    let base = LECTURE_BASE_XP as f64;
    let diff_mult = get_difficulty_multiplier(difficulty);
    let streak_mult = get_streak_multiplier(streak_days);

    (base * diff_mult * streak_mult).round() as i32
}

/// Calculate XP for quiz completion
pub fn calculate_quiz_xp(
    difficulty: Difficulty,
    score_percentage: f64,
    streak_days: u32,
) -> i32 {
    let base = QUIZ_BASE_XP as f64;
    let diff_mult = get_difficulty_multiplier(difficulty);
    let streak_mult = get_streak_multiplier(streak_days);
    let accuracy_mult = get_accuracy_multiplier(score_percentage);

    (base * diff_mult * streak_mult * accuracy_mult).round() as i32
}

/// Calculate level from total XP
/// Formula: Level N requires 100 × N^1.5 cumulative XP
pub fn calculate_level(total_xp: i32) -> u32 {
    if total_xp < 0 {
        return 1;
    }

    let mut level = 1;
    while xp_required_for_level(level + 1) <= total_xp {
        level += 1;
    }
    level
}

/// Calculate XP required to reach a specific level
pub fn xp_required_for_level(level: u32) -> i32 {
    if level <= 1 {
        return 0;
    }
    (100.0 * (level as f64).powf(1.5)).round() as i32
}

/// Calculate XP progress toward next level
/// Returns (progress, total_needed)
pub fn xp_to_next_level(current_xp: i32) -> (i32, i32) {
    let current_level = calculate_level(current_xp);
    let next_level_xp = xp_required_for_level(current_level + 1);
    let current_level_xp = xp_required_for_level(current_level);

    let xp_progress = current_xp - current_level_xp;
    let xp_total_for_level = next_level_xp - current_level_xp;

    (xp_progress, xp_total_for_level)
}

/// Update mastery score using exponential moving average
pub fn update_mastery(current_score: f64, performance: f64) -> f64 {
    let new_score = current_score + LEARNING_RATE * (performance - current_score);
    new_score.clamp(0.0, 1.0)
}

/// Get XP multiplier for quiz retakes
pub fn get_retake_multiplier(attempt_number: usize) -> f64 {
    match attempt_number {
        0 => 0.0,
        1 => 1.0,
        2 => 0.5,
        3 => 0.25,
        _ => 0.1,
    }
}

/// Get mastery multiplier for quiz retakes
pub fn get_mastery_retake_multiplier(attempt_number: usize) -> f64 {
    match attempt_number {
        0 => 0.0,
        1 => 1.0,
        2 => 0.75,
        3 => 0.5,
        _ => 0.25,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_multipliers() {
        assert_eq!(get_difficulty_multiplier(Difficulty::Easy), 1.0);
        assert_eq!(get_difficulty_multiplier(Difficulty::Medium), 1.5);
        assert_eq!(get_difficulty_multiplier(Difficulty::Hard), 2.0);
        assert_eq!(get_difficulty_multiplier(Difficulty::VeryHard), 3.0);
    }

    #[test]
    fn test_streak_multipliers() {
        assert_eq!(get_streak_multiplier(0), 1.0);
        assert_eq!(get_streak_multiplier(1), 1.0);
        assert_eq!(get_streak_multiplier(3), 1.0);
        assert_eq!(get_streak_multiplier(5), 1.1);
        assert_eq!(get_streak_multiplier(10), 1.2);
        assert_eq!(get_streak_multiplier(20), 1.3);
        assert_eq!(get_streak_multiplier(31), 1.5);
        assert_eq!(get_streak_multiplier(100), 1.5);
    }

    #[test]
    fn test_accuracy_multipliers() {
        assert_eq!(get_accuracy_multiplier(100.0), 1.5);
        assert_eq!(get_accuracy_multiplier(95.0), 1.3);
        assert_eq!(get_accuracy_multiplier(85.0), 1.1);
        assert_eq!(get_accuracy_multiplier(75.0), 1.0);
        assert_eq!(get_accuracy_multiplier(65.0), 0.8);
        assert_eq!(get_accuracy_multiplier(50.0), 0.5);
    }

    #[test]
    fn test_lecture_xp_calculation() {
        // Easy lecture, no streak
        assert_eq!(calculate_lecture_xp(Difficulty::Easy, 0), 25);
        
        // Medium lecture, 10-day streak
        assert_eq!(calculate_lecture_xp(Difficulty::Medium, 10), 45); // 25 * 1.5 * 1.2
        
        // Hard lecture, 31-day streak
        assert_eq!(calculate_lecture_xp(Difficulty::Hard, 31), 75); // 25 * 2.0 * 1.5
    }

    #[test]
    fn test_quiz_xp_calculation() {
        // Easy quiz, perfect score, no streak
        assert_eq!(calculate_quiz_xp(Difficulty::Easy, 100.0, 0), 75); // 50 * 1.0 * 1.0 * 1.5
        
        // Medium quiz, 90% score, 10-day streak
        assert_eq!(calculate_quiz_xp(Difficulty::Medium, 90.0, 10), 117); // 50 * 1.5 * 1.2 * 1.3
        
        // Hard quiz, 75% score, no streak
        assert_eq!(calculate_quiz_xp(Difficulty::Hard, 75.0, 0), 100); // 50 * 2.0 * 1.0 * 1.0
    }

    #[test]
    fn test_level_calculation() {
        assert_eq!(calculate_level(0), 1);
        assert_eq!(calculate_level(100), 1);
        assert_eq!(calculate_level(283), 2);
        assert_eq!(calculate_level(520), 3);
        assert_eq!(calculate_level(3162), 10);
        assert_eq!(calculate_level(-1), 1); // Edge case: negative XP
    }

    #[test]
    fn test_xp_required_for_level() {
        assert_eq!(xp_required_for_level(1), 0);
        assert_eq!(xp_required_for_level(2), 283);
        assert_eq!(xp_required_for_level(5), 1118);
        assert_eq!(xp_required_for_level(10), 3162);
    }

    #[test]
    fn test_xp_to_next_level() {
        // At 100 XP (level 1)
        let (progress, total) = xp_to_next_level(100);
        assert_eq!(progress, 100);
        assert_eq!(total, 283);

        // At 300 XP (level 2)
        let (progress, total) = xp_to_next_level(300);
        assert_eq!(progress, 17); // 300 - 283
        assert_eq!(total, 237); // 520 - 283
    }

    #[test]
    fn test_mastery_update() {
        // First quiz: 80% from 0
        let new = update_mastery(0.0, 0.8);
        assert_eq!(new, 0.20); // 0.0 + 0.25 * (0.8 - 0.0)
        
        // Second quiz: 90% from 0.20
        let new2 = update_mastery(0.20, 0.9);
        assert_eq!(new2, 0.375); // 0.20 + 0.25 * (0.9 - 0.20)

        // Perfect score from 0.5
        let new3 = update_mastery(0.5, 1.0);
        assert_eq!(new3, 0.625); // 0.5 + 0.25 * (1.0 - 0.5)
    }

    #[test]
    fn test_retake_multipliers() {
        assert_eq!(get_retake_multiplier(0), 0.0);
        assert_eq!(get_retake_multiplier(1), 1.0);
        assert_eq!(get_retake_multiplier(2), 0.5);
        assert_eq!(get_retake_multiplier(3), 0.25);
        assert_eq!(get_retake_multiplier(4), 0.1);
        assert_eq!(get_retake_multiplier(10), 0.1);
    }

    #[test]
    fn test_mastery_retake_multipliers() {
        assert_eq!(get_mastery_retake_multiplier(1), 1.0);
        assert_eq!(get_mastery_retake_multiplier(2), 0.75);
        assert_eq!(get_mastery_retake_multiplier(3), 0.5);
        assert_eq!(get_mastery_retake_multiplier(4), 0.25);
    }

    #[test]
    fn test_mastery_bounds() {
        // Can't go below 0
        let result = update_mastery(0.0, -1.0);
        assert_eq!(result, 0.0);

        // Can't go above 1
        let result = update_mastery(0.9, 2.0);
        assert_eq!(result, 1.0);
    }
}
