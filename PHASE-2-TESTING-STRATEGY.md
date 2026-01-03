# Phase 2 Testing Strategy

**Goal**: Ensure Phase 2 implementation (core game loop) works correctly with comprehensive testing coverage.

---

## Testing Pyramid

```
                 /\
                /  \
               / E2E \           10% - End-to-End Tests (critical user flows)
              /------\
             /        \
            /Integration\        30% - Integration Tests (backend + DB + content)
           /------------\
          /              \
         /  Unit Tests    \     60% - Unit Tests (formulas, logic, components)
        /------------------\
```

---

## 1. Unit Tests (60% coverage)

### 1.1 Rust Backend - Core Logic

**Location**: `crates/core/src/**/*.rs`

#### Formulas Module (`formulas.rs`)
```rust
// crates/core/src/gamification/formulas.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xp_calculation_base() {
        let xp = calculate_quiz_xp(Difficulty::Easy, 100.0, 0);
        assert_eq!(xp, 75); // 50 * 1.0 * 1.0 * 1.5
    }

    #[test]
    fn test_xp_with_streak() {
        let xp = calculate_quiz_xp(Difficulty::Medium, 90.0, 10);
        assert_eq!(xp, 117); // 50 * 1.5 * 1.2 * 1.3
    }

    #[test]
    fn test_level_calculation() {
        assert_eq!(calculate_level(0), 1);
        assert_eq!(calculate_level(283), 2);
        assert_eq!(calculate_level(3162), 10);
    }

    #[test]
    fn test_xp_to_next_level() {
        let (progress, total) = xp_to_next_level(100);
        assert_eq!(progress, 100);
        assert_eq!(total, 283);
    }

    #[test]
    fn test_mastery_update() {
        let new = update_mastery(0.0, 0.8);
        assert_eq!(new, 0.20); // 0.0 + 0.25 * (0.8 - 0.0)
        
        let new2 = update_mastery(0.20, 0.9);
        assert_eq!(new2, 0.375); // 0.20 + 0.25 * (0.9 - 0.20)
    }

    #[test]
    fn test_streak_multiplier() {
        assert_eq!(get_streak_multiplier(1), 1.0);
        assert_eq!(get_streak_multiplier(5), 1.1);
        assert_eq!(get_streak_multiplier(10), 1.2);
        assert_eq!(get_streak_multiplier(20), 1.3);
        assert_eq!(get_streak_multiplier(31), 1.5);
    }
}
```

#### Streak Logic (`streak.rs`)
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_streak_same_day() {
        let info = calculate_streak_info(last_activity_today(), 10);
        assert_eq!(info.current_streak, 10);
        assert!(!info.is_grace_period);
    }

    #[test]
    fn test_streak_next_day() {
        let info = calculate_streak_info(yesterday(), 10);
        assert_eq!(info.current_streak, 11);
    }

    #[test]
    fn test_streak_grace_period() {
        let info = calculate_streak_info(three_days_ago(), 10);
        assert_eq!(info.current_streak, 10);
        assert!(info.is_grace_period);
        assert_eq!(info.grace_days_remaining, 2);
    }

    #[test]
    fn test_streak_reset() {
        let info = calculate_streak_info(six_days_ago(), 10);
        assert_eq!(info.current_streak, 1);
        assert!(!info.is_grace_period);
    }
}
```

#### Quiz Grading (`quiz.rs`)
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_grade_perfect_score() {
        let quiz = create_sample_quiz();
        let answers = perfect_answers();
        let (score, correct, total) = grade_quiz(&quiz, &answers);
        assert_eq!(score, quiz.total_points());
        assert_eq!(correct, total);
    }

    #[test]
    fn test_grade_partial_score() {
        let quiz = create_sample_quiz();
        let mut answers = perfect_answers();
        answers.insert("q1".to_string(), "wrong".to_string());
        let (score, correct, total) = grade_quiz(&quiz, &answers);
        assert!(correct < total);
    }

    #[test]
    fn test_retake_xp_multiplier() {
        assert_eq!(get_retake_multiplier(1), 1.0);
        assert_eq!(get_retake_multiplier(2), 0.5);
        assert_eq!(get_retake_multiplier(3), 0.25);
        assert_eq!(get_retake_multiplier(4), 0.1);
    }
}
```

**Run**: `cargo test --lib`

---

### 1.2 Frontend - React Components

**Location**: `apps/desktop/src/components/**/*.test.tsx`

#### Component Tests (Vitest + React Testing Library)

