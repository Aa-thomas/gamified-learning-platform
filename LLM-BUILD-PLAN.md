# 🗺️ Step-by-Step Build Plan: Gamified Rust Bootcamp Platform

**Total Timeline: 8-12 weeks MVP → 4-6 weeks Beta → Production**

---

## 📐 Build Strategy

**Philosophy:**
- **Vertical slices** (end-to-end features) over horizontal layers
- **Validate risks early** (LLM grading, Docker runner)
- **One complete flow** before adding breadth
- **Checkpoint-gated** (no moving forward until evidence exists)

**Core Principle:**
> Every phase must produce a **demoable artifact** that proves the concept works.

---

# Phase 0: Risk Validation (Week 1-2)

**Goal:** Prove the three highest-risk assumptions before building anything

---

## 🎯 Milestone 0.1: LLM Grading Prototype (3-4 days)

### Deliverables
```
/prototypes/llm-grading/
├── sample_artifacts/
│   ├── design_good.md
│   ├── design_mediocre.md
│   ├── design_bad.md
│   ├── readme_good.md
│   └── readme_bad.md
├── rubrics/
│   ├── design_rubric.json
│   └── readme_rubric.json
├── grader.rs
└── results.md (human vs LLM comparison)
```

### Tasks
1. **Create 3 sample artifacts** for DESIGN.md (good/mediocre/bad)
2. **Write LLM rubric** in structured JSON format
3. **Write grading prompt** with rubric injection
4. **Test GPT-4 grading** on samples (5 runs each for consistency)
5. **Measure:**
   - Grade consistency (same input → same grade?)
   - Agreement with human judgment (>80% required)
   - API latency (p50, p95, p99)
   - API cost per grade (~$0.01-0.10)

### Acceptance Criteria
- ✅ Same artifact graded 5 times produces scores within ±5 points
- ✅ LLM agrees with human judgment on good/bad samples ≥80%
- ✅ Grading completes in <10 seconds p95
- ✅ Cost per grade documented

### Risk Mitigation
- **If consistency fails:** Add content hashing + caching strategy
- **If agreement fails:** Simplify rubric or fall back to checklist-only
- **If too slow:** Add timeout + provisional grading
- **If too expensive:** Limit retries or use cheaper model

---

## 🎯 Milestone 0.2: Docker Runner Prototype (2-3 days)

### Deliverables
```
/prototypes/docker-runner/
├── sample_challenge/
│   ├── Cargo.toml
│   ├── src/lib.rs (with TODO)
│   └── tests/test.rs
├── runner.rs
├── Dockerfile
└── test_results.md
```

### Tasks
1. **Create sample mini-challenge** with tests
2. **Build Docker image** with Rust toolchain
3. **Implement runner** that:
   - Copies code into container
   - Runs `cargo test`
   - Runs `cargo clippy`
   - Returns structured results
4. **Test edge cases:**
   - Infinite loop (timeout works?)
   - Compile error (captures error?)
   - Panic in test (reports correctly?)
   - Memory bomb (container limits work?)

### Acceptance Criteria
- ✅ Successfully runs and returns test results
- ✅ Timeout kills runaway code (30s limit)
- ✅ Captures stdout/stderr correctly
- ✅ Container cleanup works (no orphans)
- ✅ Works on macOS, Linux, Windows (if applicable)

### Risk Mitigation
- **If Docker install friction:** Document one-click installer + fallback "skip challenges" mode
- **If performance poor:** Cache base image + pre-warm container
- **If cleanup fails:** Add force-kill + manual cleanup tool

---

## 🎯 Milestone 0.3: XP/Mastery Formula Validation (1 day)

### Deliverables
```
/prototypes/gamification/
├── formulas.rs
├── simulation.rs
└── balance_report.md
```

