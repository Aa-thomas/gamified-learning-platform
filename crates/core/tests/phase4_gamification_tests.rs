//! Phase 4 Integration Tests: Gamification Features
//!
//! These tests verify that badge unlocks, mastery decay, and spaced repetition
//! work correctly and match the prototype formulas.

use chrono::{Duration, Utc};
use glp_core::{
    badges::{
        check_badge_unlocks, check_single_badge, calculate_badge_progress,
        get_all_badge_definitions, get_badge_by_id, UserStats,
    },
    models::{BadgeCategory, BadgeDefinition, BadgeProgress, MasteryScore, ReviewItem},
    spaced_repetition::{
        apply_mastery_decay, score_to_quality, ReviewQuality,
    },
};

// =============================================================================
// Badge System Tests
// =============================================================================

#[test]
fn test_badge_definitions_comprehensive() {
    let badges = get_all_badge_definitions();
    
    // Should have 10-15 badges as per spec
    assert!(badges.len() >= 10, "Expected at least 10 badges, got {}", badges.len());
    assert!(badges.len() <= 15, "Expected at most 15 badges, got {}", badges.len());
    
    // Check we have badges in each category
    let categories: Vec<_> = badges.iter().map(|b| &b.category).collect();
    assert!(categories.iter().any(|c| matches!(c, BadgeCategory::Streak)));
    assert!(categories.iter().any(|c| matches!(c, BadgeCategory::Level)));
    assert!(categories.iter().any(|c| matches!(c, BadgeCategory::Xp)));
    assert!(categories.iter().any(|c| matches!(c, BadgeCategory::Completion)));
    assert!(categories.iter().any(|c| matches!(c, BadgeCategory::Mastery)));
}

#[test]
fn test_badge_unlock_streak_progression() {
    // Test the 3 streak badges unlock at correct thresholds
    let week_warrior = get_badge_by_id("week_warrior").unwrap();
    let streak_master = get_badge_by_id("streak_master").unwrap();
    let unstoppable = get_badge_by_id("unstoppable").unwrap();
    
    assert_eq!(week_warrior.threshold, 7.0);
    assert_eq!(streak_master.threshold, 30.0);
    assert_eq!(unstoppable.threshold, 100.0);
    
    // 6-day streak should not unlock week_warrior
    let stats_6_days = UserStats {
        streak_days: 6,
        ..Default::default()
    };
    assert!(!check_single_badge(&week_warrior, &stats_6_days));
    
    // 7-day streak should unlock week_warrior but not streak_master
    let stats_7_days = UserStats {
        streak_days: 7,
        ..Default::default()
    };
    assert!(check_single_badge(&week_warrior, &stats_7_days));
    assert!(!check_single_badge(&streak_master, &stats_7_days));
    
    // 30-day streak should unlock both week_warrior and streak_master
    let stats_30_days = UserStats {
        streak_days: 30,
        ..Default::default()
    };
    assert!(check_single_badge(&week_warrior, &stats_30_days));
    assert!(check_single_badge(&streak_master, &stats_30_days));
    assert!(!check_single_badge(&unstoppable, &stats_30_days));
}

#[test]
fn test_badge_unlock_xp_progression() {
    let xp_hunter = get_badge_by_id("xp_hunter").unwrap();
    let xp_collector = get_badge_by_id("xp_collector").unwrap();
    let xp_legend = get_badge_by_id("xp_legend").unwrap();
    
    assert_eq!(xp_hunter.threshold, 1000.0);
    assert_eq!(xp_collector.threshold, 5000.0);
    assert_eq!(xp_legend.threshold, 10000.0);
    
    // Test boundary conditions
    let stats_999 = UserStats { total_xp: 999, ..Default::default() };
    let stats_1000 = UserStats { total_xp: 1000, ..Default::default() };
    let stats_5000 = UserStats { total_xp: 5000, ..Default::default() };
    
    assert!(!check_single_badge(&xp_hunter, &stats_999));
    assert!(check_single_badge(&xp_hunter, &stats_1000));
    assert!(check_single_badge(&xp_collector, &stats_5000));
}

