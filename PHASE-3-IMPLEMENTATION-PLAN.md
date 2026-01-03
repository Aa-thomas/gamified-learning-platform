# Phase 3 Implementation Plan: Verification Systems

**Project:** Gamified Rust Bootcamp Platform
**Phase:** 3 - Verification Systems (Week 7-8)
**Goal:** Integrate Docker-based code verification and LLM-based artifact grading
**Timeline:** 6-8 days (3-4 days per milestone)

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Risk Assessment & Mitigation](#risk-assessment--mitigation)
3. [Milestone 3.1: Docker Integration](#milestone-31-docker-integration)
4. [Milestone 3.2: LLM Grading Integration](#milestone-32-llm-grading-integration)
5. [Cross-Cutting Concerns](#cross-cutting-concerns)
6. [Testing Strategy](#testing-strategy)
7. [Configuration & Secrets Management](#configuration--secrets-management)
8. [Cost Analysis](#cost-analysis)
9. [Timeline & Estimates](#timeline--estimates)

---

## Executive Summary

Phase 3 introduces the two highest-risk components of the platform:

1. **Docker Integration** - Safely executes untrusted student code in isolated containers
2. **LLM Grading** - Evaluates project artifacts (DESIGN.md, README.md, etc.) using GPT-4

**Key Success Metrics:**
- Docker verification completes in <5s p95 (after container pre-warming)
- LLM grading completes in <30s p95 with caching
- Zero security breaches from student code execution
- LLM grade consistency within ±5 points for identical content
- Grade cache hit rate >70% (saves ~$0.50 per cached grade)

**High-Risk Integration Points:**
1. Docker daemon availability and version compatibility
2. Container resource limit enforcement (prevents DoS)
3. Container cleanup (prevents resource leaks)
4. LLM API rate limits and timeout handling
5. Cost runaway prevention (API call caching)
6. File I/O race conditions in temp directory management

---

## Risk Assessment & Mitigation

### Critical Risks

| Risk | Probability | Impact | Mitigation Strategy |
|------|-------------|--------|---------------------|
| **Docker not installed** | High (40%) | Blocks challenges | Graceful detection + helpful setup guide + "Skip challenges" mode |
| **Container escape** | Low (5%) | Critical security | Non-root user, no privileged mode, network isolation, resource caps |
| **Resource exhaustion** | Medium (25%) | App hangs/crashes | Hard limits: 256MB RAM, 1 CPU, 30s timeout, cleanup orphans on startup |
| **LLM API outage** | Medium (20%) | Grading unavailable | Retry with exponential backoff, queue for later, show cached results |
| **API cost runaway** | Medium (30%) | Budget overrun | SHA-256 content hashing, SQLite cache, per-user daily limits |
| **Inconsistent grading** | Medium (25%) | User frustration | Low temperature (0.3), structured prompts, cache identical content |
| **Temp dir cleanup failure** | Medium (20%) | Disk space leak | Best-effort cleanup + periodic GC task + size monitoring |

### Security Measures

**Docker Sandbox (Defense in Depth):**
```rust
// Layer 1: Resource Limits
--memory=256m              // Prevent memory bombs
--cpus=1.0                 // Prevent CPU hogging
--pids-limit=100           // Prevent fork bombs

// Layer 2: Network Isolation
--network=none             // No network access

// Layer 3: Filesystem Restrictions
--read-only                // Root FS read-only
-v <temp>:/challenge:ro    // Challenge files read-only (except /target)
--tmpfs /tmp:size=100m     // Limited temp space

// Layer 4: User Isolation
USER student (non-root)    // No privilege escalation
--security-opt=no-new-privileges

// Layer 5: Execution Limits
timeout 30s                // Hard kill after 30 seconds
--stop-timeout=5           // Force kill if graceful stop fails
```

**LLM Security:**
- Never include student PII in prompts (only code/artifacts)
- Sanitize artifact content (max 50KB per artifact)
- Rate limit: 10 grades/minute per user
- Validate JSON schema before accepting LLM response

---

## Milestone 3.1: Docker Integration

**Duration:** 3-4 days
**Dependencies:** Phase 1 (database), Phase 2 (progress system)

### Technology Decision: Docker Client Library

#### Research: Rust Docker Clients

| Library | Pros | Cons | Verdict |
|---------|------|------|---------|
| **bollard** | - Actively maintained (2024)<br>- Full API coverage<br>- Async support (Tokio)<br>- Good docs | - Larger dependency tree<br>- Requires async runtime | **RECOMMENDED** ✅ |
| **shiplift** | - Simple API<br>- Mature | - Last update 2021<br>- Limited features<br>- Maintenance uncertain | ❌ Not recommended |
| **docker-api** | - Modern async API | - Less mature<br>- Smaller community | ⚠️ Backup option |
| **Shell out to CLI** | - No dependencies<br>- Prototype proven | - Error handling fragile<br>- Performance overhead<br>- Platform-specific | ⚠️ Acceptable for MVP |

**Decision:** Use `bollard` for production, but keep prototype's shell-based approach as fallback.

**Rationale:**
- Bollard provides structured error types (better UX than parsing stderr)
- Async API integrates with Tauri's async commands
- Active maintenance = Docker API compatibility
- Can still shell out if bollard fails (graceful degradation)

#### Design: Docker Sandbox Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Tauri Backend                           │
│                                                              │
│  ┌────────────────┐         ┌─────────────────┐            │
│  │  Challenge UI  │────────▶│ ChallengeRunner │            │
│  └────────────────┘         └────────┬────────┘            │
│                                       │                      │
│                                       ▼                      │
│                            ┌──────────────────┐             │
│                            │  DockerRunner    │             │
│                            │  (bollard)       │             │
│                            └────────┬─────────┘             │
└─────────────────────────────────────┼──────────────────────┘
                                      │ IPC
                         ┌────────────▼────────────┐
                         │    Docker Daemon        │
                         └────────────┬────────────┘
                                      │
                    ┌─────────────────┼─────────────────┐
                    ▼                 ▼                 ▼
            ┌──────────────┐  ┌──────────────┐  ┌──────────────┐
            │  Container 1 │  │  Container 2 │  │ Pre-warmed   │
            │  (running)   │  │  (running)   │  │ Pool (idle)  │
            └──────────────┘  └──────────────┘  └──────────────┘
                    │
                    ▼
            ┌────────────────────────────────┐
            │  /challenge (mounted volume)   │
            │  - Cargo.toml                  │
            │  - src/lib.rs (student code)   │
            │  - tests/test.rs               │
            └────────────────────────────────┘
```

#### Data Structures

```rust
// crates/runner/src/lib.rs

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for Docker runner
#[derive(Debug, Clone)]
pub struct DockerConfig {
    pub image_name: String,
    pub memory_limit: u64,        // bytes
    pub cpu_limit: f64,            // cores (0.0-<total cores>)
    pub timeout: Duration,
    pub network_mode: NetworkMode,
    pub pre_warm_pool_size: usize, // Number of idle containers
}

impl Default for DockerConfig {
    fn default() -> Self {
        Self {
            image_name: "gamified-rust-sandbox:latest".to_string(),
            memory_limit: 256 * 1024 * 1024, // 256MB
            cpu_limit: 1.0,
            timeout: Duration::from_secs(30),
            network_mode: NetworkMode::None,
            pre_warm_pool_size: 2,
        }
    }
}

#[derive(Debug, Clone)]
pub enum NetworkMode {
    None,      // No network access (default)
    Bridge,    // For future: allow HTTP requests with whitelist
}

/// Result of running a challenge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub tests_passed: u32,
    pub tests_failed: u32,
    pub tests_total: u32,
    pub compile_error: Option<CompileError>,
    pub runtime_error: Option<RuntimeError>,
    pub resource_limit_hit: Option<ResourceLimit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileError {
    pub message: String,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuntimeError {
    Timeout,
    Panic { message: String },
    OutOfMemory,
    Unknown { stderr: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceLimit {
    Memory,
    Cpu,
    DiskSpace,
}

/// Docker runner errors
#[derive(Debug, thiserror::Error)]
pub enum DockerError {
    #[error("Docker is not installed or not running")]
    DockerNotAvailable,

    #[error("Docker image not found: {0}")]
    ImageNotFound(String),

    #[error("Failed to create container: {0}")]
    ContainerCreationFailed(String),

    #[error("Container execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Failed to cleanup container: {0}")]
    CleanupFailed(String),

    #[error("Timeout exceeded: {0}s")]
    Timeout(u64),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
```

#### Workflow: Code Submission → Results

```
┌─────────────┐
│   Student   │
│   submits   │
│    code     │
└──────┬──────┘
       │
       ▼
┌────────────────────────────────────────────┐
│ 1. Validate Submission                     │
│    - Check file size (<100KB)              │
│    - Basic syntax check (rustfmt parse)    │
│    - Detect forbidden patterns (std::fs::) │
└──────┬─────────────────────────────────────┘
       │
       ▼
┌────────────────────────────────────────────┐
│ 2. Prepare Challenge Directory             │
│    - Create temp dir: /tmp/challenge-{uuid}│
│    - Copy Cargo.toml, tests/*              │
│    - Write student code to src/lib.rs      │
└──────┬─────────────────────────────────────┘
       │
       ▼
┌────────────────────────────────────────────┐
│ 3. Get Container (pre-warmed or new)       │
│    - Try to get from pool                  │
│    - If pool empty, create new container   │
└──────┬─────────────────────────────────────┘
       │
       ▼
┌────────────────────────────────────────────┐
│ 4. Execute Tests                           │
│    - Mount challenge dir (read-only)       │
│    - Run: cargo test --message-format=json │
│    - Stream output to buffer               │
│    - Enforce 30s timeout                   │
└──────┬─────────────────────────────────────┘
       │
       ▼
┌────────────────────────────────────────────┐
│ 5. Parse Results                           │
│    - Parse JSON test events                │
│    - Extract passed/failed counts          │
│    - Capture compile errors                │
│    - Detect panics in stderr               │
└──────┬─────────────────────────────────────┘
       │
       ▼
┌────────────────────────────────────────────┐
│ 6. Cleanup                                 │
│    - Stop container (return to pool)       │
│    - Delete temp directory                 │
│    - Log metrics (duration, memory used)   │
└──────┬─────────────────────────────────────┘
       │
       ▼
┌────────────────────────────────────────────┐
│ 7. Update Progress                         │
│    - If success: award XP, unlock next     │
│    - If failure: save attempt, show hint   │
│    - Record in DB: attempts, time spent    │
└──────┬─────────────────────────────────────┘
       │
       ▼
┌─────────────┐
│   Display   │
│   results   │
│   to user   │
└─────────────┘
```

#### Container Lifecycle Management

**Pre-warming Strategy:**
```rust
// crates/runner/src/pool.rs

use std::collections::VecDeque;
use tokio::sync::Mutex;

pub struct ContainerPool {
    idle: Mutex<VecDeque<Container>>,
    config: DockerConfig,
    max_size: usize,
}

impl ContainerPool {
    pub async fn new(config: DockerConfig) -> Result<Self, DockerError> {
        let pool = Self {
            idle: Mutex::new(VecDeque::new()),
            config: config.clone(),
            max_size: config.pre_warm_pool_size,
        };

        // Pre-warm initial containers
        pool.warm_up().await?;
        Ok(pool)
    }

    async fn warm_up(&self) -> Result<(), DockerError> {
        for i in 0..self.max_size {
            let container = self.create_container().await?;
            let mut idle = self.idle.lock().await;
            idle.push_back(container);
        }
        Ok(())
    }

    pub async fn get(&self) -> Result<Container, DockerError> {
        let mut idle = self.idle.lock().await;

        // Try to get from pool
        if let Some(container) = idle.pop_front() {
            return Ok(container);
        }

        // Pool empty, create new one
        drop(idle); // Release lock
        self.create_container().await
    }

    pub async fn return_container(&self, container: Container) {
        let mut idle = self.idle.lock().await;

        // Only return to pool if under max size
        if idle.len() < self.max_size {
            idle.push_back(container);
        } else {
            // Pool full, destroy container
            tokio::spawn(async move {
                let _ = container.stop().await;
            });
        }
    }

    async fn create_container(&self) -> Result<Container, DockerError> {
        // Implementation using bollard
        todo!()
    }

    pub async fn shutdown(&self) {
        let mut idle = self.idle.lock().await;
        while let Some(container) = idle.pop_front() {
            let _ = container.stop().await;
        }
    }
}
```

**Orphan Container Cleanup:**
```rust
// Run on app startup and periodically
pub async fn cleanup_orphaned_containers(
    docker: &Docker,
) -> Result<(), DockerError> {
    let filters = vec![
        ("label", vec!["app=gamified-rust-challenge"]),
    ];

    let containers = docker
        .list_containers(Some(ListContainersOptions {
            all: true,
            filters: filters.into_iter().collect(),
            ..Default::default()
        }))
        .await?;

    for container in containers {
        let id = container.id.unwrap_or_default();

        // Check if container is stale (created >1 hour ago)
        if let Some(created) = container.created {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;

            if now - created > 3600 {
                println!("Cleaning up orphaned container: {}", id);
                let _ = docker.stop_container(&id, None).await;
                let _ = docker.remove_container(&id, None).await;
            }
        }
    }

    Ok(())
}
```

#### Test Output Parsing

**Cargo JSON Format:**
```json
{"reason":"compiler-message","package_id":"...","message":{...}}
{"reason":"build-finished","success":true}
{"reason":"test","name":"test_addition","event":"started"}
{"reason":"test","name":"test_addition","event":"ok"}
{"reason":"suite","event":"started","test_count":5}
{"reason":"suite","event":"ok","passed":5,"failed":0}
```

**Parser Implementation:**
```rust
// crates/runner/src/parser.rs

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(tag = "reason")]
enum CargoMessage {
    #[serde(rename = "compiler-message")]
    CompilerMessage { message: CompilerDiagnostic },

    #[serde(rename = "build-finished")]
    BuildFinished { success: bool },

    #[serde(rename = "test")]
    Test { name: String, event: TestEvent },

    #[serde(rename = "suite")]
    Suite { event: SuiteEvent },
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum TestEvent {
    Started,
    Ok,
    Failed,
    Ignored,
}

#[derive(Deserialize)]
#[serde(tag = "event")]
enum SuiteEvent {
    #[serde(rename = "started")]
    Started { test_count: u32 },

    #[serde(rename = "ok")]
    Ok { passed: u32, failed: u32, ignored: u32 },

    #[serde(rename = "failed")]
    Failed { passed: u32, failed: u32, ignored: u32 },
}

pub fn parse_cargo_output(output: &str) -> VerificationResult {
    let mut tests_passed = 0;
    let mut tests_failed = 0;
    let mut tests_total = 0;
    let mut compile_error = None;
    let mut build_success = true;

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() || !line.starts_with('{') {
            continue;
        }

        match serde_json::from_str::<CargoMessage>(line) {
            Ok(CargoMessage::CompilerMessage { message }) => {
                if message.level == "error" {
                    compile_error = Some(CompileError {
                        message: message.message.clone(),
                        line: message.spans.first()
                            .and_then(|s| s.line_start),
                        column: message.spans.first()
                            .and_then(|s| s.column_start),
                        file: message.spans.first()
                            .and_then(|s| s.file_name.clone()),
                    });
                }
            }
            Ok(CargoMessage::BuildFinished { success }) => {
                build_success = success;
            }
            Ok(CargoMessage::Test { event, .. }) => {
                match event {
                    TestEvent::Ok => tests_passed += 1,
                    TestEvent::Failed => tests_failed += 1,
                    _ => {}
                }
            }
            Ok(CargoMessage::Suite { event }) => {
                if let SuiteEvent::Started { test_count } = event {
                    tests_total = test_count;
                }
            }
            Err(_) => continue,
        }
    }

    VerificationResult {
        success: build_success && tests_failed == 0,
        tests_passed,
        tests_failed,
        tests_total,
        compile_error,
        // ... other fields
    }
}
```

#### Error Cases Handling

| Error Case | Detection | User Message | Recovery |
|------------|-----------|--------------|----------|
| **Infinite Loop** | 30s timeout | "Your code took too long to run. Check for infinite loops." | Kill container, show hint |
| **Out of Memory** | Docker OOM kill | "Your code used too much memory. Check for memory leaks." | Kill container, show limit |
| **Compilation Error** | `cargo build` fails | Show compiler error with line/column | Display error, allow retry |
| **Panic in Test** | stderr contains "panicked at" | Extract panic message and location | Show panic, suggest debugging |
| **Fork Bomb** | pids-limit reached | "Too many processes created." | Kill container, show limit |
| **File I/O** | stderr contains "Permission denied" | "File I/O is not allowed in challenges." | Explain sandbox restrictions |
| **Network Access** | `network=none` prevents | "Network access is not allowed." | Explain sandbox restrictions |
| **Docker Not Running** | Connection error | "Docker is not running. Please start Docker Desktop." | Link to setup guide |
| **Image Missing** | 404 error | "Docker image not found. Rebuilding..." | Auto-trigger image build |

#### Docker Detection & Setup Guide

**Detection on Startup:**
```rust
// src-tauri/src/docker_check.rs

pub async fn check_docker_setup() -> DockerStatus {
    // 1. Check if Docker is installed
    let docker = match Docker::connect_with_local_defaults() {
        Ok(d) => d,
        Err(_) => return DockerStatus::NotInstalled,
    };

    // 2. Check if Docker daemon is running
    match docker.ping().await {
        Ok(_) => {},
        Err(_) => return DockerStatus::NotRunning,
    };

    // 3. Check if image exists
    match docker.inspect_image(SANDBOX_IMAGE).await {
        Ok(_) => DockerStatus::Ready,
        Err(_) => DockerStatus::ImageMissing,
    }
}

pub enum DockerStatus {
    Ready,
    NotInstalled,
    NotRunning,
    ImageMissing,
}
```

**Setup Instructions UI:**
```typescript
// src/components/DockerSetup.tsx

const SETUP_GUIDES = {
  NotInstalled: {
    title: "Docker is not installed",
    steps: [
      {
        os: "macOS",
        instructions: [
          "1. Download Docker Desktop from docker.com",
          "2. Install and run Docker Desktop",
          "3. Restart this app"
        ],
        link: "https://docs.docker.com/desktop/install/mac-install/"
      },
      {
        os: "Windows",
        instructions: [
          "1. Enable WSL2 (wsl --install in PowerShell)",
          "2. Download Docker Desktop from docker.com",
          "3. Install and run Docker Desktop",
          "4. Restart this app"
        ],
        link: "https://docs.docker.com/desktop/install/windows-install/"
      },
      {
        os: "Linux",
        instructions: [
          "1. Run: sudo apt-get update",
          "2. Run: sudo apt-get install docker.io",
          "3. Run: sudo systemctl start docker",
          "4. Restart this app"
        ],
        link: "https://docs.docker.com/engine/install/ubuntu/"
      }
    ]
  },
  NotRunning: {
    title: "Docker is not running",
    instructions: "Please start Docker Desktop and try again.",
    action: "Retry"
  },
  ImageMissing: {
    title: "Building sandbox image...",
    instructions: "This will take 2-3 minutes the first time.",
    action: "Build Image"
  }
};
```

#### Code Editor Integration

**Decision Matrix:**

| Editor | Pros | Cons | Bundle Size | Verdict |
|--------|------|------|-------------|---------|
| **Monaco** | - VSCode's editor<br>- Syntax highlighting<br>- IntelliSense<br>- Themes | - Large bundle (4MB)<br>- Complex setup | 4MB | ✅ **Future** (Phase 6 polish) |
| **CodeMirror 6** | - Modern, modular<br>- Good Rust support<br>- Lightweight | - Steeper learning curve | 200KB | ⚠️ Backup option |
| **Textarea + Highlight** | - Simple<br>- Fast<br>- Tiny bundle | - No IntelliSense<br>- Basic UX | 50KB | ✅ **MVP** |

**MVP Implementation (Phase 3):**
```typescript
// src/components/ChallengeEditor.tsx

import { useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';

export function ChallengeEditor({ challengeId }: { challengeId: string }) {
  const [code, setCode] = useState('');
  const [result, setResult] = useState<VerificationResult | null>(null);
  const [isRunning, setIsRunning] = useState(false);

  const runTests = async () => {
    setIsRunning(true);
    try {
      const result = await invoke<VerificationResult>('run_challenge', {
        challengeId,
        studentCode: code,
      });
      setResult(result);
    } catch (error) {
      console.error('Failed to run tests:', error);
    } finally {
      setIsRunning(false);
    }
  };

  return (
    <div className="flex flex-col h-full">
      <div className="flex-1">
        <textarea
          className="w-full h-full font-mono text-sm p-4 bg-gray-900 text-gray-100"
          value={code}
          onChange={(e) => setCode(e.target.value)}
          placeholder="// Write your solution here..."
          spellCheck={false}
        />
      </div>

      <div className="border-t p-4">
        <button
          onClick={runTests}
          disabled={isRunning}
          className="btn-primary"
        >
          {isRunning ? 'Running Tests...' : 'Run Tests'}
        </button>
      </div>

      {result && (
        <div className="border-t p-4 bg-gray-50">
          <TestResults result={result} />
        </div>
      )}
    </div>
  );
}
```

**Future Enhancement (Phase 6):**
- Upgrade to Monaco Editor
- Add IntelliSense (local Rust analyzer)
- Add vim/emacs keybindings
- Split-pane view (code | output)

#### Tauri Commands

```rust
// src-tauri/src/commands.rs

use crate::runner::{DockerRunner, VerificationResult};

#[tauri::command]
pub async fn check_docker_status() -> Result<DockerStatus, String> {
    check_docker_setup().await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn build_docker_image(
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let runner = state.docker_runner.lock().await;
    runner.build_image()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn run_challenge(
    state: tauri::State<'_, AppState>,
    challenge_id: String,
    student_code: String,
) -> Result<VerificationResult, String> {
    // Load challenge from content tree
    let challenge = state
        .content
        .get_challenge(&challenge_id)
        .ok_or("Challenge not found")?;

    // Run verification
    let runner = state.docker_runner.lock().await;
    let result = runner
        .run_verification(&challenge, &student_code)
        .await
        .map_err(|e| e.to_string())?;

    // Update progress if success
    if result.success {
        let db = state.db.lock().await;
        db.update_node_progress(
            &state.user_id,
            &challenge_id,
            NodeStatus::Completed,
        )?;

        // Award XP
        let xp = calculate_xp(&challenge, &result);
        db.award_xp(&state.user_id, xp)?;
    }

    Ok(result)
}
```

---

## Milestone 3.2: LLM Grading Integration

**Duration:** 3-4 days
**Dependencies:** Milestone 3.1 (for caching pattern), Phase 2 (progress system)

### Technology Decision: OpenAI Client

#### Research: Rust OpenAI Libraries

| Library | Pros | Cons | Verdict |
|---------|------|------|---------|
| **async-openai** | - Official-ish (OpenAI org member)<br>- Full API support<br>- Type-safe<br>- Active (2024) | - Requires Tokio | ✅ **RECOMMENDED** |
| **openai-api-rs** | - Simple API<br>- Sync support | - Less feature-complete<br>- Smaller community | ⚠️ Backup |
| **reqwest + manual** | - Full control<br>- Minimal deps | - Manual schema work<br>- No retry logic | ❌ Too much work |

**Decision:** Use `async-openai` for production.

**Cargo.toml:**
```toml
[dependencies]
async-openai = "0.18"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sha2 = "0.10"  # For content hashing
```

### Design: Grading Cache Implementation

**Schema (already in Phase 1):**
```sql
CREATE TABLE grade_cache (
    content_hash TEXT PRIMARY KEY,    -- SHA-256 of artifact content
    artifact_type TEXT NOT NULL,      -- "DESIGN", "README", etc.
    grade INTEGER NOT NULL,            -- 0-100
    reasoning TEXT NOT NULL,           -- LLM feedback
    category_scores TEXT NOT NULL,     -- JSON array of category scores
    cached_at TEXT NOT NULL,           -- ISO timestamp
    hit_count INTEGER DEFAULT 0        -- Track cache effectiveness
);

CREATE INDEX idx_grade_cache_type ON grade_cache(artifact_type);
CREATE INDEX idx_grade_cache_date ON grade_cache(cached_at);
```

**Cache Implementation:**
```rust
// crates/grader/src/cache.rs

use sha2::{Sha256, Digest};
use rusqlite::{Connection, params};

pub struct GradeCache {
    conn: Connection,
}

impl GradeCache {
    pub fn new(db_path: &Path) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(db_path)?;
        Ok(Self { conn })
    }

    pub fn get(
        &self,
        content: &str,
        artifact_type: &str,
    ) -> Result<Option<CachedGrade>, rusqlite::Error> {
        let hash = Self::hash_content(content);

        let mut stmt = self.conn.prepare(
            "SELECT grade, reasoning, category_scores, cached_at
             FROM grade_cache
             WHERE content_hash = ?1 AND artifact_type = ?2"
        )?;

        let result = stmt.query_row(params![hash, artifact_type], |row| {
            Ok(CachedGrade {
                grade: row.get(0)?,
                reasoning: row.get(1)?,
                category_scores: serde_json::from_str(&row.get::<_, String>(2)?).ok()?,
                cached_at: row.get(3)?,
            })
        });

        // Increment hit count if found
        if result.is_ok() {
            let _ = self.conn.execute(
                "UPDATE grade_cache SET hit_count = hit_count + 1
                 WHERE content_hash = ?1",
                params![hash],
            );
        }

        match result {
            Ok(grade) => Ok(Some(grade)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn set(
        &self,
        content: &str,
        artifact_type: &str,
        grade: u32,
        reasoning: &str,
        category_scores: &[CategoryScore],
    ) -> Result<(), rusqlite::Error> {
        let hash = Self::hash_content(content);
        let now = chrono::Utc::now().to_rfc3339();
        let scores_json = serde_json::to_string(category_scores).unwrap();

        self.conn.execute(
            "INSERT INTO grade_cache (content_hash, artifact_type, grade, reasoning, category_scores, cached_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(content_hash) DO UPDATE SET
                grade = excluded.grade,
                reasoning = excluded.reasoning,
                category_scores = excluded.category_scores,
                cached_at = excluded.cached_at",
            params![hash, artifact_type, grade, reasoning, scores_json, now],
        )?;

        Ok(())
    }

    fn hash_content(content: &str) -> String {
        let mut hasher = Sha256::new();

        // Normalize content before hashing to improve cache hits
        let normalized = content
            .lines()
            .map(|line| line.trim_end())  // Remove trailing whitespace
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string();

        hasher.update(normalized.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn stats(&self) -> Result<CacheStats, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT COUNT(*), SUM(hit_count) FROM grade_cache"
        )?;

        let (total_entries, total_hits) = stmt.query_row([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
        })?;

        Ok(CacheStats {
            total_entries: total_entries as usize,
            total_hits: total_hits as usize,
        })
    }
}

#[derive(Debug)]
pub struct CachedGrade {
    pub grade: u32,
    pub reasoning: String,
    pub category_scores: Vec<CategoryScore>,
    pub cached_at: String,
}

pub struct CacheStats {
    pub total_entries: usize,
    pub total_hits: usize,
}
```

### Design: Rubric JSON Schema

**Unified Schema for All 5 Artifact Types:**

```typescript
// Schema definition (for documentation)
interface Rubric {
  artifact_type: "DESIGN" | "README" | "BENCH" | "RUNBOOK" | "INVARIANTS";
  total_points: number;  // Always 100
  categories: Category[];
  grading_guidelines: {
    [grade: string]: string;  // "A (90-100)": "description"
  };
  mandatory_sections: string[];
}

interface Category {
  name: string;
  points: number;
  criteria?: Criterion[];
  indicators?: {
    excellent: string;
    good: string;
    poor: string;
  };
}

interface Criterion {
  description: string;
  points: number;
  indicators: {
    excellent: string;
    good: string;
    poor: string;
  };
}
```

**Rubric Files** (build on existing DESIGN.md and README.md):

1. **DESIGN.md** - Already exists (see prototype)
2. **README.md** - Already exists (see prototype)
3. **BENCH.md** - New

```json
{
  "artifact_type": "BENCH.md",
  "total_points": 100,
  "categories": [
    {
      "name": "Benchmark Setup",
      "points": 25,
      "criteria": [
        {
          "description": "Uses Criterion.rs or similar framework",
          "points": 10,
          "indicators": {
            "excellent": "Proper Criterion configuration with statistical analysis",
            "good": "Basic Criterion setup",
            "poor": "No benchmarking framework or manual timing"
          }
        },
        {
          "description": "Multiple scenarios benchmarked",
          "points": 15,
          "indicators": {
            "excellent": "5+ scenarios covering different input sizes/types",
            "good": "2-3 scenarios",
            "poor": "Single scenario or no variety"
          }
        }
      ]
    },
    {
      "name": "Benchmark Results",
      "points": 30,
      "criteria": [
        {
          "description": "Results clearly presented",
          "points": 15,
          "indicators": {
            "excellent": "Table or graph with timing, throughput, comparisons",
            "good": "Basic timing results",
            "poor": "Vague or missing results"
          }
        },
        {
          "description": "Statistical significance discussed",
          "points": 15,
          "indicators": {
            "excellent": "Standard deviation, confidence intervals, sample size",
            "good": "Mentions variance or multiple runs",
            "poor": "Single run or no statistical analysis"
          }
        }
      ]
    },
    {
      "name": "Performance Analysis",
      "points": 25,
      "criteria": [
        {
          "description": "Bottlenecks identified",
          "points": 12,
          "indicators": {
            "excellent": "Specific hot spots with evidence (profiling, analysis)",
            "good": "General areas of slowness mentioned",
            "poor": "No performance analysis"
          }
        },
        {
          "description": "Optimization opportunities",
          "points": 13,
          "indicators": {
            "excellent": "Specific improvements suggested with rationale",
            "good": "General optimization ideas",
            "poor": "No optimization discussion"
          }
        }
      ]
    },
    {
      "name": "Methodology",
      "points": 20,
      "criteria": [
        {
          "description": "Test environment documented",
          "points": 10,
          "indicators": {
            "excellent": "Hardware specs, OS, compiler version, optimization flags",
            "good": "Basic environment info",
            "poor": "No environment details"
          }
        },
        {
          "description": "Reproducibility",
          "points": 10,
          "indicators": {
            "excellent": "Instructions to reproduce + seed data + cargo bench commands",
            "good": "Basic instructions",
            "poor": "Cannot reproduce results"
          }
        }
      ]
    }
  ],
  "grading_guidelines": {
    "A (90-100)": "Comprehensive benchmarking with proper framework, multiple scenarios, statistical analysis, and clear results.",
    "B (80-89)": "Good benchmarking with framework and results. Minor gaps in analysis or methodology.",
    "C (70-79)": "Basic benchmarks with some results. Missing statistical rigor or variety.",
    "D (60-69)": "Minimal benchmarking. Poor methodology or incomplete results.",
    "F (0-59)": "No proper benchmarks or severely lacking documentation."
  },
  "mandatory_sections": [
    "Benchmark results with timing",
    "Test environment documentation"
  ]
}
```

4. **RUNBOOK.md** - New

```json
{
  "artifact_type": "RUNBOOK.md",
  "total_points": 100,
  "categories": [
    {
      "name": "Setup Instructions",
      "points": 20,
      "criteria": [
        {
          "description": "Prerequisites listed",
          "points": 5,
          "indicators": {
            "excellent": "Specific versions and system requirements",
            "good": "Basic prerequisites",
            "poor": "Missing prerequisites"
          }
        },
        {
          "description": "Installation steps",
          "points": 15,
          "indicators": {
            "excellent": "Step-by-step with exact commands and expected output",
            "good": "Basic install commands",
            "poor": "Vague or missing installation"
          }
        }
      ]
    },
    {
      "name": "Operations Guide",
      "points": 40,
      "criteria": [
        {
          "description": "Start/stop procedures",
          "points": 15,
          "indicators": {
            "excellent": "Commands for all environments (dev, prod) with verification steps",
            "good": "Basic start/stop commands",
            "poor": "Missing operational procedures"
          }
        },
        {
          "description": "Configuration guide",
          "points": 15,
          "indicators": {
            "excellent": "All config options documented with examples and defaults",
            "good": "Main config options listed",
            "poor": "No configuration documentation"
          }
        },
        {
          "description": "Health checks",
          "points": 10,
          "indicators": {
            "excellent": "How to verify system is working + what to check",
            "good": "Basic health check mentioned",
            "poor": "No health verification"
          }
        }
      ]
    },
    {
      "name": "Troubleshooting",
      "points": 30,
      "criteria": [
        {
          "description": "Common issues documented",
          "points": 15,
          "indicators": {
            "excellent": "5+ issues with symptoms, causes, and solutions",
            "good": "2-3 common issues",
            "poor": "No troubleshooting section"
          }
        },
        {
          "description": "Debugging procedures",
          "points": 15,
          "indicators": {
            "excellent": "How to enable logging, find logs, interpret errors",
            "good": "Basic debugging info",
            "poor": "No debugging guidance"
          }
        }
      ]
    },
    {
      "name": "Maintenance",
      "points": 10,
      "criteria": [
        {
          "description": "Routine maintenance tasks",
          "points": 10,
          "indicators": {
            "excellent": "Scheduled tasks (backups, cleanup, updates) with frequency",
            "good": "Basic maintenance mentioned",
            "poor": "No maintenance procedures"
          }
        }
      ]
    }
  ],
  "grading_guidelines": {
    "A (90-100)": "Complete operational guide. Setup, operations, troubleshooting, and maintenance all well-documented.",
    "B (80-89)": "Good runbook with most operational procedures. Minor gaps in detail.",
    "C (70-79)": "Basic runbook covering essentials. Missing troubleshooting or maintenance.",
    "D (60-69)": "Minimal runbook. Hard to operate system from documentation alone.",
    "F (0-59)": "Incomplete or unusable operational documentation."
  },
  "mandatory_sections": [
    "Setup/installation instructions",
    "Start/stop procedures",
    "At least one troubleshooting scenario"
  ]
}
```

5. **INVARIANTS.md** - New

```json
{
  "artifact_type": "INVARIANTS.md",
  "total_points": 100,
  "categories": [
    {
      "name": "Data Invariants",
      "points": 35,
      "criteria": [
        {
          "description": "Type invariants documented",
          "points": 15,
          "indicators": {
            "excellent": "All types with constraints (e.g., 'User.age: 0-120', 'IDs non-empty')",
            "good": "Main types with some constraints",
            "poor": "No type invariants"
          }
        },
        {
          "description": "Relational invariants",
          "points": 20,
          "indicators": {
            "excellent": "Relationships between data (e.g., 'User must have ≥1 account')",
            "good": "Some relationships mentioned",
            "poor": "No relational invariants"
          }
        }
      ]
    },
    {
      "name": "State Invariants",
      "points": 30,
      "criteria": [
        {
          "description": "Valid state transitions",
          "points": 15,
          "indicators": {
            "excellent": "State machine diagram or explicit transition rules",
            "good": "Valid states listed",
            "poor": "No state invariants"
          }
        },
        {
          "description": "Illegal states prevented",
          "points": 15,
          "indicators": {
            "excellent": "Specific illegal states + how code prevents them",
            "good": "Some illegal states mentioned",
            "poor": "No illegal state discussion"
          }
        }
      ]
    },
    {
      "name": "System Invariants",
      "points": 25,
      "criteria": [
        {
          "description": "Concurrency invariants",
          "points": 13,
          "indicators": {
            "excellent": "Thread safety guarantees + synchronization strategy",
            "good": "Basic concurrency considerations",
            "poor": "No concurrency invariants"
          }
        },
        {
          "description": "Resource invariants",
          "points": 12,
          "indicators": {
            "excellent": "Resource limits and cleanup guarantees",
            "good": "Some resource constraints",
            "poor": "No resource invariants"
          }
        }
      ]
    },
    {
      "name": "Verification",
      "points": 10,
      "criteria": [
        {
          "description": "How invariants are enforced",
          "points": 10,
          "indicators": {
            "excellent": "Specific code mechanisms (assertions, types, tests) for each invariant",
            "good": "General enforcement strategy",
            "poor": "No verification discussion"
          }
        }
      ]
    }
  ],
  "grading_guidelines": {
    "A (90-100)": "Comprehensive invariants covering data, state, and system level. Clear enforcement mechanisms.",
    "B (80-89)": "Good invariant documentation. Minor gaps in coverage or verification.",
    "C (70-79)": "Basic invariants documented. Missing system-level or verification details.",
    "D (60-69)": "Minimal invariants. Incomplete or vague.",
    "F (0-59)": "Severely lacking invariant documentation."
  },
  "mandatory_sections": [
    "Data invariants",
    "State invariants",
    "How invariants are enforced"
  ]
}
```

### Prompt Engineering Strategy

**System Message vs User Message Split:**

```rust
// System message (constant, sets role)
const SYSTEM_MESSAGE: &str = r#"You are an expert code reviewer and educator grading student project artifacts for a Rust bootcamp.

Your role is to:
1. Evaluate artifacts against structured rubrics
2. Provide constructive, specific feedback
3. Be strict but fair in scoring
4. Help students improve their technical writing

Grading philosophy:
- Reward clarity, completeness, and technical depth
- Penalize vagueness, missing sections, and superficial analysis
- Focus on substance over style (but clarity matters)
- Compare to professional-level documentation"#;

// User message (dynamic, includes artifact + rubric)
fn build_user_message(artifact: &str, rubric: &Rubric) -> String {
    format!(r#"# GRADING TASK

## Artifact Type: {}

## Rubric
{}

## Student Submission
```
{}
```

## Instructions
1. Read the student's artifact carefully
2. Evaluate against each category in the rubric
3. Score each criterion using the indicators (excellent/good/poor)
4. Provide specific feedback citing examples from the artifact
5. Calculate total score

## Output Format
Respond with ONLY valid JSON in this exact format (no markdown, no code blocks):

{{
  "total_score": <number 0-100>,
  "overall_feedback": "<2-3 sentences summarizing quality and areas for improvement>",
  "category_scores": [
    {{
      "category": "<category name>",
      "score": <number>,
      "max_score": <number>,
      "feedback": "<specific feedback with examples>"
    }}
  ]
}}

Be specific in your feedback. Quote or reference specific parts of the artifact."#,
        rubric.artifact_type,
        serde_json::to_string_pretty(rubric).unwrap(),
        artifact
    )
}
```

**Few-Shot Examples Strategy:**

For MVP, rely on well-structured rubric. For Phase 6 improvement, add few-shot examples:

```rust
const FEW_SHOT_EXAMPLES: &str = r#"

## Example 1: Excellent DESIGN.md

```markdown
# Architecture

## Components

### Core Engine
- Responsibility: Game state management, rule enforcement
- Interface: `fn make_move(state: &GameState, move: Move) -> Result<GameState, MoveError>`
...
```

**Score: 95/100**
- Clear component breakdown ✓
- Specific interfaces ✓
- Error handling documented ✓

## Example 2: Poor DESIGN.md

```markdown
# Design

The app has a frontend and backend. The frontend is in React and the backend is in Rust.
```

**Score: 25/100**
- No component details
- No interfaces
- No data structures
"#;
```

**Temperature & Consistency:**

```rust
pub struct GraderConfig {
    pub model: String,
    pub temperature: f64,
    pub max_tokens: u32,
    pub timeout: Duration,
}

impl Default for GraderConfig {
    fn default() -> Self {
        Self {
            model: "gpt-4".to_string(),
            temperature: 0.3,  // Low for consistency
            max_tokens: 2000,  // Enough for detailed feedback
            timeout: Duration::from_secs(30),
        }
    }
}
```

### Error Handling

**Error Types:**

```rust
#[derive(Debug, thiserror::Error)]
pub enum GraderError {
    #[error("OpenAI API error: {0}")]
    ApiError(String),

    #[error("Rate limit exceeded. Retry after {0}s")]
    RateLimit(u64),

    #[error("Request timeout after {0}s")]
    Timeout(u64),

    #[error("Failed to parse LLM response: {0}")]
    ParseError(String),

    #[error("Invalid artifact: {0}")]
    InvalidArtifact(String),

    #[error("Cache error: {0}")]
    CacheError(#[from] rusqlite::Error),
}
```

**Retry Strategy:**

```rust
use tokio::time::{sleep, Duration};

pub async fn grade_with_retry(
    client: &OpenAIClient,
    prompt: &str,
    max_retries: u32,
) -> Result<GradeResult, GraderError> {
    let mut retries = 0;
    let mut backoff = Duration::from_secs(1);

    loop {
        match client.grade(prompt).await {
            Ok(result) => return Ok(result),
            Err(GraderError::RateLimit(retry_after)) => {
                if retries >= max_retries {
                    return Err(GraderError::RateLimit(retry_after));
                }
                sleep(Duration::from_secs(retry_after)).await;
                retries += 1;
            }
            Err(GraderError::Timeout(_)) => {
                if retries >= max_retries {
                    return Err(GraderError::Timeout(30));
                }
                sleep(backoff).await;
                backoff *= 2; // Exponential backoff
                retries += 1;
            }
            Err(e) => return Err(e), // Don't retry other errors
        }
    }
}
```

**User-Facing Error Messages:**

| Error | User Message | Recovery Action |
|-------|--------------|-----------------|
| **API Key Missing** | "OpenAI API key not configured. Go to Settings → API Keys" | Show settings dialog |
| **API Key Invalid** | "OpenAI API key is invalid. Please check your key." | Allow re-entry |
| **Rate Limit** | "Too many grading requests. Try again in 60 seconds." | Queue for later |
| **Timeout** | "Grading is taking longer than expected. Retrying..." | Auto-retry 3x |
| **Network Error** | "Cannot connect to OpenAI. Check your internet connection." | Retry button |
| **Parse Error** | "Received invalid response. Retrying..." | Auto-retry 1x |
| **Artifact Too Large** | "Artifact exceeds 50KB limit. Please shorten it." | Show size, block submit |

### Cost Optimization

**Caching Strategy:**

```rust
pub async fn grade_artifact(
    grader: &LLMGrader,
    cache: &GradeCache,
    artifact: &str,
    rubric: &Rubric,
) -> Result<GradeResult, GraderError> {
    // 1. Check cache first
    if let Some(cached) = cache.get(artifact, &rubric.artifact_type)? {
        println!("Cache hit! Saved ~$0.50");
        return Ok(GradeResult {
            score: cached.grade,
            reasoning: cached.reasoning,
            category_scores: cached.category_scores,
            from_cache: true,
            latency_ms: 0,
        });
    }

    // 2. Cache miss, call LLM
    println!("Cache miss. Calling OpenAI...");
    let result = grader.grade(artifact, rubric).await?;

    // 3. Cache result for next time
    cache.set(
        artifact,
        &rubric.artifact_type,
        result.score,
        &result.reasoning,
        &result.category_scores,
    )?;

    Ok(result)
}
```

**Batch Grading:**

OpenAI doesn't support batch requests in streaming API, but we can:

```rust
// Grade all 5 artifacts for a checkpoint in parallel
pub async fn grade_checkpoint(
    grader: &LLMGrader,
    cache: &GradeCache,
    artifacts: &CheckpointArtifacts,
) -> Result<CheckpointGrade, GraderError> {
    let futures = vec![
        grade_artifact(grader, cache, &artifacts.design, &DESIGN_RUBRIC),
        grade_artifact(grader, cache, &artifacts.readme, &README_RUBRIC),
        grade_artifact(grader, cache, &artifacts.bench, &BENCH_RUBRIC),
        grade_artifact(grader, cache, &artifacts.runbook, &RUNBOOK_RUBRIC),
        grade_artifact(grader, cache, &artifacts.invariants, &INVARIANTS_RUBRIC),
    ];

    // Run in parallel (OpenAI allows 3000 RPM on standard tier)
    let results = futures::future::try_join_all(futures).await?;

    Ok(CheckpointGrade {
        design: results[0].clone(),
        readme: results[1].clone(),
        bench: results[2].clone(),
        runbook: results[3].clone(),
        invariants: results[4].clone(),
        total_score: results.iter().map(|r| r.score).sum::<u32>() / 5,
    })
}
```

**Daily Limits:**

```rust
// Prevent cost runaway
pub struct GradingQuota {
    user_id: String,
    daily_limit: u32,
    db: Connection,
}

impl GradingQuota {
    pub fn check_quota(&self) -> Result<bool, rusqlite::Error> {
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

        let count: u32 = self.db.query_row(
            "SELECT COUNT(*) FROM artifact_submissions
             WHERE user_id = ?1 AND DATE(graded_at) = ?2",
            params![self.user_id, today],
            |row| row.get(0),
        )?;

        Ok(count < self.daily_limit)
    }
}

const DAILY_GRADE_LIMIT: u32 = 20; // ~$10/day max
```

### Grade Display UI

**Results Component:**

```typescript
// src/components/GradeResults.tsx

interface GradeResultsProps {
  grade: CheckpointGrade;
}

export function GradeResults({ grade }: GradeResultsProps) {
  return (
    <div className="space-y-6">
      {/* Overall Score */}
      <div className="bg-white rounded-lg shadow p-6">
        <div className="text-center">
          <div className="text-6xl font-bold text-blue-600">
            {grade.total_score}/100
          </div>
          <div className="text-xl mt-2">
            {getLetterGrade(grade.total_score)}
          </div>
        </div>
      </div>

      {/* Per-Artifact Breakdown */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {Object.entries(grade).map(([name, result]) => (
          name !== 'total_score' && (
            <ArtifactCard key={name} name={name} result={result} />
          )
        ))}
      </div>

      {/* Detailed Feedback */}
      {Object.entries(grade).map(([name, result]) => (
        name !== 'total_score' && (
          <FeedbackSection key={name} name={name} result={result} />
        )
      ))}
    </div>
  );
}

function ArtifactCard({ name, result }) {
  return (
    <div className="bg-white rounded-lg shadow p-4">
      <h3 className="font-semibold text-lg mb-2">
        {name.toUpperCase()}.md
      </h3>
      <div className="flex items-baseline">
        <span className="text-3xl font-bold text-blue-600">
          {result.score}
        </span>
        <span className="text-gray-500 ml-1">/100</span>
      </div>
      <p className="text-sm text-gray-600 mt-2 line-clamp-2">
        {result.overall_feedback}
      </p>
      {result.from_cache && (
        <span className="text-xs text-green-600 mt-2 inline-block">
          ✓ Cached result
        </span>
      )}
    </div>
  );
}

function FeedbackSection({ name, result }) {
  return (
    <div className="bg-white rounded-lg shadow p-6">
      <h3 className="text-xl font-semibold mb-4">
        {name.toUpperCase()}.md Feedback
      </h3>

      <p className="text-gray-700 mb-4">{result.overall_feedback}</p>

      <div className="space-y-4">
        {result.category_scores.map((category, idx) => (
          <div key={idx} className="border-l-4 border-blue-500 pl-4">
            <div className="flex justify-between items-start mb-2">
              <h4 className="font-medium">{category.category}</h4>
              <span className="text-sm font-semibold">
                {category.score}/{category.max_score}
              </span>
            </div>
            <p className="text-sm text-gray-600">{category.feedback}</p>
          </div>
        ))}
      </div>
    </div>
  );
}

function getLetterGrade(score: number): string {
  if (score >= 90) return 'A - Excellent';
  if (score >= 80) return 'B - Good';
  if (score >= 70) return 'C - Satisfactory';
  if (score >= 60) return 'D - Needs Improvement';
  return 'F - Unsatisfactory';
}
```

### LLM Prompt Templates

**Template 1: Standard Grading**

```rust
const STANDARD_GRADING_PROMPT: &str = r#"You are an expert code reviewer grading a student's {artifact_type} artifact.

# GRADING RUBRIC

{rubric}

# STUDENT ARTIFACT

```
{artifact_content}
```

# INSTRUCTIONS

1. Carefully read the student's artifact
2. Evaluate it against each category in the rubric
3. Provide a score for each category (be strict but fair)
4. Write specific feedback explaining the score
5. Calculate the total score

# OUTPUT FORMAT

Respond in this exact JSON format:

{{
  "total_score": <number>,
  "overall_feedback": "<2-3 sentences>",
  "category_scores": [
    {{
      "category": "<name>",
      "score": <number>,
      "max_score": <number>,
      "feedback": "<specific feedback>"
    }}
  ]
}}

Be specific. Point out what was done well and what was missing or could be improved."#;
```

**Template 2: Improvement Suggestions**

```rust
const IMPROVEMENT_PROMPT: &str = r#"You previously graded this {artifact_type} and gave it {previous_score}/100.

The student has revised it. Compare the new version to identify improvements and remaining issues.

# PREVIOUS FEEDBACK

{previous_feedback}

# REVISED ARTIFACT

```
{artifact_content}
```

# INSTRUCTIONS

1. Identify what the student improved
2. Note what still needs work
3. Provide a new score
4. Encourage progress while being honest about remaining gaps

Focus on growth and learning."#;
```

**Template 3: Quick Validation**

```rust
const VALIDATION_PROMPT: &str = r#"Quickly validate this {artifact_type} artifact.

Check for:
1. All mandatory sections present
2. Reasonable length (not too short)
3. Code blocks properly formatted
4. Clear structure

Respond with:
{{
  "valid": <true/false>,
  "issues": ["<issue>", ...]
}}

This is a fast pre-check before full grading."#;
```

### Grading Retries & Consistency

**Consistency Checking:**

```rust
pub async fn grade_with_consistency_check(
    grader: &LLMGrader,
    cache: &GradeCache,
    artifact: &str,
    rubric: &Rubric,
) -> Result<GradeResult, GraderError> {
    // Check cache first
    if let Some(cached) = cache.get(artifact, &rubric.artifact_type)? {
        return Ok(cached.into());
    }

    // Not cached, grade 3 times and check consistency
    let mut results = vec![];
    for _ in 0..3 {
        let result = grader.grade(artifact, rubric).await?;
        results.push(result);
    }

    // Check consistency (std dev <= 5 points)
    let scores: Vec<f64> = results.iter().map(|r| r.score as f64).collect();
    let mean = scores.iter().sum::<f64>() / scores.len() as f64;
    let variance = scores.iter()
        .map(|s| (s - mean).powi(2))
        .sum::<f64>() / scores.len() as f64;
    let std_dev = variance.sqrt();

    if std_dev > 5.0 {
        // Inconsistent, try once more with stricter prompt
        eprintln!("Inconsistent grading (σ={}). Retrying...", std_dev);
        let result = grader.grade_strict(artifact, rubric).await?;
        results.push(result);
    }

    // Use median score to reduce outlier impact
    let mut sorted_scores = scores.clone();
    sorted_scores.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median_idx = sorted_scores.len() / 2;
    let median_score = sorted_scores[median_idx] as u32;

    // Find result closest to median
    let final_result = results.iter()
        .min_by_key(|r| (r.score as i32 - median_score as i32).abs())
        .unwrap()
        .clone();

    // Cache the result
    cache.set(
        artifact,
        &rubric.artifact_type,
        final_result.score,
        &final_result.reasoning,
        &final_result.category_scores,
    )?;

    Ok(final_result)
}
```

**For MVP:** Skip consistency check, just cache results. Add in Phase 6 if issues arise.

---

## Cross-Cutting Concerns

### Integration with Progress System

**Mini-Challenge Completion:**

```rust
// After successful verification
pub async fn complete_mini_challenge(
    db: &Database,
    user_id: &str,
    challenge_id: &str,
    result: &VerificationResult,
) -> Result<(), DbError> {
    // Update node progress
    db.update_node_progress(user_id, challenge_id, NodeStatus::Completed)?;

    // Calculate XP (difficulty × speed bonus × streak)
    let challenge = db.get_challenge(challenge_id)?;
    let xp = calculate_challenge_xp(&challenge, result);

    // Award XP
    db.award_xp(user_id, xp)?;

    // Update mastery for relevant skills
    for skill in &challenge.skills_trained {
        db.update_mastery(user_id, skill, 0.1)?; // +10% mastery
    }

    // Unlock next node if prerequisites met
    if let Some(next_id) = challenge.unlocks {
        if db.check_prerequisites_met(user_id, &next_id)? {
            db.unlock_node(user_id, &next_id)?;
        }
    }

    // Check badge unlocks
    check_badge_unlocks(db, user_id)?;

    Ok(())
}

fn calculate_challenge_xp(
    challenge: &Challenge,
    result: &VerificationResult,
) -> u32 {
    let base_xp = challenge.difficulty * 50; // Easy=50, Medium=100, Hard=150

    // Speed bonus (if completed quickly)
    let speed_bonus = if result.duration_ms < 5000 {
        1.5 // 50% bonus
    } else {
        1.0
    };

    // First-try bonus
    let attempt_bonus = if challenge.attempts == 0 {
        1.2 // 20% bonus
    } else {
        1.0
    };

    (base_xp as f64 * speed_bonus * attempt_bonus) as u32
}
```

**Checkpoint Completion:**

```rust
pub async fn complete_checkpoint(
    db: &Database,
    user_id: &str,
    checkpoint_id: &str,
    grade: &CheckpointGrade,
) -> Result<(), DbError> {
    // Must score ≥70 overall to pass
    let passing = grade.total_score >= 70;

    let status = if passing {
        NodeStatus::Completed
    } else {
        NodeStatus::Failed
    };

    db.update_node_progress(user_id, checkpoint_id, status)?;

    if passing {
        // Award XP (checkpoints give more XP than challenges)
        let checkpoint = db.get_checkpoint(checkpoint_id)?;
        let xp = checkpoint.difficulty * 200; // 200-600 XP
        db.award_xp(user_id, xp)?;

        // Significant mastery boost
        for skill in &checkpoint.skills_trained {
            db.update_mastery(user_id, skill, 0.3)?; // +30% mastery
        }

        // Unlock next week if this was final checkpoint
        if let Some(next_id) = checkpoint.unlocks {
            if db.check_prerequisites_met(user_id, &next_id)? {
                db.unlock_node(user_id, &next_id)?;
            }
        }

        check_badge_unlocks(db, user_id)?;
    }

    Ok(())
}
```

**Daily Session Integration:**

```rust
// Update session planner to include challenges
pub fn generate_daily_session(
    db: &Database,
    user_id: &str,
) -> Result<DailySession, DbError> {
    let mut activities = vec![];

    // 1. One lecture (if available)
    if let Some(lecture) = db.get_next_unlocked_lecture(user_id)? {
        activities.push(SessionActivity::Lecture(lecture));
    }

    // 2. One quiz
    if let Some(quiz) = db.get_next_unlocked_quiz(user_id)? {
        activities.push(SessionActivity::Quiz(quiz));
    }

    // 3. One mini-challenge (new!)
    if let Some(challenge) = db.get_next_unlocked_challenge(user_id)? {
        activities.push(SessionActivity::Challenge(challenge));
    }

    // 4. Reviews (if any due)
    let reviews = db.get_due_reviews(user_id)?;
    for review in reviews.into_iter().take(3) {
        activities.push(SessionActivity::Review(review));
    }

    Ok(DailySession {
        activities,
        estimated_minutes: 30,
    })
}
```

---

## Testing Strategy

### Unit Tests

**Docker Runner:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_successful_challenge() {
        let runner = DockerRunner::new(DockerConfig::default());

        let student_code = r#"
            pub fn add(a: i32, b: i32) -> i32 {
                a + b
            }
        "#;

        let result = runner.run_verification(
            Path::new("tests/fixtures/add_challenge"),
            student_code,
        ).await.unwrap();

        assert!(result.success);
        assert_eq!(result.tests_passed, 3);
        assert_eq!(result.tests_failed, 0);
    }

    #[tokio::test]
    async fn test_compile_error() {
        let runner = DockerRunner::new(DockerConfig::default());

        let student_code = "this is not rust";

        let result = runner.run_verification(
            Path::new("tests/fixtures/add_challenge"),
            student_code,
        ).await.unwrap();

        assert!(!result.success);
        assert!(result.compile_error.is_some());
    }

    #[tokio::test]
    async fn test_timeout() {
        let runner = DockerRunner::new(DockerConfig {
            timeout: Duration::from_secs(2),
            ..Default::default()
        });

        let student_code = r#"
            pub fn add(a: i32, b: i32) -> i32 {
                loop {} // Infinite loop
            }
        "#;

        let result = runner.run_verification(
            Path::new("tests/fixtures/add_challenge"),
            student_code,
        ).await.unwrap();

        assert!(!result.success);
        assert!(matches!(result.runtime_error, Some(RuntimeError::Timeout)));
    }

    #[tokio::test]
    async fn test_memory_limit() {
        let runner = DockerRunner::new(DockerConfig::default());

        let student_code = r#"
            pub fn add(a: i32, b: i32) -> i32 {
                let v: Vec<u8> = vec![0; 1_000_000_000]; // 1GB
                a + b
            }
        "#;

        let result = runner.run_verification(
            Path::new("tests/fixtures/add_challenge"),
            student_code,
        ).await.unwrap();

        assert!(!result.success);
        assert!(matches!(result.runtime_error, Some(RuntimeError::OutOfMemory)));
    }
}
```

**LLM Grader:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_hit() {
        let cache = GradeCache::new(Path::new(":memory:")).unwrap();
        let grader = LLMGrader::new("test-key".to_string());

        let artifact = "# Test\n\nSome content";
        let rubric = Rubric::from_file(Path::new("rubrics/design.json")).unwrap();

        // First call - cache miss (mock LLM response)
        cache.set(artifact, &rubric.artifact_type, 85, "Good", &vec![]).unwrap();

        // Second call - should hit cache
        let result = cache.get(artifact, &rubric.artifact_type).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().grade, 85);
    }

    #[test]
    fn test_content_hash_normalization() {
        let cache = GradeCache::new(Path::new(":memory:")).unwrap();

        // Same content with different whitespace
        let content1 = "# Test  \n\nSome content";
        let content2 = "# Test\n\nSome content";

        let hash1 = GradeCache::hash_content(content1);
        let hash2 = GradeCache::hash_content(content2);

        assert_eq!(hash1, hash2, "Hashes should match after normalization");
    }

    #[test]
    fn test_rubric_validation() {
        let rubric = Rubric::from_file(Path::new("rubrics/design.json")).unwrap();

        assert_eq!(rubric.total_points, 100);
        assert!(!rubric.categories.is_empty());
        assert!(rubric.mandatory_sections.len() >= 1);
    }
}
```

### Integration Tests

**End-to-End Challenge Flow:**

```rust
#[tokio::test]
async fn test_challenge_completion_flow() {
    let db = setup_test_db().await;
    let docker = DockerRunner::new(DockerConfig::default());
    let user_id = "test-user";

    // 1. Get challenge
    let challenge = db.get_challenge("week1_day1_challenge1").unwrap();
    assert_eq!(challenge.status, NodeStatus::Unlocked);

    // 2. Submit solution
    let student_code = load_test_solution("correct_solution.rs");
    let result = docker.run_verification(&challenge.path, &student_code).await.unwrap();

    assert!(result.success);

    // 3. Complete challenge
    complete_mini_challenge(&db, user_id, &challenge.id, &result).await.unwrap();

    // 4. Verify progress updated
    let progress = db.get_node_progress(user_id, &challenge.id).unwrap();
    assert_eq!(progress.status, NodeStatus::Completed);

    // 5. Verify XP awarded
    let user = db.get_user(user_id).unwrap();
    assert!(user.total_xp > 0);

    // 6. Verify next node unlocked
    let next_challenge = db.get_challenge("week1_day1_challenge2").unwrap();
    assert_eq!(next_challenge.status, NodeStatus::Unlocked);
}
```

**End-to-End Checkpoint Flow:**

```rust
#[tokio::test]
async fn test_checkpoint_grading_flow() {
    let db = setup_test_db().await;
    let grader = LLMGrader::new(get_test_api_key());
    let cache = GradeCache::new(Path::new(":memory:")).unwrap();
    let user_id = "test-user";

    // 1. Load checkpoint
    let checkpoint = db.get_checkpoint("week1_checkpoint").unwrap();

    // 2. Load student artifacts
    let artifacts = CheckpointArtifacts {
        design: load_test_artifact("design.md"),
        readme: load_test_artifact("readme.md"),
        bench: load_test_artifact("bench.md"),
        runbook: load_test_artifact("runbook.md"),
        invariants: load_test_artifact("invariants.md"),
    };

    // 3. Grade all artifacts
    let grade = grade_checkpoint(&grader, &cache, &artifacts).await.unwrap();

    assert!(grade.total_score >= 70); // Should pass

    // 4. Complete checkpoint
    complete_checkpoint(&db, user_id, &checkpoint.id, &grade).await.unwrap();

    // 5. Verify completion
    let progress = db.get_node_progress(user_id, &checkpoint.id).unwrap();
    assert_eq!(progress.status, NodeStatus::Completed);
}
```

### Manual Testing Checklist

```markdown
## Docker Integration

- [ ] App detects Docker not installed → shows setup guide
- [ ] App detects Docker not running → shows helpful error
- [ ] Image builds successfully on first launch
- [ ] Challenge with correct solution passes all tests
- [ ] Challenge with incorrect solution fails tests
- [ ] Infinite loop triggers timeout (30s)
- [ ] Memory bomb triggers OOM error
- [ ] Compile error shows clear message with line number
- [ ] Panic shows panic message and location
- [ ] Container cleanup works (no orphans after 10 runs)
- [ ] Pre-warming speeds up second run
- [ ] Works on macOS
- [ ] Works on Linux
- [ ] Works on Windows (if applicable)

## LLM Grading

- [ ] API key setup flow works
- [ ] Invalid API key shows clear error
- [ ] Excellent artifact scores 90+
- [ ] Poor artifact scores <70
- [ ] Same artifact graded twice returns same score (cache hit)
- [ ] Cache hit shows "✓ Cached result" badge
- [ ] Different whitespace doesn't break cache
- [ ] Grading completes in <30s p95
- [ ] Rate limit triggers retry with backoff
- [ ] Timeout triggers retry (3x max)
- [ ] Feedback is specific and actionable
- [ ] Category scores add up to total score
- [ ] All 5 artifact types can be graded
- [ ] Daily limit prevents excessive API calls

## UI/UX

- [ ] Challenge editor accepts code input
- [ ] "Run Tests" button works
- [ ] Test results display clearly (passed/failed counts)
- [ ] stdout/stderr shown in output pane
- [ ] Grading shows progress spinner
- [ ] Grade results display nicely
- [ ] Per-artifact scores visible
- [ ] Category feedback expandable
- [ ] XP award animation plays on completion
```

---

## Configuration & Secrets Management

### Configuration Schema

```toml
# .config/gamified-rust/config.toml

[docker]
image_name = "gamified-rust-sandbox:latest"
memory_limit_mb = 256
cpu_limit = 1.0
timeout_secs = 30
pre_warm_pool_size = 2
cleanup_on_startup = true

[grading]
model = "gpt-4"
temperature = 0.3
max_tokens = 2000
timeout_secs = 30
daily_limit = 20
enable_cache = true
consistency_check = false  # Phase 6 feature

[api_keys]
# Stored separately in secure keyring, not in config file
```

### Secrets Management

**API Key Storage (secure):**

```rust
// Use OS keyring instead of plaintext config
use keyring::Entry;

pub fn get_openai_api_key() -> Result<String, KeyringError> {
    let entry = Entry::new("gamified-rust-bootcamp", "openai_api_key")?;
    entry.get_password()
}

pub fn set_openai_api_key(key: &str) -> Result<(), KeyringError> {
    let entry = Entry::new("gamified-rust-bootcamp", "openai_api_key")?;
    entry.set_password(key)
}
```

**Settings UI:**

```typescript
// src/pages/Settings.tsx

export function Settings() {
  const [apiKey, setApiKey] = useState('');
  const [apiKeyValid, setApiKeyValid] = useState<boolean | null>(null);

  const testApiKey = async () => {
    try {
      await invoke('test_openai_api_key', { apiKey });
      setApiKeyValid(true);
    } catch (error) {
      setApiKeyValid(false);
    }
  };

  const saveApiKey = async () => {
    try {
      await invoke('save_openai_api_key', { apiKey });
      alert('API key saved!');
    } catch (error) {
      alert('Failed to save API key');
    }
  };

  return (
    <div>
      <h2>OpenAI API Key</h2>
      <p>Required for checkpoint grading</p>

      <input
        type="password"
        value={apiKey}
        onChange={(e) => setApiKey(e.target.value)}
        placeholder="sk-..."
      />

      <button onClick={testApiKey}>Test Key</button>
      <button onClick={saveApiKey}>Save Key</button>

      {apiKeyValid === true && <span className="text-green-600">✓ Valid</span>}
      {apiKeyValid === false && <span className="text-red-600">✗ Invalid</span>}
    </div>
  );
}
```

### Environment Variables

```bash
# Development only
OPENAI_API_KEY=sk-...
DOCKER_HOST=unix:///var/run/docker.sock  # Override if needed
RUST_LOG=debug  # For debugging
```

**Load from .env (dev only):**

```rust
// src-tauri/src/main.rs

#[cfg(debug_assertions)]
{
    dotenv::dotenv().ok();
}
```

---

## Cost Analysis

### Per-Student Costs

**LLM Grading:**

| Artifact | Avg Tokens (Input) | Avg Tokens (Output) | Cost per Grade | Attempts | Total Cost |
|----------|-------------------|---------------------|----------------|----------|------------|
| DESIGN.md | 2,000 | 800 | $0.08 | 2 avg | $0.16 |
| README.md | 1,500 | 600 | $0.06 | 1.5 avg | $0.09 |
| BENCH.md | 1,200 | 500 | $0.05 | 1.2 avg | $0.06 |
| RUNBOOK.md | 1,800 | 700 | $0.07 | 1.3 avg | $0.09 |
| INVARIANTS.md | 1,000 | 400 | $0.04 | 1.5 avg | $0.06 |

**Per Checkpoint:** ~$0.46 (without caching)
**Per Student (14 checkpoints):** ~$6.44
**With 70% cache hit rate:** ~$1.93 per student

**Docker Costs:**

- Infrastructure: $0 (runs locally)
- Time: ~5s per challenge verification (negligible compute cost)

**Total Operating Cost per Student:** ~$2-6

### Monthly Budget Estimate

**For 100 students:**
- Low estimate (high cache hit): $200/month
- High estimate (low cache hit): $650/month

**For 1000 students:**
- Low estimate: $2,000/month
- High estimate: $6,500/month

**Cost Reduction Strategies:**

1. **Caching** (70% hit rate saves ~$4/student)
2. **GPT-3.5-Turbo for retries** ($0.002/1K tokens = 15x cheaper)
3. **Batch grading** (parallel API calls don't save money, but reduce latency)
4. **Rate limiting** (prevent accidental runaway costs)
5. **Cached rubric embedding** (future: use embeddings to detect similar artifacts)

### Cost Monitoring

```rust
// Track API costs in real-time
pub async fn track_grading_cost(
    db: &Database,
    artifact_type: &str,
    input_tokens: u32,
    output_tokens: u32,
) -> Result<(), DbError> {
    let cost = calculate_cost(input_tokens, output_tokens);

    db.execute(
        "INSERT INTO api_costs (timestamp, artifact_type, input_tokens, output_tokens, cost_usd)
         VALUES (?, ?, ?, ?, ?)",
        params![
            chrono::Utc::now().to_rfc3339(),
            artifact_type,
            input_tokens,
            output_tokens,
            cost,
        ],
    )?;

    Ok(())
}

fn calculate_cost(input_tokens: u32, output_tokens: u32) -> f64 {
    // GPT-4 pricing (as of 2024)
    const INPUT_COST_PER_1K: f64 = 0.03;
    const OUTPUT_COST_PER_1K: f64 = 0.06;

    let input_cost = (input_tokens as f64 / 1000.0) * INPUT_COST_PER_1K;
    let output_cost = (output_tokens as f64 / 1000.0) * OUTPUT_COST_PER_1K;

    input_cost + output_cost
}
```

**Daily Cost Report:**

```sql
-- Show daily API costs
SELECT
    DATE(timestamp) as date,
    COUNT(*) as api_calls,
    SUM(input_tokens) as total_input_tokens,
    SUM(output_tokens) as total_output_tokens,
    SUM(cost_usd) as total_cost_usd
FROM api_costs
WHERE timestamp >= datetime('now', '-30 days')
GROUP BY DATE(timestamp)
ORDER BY date DESC;
```

---

## Timeline & Estimates

### Milestone 3.1: Docker Integration (3-4 days)

| Task | Duration | Dependencies | Risk |
|------|----------|--------------|------|
| Set up bollard + Dockerfile | 2 hours | - | Low |
| Implement DockerRunner | 4 hours | Dockerfile | Medium |
| Build container pool | 3 hours | DockerRunner | Medium |
| Implement output parser | 2 hours | - | Low |
| Add Docker detection | 1 hour | - | Low |
| Build challenge editor UI | 4 hours | - | Low |
| Wire up Tauri commands | 2 hours | DockerRunner | Low |
| Test all error cases | 4 hours | All above | Medium |
| Write unit tests | 3 hours | All above | Low |
| **Total** | **25 hours (3-4 days)** | | |

### Milestone 3.2: LLM Grading Integration (3-4 days)

| Task | Duration | Dependencies | Risk |
|------|----------|--------------|------|
| Set up async-openai | 1 hour | - | Low |
| Implement cache layer | 3 hours | Phase 1 DB | Low |
| Port grader from prototype | 2 hours | - | Low |
| Write 3 new rubrics (BENCH, RUNBOOK, INVARIANTS) | 4 hours | - | Low |
| Build checkpoint submission UI | 4 hours | - | Low |
| Build grade results UI | 3 hours | - | Low |
| Implement retry logic | 2 hours | async-openai | Medium |
| Add API key management | 2 hours | keyring | Low |
| Wire up Tauri commands | 2 hours | Grader | Low |
| Test with real API | 3 hours | API key | High |
| Write unit tests | 2 hours | All above | Low |
| **Total** | **28 hours (3.5-4 days)** | | |

### Total Phase 3 Timeline

**Best Case:** 6 days (working efficiently, no blockers)
**Expected Case:** 7-8 days (normal development pace)
**Worst Case:** 10 days (Docker issues, API rate limits, debugging)

**Parallel Work Opportunities:**
- UI work (challenge editor, grade results) can be done in parallel with backend
- Rubric writing can be done before grader implementation
- Testing can start as soon as individual components are done

---

## Acceptance Criteria

### Phase 3 Complete When:

**Milestone 3.1:**
- [ ] Docker detection works + shows setup guide if missing
- [ ] Challenge verification runs successfully in container
- [ ] All edge cases handled (timeout, OOM, compile error, panic)
- [ ] Container pool pre-warming works
- [ ] Orphan cleanup prevents resource leaks
- [ ] Challenge editor UI functional
- [ ] XP awarded on successful completion
- [ ] Next challenge unlocks after completion

**Milestone 3.2:**
- [ ] OpenAI API integration works
- [ ] All 5 rubrics created and validated
- [ ] Grade caching prevents duplicate API calls
- [ ] Checkpoint submission UI functional
- [ ] Grade results display clearly
- [ ] Retry logic handles rate limits and timeouts
- [ ] API key management secure (keyring)
- [ ] Daily limit prevents cost runaway
- [ ] XP awarded on passing checkpoint (≥70%)
- [ ] Next week unlocks after checkpoint completion

**Evidence:**
- Video demo showing:
  1. Mini-challenge submission → verification → XP gain
  2. Checkpoint submission → grading → results display
  3. Cache hit on re-submission
  4. Error handling (e.g., timeout, API error)

---

## Risks & Mitigation Summary

| Risk | Mitigation | Owner | Status |
|------|------------|-------|--------|
| Docker not installed | Detection + setup guide + skip mode | Backend | ✅ Planned |
| Container escape | Non-root, network isolation, resource limits | Backend | ✅ Planned |
| Resource exhaustion | Hard limits + cleanup + monitoring | Backend | ✅ Planned |
| API cost runaway | Caching + daily limits + monitoring | Backend | ✅ Planned |
| Inconsistent grading | Low temperature + caching + structured prompts | Backend | ✅ Planned |
| API outage | Retry logic + queue for later + show cached | Backend | ✅ Planned |
| Temp dir leaks | Best-effort cleanup + periodic GC | Backend | ⚠️ Monitor |

---

## Next Steps After Phase 3

**Immediate (Phase 4):**
- Badge system (detect challenge/checkpoint milestones)
- Mastery decay (apply to challenges, not just quizzes)
- Review queue (add challenges to spaced repetition)

**Future Enhancements (Phase 6):**
- Upgrade to Monaco Editor
- Add code completion (local Rust analyzer)
- Implement consistency checking for LLM grades
- Add few-shot examples to prompts
- Build cost dashboard
- Add GPT-3.5-Turbo fallback for non-critical grading

---

## Appendix: Code Structure

```
/crates/
  runner/
    src/
      lib.rs           # Public API
      docker.rs        # Docker client wrapper (bollard)
      pool.rs          # Container pool management
      parser.rs        # Cargo JSON output parser
      errors.rs        # Error types
    tests/
      integration.rs   # E2E tests
      fixtures/        # Sample challenges

  grader/
    src/
      lib.rs           # Public API
      llm.rs           # OpenAI client wrapper
      cache.rs         # Grade cache (SQLite)
      rubrics.rs       # Rubric loading
      prompts.rs       # Prompt templates
      errors.rs        # Error types
    tests/
      unit.rs
      fixtures/        # Sample artifacts
    rubrics/
      design.json
      readme.json
      bench.json
      runbook.json
      invariants.json

/src-tauri/
  src/
    commands/
      docker.rs        # Docker-related commands
      grading.rs       # Grading-related commands
    state.rs           # App state (includes DockerRunner, LLMGrader)
    docker_check.rs    # Docker detection logic

/src/
  components/
    ChallengeEditor.tsx
    TestResults.tsx
    GradeResults.tsx
    DockerSetup.tsx
  pages/
    Challenge.tsx
    Checkpoint.tsx
    Settings.tsx
```

---

**End of Phase 3 Implementation Plan**

This plan covers all aspects of Docker integration and LLM grading with detailed designs, workflows, error handling, cost analysis, and testing strategies. The focus is on safety, reliability, and cost-effectiveness while maintaining a smooth user experience.
