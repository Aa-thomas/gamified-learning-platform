# RustCamp - Gamified Rust Learning Platform

A desktop application for learning Rust through gamified challenges, quizzes, and interactive content.

## Features

- 📚 **Interactive Lectures**: Markdown-based lessons with syntax highlighting
- ✍️ **Quizzes**: Test your knowledge with multiple choice and code completion questions
- 🐳 **Code Challenges**: Run real Rust code in isolated Docker containers
- 🏆 **Gamification**: Earn XP, level up, maintain streaks, and unlock badges
- 🎯 **Skill Tree**: Visual progression through the curriculum
- 📊 **Progress Tracking**: Monitor your learning journey with detailed analytics
- 🔄 **Spaced Repetition**: Review system to reinforce learning
- 🌙 **Dark Mode**: Easy on the eyes for late-night coding sessions
- ⌨️ **Keyboard Shortcuts**: Navigate efficiently with hotkeys

## Prerequisites

- **Docker**: Required for running code challenges
  - [Docker Desktop](https://www.docker.com/products/docker-desktop) (Windows/macOS)
  - `docker` package (Linux)
- **OpenAI API Key** (optional): For AI-powered artifact grading
- **Rust**: 1.75+ (for development only)
- **Node.js**: 18+ (for development only)

## Installation

### From Release (Recommended)

Download the latest release for your platform from the [Releases](../../releases) page:

- **Windows**: `RustCamp_x.x.x_x64-setup.exe`
- **macOS**: `RustCamp_x.x.x_x64.dmg`
- **Linux**: `rustcamp_x.x.x_amd64.deb` or `rustcamp-x.x.x-1.x86_64.rpm`

### From Source

```bash
# Clone the repository
git clone https://github.com/your-org/gamified-learning-platform.git
cd gamified-learning-platform/apps/desktop

# Install dependencies
npm install

# Build and run
npm run tauri dev
```

## Quick Start

1. **Launch the app** and complete the onboarding wizard
2. **Set up Docker** (follow the prompts if Docker isn't detected)
3. **Add your OpenAI API key** (optional, for checkpoint grading)
4. **Start learning!** Navigate to a lecture and begin your Rust journey

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+1` | Go to Dashboard |
| `Ctrl+2` | Go to Skill Tree |
| `Ctrl+3` | Go to Progress |
| `Ctrl+4` | Go to Review |
| `Ctrl+,` | Open Settings |
| `Ctrl+/` | Show keyboard shortcuts |
| `Esc` | Close modal / Cancel |

## Development

```bash
# Start development server
npm run tauri dev

# Run frontend tests
npm test

# Run backend tests
cargo test

# Build for production
npm run tauri build

# Lint code
npm run lint
cargo clippy
```

## Architecture

```
apps/desktop/
├── src/                 # React frontend
│   ├── components/      # Reusable UI components
│   ├── pages/           # Route pages
│   ├── stores/          # Zustand state management
│   ├── hooks/           # Custom React hooks
│   ├── types/           # TypeScript type definitions
│   └── utils/           # Utility functions
├── src-tauri/           # Rust backend
│   ├── src/
│   │   ├── commands/    # Tauri command handlers
│   │   ├── lib.rs       # Main library entry
│   │   └── main.rs      # Application entry
│   └── Cargo.toml       # Rust dependencies
└── content/             # Curriculum content
    ├── lectures/        # Markdown lecture files
    ├── quizzes/         # JSON quiz definitions
    └── challenges/      # Code challenge templates
```

## Configuration

### Environment Variables

| Variable | Description | Required |
|----------|-------------|----------|
| `OPENAI_API_KEY` | OpenAI API key for artifact grading | No |
| `RUST_LOG` | Logging level (debug, info, warn, error) | No |

### Settings

Access settings via `Ctrl+,` or the gear icon:

- **Theme**: Light / Dark / System
- **API Key**: Configure OpenAI integration
- **Data Management**: Export, import, or reset progress

## Troubleshooting

See [TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md) for common issues and solutions.

## FAQ

See [FAQ.md](docs/FAQ.md) for frequently asked questions.

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feat/amazing-feature`)
3. Commit your changes using conventional commits
4. Push to the branch (`git push origin feat/amazing-feature`)
5. Open a Pull Request

## License

MIT License - see [LICENSE](../../LICENSE) for details.
