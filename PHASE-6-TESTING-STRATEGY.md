# Phase 6: Testing Strategy

This document outlines the testing approach to verify Phase 6 implementation matches expected behavior.

---

## Testing Layers

### 1. Unit Tests (Rust backend)
- Test individual functions in isolation
- Mock external dependencies (Docker, OpenAI)
- Fast, run on every change

### 2. Integration Tests (Rust backend)
- Test command handlers end-to-end
- Use in-memory SQLite
- Verify data persistence

### 3. Component Tests (React frontend)
- Test individual components render correctly
- Test user interactions
- Mock Tauri invoke calls

### 4. E2E Manual Tests
- Full app flow verification
- Platform-specific testing
- Visual/UX verification

---

## Milestone 6.1: Error Handling Tests

### Unit Tests: `crates/core/tests/error_handling_tests.rs`

```rust
//! Tests for error handling and serialization

use glp_core::db::error::DbError;
use serde_json;

#[test]
fn test_db_error_serializes_to_json() {
    let error = DbError::NotFound("User not found".to_string());
    let json = serde_json::to_string(&error);
    assert!(json.is_ok());
}

#[test]
fn test_error_codes_are_distinct() {
    // Verify each error type produces unique code
    let errors = vec![
        DbError::NotFound("".to_string()),
        DbError::InvalidData("".to_string()),
        DbError::Migration("".to_string()),
    ];
    
    let codes: Vec<String> = errors.iter().map(|e| e.code()).collect();
    let unique: std::collections::HashSet<_> = codes.iter().collect();
    assert_eq!(codes.len(), unique.len());
}
```

### Integration Tests: `crates/core/tests/phase6_error_tests.rs`

```rust
//! Integration tests for error handling

use glp_core::db::connection::Database;
use glp_core::db::repos::UserRepository;

#[test]
fn test_get_nonexistent_user_returns_not_found() {
    let db = Database::new_in_memory().unwrap();
    let conn = db.connection();
    
    let result = UserRepository::get_by_id(conn, "nonexistent");
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn test_invalid_data_rejected() {
    let db = Database::new_in_memory().unwrap();
    let conn = db.connection();
    
    // Test with empty user ID
    let user = User::new("".to_string());
    let result = UserRepository::create(conn, &user);
    // Should either reject or handle gracefully
}
```

### Backend Command Tests: `apps/desktop/src-tauri/src/commands/system.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check_docker_status_returns_valid_response() {
        let status = check_docker_status_internal().await;
        // Should return Ok with status, not panic
        assert!(status.is_ok() || status.is_err());
    }

    #[test]
    fn test_command_error_serializes() {
        let error = CommandError {
            code: "DOCKER_NOT_RUNNING".to_string(),
            message: "Docker is not running".to_string(),
            details: Some("Start Docker Desktop".to_string()),
        };
        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("DOCKER_NOT_RUNNING"));
    }
}
```

### Frontend Tests: `apps/desktop/src/__tests__/errorHandler.test.ts`

```typescript
import { parseError, AppError } from '../utils/errorHandler';

describe('errorHandler', () => {
  it('parses Docker not running error', () => {
    const error = parseError('Docker is not installed or not running');
    expect(error.code).toBe('DOCKER_NOT_RUNNING');
    expect(error.userMessage).toContain('Docker');
    expect(error.recoveryAction).toBeDefined();
  });

  it('parses API timeout error', () => {
    const error = parseError('Request timeout after 30s');
    expect(error.code).toBe('LLM_API_TIMEOUT');
    expect(error.retryable).toBe(true);
  });

  it('handles unknown errors gracefully', () => {
    const error = parseError('Some random error');
    expect(error.code).toBe('UNKNOWN');
    expect(error.userMessage).toBeDefined();
  });
});
```

### Component Tests: `apps/desktop/src/__tests__/ErrorBanner.test.tsx`

```typescript
import { render, screen, fireEvent } from '@testing-library/react';
import { ErrorBanner } from '../components/common/ErrorBanner';

