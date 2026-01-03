# DESIGN.md - Task Tracker

## What it does

This is a task tracker CLI app. You can add tasks, list them, and mark them as done.

## Components

We have three main parts:
- CLI - handles commands
- Core - does the task stuff
- Storage - saves to a file

## How it works

The CLI uses clap to parse arguments. Then it calls the TaskManager which manages tasks. Tasks are saved in a JSON file.

## Data Structure

```rust
struct Task {
    id: u32,
    title: String,
    done: bool,
}
```

We also have a TaskManager that holds a Vec of tasks.

## Commands

- `add` - adds a task
- `list` - shows all tasks
- `done` - marks task as done
- `delete` - removes a task

## Storage

Tasks are stored in a file called `tasks.json` in the home directory. We use serde to serialize and deserialize.

## Error Handling

If something goes wrong we return an error. If the file doesn't exist we create it.

## Testing

We'll write some tests to make sure everything works.

## Dependencies

```toml
clap = "4.0"
serde = "1.0"
serde_json = "1.0"
```

## Future Ideas

Maybe add:
- priorities
- due dates
- tags
- colors

## Notes

This should be pretty straightforward to implement. The main challenge is handling the file I/O correctly.