### Tasks
1. **Implement XP formulas** (difficulty, streak, accuracy)
2. **Implement mastery formulas** (learning rate, decay)
3. **Simulate progression:**
   - Daily user (30 min/day for 20 weeks)
   - Binge user (8 hours/day for 4 weeks)
   - Casual user (2 hours/week for 40 weeks)
4. **Check balance:**
   - Does XP curve feel good? (not too fast/slow)
   - Does mastery decay motivate without punishing?
   - Are badges unlockable at reasonable intervals?

### Acceptance Criteria
- ✅ Daily user reaches Week 10 in ~10 weeks
- ✅ Mastery decay doesn't zero out after 1 week break
- ✅ At least one badge unlocks every 5-7 days

### Risk Mitigation
- **If progression too fast:** Lower base XP or increase level thresholds
- **If too slow:** Increase difficulty multipliers
- **If decay too harsh:** Reduce decay rate or cap at 0.3 minimum

---

## ✅ Phase 0 Complete When:
- [ ] LLM grading achieves ≥80% human agreement
- [ ] Docker runner handles all edge cases
- [ ] Gamification simulation shows balanced progression
- [ ] Decision made: proceed or pivot approach

**Time Check:** If >2 weeks, risks are too high. Simplify approach.

---

# Phase 1: Foundation (Week 3-4)

**Goal:** Build the skeleton - data model, storage, basic Tauri app

---

## 🎯 Milestone 1.1: Data Schema (2-3 days)

### Deliverables
```
/crates/core/
├── src/
│   ├── models/
│   │   ├── user.rs
│   │   ├── content.rs
│   │   ├── progress.rs
│   │   ├── quiz.rs
│   │   ├── challenge.rs
│   │   ├── checkpoint.rs
│   │   └── badge.rs
│   ├── db/
│   │   ├── schema.sql
│   │   ├── migrations/
│   │   └── connection.rs
│   └── lib.rs
└── tests/
    └── db_tests.rs
```

### Tasks
1. **Define Rust structs** for all data types (from our "Named Missing Pieces")
2. **Write SQLite schema** with proper indexes
3. **Implement migrations** (using `rusqlite` or `sqlx`)
4. **Write basic CRUD operations**
5. **Add serialization** (serde for JSON export)

### Schema Tables
```sql
-- users (single user for now, but future-proof)
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    created_at TEXT NOT NULL,
    last_activity TEXT NOT NULL,
    total_xp INTEGER DEFAULT 0,
    current_streak INTEGER DEFAULT 0
);

-- progress
CREATE TABLE node_progress (
    user_id TEXT,
    node_id TEXT,
    status TEXT, -- NotStarted | InProgress | Completed | Failed
    attempts INTEGER DEFAULT 0,
    time_spent_mins INTEGER DEFAULT 0,
    first_started TEXT,
    completed_at TEXT,
    PRIMARY KEY (user_id, node_id)
);

-- mastery
CREATE TABLE mastery_scores (
    user_id TEXT,
    skill_id TEXT,
    score REAL, -- 0.0-1.0
    last_updated TEXT,
    PRIMARY KEY (user_id, skill_id)
);

-- badges
CREATE TABLE badge_progress (
    user_id TEXT,
    badge_id TEXT,
    current_value REAL,
    earned_at TEXT,
    PRIMARY KEY (user_id, badge_id)
);

-- quiz attempts
CREATE TABLE quiz_attempts (
    id TEXT PRIMARY KEY,
    user_id TEXT,
    quiz_id TEXT,
    answers TEXT, -- JSON array
    score INTEGER,
    submitted_at TEXT
);

-- artifact submissions
CREATE TABLE artifact_submissions (
    id TEXT PRIMARY KEY,
    user_id TEXT,
    checkpoint_id TEXT,
    artifact_type TEXT,
    content_hash TEXT,
    grade INTEGER,
    reasoning TEXT,
    graded_at TEXT
);

-- review queue (spaced repetition)
CREATE TABLE review_items (
    user_id TEXT,
    quiz_id TEXT,
    due_date TEXT,
    ease_factor REAL,
    interval_days INTEGER,
    repetitions INTEGER,
    last_reviewed TEXT,
    PRIMARY KEY (user_id, quiz_id)
);

-- llm grade cache
CREATE TABLE grade_cache (
    content_hash TEXT PRIMARY KEY,
    artifact_type TEXT,
    grade INTEGER,
    reasoning TEXT,
    cached_at TEXT
);
```

