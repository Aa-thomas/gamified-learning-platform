# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2024-01-XX

### Added
- **Core Platform**
  - Tauri 2.0 desktop application with React/TypeScript frontend
  - SQLite database for local-first data storage
  - Markdown-based lecture content system
  - JSON quiz and challenge definitions

- **Gamification System**
  - XP calculation with difficulty, streak, and accuracy multipliers
  - Level progression (100 × N^1.5 XP per level)
  - Mastery tracking with exponential moving average
  - Mastery decay system (3-day grace period, 30% floor)
  - Badge unlocking system
  - Streak mechanics with grace periods

- **Verification Systems**
  - Docker-based code runner with security isolation (256MB RAM, 30s timeout)
  - LLM-based artifact grading with structured rubrics
  - Quiz verification with retry mechanics

- **User Experience**
  - Dark mode support with system preference detection
  - Keyboard shortcuts (Ctrl+D toggle dark, Ctrl+O open nav, Esc close modal)
  - Onboarding flow for new users
  - Global error handling with user-friendly messages
  - Backup and restore functionality

- **Auto-Update System**
  - Automatic update checking via GitHub Releases
  - In-app update notifications
  - Download and install updates without manual intervention

### Security
- Non-root Docker container execution
- Network isolation for code challenges
- Resource limits on student code execution
- Secure OpenAI API key handling

---

## Version History

### Pre-release Development
- **Phase 0**: Risk validation with prototypes (Docker, LLM grading, gamification)
- **Phase 1**: Foundation (data schema, Tauri shell, content loader)
- **Phase 2**: Core game loop (lectures, quizzes, dashboard)
- **Phase 3**: Verification systems integration
- **Phase 4**: Gamification features
- **Phase 5**: Content integration
- **Phase 6**: Polish (onboarding, dark mode, shortcuts, error handling)
- **Phase 7**: Deployment (Linux packages, auto-update)

[Unreleased]: https://github.com/Aa-thomas/gamified-learning-platform/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Aa-thomas/gamified-learning-platform/releases/tag/v0.1.0
