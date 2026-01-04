# Phase 6: Polish & Beta - Implementation Plan

**Goal:** Make the platform production-ready with robust error handling, smooth onboarding, performance optimizations, and complete documentation.

**Timeline:** 2 weeks (Week 13-14)

---

## Current State Assessment

### What Exists
- **Error types defined** in `crates/core/src/db/error.rs`, `crates/runner/src/error.rs`, `crates/grader/src/error.rs`
- **Basic loading states** in all stores (loading boolean + error string)
- **Simple onboarding** in `Home.tsx` (username input only)
- **Settings page** with placeholder "Reset Progress" button
- **No README.md** or user documentation
- **No Docker/API key setup flow**
- **No keyboard shortcuts**
- **No dark mode**

### Gap Analysis
1. **Error handling**: Errors converted to strings, no user-friendly messages or recovery actions
2. **Onboarding**: No Docker check, no OpenAI API key setup, no guided tutorial
3. **Performance**: No container pre-warming, no batch LLM calls, some missing loading indicators
4. **Polish**: No dark mode, no keyboard shortcuts, inconsistent animations
5. **Documentation**: No README, no installation guide, no troubleshooting guide

---

## Milestone 6.1: Error Handling & Edge Cases (2-3 days)

### 6.1.1 Create Unified Error Types

**File:** `apps/desktop/src/types/errors.ts`

```typescript
export type ErrorCode =
  | 'DOCKER_NOT_RUNNING'
  | 'DOCKER_NOT_INSTALLED'
  | 'LLM_API_TIMEOUT'
  | 'LLM_RATE_LIMITED'
  | 'LLM_API_KEY_INVALID'
  | 'LLM_API_KEY_MISSING'
  | 'DATABASE_LOCKED'
  | 'DATABASE_CORRUPTED'
  | 'CODE_TIMEOUT'
  | 'CODE_MEMORY_EXCEEDED'
  | 'CONTENT_NOT_FOUND'
  | 'CURRICULUM_NOT_LOADED'
  | 'NETWORK_ERROR'
  | 'UNKNOWN'

export interface AppError {
  code: ErrorCode
  message: string
  userMessage: string
  recoveryAction?: string
  retryable: boolean
}
```

**File:** `apps/desktop/src/utils/errorHandler.ts`

Implement error parsing from Rust string errors to typed `AppError` objects with:
- Pattern matching on error message strings
- User-friendly message mapping
- Recovery action suggestions

### 6.1.2 Update Backend Error Responses

**Files to modify:**
- `apps/desktop/src-tauri/src/commands/*.rs` - Return structured error JSON instead of `e.to_string()`

Create error response type:
```rust
#[derive(Serialize)]
pub struct CommandError {
    code: String,
    message: String,
    details: Option<String>,
}
```

### 6.1.3 Create Error Display Components

**File:** `apps/desktop/src/components/common/ErrorBanner.tsx`
- Dismissible error banner with icon
- Shows user message + recovery action
- Retry button for retryable errors

**File:** `apps/desktop/src/components/common/ErrorModal.tsx`
- Full-screen modal for critical errors
- Detailed error info for debugging
- "Copy Error" button for support

### 6.1.4 Add Docker Status Check

**File:** `apps/desktop/src-tauri/src/commands/system.rs`
```rust
#[tauri::command]
pub async fn check_docker_status() -> Result<DockerStatus, CommandError>

#[tauri::command]
pub async fn get_system_requirements() -> SystemRequirements
```

**Frontend:** `apps/desktop/src/stores/systemStore.ts`
- Track Docker availability
- Track OpenAI API key status
- Check on app startup

### 6.1.5 Implement Backup/Restore

**Backend commands:**
```rust
#[tauri::command]
pub fn export_user_data(path: String) -> Result<(), CommandError>

#[tauri::command]
pub fn import_user_data(path: String) -> Result<(), CommandError>

#[tauri::command]
pub fn reset_all_progress() -> Result<(), CommandError>
```

**Frontend:** Update `Settings.tsx` with:
- Export Progress button (saves JSON to chosen location)
- Import Progress button (loads JSON)
- Reset Progress button (with confirmation dialog)

### 6.1.6 Add Retry Logic

**File:** `apps/desktop/src/utils/retry.ts`
```typescript
export async function withRetry<T>(
  fn: () => Promise<T>,
  options: { maxAttempts: number; delay: number; exponential: boolean }
): Promise<T>
```

Apply to:
- LLM grading calls (3 attempts, exponential backoff)
- Docker runner calls (2 attempts)
- Database operations (2 attempts for lock errors)

