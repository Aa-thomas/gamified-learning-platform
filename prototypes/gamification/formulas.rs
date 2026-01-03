/// Gamification formulas for XP, mastery, and progression
///
/// This module implements the core formulas for the learning platform's
/// gamification system, including XP calculation, mastery tracking, and level progression.

use std::collections::HashMap;

/// Difficulty levels for content
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    VeryHard,
}

impl Difficulty {
    /// Get the XP multiplier for this difficulty
    pub fn xp_multiplier(&self) -> f64 {
        match self {
            Difficulty::Easy => 1.0,
            Difficulty::Medium => 1.5,
            Difficulty::Hard => 2.0,
            Difficulty::VeryHard => 3.0,
        }
    }

    /// Get base XP for completing content at this difficulty
    pub fn base_xp(&self) -> u32 {
        match self {
            Difficulty::Easy => 50,
            Difficulty::Medium => 100,
            Difficulty::Hard => 150,
            Difficulty::VeryHard => 250,
        }
    }
}

/// XP calculator with difficulty, streak, and accuracy bonuses
pub struct XPCalculator {
    /// Base XP values by content type
    base_values: HashMap<String, u32>,
}

impl XPCalculator {
    pub fn new() -> Self {
        let mut base_values = HashMap::new();
        base_values.insert("lecture".to_string(), 25);
        base_values.insert("quiz".to_string(), 50);
        base_values.insert("mini_challenge".to_string(), 100);
        base_values.insert("checkpoint".to_string(), 200);

        Self { base_values }
    }

    /// Calculate XP for completing content
    ///
    /// Formula: base_xp × difficulty_mult × streak_mult × accuracy_mult
    pub fn calculate_xp(
        &self,
        content_type: &str,
        difficulty: Difficulty,
        streak_days: u32,
        accuracy: f64, // 0.0-1.0
    ) -> u32 {
        let base = self.base_values.get(content_type).copied().unwrap_or(50);

        let difficulty_mult = difficulty.xp_multiplier();
        let streak_mult = self.streak_multiplier(streak_days);
        let accuracy_mult = self.accuracy_multiplier(accuracy);

        let total = base as f64 * difficulty_mult * streak_mult * accuracy_mult;
        total.round() as u32
    }

    /// Streak multiplier: Rewards consistent daily practice
    ///
    /// Formula:
    /// - Days 1-3: 1.0x (baseline)
    /// - Days 4-7: 1.1x
    /// - Days 8-14: 1.2x
    /// - Days 15-30: 1.3x
    /// - Days 31+: 1.5x (max)
    fn streak_multiplier(&self, days: u32) -> f64 {
        match days {
            0..=3 => 1.0,
            4..=7 => 1.1,
            8..=14 => 1.2,
            15..=30 => 1.3,
            _ => 1.5,
        }
    }

    /// Accuracy multiplier: Rewards getting things right on first try
    ///
    /// Formula:
    /// - 100% correct: 1.5x
    /// - 90-99%: 1.3x
    /// - 80-89%: 1.1x
    /// - 70-79%: 1.0x (baseline)
    /// - 60-69%: 0.8x
    /// - <60%: 0.5x
    fn accuracy_multiplier(&self, accuracy: f64) -> f64 {
        if accuracy >= 1.0 {
            1.5
        } else if accuracy >= 0.9 {
            1.3
        } else if accuracy >= 0.8 {
            1.1
        } else if accuracy >= 0.7 {
            1.0
        } else if accuracy >= 0.6 {
            0.8
        } else {
            0.5
        }
    }
}

/// Level progression calculator
pub struct LevelCalculator {
    /// XP required for each level (exponential growth)
    level_thresholds: Vec<u32>,
}

impl LevelCalculator {
    pub fn new() -> Self {
        // Generate level thresholds using exponential formula
        // Level N requires: 100 * (N^1.5)
        let mut thresholds = vec![0]; // Level 0
        for level in 1..=100 {
            let xp_required = (100.0 * (level as f64).powf(1.5)).round() as u32;
            thresholds.push(thresholds.last().unwrap() + xp_required);
        }

        Self {
            level_thresholds: thresholds,
        }
    }

    /// Get current level from total XP
    pub fn level_from_xp(&self, total_xp: u32) -> u32 {
        for (level, &threshold) in self.level_thresholds.iter().enumerate() {
            if total_xp < threshold {
                return level.saturating_sub(1) as u32;
            }
        }
        self.level_thresholds.len() as u32 - 1
    }

    /// Get XP required for next level
    pub fn xp_to_next_level(&self, total_xp: u32) -> u32 {
        let current_level = self.level_from_xp(total_xp) as usize;
        if current_level + 1 < self.level_thresholds.len() {
            self.level_thresholds[current_level + 1].saturating_sub(total_xp)
        } else {
            0 // Max level
        }
    }

