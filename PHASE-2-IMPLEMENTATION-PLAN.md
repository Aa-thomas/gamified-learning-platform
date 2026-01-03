# Phase 2: Core Game Loop - Detailed Implementation Plan

**Timeline**: Week 5-6 (2 weeks)
**Goal**: One complete vertical slice - lecture → quiz → XP gain

---

## Table of Contents

1. [Technology Stack Recommendations](#technology-stack-recommendations)
2. [Milestone 2.1: Lecture Viewer](#milestone-21-lecture-viewer)
3. [Milestone 2.2: Quiz System](#milestone-22-quiz-system)
4. [Milestone 2.3: Progress Dashboard](#milestone-23-progress-dashboard)
5. [Milestone 2.4: Daily Session Queue](#milestone-24-daily-session-queue)
6. [Integration & Testing](#integration--testing)
7. [Edge Cases & Error Handling](#edge-cases--error-handling)

---

## Technology Stack Recommendations

### Frontend Libraries

#### 1. Markdown Rendering

**Recommended: `react-markdown` v9+**

**Rationale**:
- Pure React component (no dangerouslySetInnerHTML)
- Excellent security (sanitizes by default)
- Extensible plugin system (remark/rehype)
- Active maintenance
- TypeScript support out of the box

**Alternatives Considered**:
- `marked`: Faster but requires manual React integration and sanitization
- `markdown-it`: Good but requires wrapper component
- `remark-react`: Lower-level, more complex setup

**Installation**:
```bash
npm install react-markdown remark-gfm rehype-raw
```

**Basic Usage**:
```tsx
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'

<ReactMarkdown
  remarkPlugins={[remarkGfm]}
  components={{
    code: CodeBlock,
    h1: ({ children }) => <h1 className="text-3xl font-bold mb-4">{children}</h1>
  }}
>
  {content}
</ReactMarkdown>
```

#### 2. Code Syntax Highlighting

**Recommended: `react-syntax-highlighter` with Prism**

**Rationale**:
- Drop-in React component
- Supports 100+ languages (including Rust)
- Multiple theme options (dark/light)
- Good performance with code splitting
- No build-time dependencies

**Alternatives Considered**:
- `Shiki`: Beautiful but slower, requires build-time setup
- `Highlight.js`: Good but manual DOM manipulation needed
- Raw `Prism.js`: Requires manual integration

**Installation**:
```bash
npm install react-syntax-highlighter
npm install --save-dev @types/react-syntax-highlighter
```

**Usage**:
```tsx
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter'
import { vscDarkPlus } from 'react-syntax-highlighter/dist/esm/styles/prism'

const CodeBlock = ({ className, children }: any) => {
  const language = className?.replace('language-', '') || 'text'
  return (
    <SyntaxHighlighter language={language} style={vscDarkPlus}>
      {children}
    </SyntaxHighlighter>
  )
}
```

#### 3. Charting Library

**Recommended: `Recharts` v2+**

**Rationale**:
- Pure React components (declarative)
- Great TypeScript support
- Responsive by default
- Built-in animations
- Easy to customize
- Good documentation

**Alternatives Considered**:
- `Chart.js`: More powerful but imperative API (harder in React)
- `D3`: Most powerful but steep learning curve, overkill for this use case
- `Victory`: Good but heavier bundle size

**Installation**:
```bash
npm install recharts
```

**Usage Example**:
```tsx
import { RadarChart, PolarGrid, PolarAngleAxis, Radar } from 'recharts'

<RadarChart width={400} height={400} data={masteryData}>
  <PolarGrid />
  <PolarAngleAxis dataKey="skill" />
  <Radar name="Mastery" dataKey="score" stroke="#8884d8" fill="#8884d8" fillOpacity={0.6} />
</RadarChart>
```

#### 4. State Management

**Recommended: `Zustand` v4+**

**Rationale**:
- Minimal boilerplate (simpler than Redux)
- TypeScript-first design
- No Provider wrapper needed
- Supports middleware (persist, devtools)
- Small bundle size (2kb)
- Perfect for Tauri apps

**Alternatives Considered**:
- React Context: Good for simple cases but verbose for complex state
- Redux Toolkit: Overkill for this project size
- Jotai/Recoil: More atomic but adds complexity

**Installation**:
```bash
npm install zustand
```

**Usage**:
```tsx
import create from 'zustand'
import { persist } from 'zustand/middleware'

interface UserState {
  xp: number
  level: number
  streak: number
  updateXP: (xp: number) => void
}

const useUserStore = create<UserState>()(
  persist(
    (set) => ({
      xp: 0,
      level: 1,
      streak: 0,
      updateXP: (xp) => set((state) => ({
        xp: state.xp + xp,
        level: calculateLevel(state.xp + xp)
      }))
    }),
    { name: 'user-storage' }
  )
)
```

### Backend/Tauri

**Database**: `rusqlite` v0.30+ (already in build plan)
**Serialization**: `serde` + `serde_json` (standard)
**Date/Time**: `chrono` v0.4+ (mature, widely used)

---

## Milestone 2.1: Lecture Viewer

**Time Estimate**: 1-2 days
**Priority**: High (blocking for other milestones)

### 1.1 Research Summary

#### Markdown Rendering
- **Winner**: `react-markdown` (see Technology Stack section)
- **Plugins needed**:
  - `remark-gfm`: GitHub Flavored Markdown (tables, strikethrough)
  - `rehype-raw`: Allow HTML in markdown (for custom components)

#### Code Highlighting
- **Winner**: `react-syntax-highlighter` with Prism (see Technology Stack section)
- **Themes**: Use `vscDarkPlus` for consistency with VS Code

### 1.2 Time Tracking Design

#### Option A: Frontend-Only Tracking (Recommended)
**Mechanism**:
```tsx
const [startTime, setStartTime] = useState(Date.now())
const [timeSpent, setTimeSpent] = useState(0)

useEffect(() => {
  const interval = setInterval(() => {
    if (document.visibilityState === 'visible') {
      setTimeSpent(Date.now() - startTime)
    }
  }, 1000)

  return () => clearInterval(interval)
}, [startTime])

// On lecture complete
const handleComplete = async () => {
  await invoke('complete_lecture', {
    lectureId,
    timeSpentMs: timeSpent
  })
}
```

**Pros**:
- Simple implementation
- No backend complexity
- Real-time updates in UI

**Cons**:
- Can be cheated (user can inspect/manipulate)
- Lost if browser crashes before completion

**Mitigation**:
- Periodic auto-save every 30s to backend
- Use visibility API to pause timer when tab not active

#### Option B: Backend Heartbeat Tracking
**Mechanism**:
- Frontend sends heartbeat every 10s to backend
- Backend calculates time as `last_heartbeat - first_heartbeat`

**Pros**:
- More accurate
- Survives frontend crashes

**Cons**:
- More complex
- Higher backend load
- Overkill for single-user desktop app

**Decision**: Use **Option A** with auto-save

### 1.3 Progress Tracking & Unlock Logic

#### Data Model
```rust
#[derive(Serialize, Deserialize)]
pub struct LectureProgress {
    pub user_id: String,
    pub lecture_id: String,
    pub status: NodeStatus,
    pub time_spent_ms: i64,
    pub scroll_position: f64, // 0.0-1.0
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize)]
pub enum NodeStatus {
    NotStarted,
    InProgress,
    Completed,
}
```

#### Unlock Algorithm
```rust
pub fn unlock_next_nodes(
    conn: &Connection,
    user_id: &str,
    completed_node_id: &str,
) -> Result<Vec<String>> {
    // 1. Get content tree
    let tree = load_content_tree()?;

    // 2. Find completed node
    let node = tree.find_node(completed_node_id)?;

    // 3. Get all nodes that have this as prerequisite
    let next_nodes = tree.get_dependent_nodes(completed_node_id);

    // 4. For each dependent node, check if ALL prerequisites met
    let mut unlocked = Vec::new();
    for next_node in next_nodes {
        let prereqs = tree.get_prerequisites(&next_node.id);
        let all_complete = prereqs.iter().all(|prereq_id| {
            is_node_completed(conn, user_id, prereq_id).unwrap_or(false)
        });

        if all_complete {
            // Update node status to unlocked
            update_node_status(conn, user_id, &next_node.id, NodeStatus::InProgress)?;
            unlocked.push(next_node.id.clone());
        }
    }

    Ok(unlocked)
}
```

#### Completion Criteria
**Lecture is considered complete when**:
1. User scrolls to bottom (scroll_position >= 0.95)
2. User clicks "Mark Complete" button
3. Minimum time spent: 30 seconds (prevents instant clicking through)

**Implementation**:
```tsx
const handleMarkComplete = async () => {
  if (timeSpent < 30000) {
    toast.error('Please spend at least 30 seconds reading the lecture')
    return
  }

  if (scrollPosition < 0.95) {
    const confirmed = await confirm('You havent scrolled to the bottom. Mark complete anyway?')
    if (!confirmed) return
  }

  await invoke('complete_lecture', { lectureId, timeSpentMs: timeSpent })
  navigate('/session') // Return to session queue
}
```

### 1.4 UI Components

#### Component Hierarchy
```
LecturePage
├── LectureHeader
│   ├── BackButton
│   ├── LectureTitle
│   └── ProgressIndicator (current/total)
├── LectureContent
│   ├── MarkdownRenderer
│   │   └── CodeBlock (syntax highlighted)
│   └── ScrollTracker (invisible)
├── LectureFooter
│   ├── TimeSpent (display)
│   ├── PreviousButton
│   ├── MarkCompleteButton
│   └── NextButton
└── CompletionModal (shows XP earned)
```

#### Component Props Interfaces

```tsx
// LecturePage.tsx
interface LecturePageProps {
  lectureId: string
}

// LectureHeader.tsx
interface LectureHeaderProps {
  title: string
  currentIndex: number
  totalLectures: number
  onBack: () => void
}

// MarkdownRenderer.tsx
interface MarkdownRendererProps {
  content: string
  onScrollChange: (position: number) => void
}

// CodeBlock.tsx
interface CodeBlockProps {
  className?: string
  children: string
  inline?: boolean
}

// LectureFooter.tsx
interface LectureFooterProps {
  timeSpent: number // milliseconds
  canMarkComplete: boolean
  hasNext: boolean
  hasPrevious: boolean
  onPrevious: () => void
  onNext: () => void
  onComplete: () => void
}

// CompletionModal.tsx
interface CompletionModalProps {
  isOpen: boolean
  xpEarned: number
  unlockedNodes: string[]
  onClose: () => void
}
```

#### Component List
1. `LecturePage` - Main container
2. `LectureHeader` - Top navigation
3. `MarkdownRenderer` - Content display with scroll tracking
4. `CodeBlock` - Syntax highlighted code
5. `LectureFooter` - Bottom navigation
6. `CompletionModal` - Success feedback
7. `TimeDisplay` - Human-readable time (reusable)

### 1.5 Tauri Backend Commands

```rust
// src-tauri/src/commands/lecture.rs

#[tauri::command]
pub async fn get_lecture(lecture_id: String) -> Result<LectureData, String> {
    let content_tree = CONTENT_TREE.lock().unwrap();
    let lecture = content_tree.get_lecture(&lecture_id)
        .ok_or("Lecture not found")?;

    // Load markdown content from file
    let content = fs::read_to_string(&lecture.path)
        .map_err(|e| format!("Failed to read lecture: {}", e))?;

    Ok(LectureData {
        id: lecture.id.clone(),
        title: lecture.title.clone(),
        content,
        difficulty: lecture.difficulty,
        skills: lecture.skills.clone(),
        xp_reward: lecture.xp_reward,
    })
}

#[tauri::command]
pub async fn start_lecture(
    user_id: String,
    lecture_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.db.lock().unwrap();

    // Insert or update progress
    db.execute(
        "INSERT INTO node_progress (user_id, node_id, status, first_started)
         VALUES (?1, ?2, 'InProgress', datetime('now'))
         ON CONFLICT(user_id, node_id) DO UPDATE SET
         status = 'InProgress'",
        params![user_id, lecture_id],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn update_lecture_time(
    user_id: String,
    lecture_id: String,
    time_spent_ms: i64,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.db.lock().unwrap();

    db.execute(
        "UPDATE node_progress
         SET time_spent_mins = ?1
         WHERE user_id = ?2 AND node_id = ?3",
        params![time_spent_ms / 60000, user_id, lecture_id],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn complete_lecture(
    user_id: String,
    lecture_id: String,
    time_spent_ms: i64,
    state: State<'_, AppState>,
) -> Result<CompletionResult, String> {
    let db = state.db.lock().unwrap();

    // Update completion
    db.execute(
        "UPDATE node_progress
         SET status = 'Completed',
             completed_at = datetime('now'),
             time_spent_mins = ?1
         WHERE user_id = ?2 AND node_id = ?3",
        params![time_spent_ms / 60000, user_id, lecture_id],
    ).map_err(|e| e.to_string())?;

    // Award XP
    let lecture = CONTENT_TREE.lock().unwrap()
        .get_lecture(&lecture_id)
        .ok_or("Lecture not found")?;

    let xp_earned = calculate_lecture_xp(
        lecture.xp_reward,
        lecture.difficulty,
        &get_user_streak(&db, &user_id)?,
    );

    award_xp(&db, &user_id, xp_earned)?;

    // Unlock next nodes
    let unlocked = unlock_next_nodes(&db, &user_id, &lecture_id)?;

    Ok(CompletionResult {
        xp_earned,
        new_total_xp: get_user_xp(&db, &user_id)?,
        new_level: calculate_level(get_user_xp(&db, &user_id)?),
        unlocked_nodes: unlocked,
    })
}
```

### 1.6 State Flow Diagram

```
[Start Lecture]
      ↓
[Load from content tree]
      ↓
[Check if already in progress] ← [Load saved progress]
      ↓
[Display markdown content]
      ↓
[Track time & scroll] → [Auto-save every 30s]
      ↓
[User scrolls to bottom]
      ↓
[Enable "Mark Complete" button]
      ↓
[User clicks complete] → [Validate min time]
      ↓
[Award XP + Update DB]
      ↓
[Unlock next nodes]
      ↓
[Show completion modal]
      ↓
[Return to session queue]
```

### 1.7 Acceptance Criteria Checklist

- [ ] Lecture displays formatted markdown correctly
- [ ] Code blocks have syntax highlighting for Rust, JavaScript, JSON, YAML
- [ ] Headers, lists, tables, blockquotes render correctly
- [ ] Scroll tracking works (updates real-time)
- [ ] Time tracking pauses when tab inactive
- [ ] Auto-save persists progress every 30s
- [ ] "Mark Complete" button disabled until scroll >= 95%
- [ ] Minimum 30s enforcement works
- [ ] Completion awards XP based on difficulty
- [ ] Next nodes unlock automatically
- [ ] Completion modal shows XP earned
- [ ] Navigation (previous/next) works correctly
- [ ] Progress persists across app restarts

---

## Milestone 2.2: Quiz System

**Time Estimate**: 3-4 days
**Priority**: High (core engagement mechanic)

### 2.1 Quiz Data Model

#### JSON Schema
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["id", "title", "questions", "difficulty", "skills", "passing_score"],
  "properties": {
    "id": {
      "type": "string",
      "description": "Unique identifier (e.g., 'week1-day2-quiz')"
    },
    "title": {
      "type": "string",
      "description": "Display name (e.g., 'Ownership Basics Quiz')"
    },
    "description": {
      "type": "string",
      "description": "Short explanation of what quiz covers"
    },
    "difficulty": {
      "type": "string",
      "enum": ["Easy", "Medium", "Hard", "VeryHard"]
    },
    "skills": {
      "type": "array",
      "items": { "type": "string" },
      "description": "Skills tested (e.g., ['ownership', 'borrowing'])"
    },
    "passing_score": {
      "type": "integer",
      "minimum": 0,
      "maximum": 100,
      "description": "Percentage required to pass (typically 70)"
    },
    "time_limit_seconds": {
      "type": "integer",
      "description": "Optional time limit (null = unlimited)"
    },
    "questions": {
      "type": "array",
      "minItems": 3,
      "items": {
        "$ref": "#/definitions/question"
      }
    }
  },
  "definitions": {
    "question": {
      "type": "object",
      "required": ["id", "type", "prompt", "options", "correct_answer", "explanation"],
      "properties": {
        "id": {
          "type": "string",
          "description": "Question ID (e.g., 'q1', 'q2')"
        },
        "type": {
          "type": "string",
          "enum": ["multiple_choice", "true_false", "code_output", "fill_blank"]
        },
        "prompt": {
          "type": "string",
          "description": "Question text (markdown supported)"
        },
        "code_snippet": {
          "type": "string",
          "description": "Optional code to display above question"
        },
        "options": {
          "type": "array",
          "items": {
            "type": "object",
            "required": ["id", "text"],
            "properties": {
              "id": { "type": "string" },
              "text": { "type": "string" }
            }
          },
          "description": "Answer choices"
        },
        "correct_answer": {
          "type": ["string", "array"],
          "description": "ID of correct option(s). Array for multiple correct answers."
        },
        "explanation": {
          "type": "string",
          "description": "Shown after answering (why correct/incorrect)"
        },
        "points": {
          "type": "integer",
          "default": 1,
          "description": "Weight of question (default 1)"
        }
      }
    }
  }
}
```

#### Example Quiz File
```json
{
  "id": "week1-day2-ownership-quiz",
  "title": "Ownership Fundamentals",
  "description": "Test your understanding of Rust's ownership rules",
  "difficulty": "Medium",
  "skills": ["ownership", "memory-safety"],
  "passing_score": 70,
  "time_limit_seconds": 600,
  "questions": [
    {
      "id": "q1",
      "type": "multiple_choice",
      "prompt": "What happens when a variable goes out of scope in Rust?",
      "options": [
        { "id": "a", "text": "Nothing, memory persists" },
        { "id": "b", "text": "Drop is called automatically" },
        { "id": "c", "text": "Compiler error" },
        { "id": "d", "text": "Undefined behavior" }
      ],
      "correct_answer": "b",
      "explanation": "Rust automatically calls `drop()` when a variable goes out of scope, ensuring memory safety without garbage collection.",
      "points": 1
    },
    {
      "id": "q2",
      "type": "code_output",
      "prompt": "What does this code print?",
      "code_snippet": "let s = String::from(\"hello\");\nlet t = s;\nprintln!(\"{}\", s);",
      "options": [
        { "id": "a", "text": "hello" },
        { "id": "b", "text": "Compiler error: value used after move" },
        { "id": "c", "text": "Empty string" },
        { "id": "d", "text": "Runtime panic" }
      ],
      "correct_answer": "b",
      "explanation": "After `let t = s`, ownership of the String moves to `t`. Using `s` afterward causes a compile error.",
      "points": 2
    },
    {
      "id": "q3",
      "type": "true_false",
      "prompt": "Multiple mutable references to the same data are allowed in the same scope.",
      "options": [
        { "id": "true", "text": "True" },
        { "id": "false", "text": "False" }
      ],
      "correct_answer": "false",
      "explanation": "Rust enforces the rule: one mutable reference OR multiple immutable references, never both simultaneously.",
      "points": 1
    }
  ]
}
```

### 2.2 Question Types Supported

#### 1. Multiple Choice (Standard)
- Single correct answer
- 3-5 options typical
- Most common type

#### 2. True/False
- Simplified multiple choice
- 2 options
- Good for concept validation

#### 3. Code Output Prediction
- Show code snippet
- Ask what it outputs (or if it compiles)
- Tests practical understanding

#### 4. Fill in the Blank (Future)
- Text input instead of options
- Requires exact match or fuzzy matching
- More difficult to auto-grade
- **Phase 2**: Skip this type, add in Phase 4

#### 5. Multiple Correct Answers (Future)
- Checkboxes instead of radio buttons
- Select all that apply
- **Phase 2**: Skip this type, add in Phase 4

**Phase 2 Implementation**: Support types 1-3 only

### 2.3 XP Calculation Formulas

Based on validated formulas from `balance_report.md`:

#### Formula
```
xp_earned = base_xp × difficulty_mult × streak_mult × accuracy_mult
```

#### Base XP by Content Type
```rust
const QUIZ_BASE_XP: i32 = 50;
```

#### Difficulty Multipliers
```rust
fn get_difficulty_multiplier(difficulty: Difficulty) -> f64 {
    match difficulty {
        Difficulty::Easy => 1.0,
        Difficulty::Medium => 1.5,
        Difficulty::Hard => 2.0,
        Difficulty::VeryHard => 3.0,
    }
}
```

#### Streak Multipliers
```rust
fn get_streak_multiplier(streak_days: u32) -> f64 {
    match streak_days {
        0..=3 => 1.0,
        4..=7 => 1.1,
        8..=14 => 1.2,
        15..=30 => 1.3,
        _ => 1.5,
    }
}
```

#### Accuracy Multipliers
```rust
fn get_accuracy_multiplier(accuracy_pct: f64) -> f64 {
    match accuracy_pct {
        a if a >= 100.0 => 1.5,
        a if a >= 90.0 => 1.3,
        a if a >= 80.0 => 1.1,
        a if a >= 70.0 => 1.0,
        a if a >= 60.0 => 0.8,
        _ => 0.5,
    }
}
```

#### Complete Calculation
```rust
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
```

#### Example Calculations
```
Example 1: Medium quiz, 10-day streak, 90% accuracy
50 × 1.5 × 1.2 × 1.3 = 117 XP

Example 2: Hard quiz, no streak, perfect score
50 × 2.0 × 1.0 × 1.5 = 150 XP

Example 3: Easy quiz, 30-day streak, 75% accuracy
50 × 1.0 × 1.5 × 1.0 = 75 XP
```

### 2.4 Mastery Score Update Algorithm

Based on exponential moving average from `balance_report.md`:

#### Formula
```
new_score = old_score + learning_rate × (performance - old_score)
```

#### Parameters
```rust
const LEARNING_RATE: f64 = 0.25; // 25% weight on new performance
const MASTERY_FLOOR: f64 = 0.30; // Never drops below 30%
```

#### Implementation
```rust
pub fn update_mastery_score(
    conn: &Connection,
    user_id: &str,
    skill_id: &str,
    performance: f64, // 0.0-1.0 (quiz score as decimal)
) -> Result<f64> {
    // Get current mastery score
    let current_score: f64 = conn.query_row(
        "SELECT score FROM mastery_scores
         WHERE user_id = ?1 AND skill_id = ?2",
        params![user_id, skill_id],
        |row| row.get(0),
    ).unwrap_or(0.0); // Default to 0 if new skill

    // Apply exponential moving average
    let new_score = current_score + LEARNING_RATE * (performance - current_score);

    // Ensure within bounds [0.0, 1.0]
    let new_score = new_score.clamp(0.0, 1.0);

    // Update database
    conn.execute(
        "INSERT INTO mastery_scores (user_id, skill_id, score, last_updated)
         VALUES (?1, ?2, ?3, datetime('now'))
         ON CONFLICT(user_id, skill_id) DO UPDATE SET
         score = ?3,
         last_updated = datetime('now')",
        params![user_id, skill_id, new_score],
    )?;

    Ok(new_score)
}

pub fn update_all_quiz_skills(
    conn: &Connection,
    user_id: &str,
    quiz: &Quiz,
    score_percentage: f64,
) -> Result<HashMap<String, f64>> {
    let mut updated_skills = HashMap::new();

    for skill_id in &quiz.skills {
        let new_score = update_mastery_score(
            conn,
            user_id,
            skill_id,
            score_percentage / 100.0, // Convert to 0.0-1.0
        )?;

        updated_skills.insert(skill_id.clone(), new_score);
    }

    Ok(updated_skills)
}
```

#### Example Progression
```
Skill: "ownership"
Initial: 0.0

Quiz 1 (80% score):
new = 0.0 + 0.25 × (0.8 - 0.0) = 0.20

Quiz 2 (90% score):
new = 0.20 + 0.25 × (0.9 - 0.20) = 0.375

Quiz 3 (100% score):
new = 0.375 + 0.25 × (1.0 - 0.375) = 0.531

Quiz 4 (95% score):
new = 0.531 + 0.25 × (0.95 - 0.531) = 0.636

After 10 high-performing quizzes:
Approaches ~0.85-0.90 (asymptotic to performance level)
```

### 2.5 Quiz Retake Handling

#### Strategy: Unlimited Retakes with Diminishing Returns

**Rationale**:
- Learning platform should encourage mastery, not punish mistakes
- But retakes shouldn't allow XP farming
- Balance: Allow retakes, but reduce rewards

#### Implementation
```rust
pub struct QuizAttemptPolicy {
    pub allow_retakes: bool,
    pub xp_multiplier_per_attempt: Vec<f64>,
    pub mastery_multiplier_per_attempt: Vec<f64>,
}

impl QuizAttemptPolicy {
    pub fn default() -> Self {
        Self {
            allow_retakes: true,
            xp_multiplier_per_attempt: vec![
                1.0,  // Attempt 1: full XP
                0.5,  // Attempt 2: half XP
                0.25, // Attempt 3: quarter XP
                0.1,  // Attempt 4+: minimal XP
            ],
            mastery_multiplier_per_attempt: vec![
                1.0,  // Attempt 1: full mastery update
                0.75, // Attempt 2: reduced mastery impact
                0.5,  // Attempt 3: further reduced
                0.25, // Attempt 4+: minimal mastery impact
            ],
        }
    }

    pub fn get_xp_multiplier(&self, attempt_number: usize) -> f64 {
        if attempt_number == 0 { return 0.0; }
        let idx = attempt_number - 1;
        if idx < self.xp_multiplier_per_attempt.len() {
            self.xp_multiplier_per_attempt[idx]
        } else {
            *self.xp_multiplier_per_attempt.last().unwrap()
        }
    }

    pub fn get_mastery_multiplier(&self, attempt_number: usize) -> f64 {
        if attempt_number == 0 { return 0.0; }
        let idx = attempt_number - 1;
        if idx < self.mastery_multiplier_per_attempt.len() {
            self.mastery_multiplier_per_attempt[idx]
        } else {
            *self.mastery_multiplier_per_attempt.last().unwrap()
        }
    }
}
```

#### Retake Flow
```rust
#[tauri::command]
pub async fn submit_quiz_attempt(
    user_id: String,
    quiz_id: String,
    answers: HashMap<String, String>,
    state: State<'_, AppState>,
) -> Result<QuizResult, String> {
    let db = state.db.lock().unwrap();

    // Count previous attempts
    let attempt_count: i32 = db.query_row(
        "SELECT COUNT(*) FROM quiz_attempts
         WHERE user_id = ?1 AND quiz_id = ?2",
        params![user_id, quiz_id],
        |row| row.get(0),
    ).unwrap_or(0);

    let attempt_number = attempt_count + 1;

    // Grade quiz
    let quiz = load_quiz(&quiz_id)?;
    let (score, correct_count, total) = grade_quiz(&quiz, &answers);
    let score_percentage = (score as f64 / total as f64) * 100.0;

    // Calculate XP with retake penalty
    let policy = QuizAttemptPolicy::default();
    let base_xp = calculate_quiz_xp(
        quiz.difficulty,
        score_percentage,
        get_user_streak(&db, &user_id)?,
    );
    let xp_earned = (base_xp as f64 * policy.get_xp_multiplier(attempt_number as usize)) as i32;

    // Update mastery with retake penalty
    let mastery_mult = policy.get_mastery_multiplier(attempt_number as usize);
    let effective_performance = (score_percentage / 100.0) * mastery_mult;

    let updated_masteries = update_all_quiz_skills(
        &db,
        &user_id,
        &quiz,
        effective_performance * 100.0,
    )?;

    // Record attempt
    save_quiz_attempt(&db, &user_id, &quiz_id, &answers, score, attempt_number)?;

    // Award XP
    award_xp(&db, &user_id, xp_earned)?;

    // Mark complete if passed
    let passed = score_percentage >= quiz.passing_score as f64;
    if passed {
        complete_quiz_node(&db, &user_id, &quiz_id)?;
    }

    Ok(QuizResult {
        score,
        total,
        score_percentage,
        passed,
        xp_earned,
        attempt_number,
        mastery_updates: updated_masteries,
        feedback: generate_feedback(&quiz, &answers),
    })
}
```

#### UI Indication
```tsx
{attemptNumber > 1 && (
  <Alert variant="warning">
    <AlertIcon />
    This is attempt #{attemptNumber}. XP reward reduced to {xpMultiplier * 100}%
  </Alert>
)}
```

### 2.6 Answer Validation Logic

#### Validation Strategy
```rust
pub fn grade_quiz(
    quiz: &Quiz,
    user_answers: &HashMap<String, String>,
) -> (i32, usize, usize) {
    let mut score = 0;
    let mut correct_count = 0;
    let total = quiz.questions.len();

    for question in &quiz.questions {
        let user_answer = user_answers.get(&question.id);

        let is_correct = match &question.correct_answer {
            CorrectAnswer::Single(correct) => {
                user_answer.map(|ans| ans == correct).unwrap_or(false)
            }
            CorrectAnswer::Multiple(correct_set) => {
                // Future: handle multiple correct answers
                // For now, assume single answer
                false
            }
        };

        if is_correct {
            score += question.points;
            correct_count += 1;
        }
    }

    (score, correct_count, total)
}

pub fn generate_feedback(
    quiz: &Quiz,
    user_answers: &HashMap<String, String>,
) -> Vec<QuestionFeedback> {
    quiz.questions.iter().map(|question| {
        let user_answer = user_answers.get(&question.id);
        let correct_answer = match &question.correct_answer {
            CorrectAnswer::Single(ans) => ans.clone(),
            CorrectAnswer::Multiple(ans) => ans.join(", "),
        };

        let is_correct = user_answer.map(|ans| ans == &correct_answer).unwrap_or(false);

        QuestionFeedback {
            question_id: question.id.clone(),
            user_answer: user_answer.cloned(),
            correct_answer,
            is_correct,
            explanation: question.explanation.clone(),
        }
    }).collect()
}
```

### 2.7 UI Components

#### Component Hierarchy
```
QuizPage
├── QuizHeader
│   ├── QuizTitle
│   ├── QuizDescription
│   ├── DifficultyBadge
│   └── TimerDisplay (if timed)
├── ProgressBar (current question / total)
├── QuizContent
│   └── QuestionCard
│       ├── QuestionPrompt (markdown)
│       ├── CodeSnippet (if present)
│       ├── AnswerOptions
│       │   ├── RadioOption (single choice)
│       │   └── CheckboxOption (multiple - future)
│       └── NavigationButtons
│           ├── PreviousQuestion
│           └── NextQuestion
├── QuizFooter
│   ├── SubmitButton
│   └── AttemptIndicator
└── ResultsModal
    ├── ScoreSummary
    ├── XPEarned
    ├── MasteryUpdates
    └── QuestionReview
        └── QuestionFeedbackCard
```

#### Component Props Interfaces

```tsx
// QuizPage.tsx
interface QuizPageProps {
  quizId: string
}

// QuestionCard.tsx
interface QuestionCardProps {
  question: Question
  selectedAnswer: string | null
  onAnswerChange: (questionId: string, answerId: string) => void
  isReviewMode?: boolean
  feedback?: QuestionFeedback
}

interface Question {
  id: string
  type: 'multiple_choice' | 'true_false' | 'code_output' | 'fill_blank'
  prompt: string
  code_snippet?: string
  options: Array<{ id: string; text: string }>
  points: number
}

// AnswerOptions.tsx
interface AnswerOptionsProps {
  options: Array<{ id: string; text: string }>
  selectedAnswer: string | null
  onSelect: (answerId: string) => void
  isReviewMode?: boolean
  correctAnswer?: string
  userAnswer?: string
}

// ResultsModal.tsx
interface ResultsModalProps {
  isOpen: boolean
  result: QuizResult
  onClose: () => void
  onRetake?: () => void
}

interface QuizResult {
  score: number
  total: number
  scorePercentage: number
  passed: boolean
  xpEarned: number
  attemptNumber: number
  masteryUpdates: Record<string, number>
  feedback: QuestionFeedback[]
}

// QuestionFeedbackCard.tsx
interface QuestionFeedbackCardProps {
  question: Question
  feedback: QuestionFeedback
}

interface QuestionFeedback {
  questionId: string
  userAnswer?: string
  correctAnswer: string
  isCorrect: boolean
  explanation: string
}

// TimerDisplay.tsx
interface TimerDisplayProps {
  totalSeconds: number
  onTimeUp: () => void
}
```

#### Component List
1. `QuizPage` - Main container
2. `QuizHeader` - Title, description, timer
3. `QuestionCard` - Individual question display
4. `AnswerOptions` - Radio/checkbox group
5. `CodeSnippet` - Syntax highlighted code
6. `QuizFooter` - Submit button, attempt counter
7. `ResultsModal` - Score summary and feedback
8. `QuestionFeedbackCard` - Individual question review
9. `TimerDisplay` - Countdown timer (if timed)
10. `ProgressBar` - Question progress indicator

### 2.8 Tauri Backend Commands

```rust
// src-tauri/src/commands/quiz.rs

#[tauri::command]
pub async fn get_quiz(quiz_id: String) -> Result<QuizData, String> {
    let content_tree = CONTENT_TREE.lock().unwrap();
    let quiz_path = content_tree.get_quiz_path(&quiz_id)
        .ok_or("Quiz not found")?;

    // Load quiz JSON
    let content = fs::read_to_string(quiz_path)
        .map_err(|e| format!("Failed to read quiz: {}", e))?;

    let quiz: Quiz = serde_json::from_str(&content)
        .map_err(|e| format!("Invalid quiz JSON: {}", e))?;

    Ok(quiz.into())
}

#[tauri::command]
pub async fn get_quiz_attempt_count(
    user_id: String,
    quiz_id: String,
    state: State<'_, AppState>,
) -> Result<i32, String> {
    let db = state.db.lock().unwrap();

    let count: i32 = db.query_row(
        "SELECT COUNT(*) FROM quiz_attempts
         WHERE user_id = ?1 AND quiz_id = ?2",
        params![user_id, quiz_id],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;

    Ok(count)
}

#[tauri::command]
pub async fn submit_quiz(
    user_id: String,
    quiz_id: String,
    answers: HashMap<String, String>,
    time_spent_seconds: i32,
    state: State<'_, AppState>,
) -> Result<QuizResult, String> {
    // Implementation from section 2.5
    submit_quiz_attempt(user_id, quiz_id, answers, state).await
}

#[tauri::command]
pub async fn get_quiz_history(
    user_id: String,
    quiz_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<QuizAttemptSummary>, String> {
    let db = state.db.lock().unwrap();

    let mut stmt = db.prepare(
        "SELECT id, score, submitted_at
         FROM quiz_attempts
         WHERE user_id = ?1 AND quiz_id = ?2
         ORDER BY submitted_at DESC"
    ).map_err(|e| e.to_string())?;

    let attempts = stmt.query_map(params![user_id, quiz_id], |row| {
        Ok(QuizAttemptSummary {
            id: row.get(0)?,
            score: row.get(1)?,
            submitted_at: row.get(2)?,
        })
    }).map_err(|e| e.to_string())?
      .collect::<Result<Vec<_>, _>>()
      .map_err(|e| e.to_string())?;

    Ok(attempts)
}
```

### 2.9 State Flow Diagram

```
[Start Quiz]
      ↓
[Load quiz from JSON]
      ↓
[Check attempt count] → [Show retake warning if > 1]
      ↓
[Display question 1]
      ↓
[User selects answers] → [Save to local state]
      ↓
[Navigate through questions]
      ↓
[User clicks Submit] → [Validate all answered]
      ↓
[Send to backend for grading]
      ↓
[Calculate score]
      ↓
[Calculate XP (with retake penalty)]
      ↓
[Update mastery scores]
      ↓
[Save attempt to DB]
      ↓
[Award XP]
      ↓
[Check if passed] → [Mark node complete]
      ↓
[Return results to frontend]
      ↓
[Show results modal]
      ↓
[User reviews feedback]
      ↓
[Options: Retake / Continue]
```

### 2.10 Acceptance Criteria Checklist

- [ ] Can load quiz from JSON
- [ ] All question types (multiple choice, true/false, code output) display correctly
- [ ] Code snippets syntax highlighted
- [ ] Can navigate between questions
- [ ] Progress indicator shows current question / total
- [ ] Timer counts down if quiz is timed
- [ ] Submit button disabled until all questions answered
- [ ] Grading calculates score correctly
- [ ] XP calculation uses correct formulas (difficulty × streak × accuracy)
- [ ] Retake penalty applied correctly (50%, 25%, 10%)
- [ ] Mastery scores update for all relevant skills
- [ ] Quiz marked complete if passed (≥70% default)
- [ ] Results modal shows score, XP, and mastery updates
- [ ] Question feedback shows correct/incorrect with explanations
- [ ] Can retake quiz (with warning)
- [ ] Next nodes unlock if quiz passed
- [ ] Attempt history saved to database

---

## Milestone 2.3: Progress Dashboard

**Time Estimate**: 2 days
**Priority**: Medium (enhances motivation but not blocking)

### 3.1 Charting Library Research

#### Winner: Recharts
(See Technology Stack section for full rationale)

**Key Features Used**:
- `<LineChart>` for XP over time
- `<BarChart>` for weekly activity
- `<RadarChart>` for mastery visualization
- `<ProgressBar>` (custom) for level progression

### 3.2 XP/Level Progression Formula

From `balance_report.md`:

#### Formula
```
Level N requires: 100 × N^1.5 cumulative XP
```

#### Implementation
```rust
pub fn calculate_level(total_xp: i32) -> u32 {
    let mut level = 1;
    while xp_required_for_level(level + 1) <= total_xp {
        level += 1;
    }
    level
}

pub fn xp_required_for_level(level: u32) -> i32 {
    if level <= 1 {
        return 0;
    }
    (100.0 * (level as f64).powf(1.5)).round() as i32
}

pub fn xp_to_next_level(current_xp: i32) -> (i32, i32) {
    let current_level = calculate_level(current_xp);
    let next_level_xp = xp_required_for_level(current_level + 1);
    let current_level_xp = xp_required_for_level(current_level);

    let xp_needed = next_level_xp - current_xp;
    let xp_progress = current_xp - current_level_xp;
    let xp_total_for_level = next_level_xp - current_level_xp;

    (xp_progress, xp_total_for_level)
}
```

#### Level Thresholds Table
```rust
pub fn get_level_thresholds() -> Vec<(u32, i32)> {
    (1..=20).map(|level| {
        (level, xp_required_for_level(level))
    }).collect()
}

// Example output:
// Level 1: 0 XP
// Level 2: 283 XP (+283)
// Level 3: 520 XP (+237)
// Level 5: 1,118 XP
// Level 10: 3,162 XP
// Level 15: 5,809 XP
// Level 20: 8,944 XP
```

### 3.3 Streak Calculation with Grace Period

From `balance_report.md`:

#### Rules
```
Same day: Streak continues (no change)
Next day: Streak increments (+1)
1-day gap: Grace period (maintains streak with warning)
2+ day gap: Streak resets to 1
```

#### Grace Period: 5 days (updated from 3 days in balance report)

#### Implementation
```rust
use chrono::{DateTime, Utc, Duration, Datelike};

pub struct StreakInfo {
    pub current_streak: u32,
    pub is_grace_period: bool,
    pub grace_days_remaining: u32,
    pub last_activity: DateTime<Utc>,
}

pub fn calculate_streak(
    conn: &Connection,
    user_id: &str,
) -> Result<StreakInfo> {
    // Get last activity date
    let last_activity: String = conn.query_row(
        "SELECT last_activity FROM users WHERE id = ?1",
        params![user_id],
        |row| row.get(0),
    )?;

    let last_activity_dt = DateTime::parse_from_rfc3339(&last_activity)?
        .with_timezone(&Utc);

    let now = Utc::now();
    let days_since = (now - last_activity_dt).num_days();

    // Get current streak from DB
    let current_streak: u32 = conn.query_row(
        "SELECT current_streak FROM users WHERE id = ?1",
        params![user_id],
        |row| row.get(0),
    )?;

    const GRACE_PERIOD_DAYS: i64 = 5;

    let (new_streak, is_grace, grace_remaining) = match days_since {
        0 => {
            // Same day - no change
            (current_streak, false, 0)
        }
        1 => {
            // Next day - increment
            (current_streak + 1, false, 0)
        }
        d if d <= GRACE_PERIOD_DAYS => {
            // Within grace period - maintain but warn
            (current_streak, true, (GRACE_PERIOD_DAYS - d) as u32)
        }
        _ => {
            // Beyond grace period - reset
            (1, false, 0)
        }
    };

    Ok(StreakInfo {
        current_streak: new_streak,
        is_grace_period: is_grace,
        grace_days_remaining: grace_remaining,
        last_activity: last_activity_dt,
    })
}

pub fn update_streak_on_activity(
    conn: &Connection,
    user_id: &str,
) -> Result<StreakInfo> {
    let streak_info = calculate_streak(conn, user_id)?;

    // Update DB
    conn.execute(
        "UPDATE users
         SET current_streak = ?1,
             last_activity = datetime('now')
         WHERE id = ?2",
        params![streak_info.current_streak, user_id],
    )?;

    Ok(streak_info)
}
```

#### Grace Period Logic Examples
```
Last activity: Jan 1
Current streak: 10

Jan 1: Activity → Streak = 10 (same day)
Jan 2: Activity → Streak = 11 (next day)
Jan 3: No activity → Streak = 11 (1 day gap, grace)
Jan 4: Activity → Streak = 12 (resumed within grace)

Alternative:
Jan 2: No activity → Grace starts (4 days left)
Jan 3: No activity → Grace continues (3 days left)
Jan 4: No activity → Grace continues (2 days left)
Jan 5: No activity → Grace continues (1 day left)
Jan 6: No activity → Grace continues (0 days left)
Jan 7: No activity → Streak resets to 0
Jan 8: Activity → Streak = 1 (fresh start)
```

### 3.4 Mastery Visualization

#### Data Structure for Radar Chart
```rust
#[derive(Serialize)]
pub struct MasteryData {
    pub skills: Vec<SkillMastery>,
    pub overall_average: f64,
    pub strongest_skill: String,
    pub weakest_skill: String,
}

#[derive(Serialize)]
pub struct SkillMastery {
    pub skill_id: String,
    pub skill_name: String,
    pub score: f64, // 0.0-1.0
    pub last_updated: DateTime<Utc>,
    pub days_since_update: i64,
    pub decay_applied: f64, // Amount of decay
}

pub fn get_mastery_data(
    conn: &Connection,
    user_id: &str,
) -> Result<MasteryData> {
    // Get all mastery scores
    let mut stmt = conn.prepare(
        "SELECT skill_id, score, last_updated
         FROM mastery_scores
         WHERE user_id = ?1
         ORDER BY skill_id"
    )?;

    let now = Utc::now();
    let mut skills: Vec<SkillMastery> = stmt.query_map(params![user_id], |row| {
        let skill_id: String = row.get(0)?;
        let score: f64 = row.get(1)?;
        let last_updated_str: String = row.get(2)?;
        let last_updated = DateTime::parse_from_rfc3339(&last_updated_str)?
            .with_timezone(&Utc);

        let days_since = (now - last_updated).num_days();

        // Calculate decay
        let decay = calculate_decay(score, days_since);
        let current_score = score * decay;

        Ok(SkillMastery {
            skill_id: skill_id.clone(),
            skill_name: get_skill_name(&skill_id),
            score: current_score,
            last_updated,
            days_since_update: days_since,
            decay_applied: 1.0 - decay,
        })
    })?.collect::<Result<Vec<_>, _>>()?;

    // Calculate statistics
    let overall_avg = skills.iter().map(|s| s.score).sum::<f64>() / skills.len() as f64;
    let strongest = skills.iter().max_by(|a, b| a.score.partial_cmp(&b.score).unwrap()).unwrap();
    let weakest = skills.iter().min_by(|a, b| a.score.partial_cmp(&b.score).unwrap()).unwrap();

    Ok(MasteryData {
        skills,
        overall_average: overall_avg,
        strongest_skill: strongest.skill_name.clone(),
        weakest_skill: weakest.skill_name.clone(),
    })
}

fn calculate_decay(base_score: f64, days_inactive: i64) -> f64 {
    const GRACE_PERIOD: i64 = 5;
    const DECAY_RATE: f64 = 0.05; // 5% per day
    const MIN_SCORE: f64 = 0.30; // 30% floor

    if days_inactive <= GRACE_PERIOD {
        return 1.0; // No decay
    }

    let decay_days = days_inactive - GRACE_PERIOD;
    let decay_factor = (-DECAY_RATE * decay_days as f64).exp();

    // Ensure doesn't drop below floor
    decay_factor.max(MIN_SCORE / base_score)
}
```

#### Radar Chart Component
```tsx
import { RadarChart, PolarGrid, PolarAngleAxis, PolarRadiusAxis, Radar, Legend } from 'recharts'

interface MasteryRadarProps {
  masteryData: MasteryData
}

export const MasteryRadar: React.FC<MasteryRadarProps> = ({ masteryData }) => {
  const chartData = masteryData.skills.map(skill => ({
    skill: skill.skill_name,
    mastery: (skill.score * 100).toFixed(0), // Convert to percentage
    fullMark: 100,
  }))

  return (
    <div className="p-4">
      <h3 className="text-xl font-bold mb-2">Skill Mastery</h3>
      <p className="text-sm text-gray-600 mb-4">
        Overall Average: {(masteryData.overall_average * 100).toFixed(1)}%
      </p>

      <RadarChart width={500} height={400} data={chartData}>
        <PolarGrid />
        <PolarAngleAxis dataKey="skill" />
        <PolarRadiusAxis angle={90} domain={[0, 100]} />
        <Radar
          name="Mastery"
          dataKey="mastery"
          stroke="#8884d8"
          fill="#8884d8"
          fillOpacity={0.6}
        />
        <Legend />
      </RadarChart>

      <div className="mt-4 text-sm">
        <p className="text-green-600">
          <strong>Strongest:</strong> {masteryData.strongest_skill}
        </p>
        <p className="text-orange-600">
          <strong>Needs Practice:</strong> {masteryData.weakest_skill}
        </p>
      </div>
    </div>
  )
}
```

### 3.5 Dashboard Components & Data Requirements

#### Component Hierarchy
```
ProgressDashboard
├── DashboardHeader
│   ├── UserAvatar
│   ├── LevelDisplay
│   └── XPProgress
├── StatsGrid
│   ├── StreakCard
│   ├── TotalXPCard
│   ├── CompletionCard
│   └── TimeSpentCard
├── ChartsSection
│   ├── XPOverTimeChart
│   ├── MasteryRadarChart
│   └── WeeklyActivityChart
├── RecentActivityFeed
│   └── ActivityItem[]
└── BadgesPreview
    └── BadgeMiniCard[]
```

#### Data Requirements by Component

##### 1. XPProgress Component
**Data Needed**:
```tsx
interface XPProgressData {
  currentXP: number
  currentLevel: number
  xpForCurrentLevel: number
  xpForNextLevel: number
  xpProgressInLevel: number
  percentToNextLevel: number
}
```

**Backend Command**:
```rust
#[tauri::command]
pub async fn get_xp_progress(
    user_id: String,
    state: State<'_, AppState>,
) -> Result<XPProgressData, String> {
    let db = state.db.lock().unwrap();

    let total_xp: i32 = get_user_xp(&db, &user_id)?;
    let level = calculate_level(total_xp);
    let (xp_progress, xp_needed) = xp_to_next_level(total_xp);

    Ok(XPProgressData {
        current_xp: total_xp,
        current_level: level,
        xp_for_current_level: xp_required_for_level(level),
        xp_for_next_level: xp_required_for_level(level + 1),
        xp_progress_in_level: xp_progress,
        percent_to_next_level: (xp_progress as f64 / xp_needed as f64 * 100.0) as u32,
    })
}
```

##### 2. StreakCard Component
**Data Needed**:
```tsx
interface StreakData {
  currentStreak: number
  longestStreak: number
  isGracePeriod: boolean
  graceDaysRemaining: number
  streakMultiplier: number
}
```

**Backend Command**:
```rust
#[tauri::command]
pub async fn get_streak_data(
    user_id: String,
    state: State<'_, AppState>,
) -> Result<StreakData, String> {
    let db = state.db.lock().unwrap();

    let streak_info = calculate_streak(&db, &user_id)?;
    let longest: u32 = db.query_row(
        "SELECT MAX(current_streak) FROM users WHERE id = ?1",
        params![user_id],
        |row| row.get(0),
    ).unwrap_or(streak_info.current_streak);

    Ok(StreakData {
        current_streak: streak_info.current_streak,
        longest_streak: longest,
        is_grace_period: streak_info.is_grace_period,
        grace_days_remaining: streak_info.grace_days_remaining,
        streak_multiplier: get_streak_multiplier(streak_info.current_streak),
    })
}
```

##### 3. XPOverTimeChart Component
**Data Needed**:
```tsx
interface XPHistoryData {
  data_points: Array<{
    date: string
    xp: number
    level: number
  }>
  total_days: number
}
```

**Backend Command**:
```rust
#[tauri::command]
pub async fn get_xp_history(
    user_id: String,
    days: i32, // Last N days
    state: State<'_, AppState>,
) -> Result<XPHistoryData, String> {
    let db = state.db.lock().unwrap();

    // Get all XP-earning events
    let mut stmt = db.prepare(
        "SELECT DATE(submitted_at) as date, SUM(score) as daily_xp
         FROM quiz_attempts
         WHERE user_id = ?1
           AND submitted_at >= date('now', '-' || ?2 || ' days')
         GROUP BY DATE(submitted_at)
         ORDER BY date ASC"
    )?;

    let data_points: Vec<XPDataPoint> = stmt.query_map(params![user_id, days], |row| {
        let date: String = row.get(0)?;
        let xp: i32 = row.get(1)?;
        Ok(XPDataPoint {
            date,
            xp,
            level: calculate_level(xp),
        })
    })?.collect::<Result<Vec<_>, _>>()?;

    Ok(XPHistoryData {
        data_points,
        total_days: days,
    })
}
```

##### 4. MasteryRadarChart Component
**Data Needed**: (Already defined in section 3.4)

##### 5. RecentActivityFeed Component
**Data Needed**:
```tsx
interface RecentActivity {
  activities: Array<{
    id: string
    type: 'lecture' | 'quiz' | 'challenge' | 'checkpoint'
    title: string
    xp_earned: number
    completed_at: string
    icon: string
  }>
}
```

**Backend Command**:
```rust
#[tauri::command]
pub async fn get_recent_activity(
    user_id: String,
    limit: i32,
    state: State<'_, AppState>,
) -> Result<Vec<ActivityItem>, String> {
    let db = state.db.lock().unwrap();

    let mut stmt = db.prepare(
        "SELECT node_id, completed_at
         FROM node_progress
         WHERE user_id = ?1 AND status = 'Completed'
         ORDER BY completed_at DESC
         LIMIT ?2"
    )?;

    let activities: Vec<ActivityItem> = stmt.query_map(params![user_id, limit], |row| {
        let node_id: String = row.get(0)?;
        let completed_at: String = row.get(1)?;

        // Get node details from content tree
        let tree = CONTENT_TREE.lock().unwrap();
        let node = tree.find_node(&node_id).ok_or("Node not found")?;

        Ok(ActivityItem {
            id: node_id,
            activity_type: node.node_type,
            title: node.title,
            xp_earned: node.xp_reward,
            completed_at,
            icon: get_icon_for_type(&node.node_type),
        })
    })?.collect::<Result<Vec<_>, _>>()?;

    Ok(activities)
}
```

##### 6. StatsCards (Total XP, Completion, Time Spent)
**Data Needed**:
```tsx
interface DashboardStats {
  total_xp: number
  nodes_completed: number
  total_nodes: number
  completion_percentage: number
  total_time_minutes: number
  average_session_minutes: number
}
```

**Backend Command**:
```rust
#[tauri::command]
pub async fn get_dashboard_stats(
    user_id: String,
    state: State<'_, AppState>,
) -> Result<DashboardStats, String> {
    let db = state.db.lock().unwrap();

    let total_xp: i32 = get_user_xp(&db, &user_id)?;

    let nodes_completed: i32 = db.query_row(
        "SELECT COUNT(*) FROM node_progress
         WHERE user_id = ?1 AND status = 'Completed'",
        params![user_id],
        |row| row.get(0),
    )?;

    let total_nodes = CONTENT_TREE.lock().unwrap().total_nodes();

    let total_time: i32 = db.query_row(
        "SELECT SUM(time_spent_mins) FROM node_progress WHERE user_id = ?1",
        params![user_id],
        |row| row.get(0),
    ).unwrap_or(0);

    let session_count: i32 = db.query_row(
        "SELECT COUNT(DISTINCT DATE(completed_at))
         FROM node_progress
         WHERE user_id = ?1 AND status = 'Completed'",
        params![user_id],
        |row| row.get(0),
    ).unwrap_or(1);

    Ok(DashboardStats {
        total_xp,
        nodes_completed,
        total_nodes,
        completion_percentage: (nodes_completed as f64 / total_nodes as f64 * 100.0) as u32,
        total_time_minutes: total_time,
        average_session_minutes: total_time / session_count,
    })
}
```

### 3.6 Component Props Interfaces

```tsx
// ProgressDashboard.tsx
interface ProgressDashboardProps {
  userId: string
}

// XPProgress.tsx
interface XPProgressProps {
  currentXP: number
  currentLevel: number
  xpForNextLevel: number
  percentToNextLevel: number
}

// StreakCard.tsx
interface StreakCardProps {
  currentStreak: number
  longestStreak: number
  isGracePeriod: boolean
  graceDaysRemaining: number
  multiplier: number
}

// XPOverTimeChart.tsx
interface XPOverTimeChartProps {
  data: Array<{ date: string; xp: number; level: number }>
  height?: number
  width?: number
}

// MasteryRadarChart.tsx
interface MasteryRadarChartProps {
  masteryData: MasteryData
  height?: number
  width?: number
}

// RecentActivityFeed.tsx
interface RecentActivityFeedProps {
  activities: ActivityItem[]
  maxItems?: number
}

interface ActivityItem {
  id: string
  type: 'lecture' | 'quiz' | 'challenge' | 'checkpoint'
  title: string
  xpEarned: number
  completedAt: string
  icon: string
}

// StatsCard.tsx (reusable)
interface StatsCardProps {
  title: string
  value: string | number
  subtitle?: string
  icon: React.ReactNode
  color?: 'blue' | 'green' | 'orange' | 'purple'
}
```

### 3.7 Component List

1. `ProgressDashboard` - Main container
2. `DashboardHeader` - User info and level
3. `XPProgress` - Progress bar to next level
4. `StreakCard` - Streak display with grace warning
5. `StatsCard` - Reusable stat display (total XP, completion, time)
6. `XPOverTimeChart` - Line chart of XP growth
7. `MasteryRadarChart` - Radar chart of skill mastery
8. `WeeklyActivityChart` - Bar chart of weekly activity
9. `RecentActivityFeed` - List of recent completions
10. `ActivityItem` - Individual activity row
11. `BadgesPreview` - Mini preview of badges (links to full page)

### 3.8 Acceptance Criteria Checklist

- [ ] Dashboard loads without errors
- [ ] XP progress bar displays correctly
- [ ] Level displays correctly
- [ ] XP to next level calculates correctly
- [ ] Streak counter shows current streak
- [ ] Grace period warning appears when in grace period
- [ ] Streak multiplier displays correctly
- [ ] XP over time chart shows historical data
- [ ] Mastery radar chart displays all skills
- [ ] Radar chart updates after quiz completion
- [ ] Recent activity feed shows last 10 items
- [ ] Activity items show correct icons
- [ ] Total XP stat matches database
- [ ] Completion percentage calculates correctly
- [ ] Time spent displays in human-readable format
- [ ] All data updates in real-time after completing activities

---

## Milestone 2.4: Daily Session Queue

**Time Estimate**: 2 days
**Priority**: High (core UX flow)

### 4.1 Session Planning Algorithm

#### Algorithm: Adaptive Session Planner

**Goals**:
1. Recommend optimal mix of activities (lecture + quiz + challenge)
2. Prioritize unlocked content the user hasn't seen
3. Balance difficulty (not all hard content)
4. Respect time constraints (estimate session length)
5. Include review items if due

**Strategy**: Greedy algorithm with priority scoring

```rust
pub struct SessionPlanner {
    content_tree: Arc<Mutex<ContentTree>>,
    db: Arc<Mutex<Connection>>,
}

#[derive(Debug, Serialize)]
pub struct SessionPlan {
    pub activities: Vec<PlannedActivity>,
    pub estimated_minutes: u32,
    pub total_xp_potential: i32,
    pub skills_covered: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct PlannedActivity {
    pub node_id: String,
    pub node_type: NodeType,
    pub title: String,
    pub difficulty: Difficulty,
    pub xp_reward: i32,
    pub estimated_minutes: u32,
    pub is_review: bool,
}

impl SessionPlanner {
    pub fn plan_daily_session(
        &self,
        user_id: &str,
        target_minutes: u32, // User's available time
    ) -> Result<SessionPlan> {
        let db = self.db.lock().unwrap();
        let tree = self.content_tree.lock().unwrap();

        let mut activities = Vec::new();
        let mut total_minutes = 0;
        let mut total_xp = 0;
        let mut skills_covered = HashSet::new();

        // Step 1: Check for due review items (priority)
        let reviews = self.get_due_reviews(&db, user_id)?;
        for review in reviews.iter().take(1) { // Max 1 review per session
            if total_minutes + review.estimated_minutes <= target_minutes {
                activities.push(review.clone());
                total_minutes += review.estimated_minutes;
                total_xp += review.xp_reward;
                skills_covered.extend(review.skills.clone());
            }
        }

        // Step 2: Get next available lecture (if any)
        if let Some(lecture) = self.get_next_lecture(&db, &tree, user_id)? {
            if total_minutes + lecture.estimated_minutes <= target_minutes {
                activities.push(lecture.clone());
                total_minutes += lecture.estimated_minutes;
                total_xp += lecture.xp_reward;
                skills_covered.extend(lecture.skills.clone());
            }
        }

        // Step 3: Get quiz for recent lecture
        if let Some(quiz) = self.get_related_quiz(&db, &tree, user_id, &activities)? {
            if total_minutes + quiz.estimated_minutes <= target_minutes {
                activities.push(quiz.clone());
                total_minutes += quiz.estimated_minutes;
                total_xp += quiz.xp_reward;
                skills_covered.extend(quiz.skills.clone());
            }
        }

        // Step 4: Add mini-challenge if time allows
        if let Some(challenge) = self.get_next_challenge(&db, &tree, user_id)? {
            if total_minutes + challenge.estimated_minutes <= target_minutes {
                activities.push(challenge.clone());
                total_minutes += challenge.estimated_minutes;
                total_xp += challenge.xp_reward;
                skills_covered.extend(challenge.skills.clone());
            }
        }

        // Step 5: Fill remaining time with short activities
        while total_minutes < target_minutes {
            if let Some(filler) = self.get_filler_activity(&db, &tree, user_id, target_minutes - total_minutes)? {
                activities.push(filler.clone());
                total_minutes += filler.estimated_minutes;
                total_xp += filler.xp_reward;
                skills_covered.extend(filler.skills.clone());
            } else {
                break; // No more activities available
            }
        }

        Ok(SessionPlan {
            activities,
            estimated_minutes: total_minutes,
            total_xp_potential: total_xp,
            skills_covered: skills_covered.into_iter().collect(),
        })
    }

    fn get_next_lecture(
        &self,
        db: &Connection,
        tree: &ContentTree,
        user_id: &str,
    ) -> Result<Option<PlannedActivity>> {
        // Find first unlocked, incomplete lecture
        let unlocked = self.get_unlocked_nodes(db, user_id)?;

        for node_id in unlocked {
            let node = tree.find_node(&node_id)?;
            if node.node_type == NodeType::Lecture {
                let status = self.get_node_status(db, user_id, &node_id)?;
                if status != NodeStatus::Completed {
                    return Ok(Some(PlannedActivity {
                        node_id: node.id.clone(),
                        node_type: node.node_type,
                        title: node.title.clone(),
                        difficulty: node.difficulty,
                        xp_reward: node.xp_reward,
                        estimated_minutes: node.estimated_minutes,
                        is_review: false,
                    }));
                }
            }
        }

        Ok(None)
    }

    fn get_related_quiz(
        &self,
        db: &Connection,
        tree: &ContentTree,
        user_id: &str,
        activities: &[PlannedActivity],
    ) -> Result<Option<PlannedActivity>> {
        // Find quiz that follows most recent lecture
        for activity in activities.iter().rev() {
            if activity.node_type == NodeType::Lecture {
                // Find quiz that depends on this lecture
                let dependents = tree.get_dependent_nodes(&activity.node_id);
                for dep in dependents {
                    if dep.node_type == NodeType::Quiz {
                        let status = self.get_node_status(db, user_id, &dep.id)?;
                        if status != NodeStatus::Completed {
                            return Ok(Some(PlannedActivity {
                                node_id: dep.id.clone(),
                                node_type: dep.node_type,
                                title: dep.title.clone(),
                                difficulty: dep.difficulty,
                                xp_reward: dep.xp_reward,
                                estimated_minutes: dep.estimated_minutes,
                                is_review: false,
                            }));
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    fn get_due_reviews(
        &self,
        db: &Connection,
        user_id: &str,
    ) -> Result<Vec<PlannedActivity>> {
        // Get quizzes due for review
        let mut stmt = db.prepare(
            "SELECT quiz_id FROM review_items
             WHERE user_id = ?1 AND due_date <= date('now')
             ORDER BY due_date ASC"
        )?;

        let reviews: Vec<PlannedActivity> = stmt.query_map(params![user_id], |row| {
            let quiz_id: String = row.get(0)?;
            // ... fetch quiz details and convert to PlannedActivity
            Ok(activity)
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(reviews)
    }
}
```

#### Session Types

**Type 1: Standard Session (30 minutes)**
```
1. Review quiz (if due) - 10 min
2. New lecture - 10 min
3. Quiz - 10 min
```

**Type 2: Learning Session (60 minutes)**
```
1. Lecture 1 - 15 min
2. Quiz 1 - 10 min
3. Lecture 2 - 15 min
4. Mini-challenge - 20 min
```

**Type 3: Practice Session (20 minutes)**
```
1. Review quiz 1 - 10 min
2. Review quiz 2 - 10 min
```

### 4.2 Session State Machine

#### States
```
NotStarted → InProgress → Paused → InProgress → Completed
```

#### State Definitions
```rust
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum SessionState {
    NotStarted,
    InProgress {
        current_activity_index: usize,
        started_at: DateTime<Utc>,
    },
    Paused {
        current_activity_index: usize,
        paused_at: DateTime<Utc>,
        time_elapsed_ms: i64,
    },
    Completed {
        completed_at: DateTime<Utc>,
        total_xp_earned: i32,
        total_time_ms: i64,
    },
}

#[derive(Debug, Serialize)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub plan: SessionPlan,
    pub state: SessionState,
    pub completed_activities: Vec<String>,
    pub xp_earned: i32,
}
```

#### State Transitions
```rust
impl Session {
    pub fn start(&mut self) -> Result<()> {
        match self.state {
            SessionState::NotStarted => {
                self.state = SessionState::InProgress {
                    current_activity_index: 0,
                    started_at: Utc::now(),
                };
                Ok(())
            }
            _ => Err("Session already started".into()),
        }
    }

    pub fn pause(&mut self) -> Result<()> {
        match &self.state {
            SessionState::InProgress { current_activity_index, started_at } => {
                let elapsed = (Utc::now() - started_at).num_milliseconds();
                self.state = SessionState::Paused {
                    current_activity_index: *current_activity_index,
                    paused_at: Utc::now(),
                    time_elapsed_ms: elapsed,
                };
                Ok(())
            }
            _ => Err("Session not in progress".into()),
        }
    }

    pub fn resume(&mut self) -> Result<()> {
        match &self.state {
            SessionState::Paused { current_activity_index, time_elapsed_ms, .. } => {
                // Adjust started_at to account for paused time
                let resumed_start = Utc::now() - Duration::milliseconds(*time_elapsed_ms);
                self.state = SessionState::InProgress {
                    current_activity_index: *current_activity_index,
                    started_at: resumed_start,
                };
                Ok(())
            }
            _ => Err("Session not paused".into()),
        }
    }

    pub fn complete_activity(&mut self, activity_id: &str, xp_earned: i32) -> Result<()> {
        match &self.state {
            SessionState::InProgress { current_activity_index, started_at } => {
                self.completed_activities.push(activity_id.to_string());
                self.xp_earned += xp_earned;

                let next_index = current_activity_index + 1;

                if next_index >= self.plan.activities.len() {
                    // Session complete
                    let total_time = (Utc::now() - started_at).num_milliseconds();
                    self.state = SessionState::Completed {
                        completed_at: Utc::now(),
                        total_xp_earned: self.xp_earned,
                        total_time_ms: total_time,
                    };
                } else {
                    // Move to next activity
                    self.state = SessionState::InProgress {
                        current_activity_index: next_index,
                        started_at: *started_at,
                    };
                }

                Ok(())
            }
            _ => Err("Session not in progress".into()),
        }
    }
}
```

#### State Diagram
```
┌─────────────┐
│ NotStarted  │
└──────┬──────┘
       │ start()
       ▼
┌─────────────┐
│ InProgress  │◄───────┐
└──────┬──────┘        │
       │               │ resume()
       │ pause()       │
       ▼               │
┌─────────────┐        │
│   Paused    │────────┘
└─────────────┘

       │ complete_activity()
       │ (last activity)
       ▼
┌─────────────┐
│  Completed  │
└─────────────┘
```

### 4.3 Resuming Interrupted Sessions

#### Strategy: Auto-save + Resume Prompt

**Implementation**:
```rust
// Database schema addition
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    plan TEXT NOT NULL, -- JSON
    state TEXT NOT NULL, -- JSON
    completed_activities TEXT, -- JSON array
    xp_earned INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

// Auto-save on every state change
pub fn save_session(
    conn: &Connection,
    session: &Session,
) -> Result<()> {
    conn.execute(
        "INSERT INTO sessions (id, user_id, plan, state, completed_activities, xp_earned, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'), datetime('now'))
         ON CONFLICT(id) DO UPDATE SET
         state = ?4,
         completed_activities = ?5,
         xp_earned = ?6,
         updated_at = datetime('now')",
        params![
            session.id,
            session.user_id,
            serde_json::to_string(&session.plan)?,
            serde_json::to_string(&session.state)?,
            serde_json::to_string(&session.completed_activities)?,
            session.xp_earned,
        ],
    )?;

    Ok(())
}

// Check for interrupted session on app startup
#[tauri::command]
pub async fn check_interrupted_session(
    user_id: String,
    state: State<'_, AppState>,
) -> Result<Option<Session>, String> {
    let db = state.db.lock().unwrap();

    let result = db.query_row(
        "SELECT id, plan, state, completed_activities, xp_earned
         FROM sessions
         WHERE user_id = ?1
           AND (state LIKE '%InProgress%' OR state LIKE '%Paused%')
         ORDER BY updated_at DESC
         LIMIT 1",
        params![user_id],
        |row| {
            let id: String = row.get(0)?;
            let plan_json: String = row.get(1)?;
            let state_json: String = row.get(2)?;
            let completed_json: String = row.get(3)?;
            let xp_earned: i32 = row.get(4)?;

            Ok(Session {
                id,
                user_id: user_id.clone(),
                plan: serde_json::from_str(&plan_json).unwrap(),
                state: serde_json::from_str(&state_json).unwrap(),
                completed_activities: serde_json::from_str(&completed_json).unwrap(),
                xp_earned,
            })
        },
    );

    match result {
        Ok(session) => Ok(Some(session)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}
```

**Frontend Flow**:
```tsx
// On app load
useEffect(() => {
  const checkInterrupted = async () => {
    const session = await invoke('check_interrupted_session', { userId })

    if (session) {
      const resume = await confirm(
        `You have an interrupted session from ${session.updated_at}. Resume?`
      )

      if (resume) {
        navigate(`/session/${session.id}`)
      } else {
        // Mark as abandoned
        await invoke('abandon_session', { sessionId: session.id })
      }
    }
  }

  checkInterrupted()
}, [])
```

### 4.4 Session Summary Data Structure

```rust
#[derive(Debug, Serialize)]
pub struct SessionSummary {
    pub session_id: String,
    pub duration_minutes: u32,
    pub total_xp_earned: i32,
    pub level_before: u32,
    pub level_after: u32,
    pub leveled_up: bool,
    pub activities_completed: Vec<CompletedActivitySummary>,
    pub skills_practiced: Vec<SkillPracticeSummary>,
    pub streak_updated: StreakUpdate,
    pub badges_earned: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct CompletedActivitySummary {
    pub title: string,
    pub type_: NodeType,
    pub xp_earned: i32,
    pub time_spent_minutes: u32,
    pub performance: Option<f64>, // For quizzes/challenges (0.0-1.0)
}

#[derive(Debug, Serialize)]
pub struct SkillPracticeSummary {
    pub skill_name: String,
    pub mastery_before: f64,
    pub mastery_after: f64,
    pub improvement: f64,
}

#[derive(Debug, Serialize)]
pub struct StreakUpdate {
    pub streak_before: u32,
    pub streak_after: u32,
    pub streak_continued: bool,
    pub multiplier_after: f64,
}

// Generate summary when session completes
pub fn generate_session_summary(
    conn: &Connection,
    session: &Session,
    user_id: &str,
) -> Result<SessionSummary> {
    let SessionState::Completed { total_xp_earned, total_time_ms, .. } = session.state else {
        return Err("Session not completed".into());
    };

    // Get level before/after
    let xp_before = get_user_xp(conn, user_id)? - total_xp_earned;
    let xp_after = get_user_xp(conn, user_id)?;
    let level_before = calculate_level(xp_before);
    let level_after = calculate_level(xp_after);

    // Get activity summaries
    let activities: Vec<CompletedActivitySummary> = session.completed_activities
        .iter()
        .map(|activity_id| {
            // Fetch from DB and content tree
            get_activity_summary(conn, user_id, activity_id)
        })
        .collect::<Result<Vec<_>>>()?;

    // Get skill improvements
    let skills = get_skill_improvements(conn, user_id, &session.plan.skills_covered)?;

    // Get streak update
    let streak_info = calculate_streak(conn, user_id)?;

    // Check for new badges
    let badges = check_badge_unlocks(conn, user_id)?;

    Ok(SessionSummary {
        session_id: session.id.clone(),
        duration_minutes: (total_time_ms / 60000) as u32,
        total_xp_earned,
        level_before,
        level_after,
        leveled_up: level_after > level_before,
        activities_completed: activities,
        skills_practiced: skills,
        streak_updated: StreakUpdate {
            streak_before: streak_info.current_streak - 1,
            streak_after: streak_info.current_streak,
            streak_continued: !streak_info.is_grace_period,
            multiplier_after: get_streak_multiplier(streak_info.current_streak),
        },
        badges_earned: badges,
    })
}
```

### 4.5 UI Components

#### Component Hierarchy
```
DailySessionPage
├── SessionHeader
│   ├── SessionTitle
│   └── ExitButton
├── SessionStatus (when NotStarted)
│   ├── SessionPlanPreview
│   │   ├── EstimatedTime
│   │   ├── TotalXPPotential
│   │   └── ActivityList
│   └── StartButton
├── SessionProgress (when InProgress/Paused)
│   ├── ProgressBar
│   ├── CurrentActivity
│   ├── Timer
│   └── PauseButton
├── ActivityContainer
│   ├── LectureViewer (if lecture)
│   ├── QuizPage (if quiz)
│   └── ChallengePage (if challenge)
└── SessionSummaryModal (when Completed)
    ├── CelebrationAnimation
    ├── XPEarned
    ├── LevelProgress
    ├── StreakUpdate
    ├── SkillImprovements
    ├── BadgesEarned
    └── CloseButton
```

#### Component Props Interfaces

```tsx
// DailySessionPage.tsx
interface DailySessionPageProps {
  sessionId?: string // If resuming
}

// SessionPlanPreview.tsx
interface SessionPlanPreviewProps {
  plan: SessionPlan
  onStart: () => void
}

interface SessionPlan {
  activities: PlannedActivity[]
  estimatedMinutes: number
  totalXpPotential: number
  skillsCovered: string[]
}

// SessionProgress.tsx
interface SessionProgressProps {
  currentIndex: number
  totalActivities: number
  elapsedMinutes: number
  isPaused: boolean
  onPause: () => void
  onResume: () => void
}

// ActivityContainer.tsx
interface ActivityContainerProps {
  activity: PlannedActivity
  onComplete: (xpEarned: number) => void
}

// SessionSummaryModal.tsx
interface SessionSummaryModalProps {
  isOpen: boolean
  summary: SessionSummary
  onClose: () => void
}

interface SessionSummary {
  durationMinutes: number
  totalXpEarned: number
  levelBefore: number
  levelAfter: number
  leveledUp: boolean
  activitiesCompleted: CompletedActivitySummary[]
  skillsPracticed: SkillPracticeSummary[]
  streakUpdated: StreakUpdate
  badgesEarned: string[]
}
```

#### Component List
1. `DailySessionPage` - Main container
2. `SessionPlanPreview` - Shows plan before starting
3. `SessionProgress` - Progress indicator
4. `ActivityContainer` - Renders current activity
5. `SessionSummaryModal` - End-of-session celebration
6. `SkillImprovementCard` - Individual skill progress
7. `LevelUpAnimation` - Celebration animation (if leveled up)
8. `StreakDisplay` - Streak continuation indicator

### 4.6 Tauri Backend Commands

```rust
// src-tauri/src/commands/session.rs

#[tauri::command]
pub async fn create_daily_session(
    user_id: String,
    target_minutes: u32,
    state: State<'_, AppState>,
) -> Result<Session, String> {
    let planner = SessionPlanner::new(state.content_tree.clone(), state.db.clone());
    let plan = planner.plan_daily_session(&user_id, target_minutes)?;

    let session = Session {
        id: Uuid::new_v4().to_string(),
        user_id,
        plan,
        state: SessionState::NotStarted,
        completed_activities: Vec::new(),
        xp_earned: 0,
    };

    // Save to DB
    save_session(&state.db.lock().unwrap(), &session)?;

    Ok(session)
}

#[tauri::command]
pub async fn start_session(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<Session, String> {
    let db = state.db.lock().unwrap();
    let mut session = load_session(&db, &session_id)?;

    session.start()?;
    save_session(&db, &session)?;

    Ok(session)
}

#[tauri::command]
pub async fn pause_session(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<Session, String> {
    let db = state.db.lock().unwrap();
    let mut session = load_session(&db, &session_id)?;

    session.pause()?;
    save_session(&db, &session)?;

    Ok(session)
}

#[tauri::command]
pub async fn resume_session(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<Session, String> {
    let db = state.db.lock().unwrap();
    let mut session = load_session(&db, &session_id)?;

    session.resume()?;
    save_session(&db, &session)?;

    Ok(session)
}

#[tauri::command]
pub async fn complete_session_activity(
    session_id: String,
    activity_id: String,
    xp_earned: i32,
    state: State<'_, AppState>,
) -> Result<Session, String> {
    let db = state.db.lock().unwrap();
    let mut session = load_session(&db, &session_id)?;

    session.complete_activity(&activity_id, xp_earned)?;
    save_session(&db, &session)?;

    Ok(session)
}

#[tauri::command]
pub async fn get_session_summary(
    session_id: String,
    user_id: String,
    state: State<'_, AppState>,
) -> Result<SessionSummary, String> {
    let db = state.db.lock().unwrap();
    let session = load_session(&db, &session_id)?;

    let summary = generate_session_summary(&db, &session, &user_id)?;

    Ok(summary)
}

#[tauri::command]
pub async fn abandon_session(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.db.lock().unwrap();

    db.execute(
        "UPDATE sessions
         SET state = ?1, updated_at = datetime('now')
         WHERE id = ?2",
        params![
            serde_json::to_string(&SessionState::Completed {
                completed_at: Utc::now(),
                total_xp_earned: 0,
                total_time_ms: 0,
            }).unwrap(),
            session_id
        ],
    ).map_err(|e| e.to_string())?;

    Ok(())
}
```

### 4.7 State Flow Diagram

```
[Home Page]
      ↓
[Click "Start Daily Session"]
      ↓
[Generate Session Plan] → [Call planning algorithm]
      ↓
[Show Plan Preview]
      ↓
[User clicks Start]
      ↓
[Session state → InProgress]
      ↓
[Load Activity 1] → [Lecture/Quiz/Challenge]
      ↓
[User completes activity]
      ↓
[Award XP + Update mastery]
      ↓
[Session.complete_activity()]
      ↓
[More activities?] ─Yes─→ [Load next activity]
      │
      No
      ↓
[Session state → Completed]
      ↓
[Generate summary]
      ↓
[Show SessionSummaryModal]
      ↓
[Update streak]
      ↓
[Check badge unlocks]
      ↓
[Return to Home]
```

### 4.8 Acceptance Criteria Checklist

- [ ] "Start Daily Session" button creates session plan
- [ ] Session plan shows estimated time, XP, and activities
- [ ] Can start session
- [ ] Progress bar updates as activities complete
- [ ] Timer tracks elapsed time
- [ ] Can pause and resume session
- [ ] Paused time doesn't count toward total
- [ ] Each activity awards XP correctly
- [ ] Completing activity advances to next
- [ ] Session summary displays when all complete
- [ ] Summary shows all XP earned, time spent, level change
- [ ] Skill improvements displayed correctly
- [ ] Streak updates correctly
- [ ] New badges shown in summary (if earned)
- [ ] Can resume interrupted session on app restart
- [ ] Abandoned sessions don't block future sessions

---

## Integration & Testing

### End-to-End Flow Testing

**Test Scenario: Complete First Session**

1. **Setup**:
   - Fresh database
   - User created
   - Week 1 Day 1 content loaded

2. **Flow**:
   ```
   [Home]
     → Click "Start Daily Session"
     → See plan: Lecture + Quiz (estimated 20 min)
     → Click Start
     → Read lecture (scroll to bottom)
     → Click "Mark Complete"
     → See +25 XP modal
     → Click Continue
     → Load quiz
     → Answer 5 questions
     → Submit quiz
     → See results: 80% (4/5)
     → See +60 XP (50 base × 1.2 medium × 1.0 streak × 1.1 accuracy)
     → See mastery update: Ownership 20%
     → Session completes
     → See summary modal:
       - 18 minutes
       - 85 XP total
       - Level 1 → 1 (not enough for level 2 yet)
       - Streak: 1 day
       - Skill: Ownership 0% → 20%
     → Click Close
     → Return to Home
   ```

3. **Validations**:
   - [ ] DB shows lecture complete
   - [ ] DB shows quiz attempt saved
   - [ ] DB shows user XP = 85
   - [ ] DB shows mastery_scores entry for "ownership" = 0.20
   - [ ] DB shows streak = 1
   - [ ] Next lecture unlocked

### Integration Checklist

- [ ] Tauri commands wire up correctly to frontend
- [ ] All database queries work
- [ ] Content tree loads correctly
- [ ] State persists across app restarts
- [ ] XP calculations match formulas
- [ ] Mastery updates correctly
- [ ] Streak updates correctly
- [ ] Unlock logic works
- [ ] All UI components render without errors
- [ ] Navigation flows work

---

## Edge Cases & Error Handling

### Edge Case Matrix

| Scenario | Handling |
|----------|----------|
| **User closes app during lecture** | Auto-save time every 30s, resume from saved position |
| **User quits mid-quiz** | Save answers to local storage, allow resume |
| **Submit quiz with unanswered questions** | Disable submit until all answered, OR show warning and mark unanswered as wrong |
| **Time limit expires on quiz** | Auto-submit with current answers |
| **Session plan finds no activities** | Show "You've completed all available content! More coming soon." |
| **Database locked (SQLite)** | Retry 3 times with exponential backoff, show error if still fails |
| **Content file missing** | Log error, show user-friendly message, skip activity |
| **Invalid JSON in quiz** | Validate on load, show error, prevent session start |
| **User changes system date (cheat streak)** | Validate date is not in future, cap streak gains to +1 per session |
| **XP overflow (>INT_MAX)** | Use i64 for XP storage, cap at 2^31-1 for display |
| **Mastery score corruption** | Clamp to [0.0, 1.0] on read, log warning |
| **Duplicate session creation** | Check for active session before creating new |
| **Level calculation underflow** | Ensure level >= 1 always |
| **Streak on first day** | Initialize to 1, not 0 |
| **Grace period on fresh account** | No grace period if no previous activity |
| **Retake quiz 100 times** | Allow unlimited, but XP approaches 0 (diminishing returns) |
| **Complete activity while offline** | Queue for sync when back online (future feature) |

### Error Handling Strategy

#### Backend Errors
```rust
#[derive(Debug, Serialize)]
pub enum AppError {
    DatabaseError(String),
    ContentNotFound(String),
    InvalidState(String),
    ValidationError(String),
}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        AppError::DatabaseError(err.to_string())
    }
}

// Return user-friendly error messages
#[tauri::command]
pub async fn some_command() -> Result<Data, String> {
    match do_something() {
        Ok(data) => Ok(data),
        Err(AppError::DatabaseError(msg)) => {
            eprintln!("DB Error: {}", msg);
            Err("Database error occurred. Please try again.".into())
        }
        Err(AppError::ContentNotFound(id)) => {
            Err(format!("Content '{}' not found. Please contact support.", id))
        }
        Err(e) => Err(format!("An error occurred: {:?}", e)),
    }
}
```

#### Frontend Error Handling
```tsx
// Global error boundary
class ErrorBoundary extends React.Component {
  componentDidCatch(error, info) {
    console.error('React Error:', error, info)
    // Log to file or error tracking service
  }

  render() {
    if (this.state.hasError) {
      return <ErrorFallback />
    }
    return this.props.children
  }
}

// Tauri command error handling
async function safeInvoke<T>(command: string, args?: any): Promise<T | null> {
  try {
    return await invoke<T>(command, args)
  } catch (error) {
    console.error(`Command ${command} failed:`, error)
    toast.error(error as string)
    return null
  }
}

// Usage
const data = await safeInvoke('get_lecture', { lectureId })
if (!data) {
  navigate('/error')
  return
}
```

### Validation Rules

#### Quiz Submission
```rust
fn validate_quiz_submission(quiz: &Quiz, answers: &HashMap<String, String>) -> Result<()> {
    // All questions answered?
    for question in &quiz.questions {
        if !answers.contains_key(&question.id) {
            return Err(format!("Question {} not answered", question.id).into());
        }
    }

    // Valid option IDs?
    for (question_id, answer_id) in answers {
        let question = quiz.questions.iter().find(|q| &q.id == question_id)
            .ok_or(format!("Invalid question ID: {}", question_id))?;

        let valid = question.options.iter().any(|opt| &opt.id == answer_id);
        if !valid {
            return Err(format!("Invalid answer ID: {}", answer_id).into());
        }
    }

    Ok(())
}
```

#### Time Validation
```rust
fn validate_activity_completion(time_spent_ms: i64, expected_min_ms: i64) -> Result<()> {
    if time_spent_ms < expected_min_ms {
        return Err("Activity completed too quickly. Please take your time.".into());
    }

    if time_spent_ms > expected_min_ms * 100 {
        // Probably left tab open for hours
        // Don't fail, but log warning
        eprintln!("Warning: Activity took {} ms (expected ~{})", time_spent_ms, expected_min_ms);
    }

    Ok(())
}
```

---

## Time Estimates Summary

### Milestone 2.1: Lecture Viewer
- **Research & setup**: 2 hours
- **MarkdownRenderer component**: 3 hours
- **Time tracking implementation**: 2 hours
- **Progress tracking & unlock logic**: 4 hours
- **Backend commands**: 3 hours
- **Testing & polish**: 2 hours
- **Total**: ~16 hours (1-2 days)

### Milestone 2.2: Quiz System
- **Quiz data model & JSON schema**: 3 hours
- **QuizPage component**: 4 hours
- **Question components (multiple types)**: 4 hours
- **Grading logic**: 3 hours
- **XP calculation**: 2 hours
- **Mastery update logic**: 3 hours
- **Retake handling**: 2 hours
- **Backend commands**: 4 hours
- **Testing & polish**: 3 hours
- **Total**: ~28 hours (3-4 days)

### Milestone 2.3: Progress Dashboard
- **Recharts setup**: 1 hour
- **XP/Level formulas**: 2 hours
- **Streak calculation**: 2 hours
- **Mastery visualization**: 3 hours
- **Dashboard components**: 6 hours
- **Backend queries**: 3 hours
- **Testing & polish**: 2 hours
- **Total**: ~19 hours (2 days)

### Milestone 2.4: Daily Session Queue
- **Session planning algorithm**: 4 hours
- **Session state machine**: 3 hours
- **Session persistence**: 2 hours
- **Resume logic**: 2 hours
- **Session summary generation**: 3 hours
- **UI components**: 5 hours
- **Backend commands**: 3 hours
- **Testing & polish**: 2 hours
- **Total**: ~24 hours (2 days)

### Integration & Testing
- **End-to-end flow testing**: 4 hours
- **Edge case handling**: 3 hours
- **Bug fixes**: 4 hours
- **Total**: ~11 hours (1 day)

---

## **Grand Total: ~98 hours (10-12 days)**

Given some inefficiencies and learning curve:
- **Optimistic**: 8 days (experienced with stack)
- **Realistic**: 10-12 days (moderate experience)
- **Conservative**: 14 days (learning as you go)

**Build Plan Estimate**: 2 weeks ✅ Validated

---

## Success Criteria for Phase 2

**Phase 2 is complete when**:

1. ✅ Can complete full session: lecture → quiz → XP gain
2. ✅ Progress persists across app restarts
3. ✅ Streak updates correctly (test by changing system date)
4. ✅ Mastery updates after quiz
5. ✅ Dashboard shows accurate real-time data
6. ✅ Session queue recommends logical activities
7. ✅ All formulas match validated balance report
8. ✅ No critical bugs in core flow
9. ✅ Screen recording demonstrates complete flow

**Evidence Required**:
- Screen recording showing complete session from start to summary
- Screenshot of dashboard with updated stats
- Database query showing correct XP/mastery/streak values
- Logs showing unlock logic working

---

## Next Steps (Phase 3 Preview)

After Phase 2 completion, Phase 3 will add:
- Docker runner for mini-challenges
- LLM grading for checkpoints
- More complex verification systems

But Phase 2 establishes the **core game loop** that makes the platform engaging. Get this right, and the rest builds on a solid foundation.

---

**End of Phase 2 Implementation Plan**