describe('ErrorBanner', () => {
  const mockError = {
    code: 'DOCKER_NOT_RUNNING',
    message: 'Docker error',
    userMessage: 'Docker is not running',
    recoveryAction: 'Start Docker Desktop',
    retryable: true,
  };

  it('renders error message', () => {
    render(<ErrorBanner error={mockError} onDismiss={() => {}} />);
    expect(screen.getByText('Docker is not running')).toBeInTheDocument();
  });

  it('shows recovery action', () => {
    render(<ErrorBanner error={mockError} onDismiss={() => {}} />);
    expect(screen.getByText('Start Docker Desktop')).toBeInTheDocument();
  });

  it('shows retry button for retryable errors', () => {
    render(<ErrorBanner error={mockError} onDismiss={() => {}} onRetry={() => {}} />);
    expect(screen.getByRole('button', { name: /retry/i })).toBeInTheDocument();
  });

  it('calls onDismiss when dismiss clicked', () => {
    const onDismiss = jest.fn();
    render(<ErrorBanner error={mockError} onDismiss={onDismiss} />);
    fireEvent.click(screen.getByRole('button', { name: /dismiss/i }));
    expect(onDismiss).toHaveBeenCalled();
  });
});
```

### Backup/Restore Tests: `crates/core/tests/backup_restore_tests.rs`

```rust
use tempfile::tempdir;
use glp_core::db::connection::Database;
use glp_core::db::repos::*;
use glp_core::models::*;

#[test]
fn test_export_import_preserves_user_data() {
    let db = Database::new_in_memory().unwrap();
    let conn = db.connection();
    
    // Create test data
    let user = User::new("test-user".to_string());
    UserRepository::create(conn, &user).unwrap();
    UserRepository::update_xp(conn, "test-user", 500).unwrap();
    
    // Export
    let temp = tempdir().unwrap();
    let export_path = temp.path().join("backup.json");
    export_user_data(conn, &export_path).unwrap();
    
    // Create fresh DB and import
    let db2 = Database::new_in_memory().unwrap();
    let conn2 = db2.connection();
    import_user_data(conn2, &export_path).unwrap();
    
    // Verify
    let imported_user = UserRepository::get_by_id(conn2, "test-user").unwrap().unwrap();
    assert_eq!(imported_user.total_xp, 500);
}

#[test]
fn test_export_import_preserves_progress() {
    // Similar test for node_progress, quiz_attempts, etc.
}

#[test]
fn test_reset_progress_clears_all_data() {
    let db = Database::new_in_memory().unwrap();
    let conn = db.connection();
    
    // Create test data
    let user = User::new("test-user".to_string());
    UserRepository::create(conn, &user).unwrap();
    
    let progress = NodeProgress::new("test-user".to_string(), "node1".to_string());
    ProgressRepository::create_or_update(conn, &progress).unwrap();
    
    // Reset
    reset_all_progress(conn).unwrap();
    
    // Verify cleared
    let user_after = UserRepository::get_by_id(conn, "test-user").unwrap();
    assert!(user_after.is_none());
    
    let progress_after = ProgressRepository::get(conn, "test-user", "node1").unwrap();
    assert!(progress_after.is_none());
}
```

### Manual Test Cases: Error Handling

| Test Case | Steps | Expected Result |
|-----------|-------|-----------------|
| Docker not installed | 1. Uninstall Docker 2. Launch app | Shows "Docker not installed" with install link |
| Docker not running | 1. Stop Docker service 2. Try challenge | Shows "Docker not running" with start instructions |
| API key invalid | 1. Enter invalid key 2. Submit artifact | Shows "Invalid API key" with setup link |
| API timeout | 1. Disconnect network during grading | Shows "Request timeout" with retry button |
| Database locked | 1. Open DB in another process 2. Try action | Shows "Database busy" with retry option |

---

## Milestone 6.2: Onboarding Tests

### Frontend Tests: `apps/desktop/src/__tests__/Welcome.test.tsx`

```typescript
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { Welcome } from '../pages/Welcome';

// Mock Tauri invoke
jest.mock('@tauri-apps/api/core', () => ({
  invoke: jest.fn(),
}));

describe('Welcome', () => {
  it('shows welcome message on first step', () => {
    render(<MemoryRouter><Welcome /></MemoryRouter>);
    expect(screen.getByText(/welcome/i)).toBeInTheDocument();
  });

  it('advances to system check on next', () => {
    render(<MemoryRouter><Welcome /></MemoryRouter>);
    fireEvent.click(screen.getByRole('button', { name: /next/i }));
    expect(screen.getByText(/system check/i)).toBeInTheDocument();
  });

  it('shows Docker setup when Docker not found', async () => {
    const { invoke } = require('@tauri-apps/api/core');
    invoke.mockResolvedValueOnce({ docker_installed: false });
    
    render(<MemoryRouter><Welcome /></MemoryRouter>);
    fireEvent.click(screen.getByRole('button', { name: /next/i }));
    
    await waitFor(() => {
      expect(screen.getByText(/install docker/i)).toBeInTheDocument();
    });
  });

  it('skips Docker setup when Docker available', async () => {
    const { invoke } = require('@tauri-apps/api/core');
    invoke.mockResolvedValueOnce({ docker_installed: true, docker_running: true });
    
    render(<MemoryRouter><Welcome /></MemoryRouter>);
    fireEvent.click(screen.getByRole('button', { name: /next/i }));
    
    await waitFor(() => {
      expect(screen.getByText(/api key/i)).toBeInTheDocument();
    });
  });
});
```

### Component Tests: `apps/desktop/src/__tests__/DockerSetup.test.tsx`

```typescript
import { render, screen, fireEvent } from '@testing-library/react';
import { DockerSetup } from '../components/onboarding/DockerSetup';

