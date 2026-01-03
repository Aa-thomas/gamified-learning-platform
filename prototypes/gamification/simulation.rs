/// Progression simulation for different user archetypes
///
/// This module simulates how different types of users progress through
/// the 20-week bootcamp to validate that the gamification formulas are balanced.

mod formulas;
use formulas::*;
use std::collections::HashMap;

/// User archetype for simulation
#[derive(Debug, Clone)]
pub enum UserType {
    Daily,    // 30 min/day for 20 weeks (dedicated learner)
    Binge,    // 8 hours/day for 4 weeks (intensive bootcamp)
    Casual,   // 2 hours/week for 40 weeks (slow and steady)
}

impl UserType {
    fn description(&self) -> &str {
        match self {
            UserType::Daily => "Daily user (30 min/day, 20 weeks)",
            UserType::Binge => "Binge user (8 hours/day, 4 weeks)",
            UserType::Casual => "Casual user (2 hours/week, 40 weeks)",
        }
    }

    fn schedule(&self) -> Schedule {
        match self {
            UserType::Daily => Schedule {
                minutes_per_session: 30,
                sessions_per_week: 7,
                total_weeks: 20,
            },
            UserType::Binge => Schedule {
                minutes_per_session: 480, // 8 hours
                sessions_per_week: 7,
                total_weeks: 4,
            },
            UserType::Casual => Schedule {
                minutes_per_session: 120, // 2 hours
                sessions_per_week: 1,
                total_weeks: 40,
            },
        }
    }
}

#[derive(Debug, Clone)]
struct Schedule {
    minutes_per_session: u32,
    sessions_per_week: u32,
    total_weeks: u32,
}

/// Simulated user progress
pub struct UserSimulation {
    user_type: UserType,
    total_xp: u32,
    current_level: u32,
    current_streak: u32,
    mastery_tracker: MasteryTracker,
    streak_tracker: StreakTracker,
    xp_calculator: XPCalculator,
    level_calculator: LevelCalculator,
    days_active: Vec<u32>,
    badges_earned: Vec<String>,
}

impl UserSimulation {
    pub fn new(user_type: UserType) -> Self {
        Self {
            user_type,
            total_xp: 0,
            current_level: 0,
            current_streak: 0,
            mastery_tracker: MasteryTracker::new(),
            streak_tracker: StreakTracker::new(),
            xp_calculator: XPCalculator::new(),
            level_calculator: LevelCalculator::new(),
            days_active: Vec::new(),
            badges_earned: Vec::new(),
        }
    }

    /// Run the full simulation
    pub fn simulate(&mut self) -> SimulationResult {
        let schedule = self.user_type.schedule();
        let total_days = schedule.total_weeks * 7;
        let sessions_per_day = if schedule.sessions_per_week == 7 { 1 } else { 0 };

        let mut current_week = 1;
        let mut current_day = 0;
        let mut content_completed = 0;

        // Bootcamp structure: 14 weeks of content
        // Each week: 5 lectures, 5 quizzes, 3 mini-challenges, 1 checkpoint
        let weeks_of_content = 14;

        while current_week <= schedule.total_weeks && current_week <= weeks_of_content {
            // Determine which days this week the user is active
            let active_days = self.get_active_days_in_week(
                current_week,
                schedule.sessions_per_week,
            );

            for day_of_week in 0..7 {
                current_day += 1;

                if active_days.contains(&day_of_week) {
                    self.days_active.push(current_day);

                    // Update streak
                    self.streak_tracker.update_streak(current_day);
                    self.current_streak = self.streak_tracker.current_streak();

                    // Complete content based on time available
                    let activities = self.plan_activities(
                        schedule.minutes_per_session,
                        current_week,
                    );

                    for activity in activities {
                        let xp = self.complete_activity(&activity, current_day);
                        self.total_xp += xp;
                        content_completed += 1;

                        // Check for badge unlocks
                        self.check_badges();
                    }

                    // Update level
                    self.current_level = self.level_calculator.level_from_xp(self.total_xp);
                }

                // Apply mastery decay for inactive skills
                self.mastery_tracker.apply_decay(current_day);
            }

            current_week += 1;
        }

        // Continue past content weeks to see if they finish
        while current_day < total_days {
            current_day += 1;
            self.mastery_tracker.apply_decay(current_day);
        }

        SimulationResult {
            user_type: self.user_type.clone(),
            total_xp: self.total_xp,
            final_level: self.current_level,
            max_streak: self.current_streak,
            average_mastery: self.mastery_tracker.average_mastery(),
            content_completed,
            badges_earned: self.badges_earned.len(),
            weeks_to_complete: (self.days_active.len() as f64 / 7.0).ceil() as u32,
        }
    }