    /// Get XP required for a specific level
    pub fn xp_for_level(&self, level: u32) -> u32 {
        self.level_thresholds.get(level as usize).copied().unwrap_or(0)
    }
}

/// Mastery score tracker with learning rate and decay
pub struct MasteryTracker {
    /// Current mastery scores by skill (0.0-1.0)
    scores: HashMap<String, f64>,
    /// Last practice date by skill
    last_practiced: HashMap<String, u32>, // Days since start
}

impl MasteryTracker {
    pub fn new() -> Self {
        Self {
            scores: HashMap::new(),
            last_practiced: HashMap::new(),
        }
    }

    /// Update mastery score after practice
    ///
    /// Formula: new_score = old_score + learning_rate × (performance - old_score)
    ///
    /// This is a weighted moving average that:
    /// - Increases faster when starting from low mastery
    /// - Increases slower when approaching mastery
    /// - Responds to performance (high performance = higher score)
    pub fn update_mastery(
        &mut self,
        skill: &str,
        performance: f64, // 0.0-1.0 (quiz/challenge score)
        current_day: u32,
        learning_rate: f64, // Typically 0.2-0.3
    ) {
        let current_score = self.scores.get(skill).copied().unwrap_or(0.0);

        // Exponential moving average
        let new_score = current_score + learning_rate * (performance - current_score);
        let clamped_score = new_score.max(0.0).min(1.0);

        self.scores.insert(skill.to_string(), clamped_score);
        self.last_practiced.insert(skill.to_string(), current_day);
    }

    /// Apply decay to inactive skills
    ///
    /// Formula: score = score × e^(-decay_rate × days_inactive)
    ///
    /// Decay parameters:
    /// - Grace period: 3 days (no decay)
    /// - Decay rate: 0.05 (5% per day after grace period)
    /// - Minimum: 0.3 (doesn't decay below 30%)
    ///
    /// This creates a forgetting curve that:
    /// - Doesn't punish short breaks (weekend)
    /// - Gradually reduces mastery over time
    /// - Never completely zeros out learned skills
    pub fn apply_decay(&mut self, current_day: u32) {
        const GRACE_PERIOD_DAYS: u32 = 3;
        const DECAY_RATE: f64 = 0.05;
        const MIN_MASTERY: f64 = 0.3;

        for (skill, score) in self.scores.iter_mut() {
            if let Some(&last_day) = self.last_practiced.get(skill) {
                let days_inactive = current_day.saturating_sub(last_day);

                if days_inactive > GRACE_PERIOD_DAYS {
                    let decay_days = days_inactive - GRACE_PERIOD_DAYS;
                    let decay_factor = (-DECAY_RATE * decay_days as f64).exp();
                    let decayed_score = *score * decay_factor;

                    *score = decayed_score.max(MIN_MASTERY);
                }
            }
        }
    }

    /// Get current mastery score for a skill
    pub fn get_mastery(&self, skill: &str) -> f64 {
        self.scores.get(skill).copied().unwrap_or(0.0)
    }

    /// Get average mastery across all skills
    pub fn average_mastery(&self) -> f64 {
        if self.scores.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.scores.values().sum();
        sum / self.scores.len() as f64
    }

    /// Get skills below mastery threshold (need practice)
    pub fn skills_needing_practice(&self, threshold: f64) -> Vec<String> {
        self.scores
            .iter()
            .filter(|(_, &score)| score < threshold)
            .map(|(skill, _)| skill.clone())
            .collect()
    }
}

/// Streak tracker with grace period
pub struct StreakTracker {
    current_streak: u32,
    last_activity_day: Option<u32>,
}

impl StreakTracker {
    pub fn new() -> Self {
        Self {
            current_streak: 0,
            last_activity_day: None,
        }
    }

    /// Update streak based on activity
    ///
    /// Rules:
    /// - Same day: streak continues
    /// - Next day: streak increments
    /// - 1 day gap: grace period, streak continues with warning
    /// - 2+ days gap: streak resets to 1
    pub fn update_streak(&mut self, current_day: u32) -> StreakStatus {
        match self.last_activity_day {
            None => {
                // First activity ever
                self.current_streak = 1;
                self.last_activity_day = Some(current_day);
                StreakStatus::Started
            }
            Some(last_day) => {
                let gap = current_day.saturating_sub(last_day);

                match gap {
                    0 => StreakStatus::Continued, // Same day
                    1 => {
                        // Next day - increment streak
                        self.current_streak += 1;
                        self.last_activity_day = Some(current_day);
                        StreakStatus::Incremented(self.current_streak)
                    }
                    2 => {
                        // Grace period - streak continues but warn user
                        self.last_activity_day = Some(current_day);
                        StreakStatus::GracePeriod(self.current_streak)
                    }
                    _ => {
                        // Streak broken
                        let old_streak = self.current_streak;
                        self.current_streak = 1;
                        self.last_activity_day = Some(current_day);
                        StreakStatus::Broken { old_streak }
                    }
                }
            }
        }
    }