describe('DockerSetup', () => {
  it('shows install instructions for Linux', () => {
    render(<DockerSetup platform="linux" onComplete={() => {}} />);
    expect(screen.getByText(/pacman/i)).toBeInTheDocument();
  });

  it('shows install instructions for macOS', () => {
    render(<DockerSetup platform="darwin" onComplete={() => {}} />);
    expect(screen.getByText(/brew/i)).toBeInTheDocument();
  });

  it('shows download link for Windows', () => {
    render(<DockerSetup platform="win32" onComplete={() => {}} />);
    expect(screen.getByText(/docker desktop/i)).toBeInTheDocument();
  });

  it('calls onComplete when skip clicked', () => {
    const onComplete = jest.fn();
    render(<DockerSetup platform="linux" onComplete={onComplete} />);
    fireEvent.click(screen.getByRole('button', { name: /skip/i }));
    expect(onComplete).toHaveBeenCalledWith({ skipped: true });
  });
});
```

### Component Tests: `apps/desktop/src/__tests__/ApiKeySetup.test.tsx`

```typescript
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { ApiKeySetup } from '../components/onboarding/ApiKeySetup';

jest.mock('@tauri-apps/api/core', () => ({
  invoke: jest.fn(),
}));

describe('ApiKeySetup', () => {
  it('shows masked input field', () => {
    render(<ApiKeySetup onComplete={() => {}} />);
    const input = screen.getByPlaceholderText(/api key/i);
    expect(input).toHaveAttribute('type', 'password');
  });

  it('enables test button when key entered', () => {
    render(<ApiKeySetup onComplete={() => {}} />);
    const input = screen.getByPlaceholderText(/api key/i);
    fireEvent.change(input, { target: { value: 'sk-test123' } });
    expect(screen.getByRole('button', { name: /test/i })).not.toBeDisabled();
  });

  it('shows success on valid key', async () => {
    const { invoke } = require('@tauri-apps/api/core');
    invoke.mockResolvedValueOnce(true);
    
    render(<ApiKeySetup onComplete={() => {}} />);
    const input = screen.getByPlaceholderText(/api key/i);
    fireEvent.change(input, { target: { value: 'sk-valid' } });
    fireEvent.click(screen.getByRole('button', { name: /test/i }));
    
    await waitFor(() => {
      expect(screen.getByText(/connected/i)).toBeInTheDocument();
    });
  });

  it('shows error on invalid key', async () => {
    const { invoke } = require('@tauri-apps/api/core');
    invoke.mockRejectedValueOnce(new Error('Invalid API key'));
    
    render(<ApiKeySetup onComplete={() => {}} />);
    const input = screen.getByPlaceholderText(/api key/i);
    fireEvent.change(input, { target: { value: 'sk-invalid' } });
    fireEvent.click(screen.getByRole('button', { name: /test/i }));
    
    await waitFor(() => {
      expect(screen.getByText(/invalid/i)).toBeInTheDocument();
    });
  });
});
```

### Backend Tests: API Key Validation

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_valid_api_key_returns_true() {
        // This would need a test key or mock
        // For CI, mock the OpenAI client
    }

    #[test]
    fn test_save_api_key_persists() {
        let temp = tempdir().unwrap();
        let config_path = temp.path().join("config");
        
        save_api_key_internal(&config_path, "sk-test123").unwrap();
        
        let loaded = load_api_key_internal(&config_path).unwrap();
        assert_eq!(loaded, Some("sk-test123".to_string()));
    }

    #[test]
    fn test_api_key_not_stored_in_plain_text() {
        let temp = tempdir().unwrap();
        let config_path = temp.path().join("config");
        
        save_api_key_internal(&config_path, "sk-secret").unwrap();
        
        // Read raw file content
        let content = std::fs::read_to_string(&config_path).unwrap();
        assert!(!content.contains("sk-secret")); // Should be encrypted
    }
}
```