### Acceptance Criteria
- [ ] No unhandled errors crash the app
- [ ] All errors show user-friendly messages
- [ ] Docker status checked on startup
- [ ] Backup/restore works correctly
- [ ] Reset progress with confirmation

---

## Milestone 6.2: Onboarding & Help (2 days)

### 6.2.1 Create Welcome Flow

**File:** `apps/desktop/src/pages/Welcome.tsx`

Multi-step wizard:
1. **Welcome Screen**: App intro, what you'll learn
2. **System Check**: Docker installed? API key set?
3. **Docker Setup** (if needed): Instructions + download links
4. **API Key Setup**: Input field, test connection button
5. **Create Profile**: Username input
6. **Quick Tutorial**: Start first lesson prompt

**Store:** `apps/desktop/src/stores/onboardingStore.ts`
```typescript
interface OnboardingState {
  step: number
  completed: boolean
  dockerReady: boolean
  apiKeySet: boolean
  skipTutorial: boolean
}
```

### 6.2.2 Docker Setup Page

**File:** `apps/desktop/src/components/onboarding/DockerSetup.tsx`

Content:
- Check Docker status with visual indicator
- Platform-specific install instructions (Linux/macOS/Windows)
- "Check Again" button
- "Skip (challenges disabled)" option

Install instructions per platform:
```markdown
## Linux (Arch)
sudo pacman -S docker
sudo systemctl start docker
sudo usermod -aG docker $USER

## Linux (Ubuntu/Debian)
sudo apt-get install docker.io
sudo systemctl start docker
sudo usermod -aG docker $USER

## macOS
brew install --cask docker
# or download from docker.com

## Windows
Download Docker Desktop from docker.com
```

### 6.2.3 API Key Setup Page

**File:** `apps/desktop/src/components/onboarding/ApiKeySetup.tsx`

Features:
- Masked input field for API key
- "Test Connection" button
- Link to OpenAI API key page
- Cost estimate warning (~$5-15 per curriculum)
- "Skip (grading disabled)" option

**Backend command:**
```rust
#[tauri::command]
pub async fn test_openai_connection(api_key: String) -> Result<bool, CommandError>

#[tauri::command]
pub fn save_api_key(api_key: String) -> Result<(), CommandError>
```

Store API key securely using `keyring` crate or encrypted local file.

### 6.2.4 Add Help Tooltips

**File:** `apps/desktop/src/components/common/Tooltip.tsx`

Simple hover tooltip component using Radix UI Tooltip.

Add tooltips to:
- XP bar (explains XP system)
- Streak counter (explains streak mechanics)
- Mastery scores (explains decay)
- Skill tree nodes (shows prerequisites)
- Badge icons (shows unlock criteria)

### 6.2.5 Create Interactive Tutorial

**File:** `apps/desktop/src/components/tutorial/TutorialOverlay.tsx`

Highlight-and-explain overlay:
1. Navigate to first lecture
2. Complete lecture (mark as done)
3. Take quiz
4. View progress update
5. Check skill tree

Store tutorial progress in `onboardingStore`.

### Acceptance Criteria
- [ ] First-time user sees welcome wizard
- [ ] Docker check with setup instructions
- [ ] API key input with validation
- [ ] Tooltips on key UI elements
- [ ] Tutorial completes without errors

---

## Milestone 6.3: Performance & Polish (2-3 days)

### 6.3.1 Docker Container Pre-warming

**File:** `crates/runner/src/pool.rs` (already exists)

Enhance `ContainerPool`:
- Pre-create 2 containers on app startup (if Docker available)
- Keep warm containers ready
- Recycle containers between runs

**Backend command:**
```rust
#[tauri::command]
pub async fn prewarm_docker_containers() -> Result<(), CommandError>
```

Call on app startup after Docker check passes.

### 6.3.2 Loading States Audit

Review and add loading indicators to:

| Component | Current State | Action Needed |
|-----------|---------------|---------------|
| Home.tsx | No spinner | Add skeleton loader |
| SkillTree.tsx | Has loading | OK |
| Quiz.tsx | Has loading | OK |
| Lecture.tsx | Has loading | OK |
| Progress.tsx | Has loading | OK |
| Badges.tsx | Has loading | OK |
| Review.tsx | Has loading | OK |
| CurriculumManager.tsx | Has loading | OK |

**File:** `apps/desktop/src/components/common/Skeleton.tsx`
- Generic skeleton loader component
- Card skeleton, text skeleton variants

### 6.3.3 Implement Dark Mode

**File:** `apps/desktop/tailwind.config.js`
```javascript
module.exports = {
  darkMode: 'class',
  // ...
}
```

**File:** `apps/desktop/src/stores/themeStore.ts`
```typescript
interface ThemeState {
  theme: 'light' | 'dark' | 'system'
  setTheme: (theme: 'light' | 'dark' | 'system') => void
}
```