    pub fn current_streak(&self) -> u32 {
        self.current_streak
    }
}

#[derive(Debug, PartialEq)]
pub enum StreakStatus {
    Started,
    Continued,
    Incremented(u32),
    GracePeriod(u32),
    Broken { old_streak: u32 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xp_calculation() {
        let calc = XPCalculator::new();

        // Base quiz, medium difficulty, no streak, perfect accuracy
        let xp = calc.calculate_xp("quiz", Difficulty::Medium, 0, 1.0);
        assert_eq!(xp, 50 * 1.5 * 1.0 * 1.5); // 112

        // Hard challenge, 10 day streak, 90% accuracy
        let xp = calc.calculate_xp("mini_challenge", Difficulty::Hard, 10, 0.9);
        // 100 * 2.0 * 1.2 * 1.3 = 312
        assert!(xp >= 310 && xp <= 315);
    }

    #[test]
    fn test_level_progression() {
        let calc = LevelCalculator::new();

        assert_eq!(calc.level_from_xp(0), 0);
        assert_eq!(calc.level_from_xp(100), 1);
        assert_eq!(calc.level_from_xp(500), 3);

        // XP to next level decreases as you earn XP
        let xp_to_2 = calc.xp_to_next_level(100);
        let xp_to_2_later = calc.xp_to_next_level(150);
        assert!(xp_to_2_later < xp_to_2);
    }

    #[test]
    fn test_mastery_learning() {
        let mut tracker = MasteryTracker::new();

        // Start with no mastery
        assert_eq!(tracker.get_mastery("ownership"), 0.0);

        // Practice with 80% performance
        tracker.update_mastery("ownership", 0.8, 1, 0.3);
        let score1 = tracker.get_mastery("ownership");
        assert!(score1 > 0.0 && score1 < 0.8);

        // Practice again with 100% performance
        tracker.update_mastery("ownership", 1.0, 2, 0.3);
        let score2 = tracker.get_mastery("ownership");
        assert!(score2 > score1);

        // Multiple practices converge toward performance
        for day in 3..10 {
            tracker.update_mastery("ownership", 0.95, day, 0.3);
        }
        let final_score = tracker.get_mastery("ownership");
        assert!(final_score > 0.9);
    }

    #[test]
    fn test_mastery_decay() {
        let mut tracker = MasteryTracker::new();

        // Build up mastery
        tracker.update_mastery("lifetimes", 0.9, 1, 0.3);
        let initial = tracker.get_mastery("lifetimes");

        // No decay in grace period (3 days)
        tracker.apply_decay(4);
        assert_eq!(tracker.get_mastery("lifetimes"), initial);

        // Decay after grace period
        tracker.apply_decay(10); // 9 days since last practice
        let decayed = tracker.get_mastery("lifetimes");
        assert!(decayed < initial);
        assert!(decayed >= 0.3); // Doesn't go below minimum
    }

    #[test]
    fn test_streak_mechanics() {
        let mut tracker = StreakTracker::new();

        // Start streak
        assert_eq!(tracker.update_streak(1), StreakStatus::Started);
        assert_eq!(tracker.current_streak(), 1);

        // Continue next day
        assert_eq!(tracker.update_streak(2), StreakStatus::Incremented(2));
        assert_eq!(tracker.current_streak(), 2);

        // Same day activity
        assert_eq!(tracker.update_streak(2), StreakStatus::Continued);
        assert_eq!(tracker.current_streak(), 2);

        // Grace period (1 day gap)
        assert_eq!(tracker.update_streak(4), StreakStatus::GracePeriod(2));
        assert_eq!(tracker.current_streak(), 2);

        // Break streak (2+ day gap)
        let status = tracker.update_streak(7);
        assert_eq!(status, StreakStatus::Broken { old_streak: 2 });
        assert_eq!(tracker.current_streak(), 1);
    }

    #[test]
    fn test_difficulty_multipliers() {
        assert_eq!(Difficulty::Easy.xp_multiplier(), 1.0);
        assert_eq!(Difficulty::Medium.xp_multiplier(), 1.5);
        assert_eq!(Difficulty::Hard.xp_multiplier(), 2.0);
        assert_eq!(Difficulty::VeryHard.xp_multiplier(), 3.0);
    }
}