### Manual Test Cases: Onboarding

| Test Case | Steps | Expected Result |
|-----------|-------|-----------------|
| First launch | 1. Delete app data 2. Launch app | Welcome wizard appears |
| Docker present | 1. Install Docker 2. Start welcome | System check passes Docker |
| Docker missing | 1. Uninstall Docker 2. Start welcome | Shows Docker install instructions |
| API key valid | 1. Enter valid key 2. Click test | Shows "Connected" success |
| API key invalid | 1. Enter random text 2. Click test | Shows "Invalid" error |
| Skip Docker | 1. On Docker step 2. Click skip | Proceeds, shows warning about challenges |
| Skip API key | 1. On API step 2. Click skip | Proceeds, shows warning about grading |
| Complete wizard | 1. Complete all steps | Redirects to Home with tutorial prompt |

---

## Milestone 6.3: Performance & Polish Tests

### Dark Mode Tests: `apps/desktop/src/__tests__/ThemeToggle.test.tsx`

```typescript
import { render, screen, fireEvent } from '@testing-library/react';
import { ThemeToggle } from '../components/layout/ThemeToggle';
import { useThemeStore } from '../stores/themeStore';

// Mock localStorage
const localStorageMock = {
  getItem: jest.fn(),
  setItem: jest.fn(),
  clear: jest.fn(),
};
Object.defineProperty(window, 'localStorage', { value: localStorageMock });

describe('ThemeToggle', () => {
  beforeEach(() => {
    useThemeStore.setState({ theme: 'light' });
  });

  it('toggles from light to dark', () => {
    render(<ThemeToggle />);
    fireEvent.click(screen.getByRole('button'));
    expect(useThemeStore.getState().theme).toBe('dark');
  });

  it('persists theme to localStorage', () => {
    render(<ThemeToggle />);
    fireEvent.click(screen.getByRole('button'));
    expect(localStorageMock.setItem).toHaveBeenCalledWith('theme', 'dark');
  });

  it('applies dark class to document', () => {
    render(<ThemeToggle />);
    fireEvent.click(screen.getByRole('button'));
    expect(document.documentElement.classList.contains('dark')).toBe(true);
  });
});
```

### Keyboard Shortcuts Tests: `apps/desktop/src/__tests__/useKeyboardShortcuts.test.ts`

```typescript
import { renderHook } from '@testing-library/react';
import { useKeyboardShortcuts } from '../hooks/useKeyboardShortcuts';

const mockNavigate = jest.fn();
jest.mock('react-router-dom', () => ({
  useNavigate: () => mockNavigate,
}));

describe('useKeyboardShortcuts', () => {
  beforeEach(() => {
    mockNavigate.mockClear();
  });

  it('navigates to home on Ctrl+1', () => {
    renderHook(() => useKeyboardShortcuts());
    
    const event = new KeyboardEvent('keydown', { key: '1', ctrlKey: true });
    document.dispatchEvent(event);
    
    expect(mockNavigate).toHaveBeenCalledWith('/');
  });

  it('navigates to skill tree on Ctrl+2', () => {
    renderHook(() => useKeyboardShortcuts());
    
    const event = new KeyboardEvent('keydown', { key: '2', ctrlKey: true });
    document.dispatchEvent(event);
    
    expect(mockNavigate).toHaveBeenCalledWith('/skill-tree');
  });

  it('opens shortcuts modal on Ctrl+?', () => {
    const onOpenShortcuts = jest.fn();
    renderHook(() => useKeyboardShortcuts({ onOpenShortcuts }));
    
    const event = new KeyboardEvent('keydown', { key: '?', ctrlKey: true });
    document.dispatchEvent(event);
    
    expect(onOpenShortcuts).toHaveBeenCalled();
  });

  it('does not trigger when typing in input', () => {
    renderHook(() => useKeyboardShortcuts());
    
    const input = document.createElement('input');
    document.body.appendChild(input);
    input.focus();
    
    const event = new KeyboardEvent('keydown', { key: '1', ctrlKey: true });
    input.dispatchEvent(event);
    
    expect(mockNavigate).not.toHaveBeenCalled();
  });
});
```

### Skeleton Loader Tests: `apps/desktop/src/__tests__/Skeleton.test.tsx`