**File:** `apps/desktop/src/components/layout/ThemeToggle.tsx`
- Toggle button in Navigation
- Persist preference to localStorage

Update all components to use Tailwind dark: variants.

### 6.3.4 Add Keyboard Shortcuts

**File:** `apps/desktop/src/hooks/useKeyboardShortcuts.ts`

Global shortcuts:
- `Ctrl/Cmd + 1`: Go to Home
- `Ctrl/Cmd + 2`: Go to Skill Tree
- `Ctrl/Cmd + 3`: Go to Progress
- `Ctrl/Cmd + 4`: Go to Badges
- `Ctrl/Cmd + ,`: Go to Settings
- `Ctrl/Cmd + ?`: Show keyboard shortcuts modal
- `Esc`: Close modal/go back

Quiz shortcuts:
- `1-4`: Select answer option
- `Enter`: Submit answer
- `N`: Next question

Lecture shortcuts:
- `Space`: Mark as complete
- `→`: Next section
- `←`: Previous section

**File:** `apps/desktop/src/components/common/KeyboardShortcutsModal.tsx`
- Modal listing all shortcuts
- Triggered by `Ctrl/Cmd + ?` or help button

### 6.3.5 Animation Polish

**File:** `apps/desktop/src/styles/animations.css`

Add smooth transitions for:
- Page transitions (fade in/out)
- Modal open/close
- Badge unlock celebration
- XP gain animation
- Level up animation
- Progress bar fills

Use Framer Motion or CSS transitions.

### 6.3.6 UI Consistency Pass

Review and fix:
- Consistent spacing (use Tailwind spacing scale)
- Button styles (primary, secondary, danger variants)
- Card shadows and borders
- Typography scale
- Color palette adherence

**File:** `apps/desktop/src/components/common/Button.tsx`
- Standardized button component with variants

### Acceptance Criteria
- [ ] Docker containers pre-warmed on startup
- [ ] All pages show loading states
- [ ] Dark mode works throughout
- [ ] Keyboard shortcuts functional
- [ ] Animations smooth and consistent
- [ ] UI visually polished

---

## Milestone 6.4: Testing & Documentation (2-3 days)

### 6.4.1 Create README.md

**File:** `README.md`

Structure:
```markdown
# Gamified Learning Platform 🦀

A desktop application for gamified Rust learning with Docker-based code verification and LLM-powered artifact grading.

## Features
- 📚 Interactive lectures with markdown rendering
- 📝 Quizzes with instant feedback
- 💻 Code challenges with Docker verification
- 🏆 XP, levels, and badges
- 📊 Mastery tracking with spaced repetition
- 🌳 Visual skill tree progression

## Screenshots
[Screenshots of key features]

## Installation

### Prerequisites
- Docker (for code challenges)
- OpenAI API key (for artifact grading)

### Download
- [Linux (.AppImage)]()
- [macOS (.dmg)]()
- [Windows (.exe)]()

### Build from Source
[Build instructions]

## Quick Start
1. Launch the app
2. Complete system setup (Docker + API key)
3. Create your profile
4. Start with the tutorial!

## Documentation
- [Installation Guide](docs/INSTALLATION.md)
- [Content Schema](docs/CONTENT_SCHEMA.md)
- [Troubleshooting](docs/TROUBLESHOOTING.md)
- [FAQ](docs/FAQ.md)

## License
MIT
```

### 6.4.2 Create Installation Guide

**File:** `docs/INSTALLATION.md`

Sections:
1. System Requirements
2. Docker Installation (per platform)
3. OpenAI API Key Setup
4. App Installation
5. First Run Setup
6. Troubleshooting Common Issues

### 6.4.3 Create Troubleshooting Guide

**File:** `docs/TROUBLESHOOTING.md`

Common issues:
1. Docker not detected
2. API key errors
3. Code timeout issues
4. Database errors
5. Content loading failures
6. Performance issues

Each with:
- Symptoms
- Causes
- Solutions

### 6.4.4 Create FAQ

**File:** `docs/FAQ.md`

Questions:
- What is this app?
- Is my data stored online?
- How much does the OpenAI API cost?
- Can I use it without Docker?
- Can I use it without OpenAI?
- How do I import my own curriculum?
- How does XP/mastery work?
- How do I reset my progress?
- How do I report bugs?

### 6.4.5 Add Integration Tests

**File:** `crates/core/tests/phase6_integration_tests.rs`

Tests:
- Error handling: Verify all error types serialize correctly
- Backup/restore: Export then import preserves all data
- System checks: Docker/API detection works