#[test]
fn test_badge_progress_calculation() {
    let badge = get_badge_by_id("week_warrior").unwrap();
    
    let stats_0 = UserStats { streak_days: 0, ..Default::default() };
    let stats_3 = UserStats { streak_days: 3, ..Default::default() };
    let stats_7 = UserStats { streak_days: 7, ..Default::default() };
    let stats_14 = UserStats { streak_days: 14, ..Default::default() };
    
    let progress_0 = calculate_badge_progress(&badge, &stats_0);
    let progress_3 = calculate_badge_progress(&badge, &stats_3);
    let progress_7 = calculate_badge_progress(&badge, &stats_7);
    let progress_14 = calculate_badge_progress(&badge, &stats_14);
    
    assert!((progress_0 - 0.0).abs() < 0.01);
    assert!((progress_3 - (3.0 / 7.0)).abs() < 0.01);
    assert!((progress_7 - 1.0).abs() < 0.01);
    assert!((progress_14 - 1.0).abs() < 0.01); // Capped at 1.0
}

#[test]
fn test_no_duplicate_badge_unlocks() {
    let stats = UserStats {
        streak_days: 10,
        level: 10,
        total_xp: 5000,
        ..Default::default()
    };
    
    // First check with no prior progress
    let unlocked_first = check_badge_unlocks(&stats, &[]);
    assert!(!unlocked_first.is_empty());
    
    // Create progress for already unlocked badges
    let progress: Vec<BadgeProgress> = unlocked_first.iter().map(|id| {
        let mut bp = BadgeProgress::new("user1".to_string(), id.clone());
        bp.update_progress(100.0, 100.0); // Mark as earned
        bp
    }).collect();
    
    // Second check should not return already unlocked badges
    let unlocked_second = check_badge_unlocks(&stats, &progress);
    for id in &unlocked_second {
        assert!(!unlocked_first.contains(id), "Badge {} should not be unlocked again", id);
    }
}

#[test]
fn test_completion_badge_specificity() {
    // Test that first_steps requires lectures, quiz_whiz requires quizzes
    let first_steps = get_badge_by_id("first_steps").unwrap();
    let quiz_whiz = get_badge_by_id("quiz_whiz").unwrap();
    
    // User with completed lectures but no quizzes
    let stats_lectures = UserStats {
        completed_lectures: 5,
        completed_quizzes: 0,
        total_completions: 5,
        ..Default::default()
    };
    
    assert!(check_single_badge(&first_steps, &stats_lectures));
    assert!(!check_single_badge(&quiz_whiz, &stats_lectures));
    
    // User with completed quizzes but no lectures
    let stats_quizzes = UserStats {
        completed_lectures: 0,
        completed_quizzes: 10,
        total_completions: 10,
        ..Default::default()
    };
    
    assert!(!check_single_badge(&first_steps, &stats_quizzes));
    assert!(check_single_badge(&quiz_whiz, &stats_quizzes));
}

// =============================================================================
// SM-2 Spaced Repetition Tests
// =============================================================================

#[test]
fn test_sm2_initial_intervals() {
    let mut item = ReviewItem::new("user1".to_string(), "quiz1".to_string());
    
    // Initial state
    assert_eq!(item.interval_days, 1);
    assert_eq!(item.repetitions, 0);
    assert!((item.ease_factor - 2.5).abs() < 0.01);
    
    // First successful review
    item.update_after_review(4); // Good
    assert_eq!(item.repetitions, 1);
    assert_eq!(item.interval_days, 1);
    
    // Second successful review
    item.update_after_review(4); // Good
    assert_eq!(item.repetitions, 2);
    assert_eq!(item.interval_days, 6);
}