```typescript
import { render, screen } from '@testing-library/react';
import { Skeleton, CardSkeleton, TextSkeleton } from '../components/common/Skeleton';

describe('Skeleton', () => {
  it('renders with correct dimensions', () => {
    render(<Skeleton width={200} height={50} />);
    const skeleton = screen.getByTestId('skeleton');
    expect(skeleton).toHaveStyle({ width: '200px', height: '50px' });
  });

  it('has animation class', () => {
    render(<Skeleton />);
    const skeleton = screen.getByTestId('skeleton');
    expect(skeleton).toHaveClass('animate-pulse');
  });
});

describe('CardSkeleton', () => {
  it('renders card-shaped skeleton', () => {
    render(<CardSkeleton />);
    expect(screen.getByTestId('card-skeleton')).toBeInTheDocument();
  });
});
```

### Container Pre-warming Tests: `crates/runner/src/pool.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_prewarm_creates_containers() {
        let pool = ContainerPool::new(2);
        pool.prewarm().await.unwrap();
        
        assert_eq!(pool.available_count(), 2);
    }

    #[tokio::test]
    async fn test_get_returns_prewarmed_container() {
        let pool = ContainerPool::new(2);
        pool.prewarm().await.unwrap();
        
        let start = std::time::Instant::now();
        let container = pool.get().await.unwrap();
        let elapsed = start.elapsed();
        
        // Should be fast since container is pre-warmed
        assert!(elapsed.as_millis() < 100);
    }

    #[tokio::test]
    async fn test_return_recycles_container() {
        let pool = ContainerPool::new(1);
        pool.prewarm().await.unwrap();
        
        let container = pool.get().await.unwrap();
        let id = container.id.clone();
        pool.return_container(container).await;
        
        let container2 = pool.get().await.unwrap();
        assert_eq!(container2.id, id); // Same container reused
    }
}
```

### Manual Test Cases: Performance & Polish

| Test Case | Steps | Expected Result |
|-----------|-------|-----------------|
| Dark mode toggle | 1. Click theme toggle | UI switches to dark colors |
| Dark mode persists | 1. Set dark mode 2. Restart app | Dark mode still active |
| Keyboard nav | 1. Press Ctrl+2 | Navigates to Skill Tree |
| Shortcuts modal | 1. Press Ctrl+? | Shows shortcuts list |
| Loading skeleton | 1. Load slow page | Shows skeleton before content |
| Animation smooth | 1. Open badge unlock | Animation plays smoothly |
| Pre-warm containers | 1. Check logs on startup | Shows "Pre-warming containers" |
| Fast challenge run | 1. Submit challenge (after warmup) | Completes in <5s |

---

## Milestone 6.4: Documentation Tests

### README Validation

```bash
# Check all links in README are valid
markdown-link-check README.md

# Check markdown syntax
markdownlint README.md

# Verify screenshots exist
for img in $(grep -oP '!\[.*?\]\(\K[^)]+' README.md); do
  [ -f "$img" ] || echo "Missing: $img"
done
```

### Documentation Structure Check

```bash
# Required files exist
test -f README.md || echo "Missing README.md"
test -f docs/INSTALLATION.md || echo "Missing INSTALLATION.md"
test -f docs/TROUBLESHOOTING.md || echo "Missing TROUBLESHOOTING.md"
test -f docs/FAQ.md || echo "Missing FAQ.md"
test -f docs/CONTENT_SCHEMA.md || echo "Missing CONTENT_SCHEMA.md"

# Check for required sections
grep -q "## Installation" README.md || echo "Missing Installation section"
grep -q "## Features" README.md || echo "Missing Features section"
grep -q "## Quick Start" README.md || echo "Missing Quick Start section"
```

### Integration Tests: Full Flow

```rust
//! End-to-end integration tests

use tempfile::tempdir;

#[test]
fn test_full_user_journey() {
    let temp = tempdir().unwrap();
    let db_path = temp.path().join("test.db");
    
    // 1. Create database
    let db = Database::new(&db_path).unwrap();
    
    // 2. Create user
    let user = User::new("test-user".to_string());
    UserRepository::create(db.connection(), &user).unwrap();
    
    // 3. Import curriculum
    let content_path = create_test_curriculum(&temp);
    let curriculum = import_curriculum(&content_path, &temp.path()).unwrap();
    
    // 4. Complete a lecture
    let progress = NodeProgress::new("test-user".to_string(), "week1-day1-lecture".to_string());
    ProgressRepository::complete(db.connection(), &progress, 25).unwrap();
    
    // 5. Verify XP updated
    let user_after = UserRepository::get_by_id(db.connection(), "test-user").unwrap().unwrap();
    assert_eq!(user_after.total_xp, 25);
    
    // 6. Export data
    let export_path = temp.path().join("backup.json");
    export_user_data(db.connection(), &export_path).unwrap();
    
    // 7. Verify export contains data
    let export_content = std::fs::read_to_string(&export_path).unwrap();
    assert!(export_content.contains("test-user"));
    assert!(export_content.contains("week1-day1-lecture"));
}
```

### Manual Test Checklist: `docs/TESTING_CHECKLIST.md`

```markdown
## Pre-Release Testing Checklist

