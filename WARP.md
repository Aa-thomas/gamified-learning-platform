# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Project Overview

This is a **gamified Rust bootcamp learning platform** built with Tauri (Rust backend) + React/TypeScript (frontend). The platform uses Docker containers to safely execute student code, LLM-based grading for written artifacts, and a sophisticated XP/mastery system to motivate learners.

**Current Status**: Phase 0 complete (prototypes validated), Phase 1-7 implementation in progress.

## Architecture

### Three-Layer System

1. **Desktop App (Tauri)**: Rust backend handles business logic, SQLite storage, Docker execution, and OpenAI API calls
2. **Frontend (React/TS)**: UI for lectures, quizzes, challenges, skill tree visualization, progress tracking
3. **External Services**: 
   - Docker containers for isolated code execution
   - OpenAI GPT-4 for artifact grading
   - SQLite embedded database (local-first, no server)

### Core Domains

- **Content Delivery**: Markdown lectures, JSON-based quizzes/challenges loaded from manifest
- **Verification Systems**: 
  - Docker runner for mini-challenges (30s timeout, 256MB RAM limit, network isolation)
  - LLM grader for checkpoint artifacts (DESIGN.md, README.md, etc.)
- **Gamification Engine**: XP calculation, mastery tracking with decay, streak mechanics, badge unlocking
- **Progress Tracking**: Node completion status, quiz attempts, artifact submissions, review queue (spaced repetition)

### Data Flow

```
User Action → Tauri Command → Domain Logic → SQLite/Docker/OpenAI → Result → Frontend State Update
```

Example: Submitting a challenge
1. Frontend sends student code via Tauri command
2. Backend creates temp directory with challenge template
3. Docker runner executes `cargo test` in isolated container
4. Results parsed and XP calculated (base × difficulty × streak × accuracy)
5. Database updated (progress, XP, mastery)
6. Frontend displays results and updated XP

## Phase-Based Development Plan

### Phase 0: Risk Validation ✅ COMPLETE
Validated three highest-risk assumptions with working prototypes:

1. **LLM Grading** (`prototypes/llm-grading/`)
   - Validates: GPT-4 can reliably grade artifacts with ≥80% agreement
   - Key finding: Temperature 0.3 + structured rubrics = consistent grading (±5 points)
   - Cost: ~$0.046/grade, ~$1.61/student with caching

2. **Docker Runner** (`prototypes/docker-runner/`)
   - Validates: Docker can safely run untrusted code with proper isolation
   - Key finding: All edge cases handled (infinite loops, memory bombs, compile errors)
   - Security: Non-root user, network isolation, 256MB RAM, 30s timeout

3. **Gamification** (`prototypes/gamification/`)
   - Validates: XP/mastery formulas create balanced progression
   - Key finding: Daily user completes bootcamp in ~16 weeks, binge user in 4 weeks
   - Simulated 3 user archetypes through 14-week curriculum

### Phases 1-7 (Upcoming)
- **Phase 1**: Foundation (data schema, Tauri shell, content loader)
- **Phase 2**: Core game loop (lectures, quizzes, progress dashboard)
- **Phase 3**: Verification systems (integrate Docker + LLM prototypes)
- **Phase 4**: Gamification (badges, mastery decay, review queue)
- **Phase 5**: Content integration (port 4 weeks of curriculum)
- **Phase 6**: Polish & beta testing
- **Phase 7**: Deployment (installers, auto-update, release)

See `LLM-BUILD-PLAN.md` for detailed milestones and acceptance criteria.

## Key Formulas

All formulas implemented in `prototypes/gamification/formulas.rs`.

### XP Calculation
```rust
xp = base_xp × difficulty_mult × streak_mult × accuracy_mult
```

- **Base XP**: Lecture (25), Quiz (50), Mini-challenge (100), Checkpoint (200)
- **Difficulty**: Easy (1.0×), Medium (1.5×), Hard (2.0×), Very Hard (3.0×)
- **Streak**: Days 1-3 (1.0×), 4-7 (1.1×), 8-14 (1.2×), 15-30 (1.3×), 31+ (1.5×)
- **Accuracy**: 100% (1.5×), 90-99% (1.3×), 80-89% (1.1×), 70-79% (1.0×), <60% (0.5×)

### Level Progression
```rust
Level N requires: 100 × N^1.5 cumulative XP
```
- Level 1: 100 XP
- Level 5: 2,118 XP
- Level 10: 10,154 XP
- Level 20: 57,195 XP

### Mastery Tracking
```rust
// Learning (exponential moving average)
new_score = old_score + 0.25 × (performance - old_score)

// Decay (after 3-day grace period)
score = score × e^(-0.05 × days_inactive)
```
- Minimum floor: 30% (never fully forgotten)
- Grace period: 3 days (no decay on weekends)

### Streak Mechanics
- Same day: Continue (no change)
- Next day: Increment +1
- 1-day gap: Grace period (maintain with warning)
- 2+ day gap: Reset to 1

## Prototype Locations

All prototypes are standalone Rust programs demonstrating core concepts:

### `prototypes/docker-runner/`
- **Purpose**: Prove Docker can safely execute student code
- **Key Files**:
  - `runner.rs`: DockerRunner implementation (330 lines)
  - `Dockerfile`: Rust 1.75 sandbox with resource limits
  - `edge_cases/`: 5 test scenarios (infinite loop, memory bomb, compile error, etc.)
- **Run**: `docker build -t rust-sandbox . && cargo run --bin test_runner`

### `prototypes/llm-grading/`
- **Purpose**: Prove GPT-4 can consistently grade artifacts
- **Key Files**:
  - `grader.rs`: LLMGrader with consistency testing (572 lines)
  - `rubrics/`: JSON rubrics for DESIGN.md and README.md
  - `sample_artifacts/`: Good/mediocre/bad examples
