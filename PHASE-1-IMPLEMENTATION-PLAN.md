# Phase 1 Implementation Plan: Foundation

**Status**: Ready for Implementation
**Estimated Duration**: 7-9 days
**Prerequisites**: Phase 0 complete (LLM grading, Docker runner, Gamification validated)

---

## Table of Contents

1. [Technology Stack Decisions](#technology-stack-decisions)
2. [Milestone 1.1: Data Schema](#milestone-11-data-schema)
3. [Milestone 1.2: Tauri Shell + Basic UI](#milestone-12-tauri-shell--basic-ui)
4. [Milestone 1.3: Content Loader](#milestone-13-content-loader)
5. [Project Structure](#project-structure)
6. [Potential Pitfalls](#potential-pitfalls)
7. [Testing Strategy](#testing-strategy)

---

## Technology Stack Decisions

### Database: SQLite with `rusqlite` (Recommended)

**Decision**: Use `rusqlite` over `sqlx`

**Rationale**:

| Aspect | rusqlite | sqlx |
|--------|----------|------|
| **Compile-time safety** | Runtime checks | Compile-time query validation |
| **Simplicity** | Simple, synchronous API | Async + complex macro setup |
| **Migration support** | Manual (simple) | Built-in (complex) |
| **Desktop app fit** | Excellent (embedded DB) | Overkill (designed for servers) |
| **Bundle size** | Smaller | Larger |
| **Learning curve** | Gentle | Steep |

**For this project**:
- Desktop app with embedded DB (not web server)
- Schema is relatively stable
- Simplicity > advanced features
- No need for async DB (I/O not bottleneck)
- Tauri apps benefit from smaller bundle size

**Winner**: `rusqlite` with manual migrations

**Dependencies to add**:
```toml
[dependencies]
rusqlite = { version = "0.30", features = ["bundled"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.6", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
sha2 = "0.10"  # For content hashing
```

---

### Frontend State Management: Zustand (Recommended)

**Decision**: Use Zustand over Context API or Redux

**Comparison**:

| Feature | Zustand | Context API | Redux |
|---------|---------|-------------|-------|
| **Boilerplate** | Minimal | Minimal | Heavy |
| **Learning curve** | Easy | Easy | Steep |
| **DevTools** | Yes | No | Yes |
| **Performance** | Excellent | Poor (re-renders) | Excellent |
| **TypeScript** | Excellent | Good | Good |
| **Middleware** | Built-in | Manual | Built-in |

**Why Zustand wins**:
1. **Minimal boilerplate** - No providers, actions, reducers
2. **Better than Context** - Avoids unnecessary re-renders
3. **Simpler than Redux** - No learning curve, direct mutations
4. **Perfect for Tauri** - Works seamlessly with Tauri commands
5. **TypeScript-first** - Great type inference

**Example store**:
```typescript
import create from 'zustand'
import { invoke } from '@tauri-apps/api/tauri'

interface User {
  id: string
  totalXp: number
  currentStreak: number
  level: number
}

interface AppState {
  user: User | null
  loading: boolean
  loadUser: () => Promise<void>
  updateXp: (xp: number) => void
}

export const useAppStore = create<AppState>((set) => ({
  user: null,
  loading: false,

  loadUser: async () => {
    set({ loading: true })
    const user = await invoke<User>('get_user_data')
    set({ user, loading: false })
  },

  updateXp: (xp: number) => {
    set((state) => ({
      user: state.user ? { ...state.user, totalXp: state.user.totalXp + xp } : null
    }))
  }
}))
```

**Winner**: Zustand

**Dependencies to add**:
```json
{
  "dependencies": {
    "zustand": "^4.4.7"
  }
}
```

---

### UI Component Library: Tailwind + shadcn/ui (Recommended)

**Decision**: Use Tailwind CSS + shadcn/ui components (not a library per se)

**Why NOT a full component library**:
- Material-UI, Ant Design, Chakra = Heavy, opinionated, hard to customize
- This app needs custom gamification UI (XP bars, skill trees, badges)
- Full libraries add bloat and lock you into their design system

**Why Tailwind + shadcn/ui**:
1. **Tailwind**: Utility-first, total design control, tiny bundle size
2. **shadcn/ui**: Copy-paste components (not a dependency), full control
3. **Radix UI primitives**: Accessible headless components for complex widgets
4. **Lucide icons**: Clean, modern icon set

**shadcn/ui Components to Use**:
- `button` - Primary buttons
- `card` - Content cards
- `badge` - XP badges, status indicators
- `progress` - XP/progress bars
- `dialog` - Modals for submissions
- `tabs` - Navigation between pages
- `tooltip` - Help tooltips

**Installation**:
```bash
# Initialize Tailwind
npm install -D tailwindcss postcss autoprefixer
npx tailwindcss init -p

# Install shadcn/ui (copies components, doesn't install library)
npx shadcn-ui@latest init

# Install icon library
npm install lucide-react
```

**Winner**: Tailwind + shadcn/ui (hybrid approach)

---

## Milestone 1.1: Data Schema

**Duration**: 2-3 days
**Deliverable**: Working database with CRUD operations

### Detailed Schema Design

#### 1. Schema SQL (with indexes and constraints)

```sql
-- /crates/core/src/db/schema.sql

-- Enable foreign keys
PRAGMA foreign_keys = ON;

-- ============================================================================
-- USERS TABLE
-- ============================================================================
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_activity TEXT NOT NULL DEFAULT (datetime('now')),
    total_xp INTEGER NOT NULL DEFAULT 0,
    current_level INTEGER NOT NULL DEFAULT 1,
    current_streak INTEGER NOT NULL DEFAULT 0,
    last_streak_date TEXT,  -- Last date streak was updated
    CHECK (total_xp >= 0),
    CHECK (current_level >= 1),
    CHECK (current_streak >= 0)
);

CREATE INDEX IF NOT EXISTS idx_users_last_activity ON users(last_activity);

-- ============================================================================
-- NODE PROGRESS TABLE
-- ============================================================================
CREATE TABLE IF NOT EXISTS node_progress (
    user_id TEXT NOT NULL,
    node_id TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'NotStarted',  -- NotStarted | InProgress | Completed | Failed
    attempts INTEGER NOT NULL DEFAULT 0,
    time_spent_mins INTEGER NOT NULL DEFAULT 0,
    first_started_at TEXT,
    completed_at TEXT,
    last_updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (user_id, node_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CHECK (status IN ('NotStarted', 'InProgress', 'Completed', 'Failed')),
    CHECK (attempts >= 0),
    CHECK (time_spent_mins >= 0)
);

CREATE INDEX IF NOT EXISTS idx_node_progress_user ON node_progress(user_id);
CREATE INDEX IF NOT EXISTS idx_node_progress_status ON node_progress(user_id, status);
CREATE INDEX IF NOT EXISTS idx_node_progress_completed ON node_progress(user_id, completed_at);

-- ============================================================================
-- MASTERY SCORES TABLE
-- ============================================================================
CREATE TABLE IF NOT EXISTS mastery_scores (
    user_id TEXT NOT NULL,
    skill_id TEXT NOT NULL,  -- e.g., "ownership", "lifetimes", "traits"
    score REAL NOT NULL DEFAULT 0.0,  -- 0.0 to 1.0
    last_updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (user_id, skill_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CHECK (score >= 0.0 AND score <= 1.0)
);

CREATE INDEX IF NOT EXISTS idx_mastery_user ON mastery_scores(user_id);
CREATE INDEX IF NOT EXISTS idx_mastery_skill ON mastery_scores(skill_id);

-- ============================================================================
-- BADGE PROGRESS TABLE
-- ============================================================================
CREATE TABLE IF NOT EXISTS badge_progress (
    user_id TEXT NOT NULL,
    badge_id TEXT NOT NULL,  -- e.g., "week_warrior", "level_5"
    current_value REAL NOT NULL DEFAULT 0.0,  -- Progress toward badge (e.g., 5/7 days)
    earned_at TEXT,  -- NULL if not earned, timestamp if earned
    PRIMARY KEY (user_id, badge_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CHECK (current_value >= 0.0)
);

CREATE INDEX IF NOT EXISTS idx_badge_earned ON badge_progress(user_id, earned_at);

-- ============================================================================
-- QUIZ ATTEMPTS TABLE
-- ============================================================================
CREATE TABLE IF NOT EXISTS quiz_attempts (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    quiz_id TEXT NOT NULL,
    node_id TEXT NOT NULL,  -- Link to content node
    answers_json TEXT NOT NULL,  -- JSON array of user answers
    score_percentage INTEGER NOT NULL,  -- 0-100
    xp_earned INTEGER NOT NULL DEFAULT 0,
    submitted_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CHECK (score_percentage >= 0 AND score_percentage <= 100),
    CHECK (xp_earned >= 0)
);

CREATE INDEX IF NOT EXISTS idx_quiz_user ON quiz_attempts(user_id);
CREATE INDEX IF NOT EXISTS idx_quiz_node ON quiz_attempts(node_id);
CREATE INDEX IF NOT EXISTS idx_quiz_submitted ON quiz_attempts(submitted_at);

-- ============================================================================
-- CHALLENGE ATTEMPTS TABLE
-- ============================================================================
CREATE TABLE IF NOT EXISTS challenge_attempts (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    challenge_id TEXT NOT NULL,
    node_id TEXT NOT NULL,
    code_hash TEXT NOT NULL,  -- SHA-256 of submitted code
    tests_passed INTEGER NOT NULL DEFAULT 0,
    tests_failed INTEGER NOT NULL DEFAULT 0,
    stdout TEXT,
    stderr TEXT,
    xp_earned INTEGER NOT NULL DEFAULT 0,
    submitted_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CHECK (tests_passed >= 0),
    CHECK (tests_failed >= 0),
    CHECK (xp_earned >= 0)
);

CREATE INDEX IF NOT EXISTS idx_challenge_user ON challenge_attempts(user_id);
CREATE INDEX IF NOT EXISTS idx_challenge_node ON challenge_attempts(node_id);
CREATE INDEX IF NOT EXISTS idx_challenge_hash ON challenge_attempts(code_hash);

-- ============================================================================
-- ARTIFACT SUBMISSIONS TABLE
-- ============================================================================
CREATE TABLE IF NOT EXISTS artifact_submissions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    checkpoint_id TEXT NOT NULL,
    artifact_type TEXT NOT NULL,  -- "README" | "DESIGN" | "BENCH" | "RUNBOOK" | "INVARIANTS"
    content_hash TEXT NOT NULL,  -- SHA-256 for caching
    grade_percentage INTEGER,  -- NULL if not graded yet, 0-100 when graded
    reasoning_json TEXT,  -- JSON with category scores + reasoning
    xp_earned INTEGER NOT NULL DEFAULT 0,
    submitted_at TEXT NOT NULL DEFAULT (datetime('now')),
    graded_at TEXT,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CHECK (artifact_type IN ('README', 'DESIGN', 'BENCH', 'RUNBOOK', 'INVARIANTS')),
    CHECK (grade_percentage IS NULL OR (grade_percentage >= 0 AND grade_percentage <= 100)),
    CHECK (xp_earned >= 0)
);

CREATE INDEX IF NOT EXISTS idx_artifact_user ON artifact_submissions(user_id);
CREATE INDEX IF NOT EXISTS idx_artifact_checkpoint ON artifact_submissions(checkpoint_id);
CREATE INDEX IF NOT EXISTS idx_artifact_hash ON artifact_submissions(content_hash);
CREATE INDEX IF NOT EXISTS idx_artifact_graded ON artifact_submissions(graded_at);

-- ============================================================================
-- REVIEW QUEUE TABLE (Spaced Repetition)
-- ============================================================================
CREATE TABLE IF NOT EXISTS review_items (
    user_id TEXT NOT NULL,
    quiz_id TEXT NOT NULL,
    due_date TEXT NOT NULL,  -- When this item is due for review
    ease_factor REAL NOT NULL DEFAULT 2.5,  -- SM-2 ease factor
    interval_days INTEGER NOT NULL DEFAULT 1,  -- Current interval
    repetitions INTEGER NOT NULL DEFAULT 0,  -- Number of successful reviews
    last_reviewed_at TEXT,
    PRIMARY KEY (user_id, quiz_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CHECK (ease_factor >= 1.3),
    CHECK (interval_days >= 1),
    CHECK (repetitions >= 0)
);

CREATE INDEX IF NOT EXISTS idx_review_due ON review_items(user_id, due_date);

-- ============================================================================
-- GRADE CACHE TABLE (LLM Grade Caching)
-- ============================================================================
CREATE TABLE IF NOT EXISTS grade_cache (
    content_hash TEXT PRIMARY KEY,  -- SHA-256 of artifact content
    artifact_type TEXT NOT NULL,
    grade_percentage INTEGER NOT NULL,
    reasoning_json TEXT NOT NULL,
    cached_at TEXT NOT NULL DEFAULT (datetime('now')),
    CHECK (artifact_type IN ('README', 'DESIGN', 'BENCH', 'RUNBOOK', 'INVARIANTS')),
    CHECK (grade_percentage >= 0 AND grade_percentage <= 100)
);

CREATE INDEX IF NOT EXISTS idx_cache_type ON grade_cache(artifact_type);
CREATE INDEX IF NOT EXISTS idx_cache_date ON grade_cache(cached_at);

-- ============================================================================
-- SESSION HISTORY TABLE (Daily activity tracking)
-- ============================================================================
CREATE TABLE IF NOT EXISTS session_history (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    started_at TEXT NOT NULL DEFAULT (datetime('now')),
    ended_at TEXT,
    total_xp_earned INTEGER NOT NULL DEFAULT 0,
    items_completed INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CHECK (total_xp_earned >= 0),
    CHECK (items_completed >= 0)
);

CREATE INDEX IF NOT EXISTS idx_session_user ON session_history(user_id);
CREATE INDEX IF NOT EXISTS idx_session_started ON session_history(started_at);
```

#### 2. Migration Strategy

**Approach**: Simple version-based migrations (good enough for desktop app)

```rust
// /crates/core/src/db/migrations.rs

use rusqlite::Connection;

pub const CURRENT_VERSION: i32 = 1;

pub fn run_migrations(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    // Get current version
    let version: i32 = conn
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap_or(0);

    if version < CURRENT_VERSION {
        println!("Running migrations from v{} to v{}", version, CURRENT_VERSION);

        // Run each migration in order
        if version < 1 {
            migrate_to_v1(conn)?;
        }

        // Future migrations go here
        // if version < 2 {
        //     migrate_to_v2(conn)?;
        // }

        // Update version
        conn.pragma_update(None, "user_version", CURRENT_VERSION)?;
        println!("Database now at version {}", CURRENT_VERSION);
    }

    Ok(())
}

fn migrate_to_v1(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    println!("  Running migration to v1 (initial schema)");

    // Read schema.sql and execute it
    let schema_sql = include_str!("schema.sql");
    conn.execute_batch(schema_sql)?;

    Ok(())
}

// Future migrations example:
// fn migrate_to_v2(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
//     println!("  Running migration to v2 (add new column)");
//     conn.execute("ALTER TABLE users ADD COLUMN email TEXT", [])?;
//     Ok(())
// }
```

**Why this approach**:
- Simple and transparent
- Version stored in SQLite's `user_version` pragma
- Each migration is a function (easy to test)
- Schema.sql is the source of truth for v1
- Future changes are incremental SQL commands

#### 3. Rust Models

```rust
// /crates/core/src/models/user.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub total_xp: i32,
    pub current_level: i32,
    pub current_streak: i32,
    pub last_streak_date: Option<DateTime<Utc>>,
}

impl User {
    pub fn new(id: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            created_at: now,
            last_activity: now,
            total_xp: 0,
            current_level: 1,
            current_streak: 0,
            last_streak_date: None,
        }
    }

    pub fn xp_for_next_level(&self) -> i32 {
        // Formula: 100 × N^1.5
        (100.0 * (self.current_level as f64 + 1.0).powf(1.5)) as i32
    }

    pub fn xp_progress_percentage(&self) -> f64 {
        let current_level_threshold = (100.0 * (self.current_level as f64).powf(1.5)) as i32;
        let next_level_threshold = self.xp_for_next_level();
        let xp_in_current_level = self.total_xp - current_level_threshold;
        let xp_needed_for_level = next_level_threshold - current_level_threshold;

        (xp_in_current_level as f64 / xp_needed_for_level as f64) * 100.0
    }
}
```

```rust
// /crates/core/src/models/progress.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeStatus {
    NotStarted,
    InProgress,
    Completed,
    Failed,
}

impl NodeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeStatus::NotStarted => "NotStarted",
            NodeStatus::InProgress => "InProgress",
            NodeStatus::Completed => "Completed",
            NodeStatus::Failed => "Failed",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "NotStarted" => Ok(NodeStatus::NotStarted),
            "InProgress" => Ok(NodeStatus::InProgress),
            "Completed" => Ok(NodeStatus::Completed),
            "Failed" => Ok(NodeStatus::Failed),
            _ => Err(format!("Invalid node status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeProgress {
    pub user_id: String,
    pub node_id: String,
    pub status: NodeStatus,
    pub attempts: i32,
    pub time_spent_mins: i32,
    pub first_started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub last_updated_at: DateTime<Utc>,
}
```

```rust
// /crates/core/src/models/mastery.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasteryScore {
    pub user_id: String,
    pub skill_id: String,  // "ownership", "lifetimes", "traits", etc.
    pub score: f64,  // 0.0 to 1.0
    pub last_updated_at: DateTime<Utc>,
}

impl MasteryScore {
    const LEARNING_RATE: f64 = 0.25;
    const DECAY_RATE: f64 = 0.05;  // 5% per day
    const GRACE_PERIOD_DAYS: i64 = 5;
    const MINIMUM_SCORE: f64 = 0.30;

    pub fn update_with_performance(&mut self, performance: f64) {
        // Exponential moving average: new = old + learning_rate × (performance - old)
        self.score = self.score + Self::LEARNING_RATE * (performance - self.score);
        self.score = self.score.clamp(0.0, 1.0);
        self.last_updated_at = Utc::now();
    }

    pub fn apply_decay(&mut self, days_inactive: i64) {
        if days_inactive <= Self::GRACE_PERIOD_DAYS {
            return; // No decay during grace period
        }

        let decay_days = days_inactive - Self::GRACE_PERIOD_DAYS;
        let decay_factor = (-Self::DECAY_RATE * decay_days as f64).exp();
        self.score = (self.score * decay_factor).max(Self::MINIMUM_SCORE);
    }
}
```

```rust
// /crates/core/src/models/quiz.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizAttempt {
    pub id: String,
    pub user_id: String,
    pub quiz_id: String,
    pub node_id: String,
    pub answers: Vec<String>,  // User's answers
    pub score_percentage: i32,
    pub xp_earned: i32,
    pub submitted_at: DateTime<Utc>,
}

impl QuizAttempt {
    pub fn new(user_id: String, quiz_id: String, node_id: String, answers: Vec<String>, score_percentage: i32, xp_earned: i32) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            quiz_id,
            node_id,
            answers,
            score_percentage,
            xp_earned,
            submitted_at: Utc::now(),
        }
    }
}
```

```rust
// /crates/core/src/models/badge.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BadgeProgress {
    pub user_id: String,
    pub badge_id: String,
    pub current_value: f64,
    pub earned_at: Option<DateTime<Utc>>,
}

impl BadgeProgress {
    pub fn is_earned(&self) -> bool {
        self.earned_at.is_some()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BadgeDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,  // Icon name or path
    pub threshold: f64,  // Value needed to unlock
    pub category: BadgeCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BadgeCategory {
    Streak,      // Week Warrior, Month Warrior
    Level,       // Level 5, Level 10, etc.
    XP,          // XP 1K, XP 5K, etc.
    Completion,  // Week 1 Complete, etc.
    Mastery,     // Ownership Master, etc.
}
```

#### 4. Database Connection & CRUD Operations

```rust
// /crates/core/src/db/connection.rs

use rusqlite::{Connection, Result};
use std::path::PathBuf;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        // Enable foreign keys
        conn.execute("PRAGMA foreign_keys = ON", [])?;

        // Run migrations
        crate::db::migrations::run_migrations(&conn)?;

        Ok(Self { conn })
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }
}

// Thread-safe wrapper for Tauri state
use std::sync::Mutex;

pub struct AppDatabase {
    pub db: Mutex<Database>,
}

impl AppDatabase {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        Ok(Self {
            db: Mutex::new(Database::new(db_path)?),
        })
    }
}
```

```rust
// /crates/core/src/db/user_repo.rs

use crate::models::user::User;
use chrono::{DateTime, Utc};
use rusqlite::{Connection, Result, params};

pub struct UserRepository;

impl UserRepository {
    pub fn create(conn: &Connection, user: &User) -> Result<()> {
        conn.execute(
            "INSERT INTO users (id, created_at, last_activity, total_xp, current_level, current_streak, last_streak_date)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                user.id,
                user.created_at.to_rfc3339(),
                user.last_activity.to_rfc3339(),
                user.total_xp,
                user.current_level,
                user.current_streak,
                user.last_streak_date.map(|d| d.to_rfc3339()),
            ],
        )?;
        Ok(())
    }

    pub fn get_by_id(conn: &Connection, user_id: &str) -> Result<Option<User>> {
        let mut stmt = conn.prepare(
            "SELECT id, created_at, last_activity, total_xp, current_level, current_streak, last_streak_date
             FROM users WHERE id = ?1"
        )?;

        let user = stmt.query_row(params![user_id], |row| {
            Ok(User {
                id: row.get(0)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(1)?)
                    .unwrap()
                    .with_timezone(&Utc),
                last_activity: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                    .unwrap()
                    .with_timezone(&Utc),
                total_xp: row.get(3)?,
                current_level: row.get(4)?,
                current_streak: row.get(5)?,
                last_streak_date: row.get::<_, Option<String>>(6)?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
            })
        }).optional()?;

        Ok(user)
    }

    pub fn update_xp(conn: &Connection, user_id: &str, xp_delta: i32) -> Result<()> {
        conn.execute(
            "UPDATE users SET total_xp = total_xp + ?1, last_activity = ?2 WHERE id = ?3",
            params![xp_delta, Utc::now().to_rfc3339(), user_id],
        )?;
        Ok(())
    }

    pub fn update_streak(conn: &Connection, user_id: &str, new_streak: i32, streak_date: DateTime<Utc>) -> Result<()> {
        conn.execute(
            "UPDATE users SET current_streak = ?1, last_streak_date = ?2, last_activity = ?3 WHERE id = ?4",
            params![new_streak, streak_date.to_rfc3339(), Utc::now().to_rfc3339(), user_id],
        )?;
        Ok(())
    }

    pub fn update_level(conn: &Connection, user_id: &str, new_level: i32) -> Result<()> {
        conn.execute(
            "UPDATE users SET current_level = ?1, last_activity = ?2 WHERE id = ?3",
            params![new_level, Utc::now().to_rfc3339(), user_id],
        )?;
        Ok(())
    }
}
```

#### 5. CRUD Operations Summary

**Users Table**:
- `create(user)` - Create new user
- `get_by_id(id)` - Get user by ID
- `update_xp(id, delta)` - Add XP (can be negative)
- `update_streak(id, streak, date)` - Update streak
- `update_level(id, level)` - Update level

**Node Progress Table**:
- `create_or_update(user_id, node_id, status)` - Upsert progress
- `get(user_id, node_id)` - Get single node progress
- `get_all_for_user(user_id)` - Get all progress for user
- `get_by_status(user_id, status)` - Get nodes by status (e.g., all completed)
- `mark_completed(user_id, node_id)` - Mark node as complete
- `increment_time(user_id, node_id, mins)` - Add time spent

**Mastery Scores Table**:
- `create_or_update(user_id, skill_id, score)` - Upsert mastery
- `get(user_id, skill_id)` - Get single skill mastery
- `get_all_for_user(user_id)` - Get all masteries
- `apply_decay_for_user(user_id)` - Apply decay to all skills

**Badge Progress Table**:
- `create_or_update(user_id, badge_id, value)` - Upsert badge progress
- `get_all_for_user(user_id)` - Get all badge progress
- `mark_earned(user_id, badge_id)` - Mark badge as earned
- `get_earned(user_id)` - Get only earned badges

**Quiz Attempts Table**:
- `create(attempt)` - Save quiz attempt
- `get_by_id(id)` - Get single attempt
- `get_for_quiz(user_id, quiz_id)` - Get all attempts for a quiz
- `get_recent(user_id, limit)` - Get N most recent attempts

**Challenge Attempts Table**:
- `create(attempt)` - Save challenge attempt
- `get_by_code_hash(hash)` - Check if code already submitted
- `get_best_for_challenge(user_id, challenge_id)` - Get best attempt

**Artifact Submissions Table**:
- `create(submission)` - Save artifact submission
- `get_by_hash(hash)` - Check cache for identical submission
- `update_grade(id, grade, reasoning)` - Update with LLM grade

**Review Items Table**:
- `create_or_update(item)` - Add/update review item
- `get_due(user_id, today)` - Get items due today
- `update_after_review(quiz_id, grade)` - Update SM-2 parameters

**Session History Table**:
- `create(session)` - Start new session
- `update_end(session_id, xp, items)` - End session
- `get_recent(user_id, days)` - Get recent sessions

### Time Estimate: 2-3 days

- **Day 1**: Write schema.sql, migrations.rs, connection.rs (4 hours)
- **Day 2**: Write all model structs + CRUD repositories (6 hours)
- **Day 3**: Write database tests, fix bugs (4 hours)

---

## Milestone 1.2: Tauri Shell + Basic UI

**Duration**: 3-4 days
**Deliverable**: Functioning Tauri app with navigation and Tauri commands

### Project Structure

```
/gamified-learning-platform/
├── apps/
│   └── desktop/
│       ├── src-tauri/
│       │   ├── src/
│       │   │   ├── main.rs
│       │   │   ├── commands/
│       │   │   │   ├── mod.rs
│       │   │   │   ├── user.rs
│       │   │   │   ├── progress.rs
│       │   │   │   ├── content.rs
│       │   │   │   └── session.rs
│       │   │   └── state.rs
│       │   ├── Cargo.toml
│       │   └── tauri.conf.json
│       ├── src/
│       │   ├── App.tsx
│       │   ├── main.tsx
│       │   ├── lib/
│       │   │   └── tauri.ts  // Typed Tauri command wrappers
│       │   ├── stores/
│       │   │   ├── appStore.ts
│       │   │   ├── userStore.ts
│       │   │   └── contentStore.ts
│       │   ├── pages/
│       │   │   ├── Home.tsx
│       │   │   ├── SkillTree.tsx
│       │   │   ├── Progress.tsx
│       │   │   └── Settings.tsx
│       │   ├── components/
│       │   │   ├── layout/
│       │   │   │   ├── Navigation.tsx
│       │   │   │   ├── StatusBar.tsx
│       │   │   │   └── Layout.tsx
│       │   │   ├── ui/
│       │   │   │   ├── XPBar.tsx
│       │   │   │   ├── StreakCounter.tsx
│       │   │   │   └── LevelBadge.tsx
│       │   │   └── ...
│       │   └── styles/
│       │       └── globals.css
│       ├── package.json
│       ├── tsconfig.json
│       ├── vite.config.ts
│       └── tailwind.config.js
├── crates/
│   ├── core/
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── models/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── user.rs
│   │   │   │   ├── progress.rs
│   │   │   │   ├── mastery.rs
│   │   │   │   ├── quiz.rs
│   │   │   │   ├── challenge.rs
│   │   │   │   └── badge.rs
│   │   │   └── db/
│   │   │       ├── mod.rs
│   │   │       ├── connection.rs
│   │   │       ├── migrations.rs
│   │   │       ├── schema.sql
│   │   │       └── repos/
│   │   │           ├── mod.rs
│   │   │           ├── user_repo.rs
│   │   │           ├── progress_repo.rs
│   │   │           └── ...
│   │   ├── Cargo.toml
│   │   └── tests/
│   └── content/
│       ├── src/
│       │   ├── lib.rs
│       │   ├── loader.rs
│       │   ├── manifest.rs
│       │   └── validator.rs
│       └── Cargo.toml
└── content/
    ├── manifest.json
    └── week1/
        └── ...
```

### Tauri Backend Setup

#### 1. Main Entry Point

```rust
// /apps/desktop/src-tauri/src/main.rs

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod state;

use state::AppState;
use std::path::PathBuf;

fn main() {
    // Determine database path (user's app data directory)
    let app_data_dir = tauri::api::path::app_data_dir(&tauri::Config::default())
        .expect("Failed to get app data directory");

    std::fs::create_dir_all(&app_data_dir).expect("Failed to create app data directory");

    let db_path = app_data_dir.join("gamified-learning.db");

    println!("Database path: {:?}", db_path);

    // Initialize database
    let db = core::db::connection::AppDatabase::new(db_path)
        .expect("Failed to initialize database");

    // Initialize app state
    let app_state = AppState::new(db);

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            // User commands
            commands::user::get_user_data,
            commands::user::create_default_user,
            commands::user::update_user_xp,

            // Progress commands
            commands::progress::get_node_progress,
            commands::progress::get_all_progress,
            commands::progress::mark_node_complete,

            // Content commands
            commands::content::get_content_tree,
            commands::content::get_node_by_id,

            // Session commands
            commands::session::start_session,
            commands::session::end_session,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

#### 2. App State

```rust
// /apps/desktop/src-tauri/src/state.rs

use core::db::connection::AppDatabase;
use std::sync::Mutex;

pub struct AppState {
    pub db: AppDatabase,
    pub current_user_id: Mutex<Option<String>>,
}

impl AppState {
    pub fn new(db: AppDatabase) -> Self {
        Self {
            db,
            current_user_id: Mutex::new(None),
        }
    }
}
```

#### 3. Tauri Commands

```rust
// /apps/desktop/src-tauri/src/commands/user.rs

use crate::state::AppState;
use core::db::repos::user_repo::UserRepository;
use core::models::user::User;
use tauri::State;
use uuid::Uuid;

#[derive(serde::Serialize)]
pub struct UserData {
    pub id: String,
    pub total_xp: i32,
    pub current_level: i32,
    pub current_streak: i32,
    pub xp_for_next_level: i32,
    pub xp_progress_percentage: f64,
}

impl From<User> for UserData {
    fn from(user: User) -> Self {
        Self {
            id: user.id.clone(),
            total_xp: user.total_xp,
            current_level: user.current_level,
            current_streak: user.current_streak,
            xp_for_next_level: user.xp_for_next_level(),
            xp_progress_percentage: user.xp_progress_percentage(),
        }
    }
}

#[tauri::command]
pub fn get_user_data(state: State<AppState>) -> Result<UserData, String> {
    let db = state.db.db.lock().map_err(|e| e.to_string())?;
    let conn = db.connection();

    // Get current user ID (for now, just use a default user)
    let user_id = state.current_user_id.lock()
        .map_err(|e| e.to_string())?
        .clone()
        .unwrap_or_else(|| "default-user".to_string());

    // Get user from database
    let user = UserRepository::get_by_id(conn, &user_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "User not found".to_string())?;

    Ok(user.into())
}

#[tauri::command]
pub fn create_default_user(state: State<AppState>) -> Result<UserData, String> {
    let db = state.db.db.lock().map_err(|e| e.to_string())?;
    let conn = db.connection();

    let user_id = Uuid::new_v4().to_string();
    let user = User::new(user_id.clone());

    UserRepository::create(conn, &user).map_err(|e| e.to_string())?;

    // Set as current user
    *state.current_user_id.lock().map_err(|e| e.to_string())? = Some(user_id);

    Ok(user.into())
}

#[tauri::command]
pub fn update_user_xp(state: State<AppState>, xp_delta: i32) -> Result<UserData, String> {
    let db = state.db.db.lock().map_err(|e| e.to_string())?;
    let conn = db.connection();

    let user_id = state.current_user_id.lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or_else(|| "No user logged in".to_string())?;

    UserRepository::update_xp(conn, &user_id, xp_delta).map_err(|e| e.to_string())?;

    // Get updated user
    let user = UserRepository::get_by_id(conn, &user_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "User not found".to_string())?;

    Ok(user.into())
}
```

```rust
// /apps/desktop/src-tauri/src/commands/progress.rs

use crate::state::AppState;
use core::db::repos::progress_repo::ProgressRepository;
use core::models::progress::{NodeProgress, NodeStatus};
use tauri::State;

#[tauri::command]
pub fn get_node_progress(state: State<AppState>, node_id: String) -> Result<Option<NodeProgress>, String> {
    let db = state.db.db.lock().map_err(|e| e.to_string())?;
    let conn = db.connection();

    let user_id = state.current_user_id.lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or_else(|| "No user logged in".to_string())?;

    ProgressRepository::get(conn, &user_id, &node_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_all_progress(state: State<AppState>) -> Result<Vec<NodeProgress>, String> {
    let db = state.db.db.lock().map_err(|e| e.to_string())?;
    let conn = db.connection();

    let user_id = state.current_user_id.lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or_else(|| "No user logged in".to_string())?;

    ProgressRepository::get_all_for_user(conn, &user_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn mark_node_complete(state: State<AppState>, node_id: String) -> Result<(), String> {
    let db = state.db.db.lock().map_err(|e| e.to_string())?;
    let conn = db.connection();

    let user_id = state.current_user_id.lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or_else(|| "No user logged in".to_string())?;

    ProgressRepository::mark_completed(conn, &user_id, &node_id)
        .map_err(|e| e.to_string())
}
```

### Frontend Setup

#### 1. Zustand Stores

```typescript
// /apps/desktop/src/stores/userStore.ts

import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/tauri'

interface UserData {
  id: string
  totalXp: number
  currentLevel: number
  currentStreak: number
  xpForNextLevel: number
  xpProgressPercentage: number
}

interface UserState {
  user: UserData | null
  loading: boolean
  error: string | null

  loadUser: () => Promise<void>
  createUser: () => Promise<void>
  addXp: (xp: number) => Promise<void>
}

export const useUserStore = create<UserState>((set, get) => ({
  user: null,
  loading: false,
  error: null,

  loadUser: async () => {
    set({ loading: true, error: null })
    try {
      const user = await invoke<UserData>('get_user_data')
      set({ user, loading: false })
    } catch (error) {
      // User doesn't exist, create default user
      await get().createUser()
    }
  },

  createUser: async () => {
    set({ loading: true, error: null })
    try {
      const user = await invoke<UserData>('create_default_user')
      set({ user, loading: false })
    } catch (error) {
      set({ error: String(error), loading: false })
    }
  },

  addXp: async (xp: number) => {
    try {
      const user = await invoke<UserData>('update_user_xp', { xpDelta: xp })
      set({ user })
    } catch (error) {
      set({ error: String(error) })
    }
  },
}))
```

```typescript
// /apps/desktop/src/stores/contentStore.ts

import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/tauri'

interface ContentNode {
  id: string
  title: string
  type: 'lecture' | 'quiz' | 'challenge' | 'checkpoint'
  difficulty: string
  prerequisites: string[]
  skills: string[]
}

interface ContentTree {
  weeks: Week[]
}

interface Week {
  id: string
  title: string
  days: Day[]
}

interface Day {
  id: string
  title: string
  nodes: ContentNode[]
}

interface ContentState {
  tree: ContentTree | null
  loading: boolean
  error: string | null

  loadContentTree: () => Promise<void>
  getNodeById: (nodeId: string) => Promise<ContentNode | null>
}

export const useContentStore = create<ContentState>((set) => ({
  tree: null,
  loading: false,
  error: null,

  loadContentTree: async () => {
    set({ loading: true, error: null })
    try {
      const tree = await invoke<ContentTree>('get_content_tree')
      set({ tree, loading: false })
    } catch (error) {
      set({ error: String(error), loading: false })
    }
  },

  getNodeById: async (nodeId: string) => {
    try {
      return await invoke<ContentNode>('get_node_by_id', { nodeId })
    } catch (error) {
      set({ error: String(error) })
      return null
    }
  },
}))
```

#### 2. UI Components

```tsx
// /apps/desktop/src/components/layout/Navigation.tsx

import { Link, useLocation } from 'react-router-dom'
import { Home, TreePine, TrendingUp, Settings } from 'lucide-react'

export function Navigation() {
  const location = useLocation()

  const links = [
    { path: '/', label: 'Home', icon: Home },
    { path: '/skill-tree', label: 'Skill Tree', icon: TreePine },
    { path: '/progress', label: 'Progress', icon: TrendingUp },
    { path: '/settings', label: 'Settings', icon: Settings },
  ]

  return (
    <nav className="flex items-center gap-2 p-4 border-b">
      {links.map(({ path, label, icon: Icon }) => (
        <Link
          key={path}
          to={path}
          className={`flex items-center gap-2 px-4 py-2 rounded-lg transition-colors ${
            location.pathname === path
              ? 'bg-primary text-primary-foreground'
              : 'hover:bg-muted'
          }`}
        >
          <Icon size={20} />
          <span>{label}</span>
        </Link>
      ))}
    </nav>
  )
}
```

```tsx
// /apps/desktop/src/components/layout/StatusBar.tsx

import { useUserStore } from '@/stores/userStore'
import { Flame, Zap } from 'lucide-react'

export function StatusBar() {
  const user = useUserStore((state) => state.user)

  if (!user) return null

  return (
    <div className="flex items-center justify-between px-4 py-2 bg-muted">
      <div className="flex items-center gap-4">
        <div className="flex items-center gap-2">
          <Zap size={16} className="text-yellow-500" />
          <span className="text-sm font-medium">Level {user.currentLevel}</span>
        </div>

        <div className="flex items-center gap-2">
          <Flame size={16} className="text-orange-500" />
          <span className="text-sm font-medium">{user.currentStreak} day streak</span>
        </div>
      </div>

      <div className="flex items-center gap-2">
        <span className="text-sm text-muted-foreground">
          {user.totalXp.toLocaleString()} XP
        </span>
      </div>
    </div>
  )
}
```

```tsx
// /apps/desktop/src/components/ui/XPBar.tsx

import { useUserStore } from '@/stores/userStore'
import { Progress } from '@/components/ui/progress'  // shadcn/ui component

export function XPBar() {
  const user = useUserStore((state) => state.user)

  if (!user) return null

  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between text-sm">
        <span className="font-medium">Level {user.currentLevel}</span>
        <span className="text-muted-foreground">
          {user.totalXp.toLocaleString()} / {user.xpForNextLevel.toLocaleString()} XP
        </span>
      </div>

      <Progress value={user.xpProgressPercentage} className="h-3" />

      <p className="text-xs text-muted-foreground text-center">
        {Math.round(user.xpProgressPercentage)}% to Level {user.currentLevel + 1}
      </p>
    </div>
  )
}
```

```tsx
// /apps/desktop/src/pages/Home.tsx

import { useEffect } from 'react'
import { useUserStore } from '@/stores/userStore'
import { XPBar } from '@/components/ui/XPBar'
import { Button } from '@/components/ui/button'

export function Home() {
  const { user, loadUser, loading } = useUserStore()

  useEffect(() => {
    loadUser()
  }, [loadUser])

  if (loading) {
    return <div className="p-8">Loading...</div>
  }

  return (
    <div className="p-8 max-w-4xl mx-auto space-y-8">
      <div>
        <h1 className="text-4xl font-bold mb-2">Welcome back!</h1>
        <p className="text-muted-foreground">Ready to continue your Rust journey?</p>
      </div>

      <XPBar />

      <div className="grid gap-4">
        <div className="p-6 border rounded-lg">
          <h2 className="text-2xl font-semibold mb-4">Today's Session</h2>
          <Button size="lg" className="w-full">
            Start Daily Session
          </Button>
        </div>

        <div className="p-6 border rounded-lg">
          <h2 className="text-2xl font-semibold mb-4">Continue Learning</h2>
          <p className="text-muted-foreground mb-4">Pick up where you left off</p>
          <Button variant="outline" className="w-full">
            View Skill Tree
          </Button>
        </div>
      </div>
    </div>
  )
}
```

```tsx
// /apps/desktop/src/App.tsx

import { BrowserRouter, Routes, Route } from 'react-router-dom'
import { Navigation } from '@/components/layout/Navigation'
import { StatusBar } from '@/components/layout/StatusBar'
import { Home } from '@/pages/Home'
import { SkillTree } from '@/pages/SkillTree'
import { Progress } from '@/pages/Progress'
import { Settings } from '@/pages/Settings'

export function App() {
  return (
    <BrowserRouter>
      <div className="flex flex-col h-screen">
        <Navigation />

        <main className="flex-1 overflow-y-auto">
          <Routes>
            <Route path="/" element={<Home />} />
            <Route path="/skill-tree" element={<SkillTree />} />
            <Route path="/progress" element={<Progress />} />
            <Route path="/settings" element={<Settings />} />
          </Routes>
        </main>

        <StatusBar />
      </div>
    </BrowserRouter>
  )
}
```

### Time Estimate: 3-4 days

- **Day 1**: Initialize Tauri app, set up basic structure (4 hours)
- **Day 2**: Write Tauri commands + state management (5 hours)
- **Day 3**: Build frontend components + stores (6 hours)
- **Day 4**: Wire everything together, test end-to-end (5 hours)

---

## Milestone 1.3: Content Loader

**Duration**: 2 days
**Deliverable**: Content loader that parses manifest and builds content tree

### Manifest Schema Design

```json
// /content/manifest.json

{
  "version": "1.0",
  "title": "14-Week Rust Bootcamp",
  "description": "From zero to production-ready Rust developer",
  "author": "Your Name",
  "created_at": "2026-01-03",
  "weeks": [
    {
      "id": "week1",
      "title": "Week 1: Foundations",
      "description": "Hello World, ownership basics, borrowing",
      "days": [
        {
          "id": "week1-day1",
          "title": "Day 1: Hello Rust",
          "description": "Getting started with Rust",
          "nodes": [
            {
              "id": "week1-day1-lecture",
              "type": "lecture",
              "title": "Introduction to Rust",
              "description": "Learn the basics of Rust syntax",
              "difficulty": "easy",
              "estimated_minutes": 20,
              "xp_reward": 25,
              "content_path": "week1/day1/lecture.md",
              "skills": ["syntax", "tooling"],
              "prerequisites": []
            },
            {
              "id": "week1-day1-quiz",
              "type": "quiz",
              "title": "Syntax Basics Quiz",
              "description": "Test your understanding of Rust syntax",
              "difficulty": "easy",
              "estimated_minutes": 10,
              "xp_reward": 50,
              "content_path": "week1/day1/quiz.json",
              "skills": ["syntax"],
              "prerequisites": ["week1-day1-lecture"]
            },
            {
              "id": "week1-day1-challenge",
              "type": "mini-challenge",
              "title": "Hello Fibonacci",
              "description": "Write a function to calculate Fibonacci numbers",
              "difficulty": "medium",
              "estimated_minutes": 30,
              "xp_reward": 100,
              "content_path": "week1/day1/challenge.json",
              "skills": ["syntax", "functions"],
              "prerequisites": ["week1-day1-quiz"]
            }
          ]
        }
      ]
    }
  ],
  "checkpoints": [
    {
      "id": "checkpoint1",
      "title": "Checkpoint 1: Simple CLI Tool",
      "description": "Build a command-line calculator",
      "week": "week2",
      "day": "week2-day7",
      "difficulty": "hard",
      "estimated_hours": 4,
      "xp_reward": 200,
      "artifacts": [
        "README.md",
        "DESIGN.md"
      ],
      "prerequisites": ["week2-day6-challenge"],
      "rubrics": {
        "README": "rubrics/readme_rubric.json",
        "DESIGN": "rubrics/design_rubric.json"
      }
    }
  ],
  "skills": [
    {
      "id": "syntax",
      "name": "Rust Syntax",
      "description": "Basic Rust language syntax"
    },
    {
      "id": "ownership",
      "name": "Ownership",
      "description": "Understanding ownership and borrowing"
    },
    {
      "id": "lifetimes",
      "name": "Lifetimes",
      "description": "Lifetime annotations and elision"
    },
    {
      "id": "traits",
      "name": "Traits",
      "description": "Trait definitions and implementations"
    },
    {
      "id": "error-handling",
      "name": "Error Handling",
      "description": "Result, Option, and error propagation"
    }
  ]
}
```

### Content Directory Structure

```
/content/
├── manifest.json
├── rubrics/
│   ├── readme_rubric.json
│   ├── design_rubric.json
│   ├── bench_rubric.json
│   ├── runbook_rubric.json
│   └── invariants_rubric.json
├── week1/
│   ├── day1/
│   │   ├── lecture.md
│   │   ├── quiz.json
│   │   └── challenge.json
│   ├── day2/
│   │   ├── lecture.md
│   │   ├── quiz.json
│   │   └── challenge.json
│   └── ...
├── week2/
│   └── ...
└── ...
```

### Quiz JSON Format

```json
// /content/week1/day1/quiz.json

{
  "id": "week1-day1-quiz",
  "title": "Syntax Basics Quiz",
  "questions": [
    {
      "id": "q1",
      "question": "What keyword is used to declare a mutable variable in Rust?",
      "type": "multiple-choice",
      "options": [
        "var",
        "let",
        "let mut",
        "mut"
      ],
      "correct_answer": 2,
      "explanation": "In Rust, `let mut` is used to declare a mutable variable. Variables are immutable by default.",
      "skills": ["syntax"]
    },
    {
      "id": "q2",
      "question": "Which of the following are valid integer types in Rust?",
      "type": "multiple-select",
      "options": [
        "i32",
        "u64",
        "int",
        "i128"
      ],
      "correct_answers": [0, 1, 3],
      "explanation": "Rust has signed (i8, i16, i32, i64, i128) and unsigned (u8, u16, u32, u64, u128) integer types. There is no generic 'int' type.",
      "skills": ["syntax"]
    }
  ]
}
```

### Challenge JSON Format

```json
// /content/week1/day1/challenge.json

{
  "id": "week1-day1-challenge",
  "title": "Hello Fibonacci",
  "description": "Implement a function to calculate Fibonacci numbers",
  "instructions": "Complete the `fibonacci` function to return the nth Fibonacci number. The sequence starts with 0 and 1.",
  "starter_code": "pub fn fibonacci(n: u32) -> u64 {\n    // TODO: Implement this\n    todo!()\n}\n",
  "test_code": "#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn test_fibonacci_base_cases() {\n        assert_eq!(fibonacci(0), 0);\n        assert_eq!(fibonacci(1), 1);\n    }\n\n    #[test]\n    fn test_fibonacci_sequence() {\n        assert_eq!(fibonacci(5), 5);\n        assert_eq!(fibonacci(10), 55);\n    }\n}\n",
  "solution": "pub fn fibonacci(n: u32) -> u64 {\n    match n {\n        0 => 0,\n        1 => 1,\n        _ => fibonacci(n - 1) + fibonacci(n - 2),\n    }\n}\n",
  "hints": [
    "Think about the base cases: what are fibonacci(0) and fibonacci(1)?",
    "Use pattern matching with the `match` keyword",
    "Recursive solution is simplest for this challenge"
  ],
  "difficulty": "medium",
  "skills": ["syntax", "functions", "recursion"]
}
```

### Content Loader Implementation

```rust
// /crates/content/src/manifest.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub version: String,
    pub title: String,
    pub description: String,
    pub author: String,
    pub created_at: String,
    pub weeks: Vec<Week>,
    pub checkpoints: Vec<Checkpoint>,
    pub skills: Vec<Skill>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Week {
    pub id: String,
    pub title: String,
    pub description: String,
    pub days: Vec<Day>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Day {
    pub id: String,
    pub title: String,
    pub description: String,
    pub nodes: Vec<ContentNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentNode {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,  // "lecture" | "quiz" | "mini-challenge" | "checkpoint"
    pub title: String,
    pub description: String,
    pub difficulty: String,  // "easy" | "medium" | "hard" | "very-hard"
    pub estimated_minutes: u32,
    pub xp_reward: u32,
    pub content_path: String,
    pub skills: Vec<String>,
    pub prerequisites: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: String,
    pub title: String,
    pub description: String,
    pub week: String,
    pub day: String,
    pub difficulty: String,
    pub estimated_hours: u32,
    pub xp_reward: u32,
    pub artifacts: Vec<String>,
    pub prerequisites: Vec<String>,
    pub rubrics: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quiz {
    pub id: String,
    pub title: String,
    pub questions: Vec<Question>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    pub id: String,
    pub question: String,
    #[serde(rename = "type")]
    pub question_type: String,  // "multiple-choice" | "multiple-select" | "true-false"
    pub options: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correct_answer: Option<usize>,  // For single-choice
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correct_answers: Option<Vec<usize>>,  // For multi-select
    pub explanation: String,
    pub skills: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Challenge {
    pub id: String,
    pub title: String,
    pub description: String,
    pub instructions: String,
    pub starter_code: String,
    pub test_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub solution: Option<String>,  // Optional, for reference
    pub hints: Vec<String>,
    pub difficulty: String,
    pub skills: Vec<String>,
}
```

```rust
// /crates/content/src/loader.rs

use crate::manifest::{Manifest, Quiz, Challenge};
use std::fs;
use std::path::{Path, PathBuf};

pub struct ContentLoader {
    content_dir: PathBuf,
    manifest: Manifest,
}

impl ContentLoader {
    pub fn new(content_dir: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let manifest_path = content_dir.join("manifest.json");

        if !manifest_path.exists() {
            return Err(format!("Manifest not found at {:?}", manifest_path).into());
        }

        let manifest_json = fs::read_to_string(&manifest_path)?;
        let manifest: Manifest = serde_json::from_str(&manifest_json)?;

        Ok(Self {
            content_dir,
            manifest,
        })
    }

    pub fn get_manifest(&self) -> &Manifest {
        &self.manifest
    }

    pub fn load_lecture(&self, content_path: &str) -> Result<String, Box<dyn std::error::Error>> {
        let path = self.content_dir.join(content_path);

        if !path.exists() {
            return Err(format!("Lecture not found at {:?}", path).into());
        }

        let content = fs::read_to_string(&path)?;
        Ok(content)
    }

    pub fn load_quiz(&self, content_path: &str) -> Result<Quiz, Box<dyn std::error::Error>> {
        let path = self.content_dir.join(content_path);

        if !path.exists() {
            return Err(format!("Quiz not found at {:?}", path).into());
        }

        let quiz_json = fs::read_to_string(&path)?;
        let quiz: Quiz = serde_json::from_str(&quiz_json)?;
        Ok(quiz)
    }

    pub fn load_challenge(&self, content_path: &str) -> Result<Challenge, Box<dyn std::error::Error>> {
        let path = self.content_dir.join(content_path);

        if !path.exists() {
            return Err(format!("Challenge not found at {:?}", path).into());
        }

        let challenge_json = fs::read_to_string(&path)?;
        let challenge: Challenge = serde_json::from_str(&challenge_json)?;
        Ok(challenge)
    }

    pub fn validate(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut errors = Vec::new();

        // Validate all content paths exist
        for week in &self.manifest.weeks {
            for day in &week.days {
                for node in &day.nodes {
                    let path = self.content_dir.join(&node.content_path);
                    if !path.exists() {
                        errors.push(format!("Missing content file: {}", node.content_path));
                    }
                }
            }
        }

        // Validate rubrics exist
        for checkpoint in &self.manifest.checkpoints {
            for (artifact_type, rubric_path) in &checkpoint.rubrics {
                let path = self.content_dir.join(rubric_path);
                if !path.exists() {
                    errors.push(format!("Missing rubric for {}: {}", artifact_type, rubric_path));
                }
            }
        }

        if errors.is_empty() {
            Ok(vec!["All content files validated successfully".to_string()])
        } else {
            Err(format!("Validation errors:\n{}", errors.join("\n")).into())
        }
    }
}
```

```rust
// /crates/content/src/validator.rs

use crate::manifest::{Manifest, ContentNode};
use std::collections::{HashMap, HashSet};

pub struct ContentValidator;

impl ContentValidator {
    pub fn validate_manifest(manifest: &Manifest) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Validate prerequisite chains
        let all_node_ids: HashSet<String> = manifest.weeks.iter()
            .flat_map(|w| &w.days)
            .flat_map(|d| &d.nodes)
            .map(|n| n.id.clone())
            .collect();

        for week in &manifest.weeks {
            for day in &week.days {
                for node in &day.nodes {
                    for prereq in &node.prerequisites {
                        if !all_node_ids.contains(prereq) {
                            errors.push(format!("Node '{}' has invalid prerequisite '{}'", node.id, prereq));
                        }
                    }
                }
            }
        }

        // Validate skill references
        let all_skill_ids: HashSet<String> = manifest.skills.iter()
            .map(|s| s.id.clone())
            .collect();

        for week in &manifest.weeks {
            for day in &week.days {
                for node in &day.nodes {
                    for skill in &node.skills {
                        if !all_skill_ids.contains(skill) {
                            errors.push(format!("Node '{}' references unknown skill '{}'", node.id, skill));
                        }
                    }
                }
            }
        }

        // Validate difficulty values
        let valid_difficulties = ["easy", "medium", "hard", "very-hard"];
        for week in &manifest.weeks {
            for day in &week.days {
                for node in &day.nodes {
                    if !valid_difficulties.contains(&node.difficulty.as_str()) {
                        errors.push(format!("Node '{}' has invalid difficulty '{}'", node.id, node.difficulty));
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
```

### Dummy Content for Testing

Create one week of minimal content:

```markdown
<!-- /content/week1/day1/lecture.md -->

# Introduction to Rust

Welcome to your first Rust lesson!

## What is Rust?

Rust is a systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety.

## Key Features

- **Memory safety** without garbage collection
- **Zero-cost abstractions**
- **Fearless concurrency**
- **Great tooling** (cargo, rustfmt, clippy)

## Hello World

```rust
fn main() {
    println!("Hello, world!");
}
```

## Next Steps

Complete the quiz to test your understanding!
```

### Time Estimate: 2 days

- **Day 1**: Write manifest schema, loader, validator (5 hours)
- **Day 2**: Create dummy content, test loading, fix bugs (4 hours)

---

## Project Structure

```
/gamified-learning-platform/
├── apps/
│   └── desktop/              # Tauri desktop app
│       ├── src-tauri/        # Rust backend
│       │   ├── src/
│       │   │   ├── main.rs
│       │   │   ├── commands/
│       │   │   └── state.rs
│       │   └── Cargo.toml
│       └── src/              # React frontend
│           ├── App.tsx
│           ├── stores/
│           ├── pages/
│           └── components/
├── crates/
│   ├── core/                 # Core business logic + DB
│   │   ├── src/
│   │   │   ├── models/
│   │   │   └── db/
│   │   └── Cargo.toml
│   └── content/              # Content loading
│       ├── src/
│       │   ├── loader.rs
│       │   ├── manifest.rs
│       │   └── validator.rs
│       └── Cargo.toml
├── content/                  # Curriculum content
│   ├── manifest.json
│   ├── rubrics/
│   └── week1/
├── prototypes/               # Phase 0 prototypes
│   ├── llm-grading/
│   ├── docker-runner/
│   └── gamification/
└── README.md
```

---

## Potential Pitfalls

### 1. Database Locking Issues

**Problem**: SQLite locks the entire database during writes. If Tauri commands try to access DB simultaneously, you'll get `database is locked` errors.

**Solution**:
- Use `Mutex` around database (already in design)
- Keep transactions short
- Don't hold locks across async boundaries
- Consider write-ahead logging (WAL mode):
  ```rust
  conn.pragma_update(None, "journal_mode", "WAL")?;
  ```

### 2. DateTime Serialization

**Problem**: SQLite doesn't have native datetime types. Storing as TEXT requires careful parsing.

**Solution**:
- Always use RFC3339 format (`to_rfc3339()`)
- Always parse with error handling
- Consider helper functions:
  ```rust
  fn datetime_to_sql(dt: DateTime<Utc>) -> String {
      dt.to_rfc3339()
  }

  fn datetime_from_sql(s: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
      DateTime::parse_from_rfc3339(s)
          .map(|dt| dt.with_timezone(&Utc))
  }
  ```

### 3. Tauri Command Error Handling

**Problem**: Errors from Rust must be serializable to JSON. `Box<dyn Error>` won't work.

**Solution**:
- Return `Result<T, String>` from commands
- Map all errors to strings: `.map_err(|e| e.to_string())?`
- For better errors, create custom error enum with serde

### 4. Frontend State Synchronization

**Problem**: User adds XP in one component, but status bar doesn't update.

**Solution**:
- Use Zustand's subscriptions
- After any mutation, re-fetch or update state
- Example:
  ```typescript
  addXp: async (xp: number) => {
    const user = await invoke<UserData>('update_user_xp', { xpDelta: xp })
    set({ user })  // This triggers re-render in all subscribers
  }
  ```

### 5. Content Path Resolution

**Problem**: Content paths in manifest are relative, but loader needs absolute paths.

**Solution**:
- Store `content_dir` as `PathBuf` in loader
- Always join: `self.content_dir.join(content_path)`
- Validate paths exist during `validate()`

### 6. Missing Content Files

**Problem**: Manifest references file that doesn't exist → app crashes.

**Solution**:
- Run `validate()` on app startup
- Show clear error if validation fails
- Graceful degradation: skip missing nodes, log warning

### 7. Circular Prerequisites

**Problem**: Node A depends on Node B, which depends on Node A.

**Solution**:
- Implement cycle detection in validator
- Use graph traversal (DFS with visited set)
- Reject manifests with cycles

### 8. Schema Migration During Development

**Problem**: You change schema, but existing DB has old schema → migration fails.

**Solution**:
- During Phase 1, just delete DB and recreate
- After Phase 1, write proper incremental migrations
- Store user_version, check on startup

---

## Testing Strategy

### Unit Tests

```rust
// /crates/core/tests/db_tests.rs

#[cfg(test)]
mod tests {
    use core::db::connection::Database;
    use core::db::repos::user_repo::UserRepository;
    use core::models::user::User;
    use tempfile::tempdir;

    #[test]
    fn test_create_and_get_user() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::new(db_path).unwrap();
        let conn = db.connection();

        let user = User::new("test-user".to_string());
        UserRepository::create(conn, &user).unwrap();

        let retrieved = UserRepository::get_by_id(conn, "test-user").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, "test-user");
    }

    #[test]
    fn test_update_xp() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::new(db_path).unwrap();
        let conn = db.connection();

        let user = User::new("test-user".to_string());
        UserRepository::create(conn, &user).unwrap();

        UserRepository::update_xp(conn, "test-user", 100).unwrap();

        let updated = UserRepository::get_by_id(conn, "test-user").unwrap().unwrap();
        assert_eq!(updated.total_xp, 100);
    }
}
```

### Integration Tests

```rust
// /apps/desktop/src-tauri/tests/integration_tests.rs

#[cfg(test)]
mod tests {
    use tauri::test::{MockRuntime, mock_builder};

    #[test]
    fn test_get_user_data_command() {
        let app = mock_builder().build::<MockRuntime>(tauri::generate_context!()).unwrap();

        // Test command invocation
        let result = tauri::test::invoke_handler(
            &app,
            "get_user_data",
            tauri::test::InvokePayload::default(),
        );

        // Assert result
        assert!(result.is_ok());
    }
}
```

### Manual Testing Checklist

- [ ] App launches without errors
- [ ] Database is created in correct location
- [ ] Can navigate between pages
- [ ] User data loads on startup
- [ ] XP updates reflect in status bar
- [ ] Manifest loads successfully
- [ ] Missing content files show clear errors
- [ ] App restarts preserve data

---

## Time Estimates Summary

| Milestone | Task | Time |
|-----------|------|------|
| **1.1** | Schema + Migrations | 4 hours |
| **1.1** | Models + CRUD | 6 hours |
| **1.1** | Testing | 4 hours |
| **1.2** | Tauri setup | 4 hours |
| **1.2** | Commands + State | 5 hours |
| **1.2** | Frontend components | 6 hours |
| **1.2** | Integration + Testing | 5 hours |
| **1.3** | Manifest + Loader | 5 hours |
| **1.3** | Dummy content + Validation | 4 hours |
| | **TOTAL** | **43 hours (~7-9 days)** |

**Reality Buffer**: Add 20% for debugging, documentation, and unexpected issues → **9-11 days total**

---

## Success Criteria (Phase 1 Complete When)

- [ ] Database schema is stable and tested
- [ ] Tauri app launches and connects to DB
- [ ] Content loader works end-to-end
- [ ] Can view skill tree skeleton in UI (even if empty)
- [ ] User data persists across app restarts
- [ ] No crashes or unhandled errors
- [ ] All unit tests pass
- [ ] Screenshot evidence of working app

---

## Next Steps After Phase 1

Once Phase 1 is complete, you'll have:
1. Working database with all tables
2. Tauri app with navigation
3. Content loading system
4. Foundation for Phase 2 (Core Game Loop)

**Phase 2 Preview**:
- Lecture viewer with markdown rendering
- Quiz system with grading
- XP calculation and awarding
- Progress tracking
- Daily session planner

---

## Additional Resources

### Documentation Links

- **rusqlite**: https://docs.rs/rusqlite/latest/rusqlite/
- **Tauri Commands**: https://tauri.app/v1/guides/features/command/
- **Zustand**: https://docs.pmnd.rs/zustand/getting-started/introduction
- **shadcn/ui**: https://ui.shadcn.com/docs
- **Tailwind CSS**: https://tailwindcss.com/docs

### Example Code Repositories

- Tauri + React: https://github.com/tauri-apps/tauri/tree/dev/examples
- Zustand examples: https://github.com/pmndrs/zustand/tree/main/examples

---

## Questions to Resolve Before Starting

1. **Where should the database file live?**
   - Recommendation: Use Tauri's `app_data_dir()` for cross-platform compatibility

2. **Should we support multiple users?**
   - Recommendation: Start with single user, design allows future expansion

3. **How to handle content updates?**
   - Recommendation: Phase 1 just loads on startup, Phase 6 add hot-reload

4. **What happens if manifest is invalid?**
   - Recommendation: Show error dialog, don't start app (fail fast)

---

**End of Phase 1 Implementation Plan**

This document provides all the technical details needed to implement Phase 1. Start with Milestone 1.1 (Data Schema), then 1.2 (Tauri Shell), then 1.3 (Content Loader). Follow the code examples closely and refer back to this document when you encounter issues.

Good luck!