#[test]
fn test_sm2_ease_factor_adjustment() {
    let mut item = ReviewItem::new("user1".to_string(), "quiz1".to_string());
    let initial_ease = item.ease_factor;
    
    // Perfect response should increase ease factor
    item.update_after_review(5); // Perfect
    assert!(item.ease_factor > initial_ease);
    
    let perfect_ease = item.ease_factor;
    
    // Reset for hard response test
    let mut item2 = ReviewItem::new("user1".to_string(), "quiz2".to_string());
    item2.update_after_review(3); // Difficult
    assert!(item2.ease_factor < initial_ease);
    
    // Ease factor has minimum of 1.3
    let mut item3 = ReviewItem::new("user1".to_string(), "quiz3".to_string());
    for _ in 0..20 {
        item3.update_after_review(3); // Keep answering difficult
    }
    assert!(item3.ease_factor >= 1.3);
}

#[test]
fn test_sm2_failed_review_resets() {
    let mut item = ReviewItem::new("user1".to_string(), "quiz1".to_string());
    
    // Build up some progress
    item.update_after_review(4);
    item.update_after_review(4);
    item.update_after_review(4);
    
    assert!(item.repetitions >= 3);
    assert!(item.interval_days > 1);
    
    // Fail the review
    item.update_after_review(2); // Hard (fail)
    
    // Should reset
    assert_eq!(item.repetitions, 0);
    assert_eq!(item.interval_days, 1);
}

#[test]
fn test_score_to_quality_mapping() {
    // Verify score percentages map to correct quality ratings
    assert_eq!(score_to_quality(100.0), ReviewQuality::Perfect);
    assert_eq!(score_to_quality(95.0), ReviewQuality::Good);
    assert_eq!(score_to_quality(85.0), ReviewQuality::Difficult);
    assert_eq!(score_to_quality(70.0), ReviewQuality::Hard);
    assert_eq!(score_to_quality(50.0), ReviewQuality::Wrong);
    assert_eq!(score_to_quality(30.0), ReviewQuality::Blackout);
}

#[test]
fn test_review_due_date_calculation() {
    let mut item = ReviewItem::new("user1".to_string(), "quiz1".to_string());
    let now = Utc::now();
    
    // Initial due date is tomorrow
    assert!(item.due_date > now);
    assert!(item.due_date < now + Duration::days(2));
    
    // After successful review, due date should be further out
    item.update_after_review(4);
    assert!(item.due_date > now);
    
    item.update_after_review(4);
    // After second review, interval should be 6 days
    assert!(item.due_date > now + Duration::days(5));
}

// =============================================================================
// Mastery Decay Tests
// =============================================================================

#[test]
fn test_mastery_decay_grace_period() {
    // Skills practiced within 3 days should not decay
    let mut masteries = vec![
        MasteryScore {
            user_id: "user1".to_string(),
            skill_id: "skill1".to_string(),
            score: 0.8,
            last_updated_at: Utc::now() - Duration::days(2),
        },
        MasteryScore {
            user_id: "user1".to_string(),
            skill_id: "skill2".to_string(),
            score: 0.8,
            last_updated_at: Utc::now() - Duration::days(3),
        },
    ];
    
    let decayed = apply_mastery_decay(&mut masteries, Utc::now());
    
    assert_eq!(decayed, 0);
    assert_eq!(masteries[0].score, 0.8);
    assert_eq!(masteries[1].score, 0.8);
}

#[test]
fn test_mastery_decay_after_grace_period() {
    // Skills inactive beyond 3 days should decay
    let mut masteries = vec![
        MasteryScore {
            user_id: "user1".to_string(),
            skill_id: "skill1".to_string(),
            score: 0.8,
            last_updated_at: Utc::now() - Duration::days(10),
        },
    ];
    
    let decayed = apply_mastery_decay(&mut masteries, Utc::now());
    
    assert_eq!(decayed, 1);
    assert!(masteries[0].score < 0.8);
    
    // Verify decay formula: score * e^(-0.05 * days_after_grace)
    // days_inactive = 10, grace = 3, decay_days = 7
    // expected = 0.8 * e^(-0.05 * 7) ≈ 0.8 * 0.7047 ≈ 0.564
    assert!((masteries[0].score - 0.564).abs() < 0.02);
}