### Acceptance Criteria
- ✅ All tables created with constraints
- ✅ Basic CRUD works (insert, query, update)
- ✅ Can export full DB to JSON
- ✅ Can import from JSON (for backups)
- ✅ Tests cover happy path + edge cases

---

## 🎯 Milestone 1.2: Tauri Shell + Basic UI (3-4 days)

### Deliverables
```
/apps/desktop/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   └── commands.rs
│   └── Cargo.toml
├── src/
│   ├── App.tsx
│   ├── pages/
│   │   ├── Home.tsx
│   │   └── SkillTree.tsx
│   └── components/
│       └── Navigation.tsx
└── package.json
```

### Tasks
1. **Initialize Tauri app** (`cargo tauri init`)
2. **Set up frontend** (React + TypeScript + Tailwind)
3. **Create basic layout:**
   - Navigation bar (Home, Skill Tree, Progress)
   - Main content area
   - Status bar (XP, streak, level)
4. **Implement Tauri commands:**
   ```rust
   #[tauri::command]
   fn get_user_progress() -> UserProgress { }
   
   #[tauri::command]
   fn get_skill_tree() -> ContentTree { }
   ```
5. **Wire up state management** (Zustand or Context)

### Acceptance Criteria
- ✅ App launches on desktop
- ✅ Can navigate between pages
- ✅ Frontend can call backend commands
- ✅ DB connection works from Tauri backend
- ✅ Basic styling complete (doesn't have to be pretty)

---

## 🎯 Milestone 1.3: Content Loader (2 days)

### Deliverables
```
/crates/content/
├── src/
│   ├── loader.rs
│   ├── manifest.rs
│   └── validator.rs
└── tests/

/content/
├── manifest.json
└── week1/
    └── day1/
        ├── lecture.md
        ├── quiz.json
        └── challenge.json
```

### Tasks
1. **Define manifest format** (from our "Named Missing Pieces")
2. **Write content loader** that reads manifest + parses files
3. **Validate content** (required fields, paths exist)
4. **Build content tree** in memory
5. **Create 1 week of dummy content** for testing

### Acceptance Criteria
- ✅ Manifest loads without errors
- ✅ Content tree builds correctly
- ✅ Missing files trigger clear errors
- ✅ Invalid JSON rejected with helpful messages

---

## ✅ Phase 1 Complete When:
- [ ] Database schema is stable
- [ ] Tauri app launches + connects to DB
- [ ] Content loader works end-to-end
- [ ] Can view (empty) skill tree in UI

**Evidence:** Screenshot of app showing skill tree skeleton

---

# Phase 2: Core Game Loop (Week 5-6)

**Goal:** One complete vertical slice - lecture → quiz → XP gain

---

## 🎯 Milestone 2.1: Lecture Viewer (1-2 days)

### Deliverables
```
/src/pages/Lecture.tsx
/src/components/MarkdownRenderer.tsx
```

### Tasks
1. **Render markdown lectures** (using `react-markdown`)
2. **Add navigation** (previous/next)
3. **Track time spent**
4. **Mark as complete** when user reaches end

### Acceptance Criteria
- ✅ Lecture displays formatted markdown
- ✅ Code blocks syntax highlighted
- ✅ "Mark Complete" button appears at end
- ✅ Completion updates DB + unlocks next node

---

## 🎯 Milestone 2.2: Quiz System (3-4 days)

### Deliverables
```
/src/pages/Quiz.tsx
/src/components/QuizQuestion.tsx
/crates/core/src/quiz/grader.rs
```

### Tasks
1. **Build quiz UI:**
   - Show question + options (radio buttons)
   - Submit button
   - Show result + explanation
2. **Implement grading logic:**
   - Check answer correctness
   - Calculate XP (difficulty × accuracy × streak)
   - Update mastery scores
3. **Save quiz attempt** to DB
4. **Update progress + unlock next node**

### Acceptance Criteria
- ✅ Can answer quiz questions
- ✅ Correct/incorrect shown immediately
- ✅ XP awarded and displayed
- ✅ Mastery updates for relevant skills
- ✅ Next node unlocks if quiz passed

---

## 🎯 Milestone 2.3: Progress Dashboard (2 days)

### Deliverables
```
/src/pages/Progress.tsx
/src/components/XPBar.tsx
/src/components/StreakCounter.tsx
/src/components/MasteryRadar.tsx
```

### Tasks
1. **Build XP/level display** (progress bar to next level)
2. **Build streak counter** (days + grace period warning)
3. **Build mastery chart** (radar or bar chart per skill)
4. **Show recent activity** (last 10 completed nodes)

### Acceptance Criteria
- ✅ XP displays correctly + updates after quiz
- ✅ Streak updates daily
- ✅ Mastery chart shows all tracked skills
- ✅ Activity log shows completions

---

## 🎯 Milestone 2.4: Daily Session Queue (2 days)

### Deliverables
```
/src/pages/DailySession.tsx
/crates/core/src/session/planner.rs
```

### Tasks
1. **Build session planner** that recommends:
   - 1 lecture (if available)
   - 1 quiz
   - 1 mini-challenge (if unlocked)
2. **Track session progress** (current activity, XP earned)
3. **Show session summary** on completion

### Acceptance Criteria
- ✅ "Start Daily Session" button works
- ✅ Shows ordered list of recommended activities
- ✅ Tracks current position in session
- ✅ Summary shows total XP + time spent

---

## ✅ Phase 2 Complete When:
- [ ] Can complete full session: lecture → quiz → XP gain
- [ ] Progress persists across app restarts
- [ ] Streak updates correctly (test by changing system date)
- [ ] Mastery updates after quiz

**Evidence:** Screen recording of complete session flow

---

# Phase 3: Verification Systems (Week 7-8)

**Goal:** Add Docker runner + LLM grading (the two risky pieces)

---

## 🎯 Milestone 3.1: Docker Integration (3-4 days)

### Deliverables
```
/crates/runner/
├── src/
│   ├── docker.rs
│   ├── verification.rs
│   └── errors.rs
└── tests/

/docker/
└── rust-sandbox/
    └── Dockerfile
```

### Tasks
1. **Build Docker image** with Rust toolchain
2. **Implement DockerRunner:**
   ```rust
   impl ChallengeRunner for DockerRunner {
       fn run_verification(...) -> Result<VerificationResult> {
           // 1. Create temp dir with student code
           // 2. docker run with timeout
           // 3. Parse output
           // 4. Clean up
       }
   }
   ```
3. **Add Docker detection** on app startup
4. **Build mini-challenge UI:**
   - Code editor (Monaco or textarea)
   - "Run Tests" button
   - Output display (stdout/stderr)

### Acceptance Criteria
- ✅ Docker detection works + shows helpful error if missing
- ✅ Can run student code in container
- ✅ Timeout kills infinite loops
- ✅ Test output displays correctly
- ✅ Passing challenge awards XP + unlocks next

---

## 🎯 Milestone 3.2: LLM Grading Integration (3-4 days)

### Deliverables
```
/crates/grader/
├── src/
│   ├── llm.rs
│   ├── rubrics/
│   │   ├── design.json
│   │   ├── readme.json
│   │   └── bench.json
│   ├── cache.rs
│   └── prompts.rs
```

### Tasks
1. **Port prototype LLM grader** into real crate
2. **Implement grade caching:**
   ```rust
   fn grade_with_cache(content: &str) -> Result<Grade> {
       let hash = sha256(content);
       if let Some(cached) = cache.get(hash) {
           return Ok(cached);
       }
       let grade = call_openai(content).await?;
       cache.set(hash, &grade);
       Ok(grade)
   }
   ```
3. **Write all 5 rubrics** (README, DESIGN, BENCH, RUNBOOK, INVARIANTS)
4. **Build checkpoint submission UI:**
   - File picker for artifact directory
   - Shows checklist (files present?)
   - "Submit for Grading" button
   - Progress spinner during grading
   - Results display with per-artifact scores

### Acceptance Criteria
- ✅ Can submit artifacts for grading
- ✅ Grading completes in <30s p95
- ✅ Cache prevents duplicate API calls
- ✅ Grade reasoning displays clearly
- ✅ Passing checkpoint awards XP + unlocks next

---

## ✅ Phase 3 Complete When:
- [ ] Mini-challenge verification works end-to-end
- [ ] Checkpoint grading works end-to-end
- [ ] Both use caching correctly
- [ ] Error states handled gracefully

**Evidence:** Video showing challenge run + checkpoint grade

---

# Phase 4: Gamification (Week 9-10)

**Goal:** Complete the motivation loop - badges, mastery decay, review queue

---

## 🎯 Milestone 4.1: Badge System (2-3 days)

### Deliverables
```
/crates/core/src/badges/
├── definitions.rs
├── tracker.rs
└── unlocks.rs

/src/components/BadgeDisplay.tsx
```

### Tasks
1. **Define 10-15 badges** with criteria
2. **Implement badge tracker:**
   ```rust
   fn check_badge_unlocks(user: &User) -> Vec<Badge> {
       // Check criteria for all badges
       // Return newly unlocked badges
   }
   ```
3. **Build badge UI:**
   - Badge collection page (earned + locked)
   - Unlock animation/notification
   - Badge details (how to earn)

### Acceptance Criteria
- ✅ Badges unlock when criteria met
- ✅ Unlock notification appears
- ✅ Badge collection displays correctly
- ✅ At least 5 badges definable via config

---

## 🎯 Milestone 4.2: Mastery Decay + Review Queue (3-4 days)

### Deliverables
```
/crates/core/src/spaced_repetition/
├── sm2.rs
├── scheduler.rs
└── review.rs

/src/pages/Review.tsx
```

### Tasks
1. **Implement Anki's modified SM-2:**
   ```rust
   fn update_review_schedule(item: &mut ReviewItem, grade: u8) {
       // Update ease factor, interval, reps
   }
   ```
2. **Build mastery decay worker:**
   - Run on app startup
   - Apply exponential decay to inactive skills
3. **Build review UI:**
   - Shows due reviews
   - Quiz interface (same as regular quiz)
   - Updates ease factor based on performance

### Acceptance Criteria
- ✅ Mastery decays correctly when inactive
- ✅ Review items scheduled correctly
- ✅ Review UI shows due count
- ✅ Completing review updates schedule

---

## 🎯 Milestone 4.3: Skill Tree Visualization (2-3 days)

### Deliverables
```
/src/pages/SkillTree.tsx
/src/components/TreeNode.tsx
```

### Tasks
1. **Build interactive skill tree:**
   - Nodes (circles/icons)
   - Edges (lines showing prerequisites)
   - Colors: locked (gray), unlocked (blue), completed (green)
2. **Add click handlers:**
   - Click node → show details
   - Shows XP reward, difficulty, skills trained
3. **Add visual polish:**
   - Smooth animations
   - Hover states
   - Current path highlighted

### Acceptance Criteria
- ✅ Tree renders correctly
- ✅ Node states (locked/unlocked/complete) display correctly
- ✅ Can click to see node details
- ✅ Prerequisites visually clear

---

## ✅ Phase 4 Complete When:
- [ ] Badges unlock correctly
- [ ] Mastery decays over time
- [ ] Review queue populates
- [ ] Skill tree is navigable

**Evidence:** Screenshot of badge unlock + skill tree + review queue

---

# Phase 5: Content Integration (Week 11-12)

**Goal:** Port real curriculum content into the system

---

## 🎯 Milestone 5.1: Content Authoring Pipeline (2-3 days)

### Deliverables
```
/tools/content-builder/
├── src/
│   ├── quiz_generator.rs
│   ├── challenge_generator.rs
│   └── manifest_builder.rs
└── templates/

/scripts/
└── build-content.sh
```

### Tasks
1. **Build quiz generator** from your quiz template
2. **Build challenge generator** from your challenge template
3. **Build manifest builder** from syllabus structure
4. **Validate generated content:**
   - All required fields present
   - Paths resolve
   - JSON parses

### Acceptance Criteria
- ✅ Can generate quiz JSON from markdown
- ✅ Can generate challenge specs from outline
- ✅ Manifest builds from syllabus
- ✅ Validation catches errors

---

## 🎯 Milestone 5.2: Port Week 1 Content (3-4 days)

### Tasks
1. **Convert Week 1 lectures** to markdown
2. **Generate Week 1 quizzes** (use your quiz generator template)
3. **Create Week 1 mini-challenges** with Docker verification
4. **Define Week 1 checkpoint** with LLM rubrics
5. **Test full week flow** end-to-end

### Acceptance Criteria
- ✅ All Week 1 content loads
- ✅ Can complete full week session-by-session
- ✅ Checkpoint submission works
- ✅ XP/mastery progression feels balanced

---

## 🎯 Milestone 5.3: Port Weeks 2-4 (Parallel) (4-5 days)

### Tasks
1. **Batch generate content** for Weeks 2-4
2. **Create checkpoint rubrics**
3. **Test progression** through all 4 weeks
4. **Balance XP** if needed (adjust difficulty multipliers)

### Acceptance Criteria
- ✅ 4 weeks of content available
- ✅ Progression feels smooth
- ✅ No content gaps or broken links

---

## ✅ Phase 5 Complete When:
- [ ] 4 weeks of real content in system
- [ ] Content authoring pipeline documented
- [ ] Can complete Week 1-4 end-to-end

**Evidence:** Progress report showing completion of Week 1-4

---

# Phase 6: Polish & Beta (Week 13-14)

**Goal:** Make it production-ready

---

## 🎯 Milestone 6.1: Error Handling & Edge Cases (2-3 days)

### Tasks
1. **Implement all error states** (from our edge case analysis)
2. **Add user-facing error messages:**
   - Docker not running
   - LLM API timeout
   - SQLite locked
   - Code timeout
3. **Add retry logic** where appropriate
4. **Add manual backup/restore**

### Acceptance Criteria
- ✅ No panics or unhandled errors
- ✅ All error messages actionable
- ✅ Backup/restore works

---

## 🎯 Milestone 6.2: Onboarding & Help (2 days)

### Tasks
1. **Build welcome flow:**
   - App intro
   - Docker check
   - OpenAI API key setup
2. **Add help tooltips** throughout UI
3. **Create demo/tutorial** (first lecture + quiz)

### Acceptance Criteria
- ✅ First-time user can get started
- ✅ Docker setup is clear
- ✅ Tutorial completes successfully

---

## 🎯 Milestone 6.3: Performance & Polish (2-3 days)

### Tasks
1. **Optimize Docker runner** (pre-warm containers)
2. **Optimize LLM calls** (batch if possible)
3. **Add loading states** everywhere
4. **Polish UI:**
   - Consistent spacing
   - Smooth animations
   - Dark mode (if not done)
5. **Add keyboard shortcuts**

### Acceptance Criteria
- ✅ Docker runs complete in <5s p95
- ✅ No janky animations
- ✅ UI feels responsive

---

## 🎯 Milestone 6.4: Testing & Documentation (2-3 days)

### Tasks
1. **Write user documentation:**
   - Installation guide
   - Troubleshooting guide
   - FAQ
2. **Record demo video** (5-10 min)
3. **Create README** with screenshots
4. **Run full regression test** (Weeks 1-4 complete)

### Acceptance Criteria
- ✅ Documentation complete
- ✅ Demo video published
- ✅ Can complete Weeks 1-4 without bugs

---

## ✅ Phase 6 Complete When:
- [ ] All error states handled
- [ ] Onboarding smooth
- [ ] Documentation complete
- [ ] 3 beta testers can use successfully

**Evidence:** Beta tester feedback + bug reports

---

# 🎯 Deployment (Week 15-16)

## Milestone 7.1: Packaging (2-3 days)

### Tasks
1. **Build installers:**
   - macOS: `.dmg`
   - Windows: `.exe` (NSIS installer)
   - Linux: `.AppImage` or `.deb`
2. **Code signing** (macOS required)
3. **Auto-update setup** (Tauri updater)

### Acceptance Criteria
- ✅ Installers work on all platforms
- ✅ App launches after install
- ✅ Auto-update works

---

## Milestone 7.2: Release (1-2 days)

### Tasks
1. **GitHub Release** with installers
2. **Landing page** (simple, explains what it is)
3. **Distribution:** Share with bootcamp students

### Acceptance Criteria
- ✅ Downloads work
- ✅ Installation instructions clear
- ✅ Support channel set up (Discord/GitHub Issues)

---

# 📊 Summary Timeline

```
Phase 0: Risk Validation         [2 weeks]  ████████
Phase 1: Foundation              [2 weeks]  ████████
Phase 2: Core Game Loop          [2 weeks]  ████████
Phase 3: Verification Systems    [2 weeks]  ████████
Phase 4: Gamification            [2 weeks]  ████████
Phase 5: Content Integration     [2 weeks]  ████████
Phase 6: Polish & Beta           [2 weeks]  ████████
Phase 7: Deployment              [2 weeks]  ████████
                                 ─────────────────────
                        TOTAL:   14-16 weeks
```

---

# 🎯 Checkpoint Gates (DO NOT SKIP)

**After Phase 0:**
- [ ] LLM grading proven reliable
- [ ] Docker runner proven safe
- [ ] Gamification balanced

**After Phase 2:**
- [ ] Can complete one full session
- [ ] Progress persists
- [ ] XP/mastery updates correctly

**After Phase 3:**
- [ ] Challenge verification works
- [ ] Checkpoint grading works
- [ ] Caching prevents duplicate costs

**After Phase 5:**
- [ ] 4 weeks of content complete
- [ ] Content quality validated

**Before Release:**
- [ ] 3 beta testers complete Week 1
- [ ] No critical bugs
- [ ] Documentation complete

---

# 📋 Tech Stack Summary

```
Frontend:        React + TypeScript + Tailwind
Desktop:         Tauri (Rust)
Database:        SQLite
Code Runner:     Docker
LLM:             OpenAI API (GPT-4)
Charts:          Recharts or D3
Markdown:        react-markdown
Code Editor:     Monaco (optional) or textarea
Spaced Rep:      Custom (Anki's SM-2)
```

---

# 💰 Cost Estimate

**Development:**
- Time: 14-16 weeks solo (8-10 weeks with 2 people)

**Per-User Operating Costs:**
- LLM API: ~$5-15 per student (14 checkpoints × ~$0.50 avg)
- Infrastructure: $0 (local-only)

**One-Time Costs:**
- Code signing cert: ~$100/year (macOS)
- Domain: ~$15/year (optional)

---

