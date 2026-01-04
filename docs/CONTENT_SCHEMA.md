# Content Pack Schema

This document describes the expected format for content packs that can be imported into the Gamified Learning Platform.

## Overview

A content pack is a folder containing:
- A `manifest.json` file (required)
- Content files (lectures, quizzes, challenges) referenced by the manifest

## Folder Structure

```
my-curriculum/
├── manifest.json              # Required: Defines structure
├── week1/
│   ├── day1/
│   │   ├── lecture.md         # Lecture content (markdown)
│   │   ├── quiz.json          # Quiz questions
│   │   └── challenge.json     # Coding challenge
│   ├── day2/
│   │   └── ...
│   └── checkpoint.json        # Weekly checkpoint (optional)
├── week2/
│   └── ...
└── assets/                    # Optional: Images, etc.
    └── ...
```

## manifest.json

The manifest defines the curriculum structure and metadata.

### Required Fields

```json
{
  "version": "1.0",
  "title": "My Curriculum",
  "description": "A comprehensive course on...",
  "author": "Your Name",
  "created_at": "2026-01-01",
  "weeks": [],
  "skills": []
}
```

### Complete Example

```json
{
  "version": "1.0",
  "title": "Trading Systems Bootcamp",
  "description": "14-week course on building trading systems in Rust",
  "author": "Aaron",
  "created_at": "2026-01-01",
  "weeks": [
    {
      "id": "week1",
      "title": "Week 1: Order Flow Simulator",
      "description": "Build a deterministic event-driven simulator",
      "days": [
        {
          "id": "week1-day1",
          "title": "Day 1: Event Loop Foundations",
          "description": "Understanding event-driven architecture",
          "nodes": [
            {
              "id": "week1-day1-lecture",
              "type": "lecture",
              "title": "Event Loop Anatomy",
              "description": "Learn the basics of event-driven systems",
              "difficulty": "easy",
              "estimated_minutes": 20,
              "xp_reward": 25,
              "content_path": "week1/day1/lecture.md",
              "skills": ["event-loops", "rust-basics"],
              "prerequisites": []
            },
            {
              "id": "week1-day1-quiz",
              "type": "quiz",
              "title": "Event Loop Quiz",
              "description": "Test your understanding",
              "difficulty": "easy",
              "estimated_minutes": 10,
              "xp_reward": 50,
              "content_path": "week1/day1/quiz.json",
              "skills": ["event-loops"],
              "prerequisites": ["week1-day1-lecture"]
            },
            {
              "id": "week1-day1-challenge",
              "type": "mini-challenge",
              "title": "Build a Simple Event Loop",
              "description": "Implement a basic event loop",
              "difficulty": "medium",
              "estimated_minutes": 30,
              "xp_reward": 100,
              "content_path": "week1/day1/challenge.json",
              "skills": ["event-loops", "rust-basics"],
              "prerequisites": ["week1-day1-quiz"]
            }
          ]
        }
      ]
    }
  ],
  "checkpoints": [
    {
      "id": "week1-checkpoint",
      "title": "Week 1 Checkpoint",
      "description": "Submit your order flow simulator",
      "week": "week1",
      "day": "week1-day5",
      "difficulty": "hard",
      "estimated_hours": 4,
      "xp_reward": 200,
      "artifacts": ["README.md", "DESIGN.md"],
      "prerequisites": ["week1-day5-challenge"],
      "rubrics": {
        "README": "rubrics/readme.json",
        "DESIGN": "rubrics/design.json"
      }
    }
  ],
  "skills": [
    {
      "id": "event-loops",
      "name": "Event Loops",
      "description": "Understanding event-driven architecture"
    },
    {
      "id": "rust-basics",
      "name": "Rust Basics",
      "description": "Core Rust language features"
    }
  ]
}
```

## Node Types

### lecture

Lecture content is written in Markdown.

**File:** `*.md`

```markdown
# Event Loop Anatomy

## Learning Objectives
- Understand the event loop pattern
- Learn how to process events deterministically

## Introduction

An event loop is the core of any trading system...

## Key Concepts

### Events
Events represent discrete occurrences in your system...

### The Loop
```rust
loop {
    let event = queue.pop();
    process(event);
}
```

## Summary
Today we learned about...
```

### quiz

Quiz files define multiple-choice questions.

**File:** `quiz.json`

```json
{
  "id": "week1-day1-quiz",
  "title": "Event Loop Quiz",
  "questions": [
    {
      "id": "q1",
      "question": "What is the main purpose of an event loop?",
      "type": "multiple-choice",
      "options": [
        "To create infinite loops",
        "To process events in order",
        "To slow down execution",
        "To handle user input only"
      ],
      "correct_answer": 1,
      "explanation": "Event loops process events in a deterministic order, which is crucial for trading systems.",
      "skills": ["event-loops"]
    },
    {
      "id": "q2",
      "question": "Which of the following are valid event types? (Select all that apply)",
      "type": "multiple-select",
      "options": [
        "NewOrder",
        "Cancel",
        "Fill",
        "Sleep"
      ],
      "correct_answers": [0, 1, 2],
      "explanation": "NewOrder, Cancel, and Fill are typical trading events. Sleep is not an event type.",
      "skills": ["event-loops"]
    }
  ]
}
```

