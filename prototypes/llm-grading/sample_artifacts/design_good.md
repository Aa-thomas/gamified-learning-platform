# DESIGN.md - Task Tracker CLI

## Overview

A command-line task management application written in Rust that allows users to create, organize, and track their daily tasks with a focus on simplicity and performance.

## Architecture

### High-Level Components

```
┌─────────────┐
│   CLI Layer │  (clap for arg parsing)
└──────┬──────┘
       │
┌──────▼──────┐
│  Core Logic │  (business rules, validation)
└──────┬──────┘
       │
┌──────▼──────┐
│   Storage   │  (JSON file persistence)
└─────────────┘
```

### Component Details

#### 1. CLI Layer (`src/cli.rs`)
- **Responsibility**: Parse command-line arguments and delegate to core logic
- **Dependencies**: `clap` crate for argument parsing
- **Interface**:
  ```rust
  pub enum Command {
      Add { title: String, priority: Priority },
      List { filter: Option<Status> },
      Complete { id: u32 },
      Delete { id: u32 },
  }
  ```

#### 2. Core Logic (`src/task.rs`, `src/manager.rs`)
- **Responsibility**: Business logic, task validation, state management
- **Key Types**:
  ```rust
  pub struct Task {
      id: u32,
      title: String,
      priority: Priority,
      status: Status,
      created_at: DateTime<Utc>,
      completed_at: Option<DateTime<Utc>>,
  }

  pub struct TaskManager {
      tasks: Vec<Task>,
      next_id: u32,
  }
  ```
- **Invariants**:
  - Task IDs are unique and monotonically increasing
  - Completed tasks must have a `completed_at` timestamp
  - Task titles must be 1-200 characters

#### 3. Storage Layer (`src/storage.rs`)
- **Responsibility**: Persist tasks to disk, handle serialization/deserialization
- **Format**: JSON file at `~/.task-tracker/tasks.json`
- **Error Handling**:
  - File not found → create new empty task list
  - Parse errors → return descriptive error, don't corrupt data
  - Write errors → atomic write (temp file + rename)

## Data Flow

### Adding a Task
1. User runs: `task-tracker add "Fix bug" --priority high`
2. CLI layer parses arguments into `Command::Add`
3. TaskManager validates input (title length, priority valid)
4. TaskManager creates Task with unique ID
5. Storage layer persists updated task list to JSON
6. CLI displays confirmation message

### Listing Tasks
1. User runs: `task-tracker list --status pending`
2. CLI layer parses into `Command::List`
3. TaskManager filters tasks by status
4. CLI formats and displays tasks in a table

## Error Handling

### Error Types
```rust
pub enum Error {
    InvalidInput(String),
    StorageError(std::io::Error),
    ParseError(serde_json::Error),
}
```

### Error Recovery
- **Invalid input**: Display helpful error message, exit with code 1
- **Storage errors**: Attempt retry once, then fail gracefully
- **Parse errors**: Never overwrite corrupted data, prompt user to backup

## Testing Strategy

### Unit Tests
- Task validation logic (title length, valid enums)
- TaskManager operations (add, complete, delete)
- Storage serialization/deserialization

### Integration Tests
- Full command flows (add → list → complete)
- Error scenarios (invalid input, missing files)
- Concurrent access (file locking)

### Test Data
```rust
fn sample_task() -> Task {
    Task {
        id: 1,
        title: "Test task".to_string(),
        priority: Priority::Medium,
        status: Status::Pending,
        created_at: Utc::now(),
        completed_at: None,
    }
}
```

## Performance Considerations

- **File I/O**: All tasks loaded into memory (acceptable for <10,000 tasks)
- **Serialization**: Use `serde_json` with buffered writer
- **Startup Time**: Target <100ms for typical operations
- **Memory**: Estimated ~100 bytes per task, <1MB for 10K tasks

## Future Enhancements

1. **Search**: Full-text search across task titles
2. **Tags**: Add tags to tasks for better organization
3. **Due Dates**: Add deadline tracking with notifications
4. **Sync**: Optional cloud sync via REST API

## Dependencies

```toml
[dependencies]
clap = "4.0"           # CLI argument parsing
serde = "1.0"          # Serialization framework
serde_json = "1.0"     # JSON support
chrono = "0.4"         # Date/time handling
```

## Security Considerations

- File permissions: `tasks.json` should be user-readable only (0600)
- Input validation: Sanitize task titles (no control characters)
- Path traversal: Always use canonical paths for file operations

## Trade-offs

### JSON vs SQLite
- **Chosen**: JSON file
- **Rationale**: Simpler, easier to debug, sufficient for single-user CLI
- **Trade-off**: Less efficient for large datasets, no concurrent access

### In-Memory vs Streaming
- **Chosen**: Load all tasks into memory
- **Rationale**: Better performance for typical use cases
- **Trade-off**: Not suitable for >100K tasks (acceptable limitation)