```tsx
// apps/desktop/src/components/quiz/QuestionCard.test.tsx
import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect } from 'vitest'
import { QuestionCard } from './QuestionCard'

describe('QuestionCard', () => {
  it('renders question prompt', () => {
    render(<QuestionCard question={sampleQuestion} />)
    expect(screen.getByText('What is ownership?')).toBeInTheDocument()
  })

  it('allows answer selection', () => {
    const onSelect = vi.fn()
    render(<QuestionCard question={sampleQuestion} onAnswerChange={onSelect} />)
    
    fireEvent.click(screen.getByText('Option A'))
    expect(onSelect).toHaveBeenCalledWith('q1', 'a')
  })

  it('shows feedback in review mode', () => {
    render(
      <QuestionCard 
        question={sampleQuestion} 
        isReviewMode 
        feedback={correctFeedback}
      />
    )
    expect(screen.getByText('✓ Correct!')).toBeInTheDocument()
  })
})
```

```tsx
// apps/desktop/src/components/progress/XPProgress.test.tsx
describe('XPProgress', () => {
  it('calculates progress percentage correctly', () => {
    render(<XPProgress currentXP={150} currentLevel={1} xpForNextLevel={283} />)
    const progress = screen.getByRole('progressbar')
    expect(progress).toHaveAttribute('aria-valuenow', '53') // 150/283 ≈ 53%
  })

  it('shows level up when at threshold', () => {
    render(<XPProgress currentXP={283} currentLevel={2} xpForNextLevel={520} />)
    expect(screen.getByText('Level 2')).toBeInTheDocument()
  })
})
```

**Setup** (`apps/desktop/vitest.config.ts`):
```ts
import { defineConfig } from 'vitest/config'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  test: {
    environment: 'jsdom',
    setupFiles: './src/test/setup.ts',
    globals: true,
  },
})
```

**Run**: `npm test` or `npm run test:watch`

---

## 2. Integration Tests (30% coverage)

### 2.1 Backend Integration - Database + Logic

**Location**: `crates/core/tests/integration/**/*.rs`

```rust
// crates/core/tests/integration/lecture_flow.rs
use gamified_core::db::Connection;
use gamified_core::repos::*;

#[test]
fn test_complete_lecture_flow() {
    let conn = setup_test_db();
    
    // Start lecture
    let result = start_lecture(&conn, "user1", "lecture1").unwrap();
    assert!(result.is_ok());
    
    // Complete lecture
    let completion = complete_lecture(&conn, "user1", "lecture1", 600_000).unwrap();
    assert_eq!(completion.xp_earned, 25); // Base lecture XP
    
    // Check progress
    let progress = get_node_progress(&conn, "user1", "lecture1").unwrap();
    assert_eq!(progress.status, NodeStatus::Completed);
}

#[test]
fn test_quiz_with_mastery_update() {
    let conn = setup_test_db();
    
    // Submit quiz
    let mut answers = HashMap::new();
    answers.insert("q1".to_string(), "b".to_string());
    
    let result = submit_quiz(&conn, "user1", "quiz1", answers).unwrap();
    assert_eq!(result.score_percentage, 100.0);
    assert!(result.passed);
    
    // Check mastery updated
    let mastery = get_mastery_score(&conn, "user1", "ownership").unwrap();
    assert!(mastery > 0.0);
}

#[test]
fn test_unlock_logic() {
    let conn = setup_test_db();
    
    // Complete prerequisite
    complete_lecture(&conn, "user1", "lecture1").unwrap();
    
    // Check dependent unlocked
    let unlocked = get_unlocked_nodes(&conn, "user1").unwrap();
    assert!(unlocked.contains(&"quiz1".to_string()));
}
```

**Run**: `cargo test --test integration`

---

### 2.2 Content Loading Integration

```rust
// crates/content/tests/integration/loader.rs
#[test]
fn test_load_manifest() {
    let tree = ContentTree::load("./content/manifest.json").unwrap();
    assert!(tree.total_nodes() > 0);
}

#[test]
fn test_load_lecture() {
    let tree = ContentTree::load("./content/manifest.json").unwrap();
    let lecture = tree.get_lecture("week1-day1-intro").unwrap();
    assert_eq!(lecture.title, "Introduction to Rust");
}

#[test]
fn test_load_quiz_validates() {
    let tree = ContentTree::load("./content/manifest.json").unwrap();
    let quiz = tree.get_quiz("week1-day1-quiz").unwrap();
    
    // Validate structure
    assert!(quiz.questions.len() >= 3);
    assert!(quiz.passing_score >= 70);
    
    for question in &quiz.questions {
        assert!(question.options.len() >= 2);
        assert!(!question.explanation.is_empty());
    }
}
```