**Question Types:**
- `multiple-choice`: Single correct answer (use `correct_answer` index)
- `multiple-select`: Multiple correct answers (use `correct_answers` array)

### mini-challenge

Challenges are coding exercises verified by running tests in Docker.

**File:** `challenge.json`

```json
{
  "id": "week1-day1-challenge",
  "title": "Build a Simple Event Loop",
  "description": "Implement a basic event loop that processes events",
  "instructions": "## Your Task\n\nImplement the `EventLoop` struct with the following methods:\n\n1. `new()` - Create a new event loop\n2. `push(&mut self, event: Event)` - Add an event to the queue\n3. `run(&mut self)` - Process all events in order\n\n## Requirements\n- Events must be processed in FIFO order\n- No panics allowed\n- Return typed errors\n",
  "starter_code": "use std::collections::VecDeque;\n\n#[derive(Debug, Clone)]\npub enum Event {\n    NewOrder { id: u64, symbol: String },\n    Cancel { id: u64 },\n    Fill { id: u64, quantity: u32 },\n}\n\npub struct EventLoop {\n    // TODO: Add your fields here\n}\n\nimpl EventLoop {\n    pub fn new() -> Self {\n        todo!()\n    }\n\n    pub fn push(&mut self, event: Event) {\n        todo!()\n    }\n\n    pub fn run(&mut self) -> Vec<Event> {\n        todo!()\n    }\n}\n",
  "test_code": "#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn test_new() {\n        let loop_ = EventLoop::new();\n        assert!(loop_.run().is_empty());\n    }\n\n    #[test]\n    fn test_push_and_run() {\n        let mut loop_ = EventLoop::new();\n        loop_.push(Event::NewOrder { id: 1, symbol: \"BTC\".to_string() });\n        loop_.push(Event::Cancel { id: 1 });\n        \n        let events = loop_.run();\n        assert_eq!(events.len(), 2);\n    }\n\n    #[test]\n    fn test_fifo_order() {\n        let mut loop_ = EventLoop::new();\n        loop_.push(Event::NewOrder { id: 1, symbol: \"BTC\".to_string() });\n        loop_.push(Event::NewOrder { id: 2, symbol: \"ETH\".to_string() });\n        \n        let events = loop_.run();\n        match &events[0] {\n            Event::NewOrder { id, .. } => assert_eq!(*id, 1),\n            _ => panic!(\"Expected NewOrder\"),\n        }\n    }\n}\n",
  "solution": "pub struct EventLoop {\n    queue: VecDeque<Event>,\n}\n\nimpl EventLoop {\n    pub fn new() -> Self {\n        Self {\n            queue: VecDeque::new(),\n        }\n    }\n\n    pub fn push(&mut self, event: Event) {\n        self.queue.push_back(event);\n    }\n\n    pub fn run(&mut self) -> Vec<Event> {\n        let mut processed = Vec::new();\n        while let Some(event) = self.queue.pop_front() {\n            processed.push(event);\n        }\n        processed\n    }\n}\n",
  "hints": [
    "Use VecDeque for the event queue",
    "Remember to process events in FIFO order",
    "The run() method should drain the queue"
  ],
  "difficulty": "medium",
  "skills": ["event-loops", "rust-basics"]
}
```

## Difficulty Levels

The platform recognizes these difficulty levels with XP multipliers:

| Level | Multiplier | Use For |
|-------|------------|---------|
| `easy` | 1.0x | Introductory content |
| `medium` | 1.5x | Standard difficulty |
| `hard` | 2.0x | Challenging content |
| `very-hard` | 3.0x | Advanced/stretch content |

## XP Rewards

Recommended base XP values:

| Node Type | Base XP |
|-----------|---------|
| `lecture` | 25 |
| `quiz` | 50 |
| `mini-challenge` | 100 |
| `checkpoint` | 200 |

## Prerequisites

Prerequisites are specified as an array of node IDs. The platform enforces:
- A node is locked until all prerequisites are completed
- Prerequisites must reference valid node IDs in the same manifest

## Content Generation Pipeline

You can use the provided generator templates to create content:

1. **Syllabus** (`syllabus_updated.md`) - Define weekly project outlines
2. **Project Outline** (`project-outline-template.md`) - Create detailed specs
3. **Pre-Project Generator** (`preproject-quiz-challenge-generator.md`) - Generate daily content

### Workflow

1. Define your week's project in the syllabus
2. Create a project outline using the template
3. Use the pre-project generator to create 5 days of prep content
4. Organize output into the folder structure above
5. Create `manifest.json` referencing all content
6. Import into the platform

## Validation

The platform validates content packs before import:

### Errors (block import)
- Missing `manifest.json`
- Invalid JSON syntax
- Missing required manifest fields
- Missing content files referenced by nodes
- Duplicate node IDs
- Invalid prerequisite references

### Warnings (allow import)
- Non-standard node types
- Non-standard difficulty levels

## Best Practices

1. **Use descriptive IDs**: `week1-day1-lecture` not `l1`
2. **Keep content paths consistent**: `weekN/dayN/type.ext`
3. **Provide good explanations**: Quiz explanations help learning
4. **Include hints for challenges**: Progressive hints reduce frustration
5. **Balance difficulty**: Mix easy and hard content
6. **Test your content**: Verify quizzes have correct answers, challenges compile
7. **Use version numbers**: Update version when content changes significantly
