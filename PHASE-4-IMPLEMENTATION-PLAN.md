# Phase 4: Gamification Implementation Plan

**Status:** ✅ COMPLETE  
**Duration:** Week 9-10 (as per LLM-BUILD-PLAN.md)  
**Goal:** Complete the motivation loop - badges, mastery decay, review queue

---

## Overview

Phase 4 implements the gamification features that create the motivation loop for learners:

1. **Badge System** - 15 achievement badges across 5 categories
2. **Spaced Repetition** - SM-2 algorithm for review scheduling
3. **Mastery Decay** - Forgetting curve implementation
4. **Skill Tree Visualization** - Enhanced UI with node details

---

## Milestone 4.1: Badge System ✅

### Backend (Rust)

**Location:** `crates/core/src/badges/`

#### definitions.rs
- 15 badge definitions across 5 categories:
  - **Streak:** Week Warrior (7d), Streak Master (30d), Unstoppable (100d)
  - **Level:** Rising Star (L5), Apprentice (L10), Journeyman (L20)
  - **XP:** XP Hunter (1K), XP Collector (5K), XP Legend (10K)
  - **Completion:** First Steps (1 lecture), Quiz Whiz (10 quizzes), Completionist (50 total), Perfect Score (100%)
  - **Mastery:** Skill Seeker (50%), Skill Master (90%)

#### tracker.rs
- `check_badge_unlocks()` - Evaluates all badges against user stats
- `check_single_badge()` - Evaluates one badge
- `calculate_badge_progress()` - Returns 0.0-1.0 progress
- `UserStats` struct - Aggregates user metrics for evaluation

### Tauri Commands

**Location:** `apps/desktop/src-tauri/src/commands/badge.rs`

- `get_all_badges()` - Returns badges with progress info
- `get_earned_badges()` - Filters to only earned badges
- `check_and_unlock_badges()` - Checks and unlocks new badges
- `update_badge_progress()` - Updates progress for specific badge

### Frontend (React/TypeScript)

**Components:**
- `Badges.tsx` - Badge collection page with earned/locked sections
- `BadgeUnlockNotification.tsx` - Celebration modal with animations

**Store:** `badgeStore.ts` - Zustand store for badge state

---

## Milestone 4.2: Mastery Decay + Review Queue ✅

### SM-2 Algorithm

**Location:** `crates/core/src/models/review.rs`

```rust
// Key constants
const MIN_EASE_FACTOR: f64 = 1.3;
const INITIAL_EASE_FACTOR: f64 = 2.5;

// Interval progression
// Rep 0: 1 day
// Rep 1: 6 days
// Rep 2+: interval * ease_factor
```

**Location:** `crates/core/src/spaced_repetition/scheduler.rs`

- `schedule_initial_review()` - Creates new review item
- `is_due_now()` - Checks if review is due
- `get_due_reviews()` - Filters to due items
- `score_to_quality()` - Maps quiz score to SM-2 quality (0-5)

### Mastery Decay

**Formula:** `score = score × e^(-0.05 × days_after_grace)`

- **Grace Period:** 3 days (no decay)
- **Decay Rate:** 5% per day after grace
- **Minimum Floor:** 30% (skills never fully forgotten)

**Location:** `crates/core/src/models/mastery.rs`

```rust
pub fn apply_decay(&mut self, days_since_update: i64) {
    const GRACE_PERIOD: i64 = 3;
    const DECAY_RATE: f64 = 0.05;
    const MIN_SCORE: f64 = 0.3;
    
    if days_since_update > GRACE_PERIOD {
        let decay_days = days_since_update - GRACE_PERIOD;
        let decay_factor = (-DECAY_RATE * decay_days as f64).exp();
        self.score = (self.score * decay_factor).max(MIN_SCORE);
    }
}
```

### Tauri Commands

**Location:** `apps/desktop/src-tauri/src/commands/review.rs`