    /// Determine which days of the week user is active
    fn get_active_days_in_week(&self, _week: u32, sessions_per_week: u32) -> Vec<u32> {
        if sessions_per_week == 7 {
            vec![0, 1, 2, 3, 4, 5, 6] // Every day
        } else if sessions_per_week == 1 {
            vec![0] // Once a week (Monday)
        } else {
            // Spread evenly through week
            (0..sessions_per_week).collect()
        }
    }

    /// Plan activities for a session based on available time
    fn plan_activities(&self, minutes: u32, week: u32) -> Vec<Activity> {
        let mut activities = Vec::new();
        let mut remaining_minutes = minutes;

        // Week 1-14: Normal content
        // Each week has: lectures, quizzes, challenges, checkpoint

        // Lectures (5 min each)
        while remaining_minutes >= 5 && activities.len() < 5 {
            activities.push(Activity {
                content_type: "lecture".to_string(),
                difficulty: Difficulty::Medium,
                skill: format!("week{}_concept", week),
                duration_minutes: 5,
                expected_performance: 1.0, // Lectures always "complete"
            });
            remaining_minutes -= 5;
        }

        // Quizzes (10 min each)
        while remaining_minutes >= 10 && activities.iter().filter(|a| a.content_type == "quiz").count() < 5 {
            activities.push(Activity {
                content_type: "quiz".to_string(),
                difficulty: Difficulty::Medium,
                skill: format!("week{}_concept", week),
                duration_minutes: 10,
                expected_performance: 0.85, // Average 85% on quizzes
            });
            remaining_minutes -= 10;
        }

        // Mini challenges (30 min each)
        while remaining_minutes >= 30 && activities.iter().filter(|a| a.content_type == "mini_challenge").count() < 3 {
            activities.push(Activity {
                content_type: "mini_challenge".to_string(),
                difficulty: Difficulty::Hard,
                skill: format!("week{}_coding", week),
                duration_minutes: 30,
                expected_performance: 0.80, // Average 80% on challenges
            });
            remaining_minutes -= 30;
        }

        // Checkpoint (60 min)
        if remaining_minutes >= 60 && !activities.iter().any(|a| a.content_type == "checkpoint") {
            activities.push(Activity {
                content_type: "checkpoint".to_string(),
                difficulty: Difficulty::VeryHard,
                skill: format!("week{}_project", week),
                duration_minutes: 60,
                expected_performance: 0.75, // Average 75% on checkpoints
            });
        }

        activities
    }

    /// Complete an activity and return XP earned
    fn complete_activity(&mut self, activity: &Activity, current_day: u32) -> u32 {
        // Calculate XP
        let xp = self.xp_calculator.calculate_xp(
            &activity.content_type,
            activity.difficulty,
            self.current_streak,
            activity.expected_performance,
        );

        // Update mastery
        self.mastery_tracker.update_mastery(
            &activity.skill,
            activity.expected_performance,
            current_day,
            0.25, // Learning rate
        );

        xp
    }

    /// Check for badge unlocks
    fn check_badges(&mut self) {
        // First Steps: Complete first lecture
        if self.total_xp >= 25 && !self.badges_earned.contains(&"first_steps".to_string()) {
            self.badges_earned.push("first_steps".to_string());
        }

        // Week Warrior: 7 day streak
        if self.current_streak >= 7 && !self.badges_earned.contains(&"week_warrior".to_string()) {
            self.badges_earned.push("week_warrior".to_string());
        }

        // Level Milestones
        if self.current_level >= 5 && !self.badges_earned.contains(&"level_5".to_string()) {
            self.badges_earned.push("level_5".to_string());
        }

        if self.current_level >= 10 && !self.badges_earned.contains(&"level_10".to_string()) {
            self.badges_earned.push("level_10".to_string());
        }

        // XP Milestones
        if self.total_xp >= 1000 && !self.badges_earned.contains(&"xp_1k".to_string()) {
            self.badges_earned.push("xp_1k".to_string());
        }

        if self.total_xp >= 5000 && !self.badges_earned.contains(&"xp_5k".to_string()) {
            self.badges_earned.push("xp_5k".to_string());
        }

        // Mastery badges
        if self.mastery_tracker.average_mastery() >= 0.8 && !self.badges_earned.contains(&"mastery_80".to_string()) {
            self.badges_earned.push("mastery_80".to_string());
        }
    }
}