**File:** `apps/desktop/src/__tests__/onboarding.test.tsx`

Frontend tests:
- Welcome flow completes
- Docker setup shows correct instructions
- API key validation works

### 6.4.6 Manual Testing Checklist

Create `docs/TESTING_CHECKLIST.md`:

```markdown
## Pre-Release Testing Checklist

### Installation
- [ ] Fresh install on Linux works
- [ ] Fresh install on macOS works
- [ ] Fresh install on Windows works

### Onboarding
- [ ] Welcome wizard appears on first launch
- [ ] Docker detection accurate
- [ ] API key test works
- [ ] Profile creation works
- [ ] Tutorial completes

### Core Features
- [ ] Curriculum import works
- [ ] Lectures render correctly
- [ ] Quizzes grade correctly
- [ ] XP awards correctly
- [ ] Badges unlock
- [ ] Skill tree updates
- [ ] Review queue works

### Error Handling
- [ ] Docker not running shows helpful error
- [ ] API timeout handled gracefully
- [ ] Network error handled gracefully
- [ ] Code timeout displays correctly

### Settings
- [ ] Dark mode toggles
- [ ] Backup export works
- [ ] Backup import works
- [ ] Reset progress works

### Performance
- [ ] App launches in <3s
- [ ] Docker runs complete in <5s
- [ ] No UI jank during operations
```

### Acceptance Criteria
- [ ] README complete with screenshots
- [ ] Installation guide covers all platforms
- [ ] Troubleshooting covers common issues
- [ ] FAQ answers common questions
- [ ] Integration tests pass
- [ ] Manual testing checklist completed

---

## Implementation Order

### Day 1-2: Error Handling Foundation
1. Create unified error types (frontend)
2. Update backend commands with structured errors
3. Create ErrorBanner and ErrorModal components
4. Add Docker status check command
5. Implement backup/restore commands

### Day 3-4: Onboarding Flow
1. Create Welcome page with wizard steps
2. Build DockerSetup component
3. Build ApiKeySetup component
4. Add tooltips to key components
5. Create tutorial overlay

### Day 5-6: Performance & Polish
1. Implement container pre-warming
2. Add skeleton loaders
3. Implement dark mode
4. Add keyboard shortcuts
5. Polish animations

### Day 7-8: Documentation & Testing
1. Write README.md
2. Write Installation guide
3. Write Troubleshooting guide
4. Write FAQ
5. Add integration tests
6. Complete manual testing

---

## Files to Create

```
apps/desktop/src/
├── components/
│   ├── common/
│   │   ├── Button.tsx
│   │   ├── ErrorBanner.tsx
│   │   ├── ErrorModal.tsx
│   │   ├── KeyboardShortcutsModal.tsx
│   │   ├── Skeleton.tsx
│   │   └── Tooltip.tsx
│   ├── layout/
│   │   └── ThemeToggle.tsx
│   ├── onboarding/
│   │   ├── ApiKeySetup.tsx
│   │   ├── DockerSetup.tsx
│   │   └── SystemCheck.tsx
│   └── tutorial/
│       └── TutorialOverlay.tsx
├── hooks/
│   └── useKeyboardShortcuts.ts
├── pages/
│   └── Welcome.tsx
├── stores/
│   ├── onboardingStore.ts
│   ├── systemStore.ts
│   └── themeStore.ts
├── styles/
│   └── animations.css
├── types/
│   └── errors.ts
└── utils/
    ├── errorHandler.ts
    └── retry.ts

apps/desktop/src-tauri/src/commands/
└── system.rs

docs/
├── FAQ.md
├── INSTALLATION.md
├── TESTING_CHECKLIST.md
└── TROUBLESHOOTING.md

crates/core/tests/
└── phase6_integration_tests.rs

README.md
```

## Files to Modify

```
apps/desktop/src/
├── App.tsx (add Welcome route, keyboard shortcuts)
├── components/layout/Navigation.tsx (add ThemeToggle, help button)
├── pages/Home.tsx (add skeleton loader)
├── pages/Settings.tsx (add backup/restore, dark mode)
├── stores/*.ts (update error handling)

apps/desktop/src-tauri/src/
├── commands/mod.rs (add system module)
├── lib.rs (register new commands)

apps/desktop/
├── tailwind.config.js (add dark mode)
```

---

## Success Metrics

- **Error handling**: Zero unhandled panics, all errors show recovery options
- **Onboarding**: New user can complete setup in <5 minutes
- **Performance**: App startup <3s, Docker runs <5s p95
- **Polish**: Dark mode works, animations smooth, keyboard shortcuts functional
- **Documentation**: User can install and start learning without external help
- **Testing**: All automated tests pass, manual checklist complete