**Run**: `cargo test --test content_integration`

---

### 2.3 Tauri Commands Integration

```rust
// apps/desktop/src-tauri/tests/commands.rs
#[tokio::test]
async fn test_get_lecture_command() {
    let app = setup_test_app();
    
    let result = get_lecture("lecture1".to_string()).await;
    assert!(result.is_ok());
    
    let lecture = result.unwrap();
    assert!(!lecture.content.is_empty());
}

#[tokio::test]
async fn test_complete_lecture_updates_progress() {
    let app = setup_test_app();
    
    let result = complete_lecture(
        "user1".to_string(),
        "lecture1".to_string(),
        60_000,
        app.state()
    ).await;
    
    assert!(result.is_ok());
    let completion = result.unwrap();
    assert!(completion.xp_earned > 0);
}
```

**Run**: `cargo test --package gamified-desktop`

---

## 3. End-to-End Tests (10% coverage)

### 3.1 Playwright E2E Tests

**Location**: `apps/desktop/e2e/**/*.spec.ts`

```typescript
// apps/desktop/e2e/complete-session.spec.ts
import { test, expect } from '@playwright/test'

test.describe('Complete Session Flow', () => {
  test('user completes lecture → quiz → sees summary', async ({ page }) => {
    await page.goto('/')
    
    // Start session
    await page.click('button:has-text("Start Daily Session")')
    await expect(page).toHaveURL(/\/session\//)
    
    // Complete lecture
    await page.click('button:has-text("Start")')
    await page.evaluate(() => window.scrollTo(0, document.body.scrollHeight))
    await page.waitForTimeout(30000) // Min time requirement
    await page.click('button:has-text("Mark Complete")')
    
    // Verify XP modal
    await expect(page.locator('text=+25 XP')).toBeVisible()
    await page.click('button:has-text("Continue")')
    
    // Complete quiz
    await page.click('input[value="b"]') // Answer Q1
    await page.click('input[value="a"]') // Answer Q2
    await page.click('button:has-text("Submit")')
    
    // Verify results
    await expect(page.locator('text=Score:')).toBeVisible()
    await expect(page.locator('text=XP Earned:')).toBeVisible()
    
    // Session summary
    await expect(page.locator('text=Session Complete')).toBeVisible()
    await expect(page.locator('text=Streak: 1 day')).toBeVisible()
  })

  test('progress persists after app restart', async ({ page, context }) => {
    // Complete lecture
    await page.goto('/')
    await completeFirstLecture(page)
    
    // Close and reopen
    await page.close()
    const newPage = await context.newPage()
    await newPage.goto('/')
    
    // Verify lecture marked complete
    await newPage.goto('/progress')
    await expect(newPage.locator('text=Week 1 Day 1 Intro')).toHaveClass(/completed/)
  })
})
```

**Run**: `npm run test:e2e`

---

### 3.2 Critical User Flows

**Test Cases**:

1. **First Session Flow**
   - New user → Start session → Complete lecture → Complete quiz → See summary ✓

2. **Streak Maintenance**
   - Complete session day 1 → Complete session day 2 → Verify streak = 2 ✓

3. **Grace Period**
   - Complete session → Skip 3 days → Complete session → Verify streak maintained with warning ✓

4. **Retake Quiz**
   - Fail quiz (60%) → Retake → Pass (80%) → Verify reduced XP (50%) ✓

5. **Level Up**
   - Complete activities until XP threshold → Verify level up animation ✓

6. **Dashboard Accuracy**
   - Complete 5 activities → Check dashboard stats match DB ✓

---

## 4. Testing Strategy by Milestone

### Milestone 2.1: Lecture Viewer
- **Unit**: Markdown rendering, time tracking logic, scroll position calculation
- **Integration**: Complete lecture command → DB update → unlock next nodes
- **E2E**: Load lecture → scroll to bottom → mark complete → verify XP

### Milestone 2.2: Quiz System
- **Unit**: Grading logic, XP calculation, mastery update formula, retake multiplier
- **Integration**: Submit quiz → grade → update mastery → save attempt
- **E2E**: Complete quiz → see results → verify feedback → retake quiz

