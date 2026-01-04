# Release Checklist

This document outlines the steps to create a new release of RustCamp.

## Pre-Release Checks

### Code Quality
- [ ] All tests pass: `cargo test --workspace`
- [ ] Frontend builds without errors: `cd apps/desktop && npm run build`
- [ ] No compiler warnings: `cargo build --release 2>&1 | grep -i warning`
- [ ] TypeScript checks pass: `npm run typecheck`
- [ ] Lint checks pass: `npm run lint`

### Functionality
- [ ] Application starts without errors
- [ ] Onboarding flow works for new users
- [ ] Content loads correctly (lectures, quizzes, challenges)
- [ ] Dark mode toggle works
- [ ] Keyboard shortcuts function
- [ ] Backup and restore works
- [ ] Settings persist across restarts

### Documentation
- [ ] CHANGELOG.md updated with release notes
- [ ] README.md reflects current features
- [ ] Version numbers consistent across all files

## Release Process

### 1. Update Version
```bash
# Bump version in all files
./scripts/bump-version.sh X.Y.Z

# Verify changes
git diff
```

Files updated:
- `apps/desktop/package.json`
- `apps/desktop/src-tauri/tauri.conf.json`
- `apps/desktop/src-tauri/Cargo.toml`
- `Cargo.toml` (workspace version)

### 2. Update CHANGELOG
Add release notes to `CHANGELOG.md` under a new version heading:
```markdown
## [X.Y.Z] - YYYY-MM-DD

### Added
- New feature descriptions

### Changed
- Modified behavior descriptions

### Fixed
- Bug fix descriptions
```

### 3. Commit and Tag
```bash
# Stage all changes
git add -A

# Commit with conventional format
git commit -m "chore: release vX.Y.Z

- Update version to X.Y.Z
- Add release notes to CHANGELOG

Co-Authored-By: Warp <agent@warp.dev>"

# Create annotated tag
git tag -a vX.Y.Z -m "Release vX.Y.Z"
```

### 4. Push to Trigger Build
```bash
# Push commit and tag
git push origin main --tags
```

This triggers the GitHub Actions workflow which:
1. Builds Linux packages (.deb, .AppImage)
2. Generates update manifest (latest.json)
3. Creates GitHub Release with artifacts

### 5. Verify Release
- [ ] GitHub Actions workflow completed successfully
- [ ] Release page shows correct version
- [ ] All artifacts are attached:
  - `rustcamp_X.Y.Z_amd64.deb`
  - `rustcamp_X.Y.Z_amd64.AppImage`
  - `latest.json`
- [ ] Release notes are accurate
- [ ] Download and test packages locally

## Local Build (Optional)

To build release packages locally:
```bash
./scripts/build-release.sh
```

Artifacts will be in `apps/desktop/src-tauri/target/release/bundle/`

## Rollback Process

If a release has critical issues:

1. Delete the problematic release from GitHub
2. Delete the tag: `git tag -d vX.Y.Z && git push origin :refs/tags/vX.Y.Z`
3. Revert the version bump commit if needed
4. Fix the issues
5. Create a new patch release (X.Y.Z+1)

## Version Numbering

This project uses [Semantic Versioning](https://semver.org/):

- **MAJOR** (X): Incompatible changes (content format, database schema)
- **MINOR** (Y): New features, backwards compatible
- **PATCH** (Z): Bug fixes, backwards compatible

Examples:
- `0.1.0` → `0.2.0`: New feature (e.g., skill tree visualization)
- `0.1.0` → `0.1.1`: Bug fix
- `0.9.0` → `1.0.0`: First stable release