### Installation (Linux)
- [ ] Download .AppImage
- [ ] Make executable: `chmod +x *.AppImage`
- [ ] Run successfully
- [ ] Welcome wizard appears

### Onboarding Flow
- [ ] Welcome screen shows app intro
- [ ] System check detects Docker status correctly
- [ ] Docker instructions match platform (Arch: pacman)
- [ ] "Check Again" button works
- [ ] API key input masked
- [ ] "Test Connection" validates key
- [ ] Invalid key shows error
- [ ] Profile creation works
- [ ] Tutorial prompt appears

### Error Handling
- [ ] Stop Docker → shows clear error message
- [ ] Enter invalid API key → shows validation error
- [ ] Disconnect network during grading → timeout with retry
- [ ] All errors have recovery suggestions

### Core Features
- [ ] Import curriculum from folder
- [ ] View skill tree with correct node states
- [ ] Complete lecture → XP awarded
- [ ] Take quiz → score calculated correctly
- [ ] Badge unlocks when criteria met
- [ ] Review queue shows due items

### Settings & Data
- [ ] Dark mode toggles correctly
- [ ] Dark mode persists after restart
- [ ] Export creates valid JSON file
- [ ] Import restores all progress
- [ ] Reset clears all data (with confirmation)

### Performance
- [ ] App starts in < 3 seconds
- [ ] Docker challenge runs in < 5 seconds (after warmup)
- [ ] No UI freezing during operations
- [ ] Animations smooth (60fps)

### Keyboard Shortcuts
- [ ] Ctrl+1 → Home
- [ ] Ctrl+2 → Skill Tree
- [ ] Ctrl+3 → Progress
- [ ] Ctrl+4 → Badges
- [ ] Ctrl+, → Settings
- [ ] Ctrl+? → Shortcuts modal
- [ ] Esc → Close modal
- [ ] 1-4 in quiz → Select option
- [ ] Enter in quiz → Submit

### Documentation
- [ ] README has all sections
- [ ] Screenshots match current UI
- [ ] Installation guide is accurate
- [ ] Troubleshooting covers common issues
- [ ] FAQ answers likely questions
```

---

## Test Execution Plan

### Automated Tests (CI)

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  rust-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run Rust tests
        run: cargo test --all

  frontend-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install dependencies
        run: cd apps/desktop && npm ci
      - name: Run tests
        run: cd apps/desktop && npm test

  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Rust lint
        run: cargo clippy --all -- -D warnings
      - name: Frontend lint
        run: cd apps/desktop && npm run lint
```

### Pre-Commit Verification

```bash
#!/bin/bash
# scripts/pre-commit.sh

echo "Running pre-commit checks..."

# Rust tests
cargo test --lib || exit 1

# Frontend tests
cd apps/desktop && npm test || exit 1

# Lint
cargo clippy -- -D warnings || exit 1
cd apps/desktop && npm run lint || exit 1

echo "All checks passed!"
```

### Manual Test Schedule

| Phase | When | Duration | Focus |
|-------|------|----------|-------|
| Milestone 6.1 complete | Day 2 | 1 hour | Error handling scenarios |
| Milestone 6.2 complete | Day 4 | 1 hour | Onboarding flow |
| Milestone 6.3 complete | Day 6 | 1 hour | Dark mode, shortcuts, performance |
| Milestone 6.4 complete | Day 8 | 2 hours | Full checklist, documentation |

---

## Success Criteria

### Automated Tests
- [ ] All Rust unit tests pass
- [ ] All Rust integration tests pass
- [ ] All frontend component tests pass
- [ ] No clippy warnings
- [ ] No ESLint errors

### Manual Tests
- [ ] Complete testing checklist with all items checked
- [ ] No critical bugs found
- [ ] No UI/UX issues blocking usage

### Documentation
- [ ] README renders correctly on GitHub
- [ ] All links valid
- [ ] Screenshots up to date
- [ ] Installation guide tested on fresh system