#[test]
fn test_mastery_minimum_floor() {
    // Mastery should never go below 30%
    let mut masteries = vec![
        MasteryScore {
            user_id: "user1".to_string(),
            skill_id: "skill1".to_string(),
            score: 0.4,
            last_updated_at: Utc::now() - Duration::days(100), // Very old
        },
    ];
    
    apply_mastery_decay(&mut masteries, Utc::now());
    
    assert!(masteries[0].score >= 0.3, "Mastery should not go below 30%, got {}", masteries[0].score);
}

#[test]
fn test_mastery_decay_mixed_skills() {
    // Test with a mix of fresh and stale skills
    let mut masteries = vec![
        MasteryScore {
            user_id: "user1".to_string(),
            skill_id: "fresh".to_string(),
            score: 0.9,
            last_updated_at: Utc::now() - Duration::days(1),
        },
        MasteryScore {
            user_id: "user1".to_string(),
            skill_id: "medium".to_string(),
            score: 0.8,
            last_updated_at: Utc::now() - Duration::days(7),
        },
        MasteryScore {
            user_id: "user1".to_string(),
            skill_id: "stale".to_string(),
            score: 0.7,
            last_updated_at: Utc::now() - Duration::days(30),
        },
    ];
    
    let decayed = apply_mastery_decay(&mut masteries, Utc::now());
    
    // Fresh skill should not decay
    assert_eq!(masteries[0].score, 0.9);
    
    // Medium skill should decay slightly (4 days after grace)
    assert!(masteries[1].score < 0.8);
    assert!(masteries[1].score > 0.6);
    
    // Stale skill should decay more (27 days after grace)
    assert!(masteries[2].score < 0.7);
    assert!(masteries[2].score >= 0.3); // But not below floor
    
    assert_eq!(decayed, 2);
}

// =============================================================================
// Integration: Badge + Mastery Interaction
// =============================================================================

#[test]
fn test_mastery_badge_integration() {
    let skill_seeker = get_badge_by_id("skill_seeker").unwrap();
    let skill_master = get_badge_by_id("skill_master").unwrap();
    
    assert_eq!(skill_seeker.threshold, 0.5);
    assert_eq!(skill_master.threshold, 0.9);
    
    // User with 50% max mastery should unlock skill_seeker
    let stats_50 = UserStats {
        max_mastery_score: 0.5,
        ..Default::default()
    };
    assert!(check_single_badge(&skill_seeker, &stats_50));
    assert!(!check_single_badge(&skill_master, &stats_50));
    
    // User with 90% max mastery should unlock both
    let stats_90 = UserStats {
        max_mastery_score: 0.9,
        ..Default::default()
    };
    assert!(check_single_badge(&skill_seeker, &stats_90));
    assert!(check_single_badge(&skill_master, &stats_90));
}

// =============================================================================
// Formula Validation Against Prototype
// =============================================================================

#[test]
fn test_decay_formula_matches_prototype() {
    // Prototype formula: score = score × e^(-0.05 × days_inactive)
    // with grace period of 3 days and minimum of 0.3
    
    let test_cases = vec![
        (0.8, 5, 0.8 * (-0.05 * 2.0_f64).exp()),  // 5 days = 2 after grace
        (0.8, 10, 0.8 * (-0.05 * 7.0_f64).exp()), // 10 days = 7 after grace
        (0.5, 20, 0.5 * (-0.05 * 17.0_f64).exp()), // May hit floor
    ];
    
    for (initial_score, days_inactive, expected) in test_cases {
        let mut masteries = vec![
            MasteryScore {
                user_id: "user1".to_string(),
                skill_id: "test".to_string(),
                score: initial_score,
                last_updated_at: Utc::now() - Duration::days(days_inactive),
            },
        ];
        
        apply_mastery_decay(&mut masteries, Utc::now());
        
        let expected_clamped = expected.max(0.3);
        assert!(
            (masteries[0].score - expected_clamped).abs() < 0.02,
            "For initial={}, days={}: expected {}, got {}",
            initial_score, days_inactive, expected_clamped, masteries[0].score
        );
    }
}
