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
    last_streak_date TEXT,
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
    status TEXT NOT NULL DEFAULT 'NotStarted',
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
    skill_id TEXT NOT NULL,
    score REAL NOT NULL DEFAULT 0.0,
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
    badge_id TEXT NOT NULL,
    current_value REAL NOT NULL DEFAULT 0.0,
    earned_at TEXT,
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
    node_id TEXT NOT NULL,
    answers_json TEXT NOT NULL,
    score_percentage INTEGER NOT NULL,
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
    code_hash TEXT NOT NULL,
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
    artifact_type TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    grade_percentage INTEGER,
    reasoning_json TEXT,
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
    due_date TEXT NOT NULL,
    ease_factor REAL NOT NULL DEFAULT 2.5,
    interval_days INTEGER NOT NULL DEFAULT 1,
    repetitions INTEGER NOT NULL DEFAULT 0,
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
    content_hash TEXT PRIMARY KEY,
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