### Milestone 2.3: Progress Dashboard
- **Unit**: Level calculation, streak logic, chart data transformations
- **Integration**: Fetch dashboard stats → verify against DB
- **E2E**: Complete activities → refresh dashboard → verify real-time updates

### Milestone 2.4: Daily Session Queue
- **Unit**: Session planning algorithm, state machine transitions
- **Integration**: Create session → complete activities → generate summary
- **E2E**: Full session flow from start to summary modal

---

## 5. Test Data Setup

### 5.1 Test Database Schema

```sql
-- tests/fixtures/test_schema.sql
-- Minimal content for testing

INSERT INTO users (id, username, total_xp, current_level, current_streak) 
VALUES ('user1', 'testuser', 0, 1, 0);

INSERT INTO content_nodes (id, title, node_type, difficulty, xp_reward)
VALUES 
  ('lecture1', 'Intro Lecture', 'Lecture', 'Easy', 25),
  ('quiz1', 'Intro Quiz', 'Quiz', 'Easy', 50);
```

### 5.2 Test Fixtures

```rust
// tests/fixtures/mod.rs
pub fn create_sample_quiz() -> Quiz {
    Quiz {
        id: "test-quiz".to_string(),
        questions: vec![
            Question {
                id: "q1".to_string(),
                prompt: "What is 2+2?".to_string(),
                options: vec![
                    Option { id: "a", text: "3" },
                    Option { id: "b", text: "4" },
                ],
                correct_answer: "b".to_string(),
                explanation: "Basic math".to_string(),
                points: 1,
            }
        ],
        passing_score: 70,
        difficulty: Difficulty::Easy,
        skills: vec!["math".to_string()],
    }
}
```

---

## 6. Testing Tools & Setup

### 6.1 Rust Testing
```toml
# Cargo.toml
[dev-dependencies]
tokio = { version = "1", features = ["test-util"] }
mockall = "0.12"
tempfile = "3.8"
```

### 6.2 Frontend Testing
```json
{
  "devDependencies": {
    "vitest": "^1.0.0",
    "@testing-library/react": "^14.0.0",
    "@testing-library/user-event": "^14.5.0",
    "@playwright/test": "^1.40.0"
  }
}
```

### 6.3 Test Scripts
```json
{
  "scripts": {
    "test": "vitest",
    "test:watch": "vitest --watch",
    "test:e2e": "playwright test",
    "test:all": "cargo test && npm test && npm run test:e2e"
  }
}
```

---

## 7. Verification Checklist

### Before Merging Phase 2
- [ ] All unit tests pass (`cargo test --lib`)
- [ ] All integration tests pass (`cargo test --test`)
- [ ] All frontend tests pass (`npm test`)
- [ ] E2E critical flows pass (`npm run test:e2e`)
- [ ] Code coverage ≥60% (run `cargo tarpaulin`)
- [ ] No TypeScript errors (`npm run typecheck`)
- [ ] Linting passes (`npm run lint`)
- [ ] Manual smoke test of full session flow

### Manual Testing Checklist
- [ ] Complete lecture → quiz → summary flow works
- [ ] Progress persists after app restart
- [ ] Streak updates correctly
- [ ] Grace period warning appears
- [ ] Mastery radar chart updates
- [ ] Dashboard shows accurate stats
- [ ] Retake quiz shows reduced XP
- [ ] Level up animation triggers correctly
- [ ] All formulas match balance report values

---

## 8. CI/CD Integration

```yaml
# .github/workflows/test.yml
name: Tests
on: [push, pull_request]

jobs:
  rust-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
      - run: cargo test --all

  frontend-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
      - run: npm ci
      - run: npm test

  e2e-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
      - run: npm ci
      - run: npx playwright install
      - run: npm run test:e2e
```

---

## 9. Testing Anti-Patterns to Avoid

❌ **Don't**:
- Test implementation details (internal state)
- Mock everything (integration tests should use real DB)
- Write flaky E2E tests that depend on timing
- Skip edge cases (null values, empty arrays, etc.)
- Test multiple things in one test

✅ **Do**:
- Test behavior and outcomes
- Use in-memory DB for integration tests
- Use explicit waits in E2E tests
- Test happy path + error cases
- One assertion per test (or related assertions)

---

## 10. Success Criteria

**Phase 2 testing is complete when**:
1. ≥60% unit test coverage
2. ≥30% integration test coverage
3. All critical E2E flows pass
4. No regressions in core formulas (XP, level, mastery, streak)
5. Manual testing checklist completed
6. CI pipeline green

**Time Estimate**: 2-3 days of testing work distributed across implementation.
