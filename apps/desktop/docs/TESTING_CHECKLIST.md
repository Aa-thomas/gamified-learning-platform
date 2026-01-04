# Testing Checklist

Manual testing checklist for RustCamp releases.

## Pre-Test Setup

- [ ] Fresh install (or clear user data)
- [ ] Docker installed and running
- [ ] OpenAI API key available (for checkpoint tests)

## Phase 6 Features

### 6.1 Error Handling

#### Error Display
- [ ] Network errors show user-friendly messages
- [ ] Docker errors include troubleshooting guidance
- [ ] ErrorBanner appears for errors
- [ ] ErrorBanner can be dismissed
- [ ] Retry button works when shown

#### System Status
- [ ] Settings shows Docker status
- [ ] Settings shows API key status
- [ ] Invalid operations show appropriate errors

### 6.2 Onboarding

#### Welcome Page
- [ ] Shows on first launch
- [ ] Step indicator shows progress (1/3, 2/3, 3/3)
- [ ] Back button works (except on step 1)
- [ ] Next/Continue button advances steps

#### Docker Setup Step
- [ ] Shows Docker status (running/not running)
- [ ] Check Again button refreshes status
- [ ] Can skip if Docker not available
- [ ] Installation links work

#### API Key Step
- [ ] Input accepts API key
- [ ] Shows masked preview when saved
- [ ] Skip option available
- [ ] Validates key format (starts with sk-)

#### Completion
- [ ] Shows summary of setup
- [ ] Get Started button navigates to dashboard
- [ ] Onboarding doesn't show on subsequent launches
- [ ] Can trigger onboarding again from Settings

### 6.3 UI Improvements

#### Dark Mode
- [ ] Light theme applies correctly
- [ ] Dark theme applies correctly
- [ ] System theme follows OS preference
- [ ] Theme persists after restart
- [ ] All pages render correctly in dark mode:
  - [ ] Dashboard
  - [ ] Skill Tree
  - [ ] Progress
  - [ ] Badges
  - [ ] Review
  - [ ] Settings
  - [ ] Curriculum Manager
  - [ ] Lectures
  - [ ] Quizzes

#### Theme Toggle
- [ ] Toggle appears in navigation
- [ ] Clicking cycles through themes
- [ ] Icon updates to reflect current theme

#### Keyboard Shortcuts
- [ ] Ctrl+1 navigates to Dashboard
- [ ] Ctrl+2 navigates to Skill Tree
- [ ] Ctrl+3 navigates to Progress
- [ ] Ctrl+4 navigates to Review
- [ ] Ctrl+, opens Settings
- [ ] Ctrl+/ opens shortcuts modal
- [ ] Esc closes shortcuts modal
- [ ] Shortcuts don't trigger when typing in inputs

#### Shortcuts Modal
- [ ] Lists all available shortcuts
- [ ] Grouped by category
- [ ] Closes on background click
- [ ] Closes on X button
- [ ] Closes on Esc key

#### Loading States
- [ ] Skeleton shows during data loading
- [ ] Buttons show loading spinner when processing
- [ ] No layout shift when content loads

#### Tooltips
- [ ] Tooltips appear on hover
- [ ] Tooltips disappear on mouse leave
- [ ] Tooltips position correctly (not cut off)

### 6.4 Data Management

#### Export
- [ ] Export button prompts for save location
- [ ] JSON file is created
- [ ] File contains progress data
- [ ] File contains badge data
- [ ] File contains settings

#### Import
- [ ] Import button prompts for file selection
- [ ] Valid file imports successfully
- [ ] Invalid file shows error
- [ ] Confirmation shown before overwriting

#### Reset
- [ ] Reset shows confirmation dialog
- [ ] Cancel doesn't reset
- [ ] Confirm resets all progress
- [ ] Badges are reset
- [ ] XP is reset
- [ ] Completed items are reset

## Core Features (Regression)

### Navigation
- [ ] All nav links work
- [ ] Active page is highlighted
- [ ] Mobile menu works (if applicable)

### Dashboard
- [ ] Shows current streak
- [ ] Shows XP/level
- [ ] Shows recommended next items
- [ ] Continue Learning works

### Skill Tree
- [ ] Tree renders correctly
- [ ] Nodes show correct status (locked/available/completed)
- [ ] Clicking node shows details
- [ ] Start button navigates to content

### Lectures
- [ ] Markdown renders correctly
- [ ] Code blocks have syntax highlighting
- [ ] Complete button marks as done
- [ ] XP is awarded

### Quizzes
- [ ] Questions display correctly
- [ ] Multiple choice selection works
- [ ] Code completion input works
- [ ] Submit shows results
- [ ] Score is calculated correctly
- [ ] XP is awarded based on accuracy

### Challenges (requires Docker)
- [ ] Challenge loads with template
- [ ] Code editor accepts input
- [ ] Run executes in container
- [ ] Test results display
- [ ] Timeout works (30s)
- [ ] Memory limit enforced

### Progress
- [ ] Stats are accurate
- [ ] Charts render
- [ ] Mastery levels shown
- [ ] Recent activity listed

### Review
- [ ] Due items shown
- [ ] Review session works
- [ ] Items reschedule correctly

### Badges
- [ ] Earned badges display
- [ ] Unearned badges show requirements
- [ ] Unlock notification appears
- [ ] Badge details show progress

## Performance

- [ ] App launches in <5 seconds
- [ ] Navigation is instant (<100ms)
- [ ] No visible jank/stuttering
- [ ] Memory usage stable (<500MB)

## Platform-Specific

### Windows
- [ ] Installer works
- [ ] App launches from Start menu
- [ ] File dialogs use native style

### macOS
- [ ] DMG opens correctly
- [ ] App runs after drag to Applications
- [ ] Gatekeeper warning can be bypassed
- [ ] Native menu bar works

### Linux
- [ ] .deb installs on Debian/Ubuntu
- [ ] .rpm installs on Fedora
- [ ] AppImage runs
- [ ] File dialogs work

## Accessibility

- [ ] Keyboard navigation works
- [ ] Focus indicators visible
- [ ] Screen reader landmarks present
- [ ] Color contrast sufficient

## Notes

Add any issues found during testing:

| Issue | Steps to Reproduce | Severity |
|-------|-------------------|----------|
|       |                   |          |
|       |                   |          |
|       |                   |          |