#[derive(Debug, Clone)]
struct Activity {
    content_type: String,
    difficulty: Difficulty,
    skill: String,
    duration_minutes: u32,
    expected_performance: f64,
}

/// Result of a simulation run
#[derive(Debug, Clone)]
pub struct SimulationResult {
    pub user_type: UserType,
    pub total_xp: u32,
    pub final_level: u32,
    pub max_streak: u32,
    pub average_mastery: f64,
    pub content_completed: u32,
    pub badges_earned: u32,
    pub weeks_to_complete: u32,
}

impl SimulationResult {
    pub fn print_report(&self) {
        println!("\n=== {} ===", self.user_type.description());
        println!("Total XP: {}", self.total_xp);
        println!("Final Level: {}", self.final_level);
        println!("Max Streak: {} days", self.max_streak);
        println!("Average Mastery: {:.1}%", self.average_mastery * 100.0);
        println!("Content Completed: {} items", self.content_completed);
        println!("Badges Earned: {}", self.badges_earned);
        println!("Weeks to Finish: {}", self.weeks_to_complete);
    }
}

/// Run all simulations and generate balance report
pub fn run_all_simulations() -> Vec<SimulationResult> {
    let user_types = vec![UserType::Daily, UserType::Binge, UserType::Casual];
    let mut results = Vec::new();

    for user_type in user_types {
        let mut sim = UserSimulation::new(user_type);
        let result = sim.simulate();
        result.print_report();
        results.push(result);
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daily_user_simulation() {
        let mut sim = UserSimulation::new(UserType::Daily);
        let result = sim.simulate();

        // Daily user should complete in ~20 weeks
        assert!(result.weeks_to_complete >= 14);
        assert!(result.weeks_to_complete <= 25);

        // Should earn reasonable XP
        assert!(result.total_xp > 5000);

        // Should reach a good level
        assert!(result.final_level >= 10);
    }

    #[test]
    fn test_binge_user_simulation() {
        let mut sim = UserSimulation::new(UserType::Binge);
        let result = sim.simulate();

        // Binge user completes faster
        assert!(result.weeks_to_complete <= 6);

        // High XP in short time
        assert!(result.total_xp > 5000);
    }

    #[test]
    fn test_casual_user_simulation() {
        let mut sim = UserSimulation::new(UserType::Casual);
        let result = sim.simulate();

        // Casual user takes longer
        assert!(result.weeks_to_complete >= 30);

        // But still makes progress
        assert!(result.total_xp > 2000);
    }
}

fn main() {
    println!("=== Gamification Balance Simulation ===\n");
    println!("Simulating 3 user archetypes through 14-week bootcamp...\n");

    let results = run_all_simulations();

    println!("\n=== Balance Analysis ===");

    // Check if progression feels balanced
    for result in &results {
        print!("\n{}: ", result.user_type.description());

        match result.user_type {
            UserType::Daily => {
                if result.weeks_to_complete >= 14 && result.weeks_to_complete <= 20 {
                    println!("✅ Completes in expected timeframe");
                } else {
                    println!("⚠️  Takes {} weeks (expected ~14-20)", result.weeks_to_complete);
                }

                if result.total_xp >= 8000 && result.total_xp <= 15000 {
                    println!("✅ XP progression balanced");
                } else {
                    println!("⚠️  Total XP: {} (expected ~8K-15K)", result.total_xp);
                }
            }
            UserType::Binge => {
                if result.weeks_to_complete <= 6 {
                    println!("✅ Completes quickly as expected");
                } else {
                    println!("⚠️  Takes {} weeks (expected ~4)", result.weeks_to_complete);
                }
            }
            UserType::Casual => {
                if result.weeks_to_complete >= 30 {
                    println!("✅ Takes longer as expected");
                } else {
                    println!("⚠️  Completes too quickly: {} weeks", result.weeks_to_complete);
                }
            }
        }

        // Check mastery doesn't decay to zero
        if result.average_mastery >= 0.3 {
            println!("✅ Mastery maintained ({:.0}%)", result.average_mastery * 100.0);
        } else {
            println!("⚠️  Mastery too low: {:.0}%", result.average_mastery * 100.0);
        }

        // Check badges unlock regularly
        let badges_per_week = result.badges_earned as f64 / result.weeks_to_complete as f64;
        if badges_per_week >= 0.15 && badges_per_week <= 1.5 {
            println!("✅ Badge frequency feels good (~{:.1} per week)", badges_per_week);
        } else {
            println!("⚠️  Badge frequency: {:.1} per week", badges_per_week);
        }
    }

    println!("\n=== Simulation Complete ===");
}
