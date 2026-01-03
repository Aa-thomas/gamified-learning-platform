# Task Tracker

A fast, simple command-line task manager written in Rust.

![Build Status](https://img.shields.io/badge/build-passing-brightgreen)
![License](https://img.shields.io/badge/license-MIT-blue)

## Features

- ✅ Add and manage tasks from the command line
- 🎯 Set priorities (low, medium, high)
- 📋 Filter tasks by status
- 💾 Automatic persistence to JSON
- ⚡ Fast startup (<100ms)
- 🔒 Safe concurrent access

## Installation

### From Source

```bash
git clone https://github.com/username/task-tracker.git
cd task-tracker
cargo build --release
sudo cp target/release/task-tracker /usr/local/bin/
```

### Via Cargo

```bash
cargo install task-tracker
```

### Requirements

- Rust 1.70 or higher
- Linux, macOS, or Windows

## Quick Start

```bash
# Add a task
task-tracker add "Write documentation" --priority high

# List all tasks
task-tracker list

# List only pending tasks
task-tracker list --status pending

# Complete a task
task-tracker complete 1

# Delete a task
task-tracker delete 2
```

## Usage

### Adding Tasks

```bash
task-tracker add <TITLE> [OPTIONS]

Options:
  -p, --priority <PRIORITY>  Set priority [low, medium, high] [default: medium]

Examples:
  task-tracker add "Fix bug #123" --priority high
  task-tracker add "Review PR"
```

### Listing Tasks

```bash
task-tracker list [OPTIONS]

Options:
  -s, --status <STATUS>  Filter by status [pending, completed] [default: all]
  -p, --priority <PRIORITY>  Filter by priority [low, medium, high]

Examples:
  task-tracker list
  task-tracker list --status pending
  task-tracker list --priority high
```

### Completing Tasks

```bash
task-tracker complete <ID>

Examples:
  task-tracker complete 1
```

### Deleting Tasks

```bash
task-tracker delete <ID>

Examples:
  task-tracker delete 1
```

## Configuration

Tasks are stored in `~/.task-tracker/tasks.json`. This file is created automatically on first run.

### File Location

- Linux/macOS: `~/.task-tracker/tasks.json`
- Windows: `%USERPROFILE%\.task-tracker\tasks.json`

### Manual Backup

```bash
cp ~/.task-tracker/tasks.json ~/.task-tracker/tasks.backup.json
```

## Development

### Building

```bash
cargo build
```

### Running Tests

```bash
cargo test
```

### Running with Debug Output

```bash
RUST_LOG=debug cargo run -- add "Test task"
```

### Code Coverage

```bash
cargo tarpaulin --out Html
```

## Project Structure

```
task-tracker/
├── src/
│   ├── main.rs       # Entry point
│   ├── cli.rs        # CLI argument parsing
│   ├── task.rs       # Task data structure
│   ├── manager.rs    # Task management logic
│   └── storage.rs    # File persistence
├── tests/
│   ├── integration_tests.rs
│   └── fixtures/
├── Cargo.toml
└── README.md
```

## Architecture

See [DESIGN.md](DESIGN.md) for detailed architecture documentation.

## Contributing

Contributions are welcome! Please follow these steps:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Code Style

- Run `cargo fmt` before committing
- Ensure `cargo clippy` passes with no warnings
- Add tests for new functionality

## Troubleshooting

### Permission Denied

If you get a permission error when running the installed binary:

```bash
chmod +x /usr/local/bin/task-tracker
```

### Tasks Not Persisting

Check that the tasks file exists and is writable:

```bash
ls -la ~/.task-tracker/tasks.json
```

If the file is corrupted, restore from backup or delete it to start fresh.

### Build Errors

Ensure you have the latest Rust toolchain:

```bash
rustup update stable
```

## Performance

- Startup time: ~50ms (tested with 1000 tasks)
- Memory usage: ~2MB (typical workload)
- File operations: Atomic writes prevent data corruption

## Limitations

- Maximum ~10,000 tasks (all loaded into memory)
- Single-user only (no multi-user support)
- No cloud sync (local storage only)

## License

MIT License - see [LICENSE](LICENSE) for details

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history.

## Authors

- Your Name - [@yourhandle](https://github.com/yourhandle)

## Acknowledgments

- Built with [clap](https://github.com/clap-rs/clap)
- Inspired by [todo.txt](http://todotxt.org/)