- **Run**: Set `OPENAI_API_KEY` then `cargo run`

### `prototypes/gamification/`
- **Purpose**: Validate XP/mastery formulas are balanced
- **Key Files**:
  - `formulas.rs`: XP, level, mastery calculators (500+ lines)
  - `simulation.rs`: User archetype simulator (450+ lines)
  - `balance_report.md`: Simulation results for 3 user types
- **Run**: `cargo test` to verify formulas

## Technology Decisions

### Database: `rusqlite` (not `sqlx`)
- **Rationale**: Desktop app with embedded DB, simpler than async server-oriented `sqlx`
- **Schema**: See `PHASE-1-IMPLEMENTATION-PLAN.md` lines 182-274

### State Management: Zustand (not Redux/Context)
- **Rationale**: Minimal boilerplate, no re-render issues, perfect for Tauri
- **Pattern**: Direct store mutations, invoke Tauri commands, TypeScript-first

### UI: Tailwind + shadcn/ui (not Material-UI/Chakra)
- **Rationale**: Custom gamification UI needs (XP bars, skill trees), full design control
- **Components**: Copy-paste Radix primitives, not a dependency

### Content Format: Markdown + JSON
- **Lectures**: Markdown with frontmatter
- **Quizzes/Challenges**: JSON with structured schemas
- **Manifest**: `content/manifest.json` defines tree structure

## Development Commands

**Note**: This project is in early stage (Phase 0 complete). Full dev environment will be set up in Phase 1.

### Prototype Testing
```bash
# Test Docker runner (requires Docker installed)
cd prototypes/docker-runner
docker build -t rust-sandbox .
cargo run --bin test_runner

# Test gamification formulas
cd prototypes/gamification
cargo test

# Test LLM grading (requires OPENAI_API_KEY)
cd prototypes/llm-grading
export OPENAI_API_KEY=your_key_here
cargo run
```

### Future Commands (Phase 1+)
Once Tauri app is initialized, commands will include:
```bash
# Install dependencies
npm install
cargo build

# Run development server
npm run tauri dev

# Build production app
npm run tauri build

# Run tests
cargo test                          # Backend tests
npm test                            # Frontend tests
cargo test test_name --lib         # Single Rust test
```

## Git Workflow

### Branch Naming
Use conventional commit type prefixes:
- `feat/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation updates
- `refactor/` - Code refactoring
- `perf/` - Performance improvements
- `test/` - Test additions/updates
- `chore/` - Maintenance tasks

**Branch type must match commit type** (e.g., `feat/user-auth` → `feat: implement user authentication`)

### Commit Messages
Follow Conventional Commits format:
```
type(scope): description

- Optional bullet points for complex changes

Co-Authored-By: Warp <agent@warp.dev>
```

**Types**: feat, fix, docs, style, refactor, perf, test, chore

**Examples**:
- `feat(docker): add Docker runner prototype with edge case testing`
- `docs(phase-0): complete risk validation milestone documentation`
- `refactor(gamification): optimize XP calculation formulas`

### Pull Requests
**Title**: Use conventional commit format (e.g., `feat: implement OAuth2 authentication`)

**Description Template**:
```markdown
Closes #123

## Why
[Explain the problem being solved]

## What
- [Key changes made]
- [New dependencies added]
- [Architectural decisions]

## How
- [Testing approach]
- [Verification steps]
```

**Requirements**:
- Link issues using `Closes #123` or `Fixes #456`
- Include context, implementation details, and testing info
- Highlight complex areas for reviewers
- Include `Co-Authored-By: Warp <agent@warp.dev>` in commit messages

### Git Commands
**Always use `--no-pager` flag** to avoid pagination:
```bash
git --no-pager status
git --no-pager log --oneline -10
git --no-pager diff
```

## AI Rules (.ai-rules/)

This repository uses `.ai-rules/` for AI assistant configuration:

- **Location**: All `.mdc` files must live in `.ai-rules/` directory (not subdirectories)
- **Scope**: One rule per `.mdc` file (single responsibility principle)
- **Format**: MDC format with YAML frontmatter and `<rule>` blocks

**Do not create `.mdc` files anywhere else** (not in `.cursor/rules/` or project root).

## Important Notes

### Docker Setup Required
Mini-challenges and verification require Docker:
```bash
# Ubuntu/Debian
sudo apt-get install docker.io
sudo systemctl start docker
sudo usermod -aG docker $USER

# Arch Linux
sudo pacman -S docker
sudo systemctl start docker
sudo usermod -aG docker $USER

# macOS
brew install --cask docker
```

### OpenAI API Key Required
Checkpoint grading requires GPT-4 access:
```bash
export OPENAI_API_KEY=your_key_here
```
Cost: ~$1.61 per student (70 artifacts with 50% cache hit rate)

### Codebase Status
- ✅ **Phase 0 Complete**: All prototypes validated, low risk
- 🚧 **Phase 1 Next**: Data schema + Tauri shell (2-3 weeks)
- 📋 **No Production Code Yet**: Only planning docs and prototypes exist

## Reference Documents

- `LLM-BUILD-PLAN.md` - Complete 8-phase build plan with milestones
- `PHASE_0_COMPLETE.md` - Phase 0 validation results and metrics
- `PHASE-1-IMPLEMENTATION-PLAN.md` - Detailed Phase 1 tasks and decisions
- `PHASE-2-IMPLEMENTATION-PLAN.md` - Phase 2 planning (core game loop)
- `PHASE-3-IMPLEMENTATION-PLAN.md` - Phase 3 planning (verification systems)
- `.claude/commands/commit.md` - Git commit automation guidelines
