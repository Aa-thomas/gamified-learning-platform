# Frequently Asked Questions

## General

### What is RustCamp?

RustCamp is a gamified desktop application for learning the Rust programming language. It combines interactive lectures, quizzes, coding challenges, and a progression system with XP and badges to make learning engaging and fun.

### Do I need programming experience?

Basic programming knowledge is recommended. RustCamp assumes familiarity with concepts like variables, functions, and control flow from any language. Complete beginners should consider starting with a general programming introduction first.

### Is RustCamp free?

Yes, RustCamp is open-source and free to use. The AI-powered checkpoint grading feature requires an OpenAI API key, which has its own costs.

### What platforms does RustCamp support?

RustCamp runs on:
- Windows 10+
- macOS 10.15+
- Linux (Ubuntu 20.04+, Fedora 34+, Arch, and most modern distributions)

## Learning Content

### How long does the curriculum take to complete?

The full bootcamp takes approximately:
- **Casual learner** (30 min/day): ~16 weeks
- **Regular learner** (1-2 hrs/day): ~8 weeks
- **Intensive learner** (4+ hrs/day): ~4 weeks

Progress varies based on prior experience and learning style.

### Can I skip content I already know?

Yes! Navigate to any unlocked content in the Skill Tree. However, some advanced topics require completing prerequisites. You can also take quizzes without reading lectures if you're confident in the material.

### Is the content up-to-date with current Rust?

Content is regularly updated for recent Rust editions. The curriculum targets Rust 1.75+ and follows current best practices from the official Rust documentation.

### Can I add my own curriculum?

Yes! RustCamp supports user-uploaded curricula. You can create custom courses or import curricula shared by others. See the Curriculum Manager in Settings for details.

## Technical

### Why do I need Docker?

Docker provides secure, isolated containers for running your Rust code. This:
- Prevents malicious code from affecting your system
- Ensures consistent compilation environment
- Allows safe testing of all code, including potentially harmful experiments

### Can I use RustCamp without Docker?

The app will run, but code challenges won't work. Lectures, quizzes, and progress tracking function normally without Docker.

### How much disk space does RustCamp use?

- **Base installation**: ~200 MB
- **Docker images**: ~1-2 GB (downloaded on first challenge)
- **User data**: Varies based on progress (typically <50 MB)

### Is my data stored locally or in the cloud?

All data is stored locally on your machine:
- Progress and XP in SQLite database
- Settings in JSON configuration file
- Curriculum content in markdown/JSON files

No data is sent to external servers except:
- OpenAI API calls for checkpoint grading (if enabled)
- Update checks to GitHub releases

## Gamification

### How does the XP system work?

You earn XP for:
- Completing lectures: 25 base XP
- Passing quizzes: 50 base XP
- Completing challenges: 100 base XP
- Finishing checkpoints: 200 base XP

XP is multiplied by:
- **Difficulty**: Easy (1×) to Very Hard (3×)
- **Streak**: Up to 1.5× for 31+ day streaks
- **Accuracy**: Up to 1.5× for perfect scores

### What are streaks?

Streaks track consecutive days of learning. Each day you complete at least one activity, your streak increases. Miss a day and it resets. Longer streaks give XP bonuses:
- Days 1-3: 1.0×
- Days 4-7: 1.1×
- Days 8-14: 1.2×
- Days 15-30: 1.3×
- Days 31+: 1.5×

You get a one-day grace period if you miss a single day.

### How does mastery decay work?

Mastery represents long-term retention. After a 3-day grace period without practicing a topic:
- Mastery decays exponentially (~5% per day)
- Minimum floor is 30% (never fully forgotten)
- Practice the topic to restore mastery

This encourages spaced repetition for long-term learning.

### What are badges?

Badges are achievements for accomplishments like:
- Completing modules
- Maintaining long streaks
- Reaching XP milestones
- Perfect quiz scores

View your badges in the Badges page. Some badges unlock based on cumulative progress, others for specific achievements.

### What are levels?

Levels reflect your total XP earned. Higher levels require exponentially more XP:
- Level 1: 100 XP
- Level 5: ~2,100 XP
- Level 10: ~10,000 XP
- Level 20: ~57,000 XP

Levels are for bragging rights and don't affect content access.

## Progress & Data

### How do I back up my progress?

Settings → Data Management → Export Data

This creates a JSON file with all your progress that can be imported later.

### Can I sync across devices?

Not automatically. You can:
1. Export your progress on one device
2. Import the file on another device

Cloud sync may be added in a future version.

### I lost my progress! What do I do?

1. Check if you have a backup file
2. Check your data directory for the database file
3. If using version control for the app, check for uncommitted changes

If progress is truly lost, you'll need to start fresh. We recommend regular exports.

### Can I reset specific topics?

Currently, you can only reset all progress. Selective reset may be added in a future version. Consider using the Review feature instead of resetting.

## Checkpoint Grading

### What is checkpoint grading?

Checkpoints are larger assessments (like designing a module or writing documentation). AI grading using GPT-4 evaluates your submissions against a rubric.

### Do I need an OpenAI API key?

Only for checkpoint grading. All other features (lectures, quizzes, coding challenges) work without it.

### How much does grading cost?

Approximately $0.05 per checkpoint grade. For a full bootcamp with ~70 checkpoints, expect ~$1.50 total if cache hit rate is 50%.

### How accurate is AI grading?

Testing shows ~80%+ agreement with human graders, with score variance of ±5 points. The system uses structured rubrics and consistent temperature settings for reliability.

## Keyboard Shortcuts

### What shortcuts are available?

| Shortcut | Action |
|----------|--------|
| `Ctrl+1` | Go to Dashboard |
| `Ctrl+2` | Go to Skill Tree |
| `Ctrl+3` | Go to Progress |
| `Ctrl+4` | Go to Review |
| `Ctrl+,` | Open Settings |
| `Ctrl+/` | Show all shortcuts |
| `Esc` | Close modal |

### Can I customize shortcuts?

Not currently. Custom keybindings may be added in a future version.

## Troubleshooting

### Where do I find help for specific issues?

See [TROUBLESHOOTING.md](TROUBLESHOOTING.md) for detailed solutions to common problems.

### How do I report a bug?

1. Check if it's already reported in GitHub Issues
2. If not, create a new issue with:
   - App version (Settings → About)
   - Operating system
   - Steps to reproduce
   - Error messages/screenshots

### How do I suggest a feature?

Open a GitHub Issue with the "enhancement" label describing:
- What you'd like added
- Why it would be useful
- How you envision it working