- `get_due_reviews()` - Returns reviews due today
- `get_due_review_count()` - Returns count for badge
- `submit_review()` - Updates SM-2 schedule
- `create_review_item()` - Creates new review after quiz
- `apply_mastery_decay_on_startup()` - Background decay worker
- `get_low_mastery_skills()` - Skills below threshold

### Frontend

**Components:**
- `Review.tsx` - Review queue page with stats and due list

**Store:** `reviewStore.ts` - Zustand store for review state

---

## Milestone 4.3: Skill Tree Visualization ✅

### Enhanced SkillTree Page

**Location:** `apps/desktop/src/pages/SkillTree.tsx`

Features:
- Overall progress bar with percentage
- Color-coded legend (completed/in-progress/available/locked)
- Week-based organization with visual connectors
- Day indicators with connecting lines
- Node status styling with pulse animation for in-progress

### Node Details Modal

**Location:** `apps/desktop/src/components/skilltree/NodeDetailsModal.tsx`

Displays:
- Node icon (lecture/quiz/challenge)
- Status badge (locked/available/in-progress/completed)
- XP reward with icon
- Estimated time
- Difficulty stars (1-4)
- Skills trained (tags)
- Prerequisites (list)
- Action button (Start/Continue/Review Again/Locked)

---

## Navigation Updates

**Location:** `apps/desktop/src/components/layout/Navigation.tsx`

- Added Badges link with Trophy icon
- Added Review link with Brain icon
- Review link shows due count badge (red circle)

---

## Testing

### Unit Tests (94 tests in glp_core)

- Badge unlock criteria evaluation
- SM-2 scheduling calculations
- Mastery decay formulas
- Review item state transitions

### Integration Tests (17 tests)

**Location:** `crates/core/tests/phase4_gamification_tests.rs`

- `test_badge_definitions_comprehensive` - Validates 10-15 badges
- `test_badge_unlock_streak_progression` - Streak badge thresholds
- `test_badge_unlock_xp_progression` - XP badge thresholds
- `test_badge_progress_calculation` - Progress percentage
- `test_no_duplicate_badge_unlocks` - Prevents re-unlocking
- `test_completion_badge_specificity` - Type-specific badges
- `test_sm2_initial_intervals` - SM-2 first reviews
- `test_sm2_ease_factor_adjustment` - Ease factor bounds
- `test_sm2_failed_review_resets` - Reset on failure
- `test_score_to_quality_mapping` - Score→quality
- `test_review_due_date_calculation` - Due date math
- `test_mastery_decay_grace_period` - 3-day grace
- `test_mastery_decay_after_grace_period` - Decay formula
- `test_mastery_minimum_floor` - 30% minimum
- `test_mastery_decay_mixed_skills` - Multiple skills
- `test_mastery_badge_integration` - Badge+mastery
- `test_decay_formula_matches_prototype` - Formula validation

---

## Phase 4 Completion Checklist

- [x] At least 10 badges defined with clear criteria (15 implemented)
- [x] Badges unlock automatically when criteria met
- [x] Unlock notification displays with animation
- [x] Badge collection page shows earned vs locked badges
- [x] Tests verify unlock logic for all badge types
- [x] Mastery decays correctly (validated against prototype)
- [x] Review items scheduled with SM-2 algorithm
- [x] Review UI shows due count and quiz content
- [x] Completing review updates ease factor and next interval
- [x] Grace period prevents weekend decay (3 days)
- [x] Skill tree renders all nodes in logical layout
- [x] Node colors reflect state (locked/unlocked/complete)
- [x] Clicking node shows details modal
- [x] Prerequisites visually clear via edges
- [x] All tests passing (94 unit + 17 integration)

---

## Next Steps: Phase 5

Phase 5 focuses on content integration:
- Content authoring pipeline
- Port Week 1-4 curriculum
- Quiz/challenge generation from templates
- Balance XP if needed

See `LLM-BUILD-PLAN.md` for Phase 5 details.
