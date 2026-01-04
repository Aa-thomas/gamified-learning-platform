# Phase 7: Deployment Testing Strategy (Linux Only)

## Overview

This document defines the expected behaviors and test cases for Phase 7 implementation. Each test specifies inputs, expected outputs, and verification commands.

---

## 7.1 Build Configuration Tests

### Test 7.1.1: tauri.conf.json Production Settings

**Expected Behavior:**
- `productName` is "RustCamp"
- `identifier` is "dev.rustcamp.app"
- `bundle.targets` includes "deb" and "appimage"
- `bundle.category` is "Education"
- Linux-specific settings configured

**Verification:**
```bash
# Check config values
cat apps/desktop/src-tauri/tauri.conf.json | jq '.productName'
# Expected: "RustCamp"

cat apps/desktop/src-tauri/tauri.conf.json | jq '.identifier'
# Expected: "dev.rustcamp.app"

cat apps/desktop/src-tauri/tauri.conf.json | jq '.bundle.targets'
# Expected: ["deb", "appimage"] or "all"

cat apps/desktop/src-tauri/tauri.conf.json | jq '.bundle.category'
# Expected: "Education"
```

### Test 7.1.2: Application Icons Exist

**Expected Behavior:**
- Icon files exist in `apps/desktop/src-tauri/icons/`
- Required sizes: 32x32, 128x128, 128x128@2x (256x256), icon.png (512x512)
- Files are valid PNG images

**Verification:**
```bash
# Check icon files exist
ls -la apps/desktop/src-tauri/icons/
# Expected files:
#   32x32.png
#   128x128.png
#   128x128@2x.png
#   icon.png

# Verify PNG format and dimensions
file apps/desktop/src-tauri/icons/32x32.png
# Expected: PNG image data, 32 x 32

file apps/desktop/src-tauri/icons/128x128.png
# Expected: PNG image data, 128 x 128

file apps/desktop/src-tauri/icons/icon.png
# Expected: PNG image data, 512 x 512 (or larger)
```

### Test 7.1.3: Build Compiles Successfully

**Expected Behavior:**
- `npm run tauri build` completes without errors
- Build produces .deb and .AppImage artifacts
- Artifacts are in `src-tauri/target/release/bundle/`

**Verification:**
```bash
cd apps/desktop
npm run tauri build 2>&1 | tail -20
# Expected: "Finished" with no errors

# Check artifacts exist
ls -la src-tauri/target/release/bundle/deb/
# Expected: rustcamp_*.deb file

ls -la src-tauri/target/release/bundle/appimage/
# Expected: rustcamp_*.AppImage file
```

---

## 7.2 Auto-Update System Tests

### Test 7.2.1: Updater Plugin Configured

**Expected Behavior:**
- `tauri-plugin-updater` is in Cargo.toml dependencies
- Plugin is registered in lib.rs
- Updater config exists in tauri.conf.json with endpoint and pubkey

**Verification:**
```bash
# Check Cargo.toml
grep "tauri-plugin-updater" apps/desktop/src-tauri/Cargo.toml
# Expected: tauri-plugin-updater = "2"

# Check plugin registration
grep "plugin-updater" apps/desktop/src-tauri/src/lib.rs
# Expected: .plugin(tauri_plugin_updater::Builder::new().build())

# Check tauri.conf.json updater config
cat apps/desktop/src-tauri/tauri.conf.json | jq '.plugins.updater'
# Expected: Object with "endpoints" array and "pubkey" string
```

### Test 7.2.2: Update Commands Exist

**Expected Behavior:**
- `check_for_update` command exists and is registered
- `download_and_install_update` command exists and is registered
- Commands return proper types

**Verification:**
```bash
# Check update.rs exists
test -f apps/desktop/src-tauri/src/commands/update.rs && echo "EXISTS"
# Expected: EXISTS

# Check commands are defined
grep "check_for_update" apps/desktop/src-tauri/src/commands/update.rs
# Expected: pub async fn check_for_update

grep "download_and_install_update" apps/desktop/src-tauri/src/commands/update.rs
# Expected: pub async fn download_and_install_update

# Check commands are registered in lib.rs
grep "check_for_update" apps/desktop/src-tauri/src/lib.rs
# Expected: commands::update::check_for_update
```

### Test 7.2.3: Update Store Functions

**Expected Behavior:**
- `updateStore.ts` exports store with state and actions
- State includes: `updateAvailable`, `updateInfo`, `downloading`, `progress`
- Actions include: `checkForUpdate`, `downloadUpdate`

**Verification:**
```bash
# Check store exists
test -f apps/desktop/src/stores/updateStore.ts && echo "EXISTS"
# Expected: EXISTS

# Check state properties
grep -E "(updateAvailable|updateInfo|downloading|progress)" apps/desktop/src/stores/updateStore.ts
# Expected: All four properties found

# Check actions
grep -E "(checkForUpdate|downloadUpdate)" apps/desktop/src/stores/updateStore.ts
# Expected: Both actions found
```

### Test 7.2.4: UpdateChecker Component

**Expected Behavior:**
- Component renders update notification when update available
- Shows "Update Now" button
- Displays download progress during update

**Verification:**
```bash
# Check component exists
test -f apps/desktop/src/components/UpdateChecker.tsx && echo "EXISTS"
# Expected: EXISTS

# Check for key UI elements
grep "Update" apps/desktop/src/components/UpdateChecker.tsx
# Expected: References to update-related text/buttons

# TypeScript compiles without errors
cd apps/desktop && npm run build 2>&1 | grep -i error
# Expected: No error output
```

---

## 7.3 Build Pipeline Tests

### Test 7.3.1: GitHub Actions Workflow Valid

**Expected Behavior:**
- `.github/workflows/release.yml` exists
- Triggers on tag push `v*` and manual dispatch
- Contains Linux build job with correct steps
- Uploads artifacts to release

**Verification:**
```bash
# Check workflow exists
test -f .github/workflows/release.yml && echo "EXISTS"
# Expected: EXISTS

# Check triggers
grep -A5 "on:" .github/workflows/release.yml | grep -E "(push|workflow_dispatch|tags)"
# Expected: Both push (with tags) and workflow_dispatch

# Check Linux job
grep "ubuntu" .github/workflows/release.yml
# Expected: ubuntu-22.04 or ubuntu-latest

# Check artifact upload
grep -i "upload" .github/workflows/release.yml
# Expected: upload-artifact or upload-release-asset action
```

### Test 7.3.2: Build Script Works

**Expected Behavior:**
- `scripts/build-release.sh` exists and is executable
- Script runs tauri build
- Exits 0 on success

**Verification:**
```bash
# Check script exists
test -f scripts/build-release.sh && echo "EXISTS"
# Expected: EXISTS

# Check executable
test -x scripts/build-release.sh && echo "EXECUTABLE"
# Expected: EXECUTABLE

# Verify script content
head -5 scripts/build-release.sh
# Expected: #!/bin/bash and set -e
```

### Test 7.3.3: Version Bump Script Works

**Expected Behavior:**
- `scripts/bump-version.sh` exists
- Updates version in package.json, Cargo.toml, tauri.conf.json
- Can bump to specified version

**Verification:**
```bash
# Check script exists
test -f scripts/bump-version.sh && echo "EXISTS"
# Expected: EXISTS

# Check it references all version files
grep -E "(package.json|Cargo.toml|tauri.conf.json)" scripts/bump-version.sh
# Expected: All three files referenced
```

---

## 7.4 Release Preparation Tests

### Test 7.4.1: CHANGELOG.md Exists

**Expected Behavior:**
- `CHANGELOG.md` exists at repo root
- Contains version history format
- Includes [Unreleased] section

**Verification:**
```bash
# Check file exists
test -f CHANGELOG.md && echo "EXISTS"
# Expected: EXISTS

# Check format (Keep a Changelog style)
head -20 CHANGELOG.md
# Expected: # Changelog header, [Unreleased] section
```

### Test 7.4.2: Issue Templates Exist

**Expected Behavior:**
- Bug report template exists
- Feature request template exists
- Templates have required fields

**Verification:**
```bash
# Check templates exist
ls -la .github/ISSUE_TEMPLATE/
# Expected: bug_report.md, feature_request.md (or .yml)

# Check bug report has required fields
grep -E "(title|description|steps)" .github/ISSUE_TEMPLATE/bug_report.md
# Expected: Required fields present
```

### Test 7.4.3: Release Checklist Exists

**Expected Behavior:**
- `docs/RELEASE_CHECKLIST.md` exists
- Contains all release steps as checkboxes

**Verification:**
```bash
# Check file exists
test -f apps/desktop/docs/RELEASE_CHECKLIST.md && echo "EXISTS"
# Expected: EXISTS

# Check for checkbox items
grep -c "\- \[ \]" apps/desktop/docs/RELEASE_CHECKLIST.md
# Expected: >= 5 checkbox items
```

---

## 7.5 Integration Tests

### Test 7.5.1: Full Build and Package

**Expected Behavior:**
- Fresh build produces valid artifacts
- .deb file installable (on Debian-based systems)
- .AppImage runs without installation

**Verification (Arch Linux with debtap or Docker):**
```bash
# Build
cd apps/desktop
npm run tauri build

# Check .deb exists and has correct structure
ar -t src-tauri/target/release/bundle/deb/*.deb
# Expected: debian-binary, control.tar.*, data.tar.*

# Check AppImage exists and is executable
chmod +x src-tauri/target/release/bundle/appimage/*.AppImage
file src-tauri/target/release/bundle/appimage/*.AppImage
# Expected: ELF 64-bit LSB executable
```

### Test 7.5.2: Application Launches from Package

**Expected Behavior:**
- AppImage launches without errors
- Shows main window
- Can navigate to all pages

**Verification:**
```bash
# Run AppImage
./src-tauri/target/release/bundle/appimage/*.AppImage &
APP_PID=$!
sleep 5

# Check process is running
ps -p $APP_PID > /dev/null && echo "RUNNING"
# Expected: RUNNING

# Kill test app
kill $APP_PID
```

### Test 7.5.3: Rust Tests Still Pass

**Expected Behavior:**
- All existing tests continue to pass
- No regressions from Phase 7 changes

**Verification:**
```bash
cargo test --workspace 2>&1 | tail -10
# Expected: test result: ok. 227+ passed
```

### Test 7.5.4: Frontend Builds Without Errors

**Expected Behavior:**
- TypeScript compiles without errors
- No missing imports or type errors

**Verification:**
```bash
cd apps/desktop
npm run build 2>&1
# Expected: "built in" message, exit code 0
echo $?
# Expected: 0
```

---

## 7.6 Update System Integration Test

### Test 7.6.1: Update Check Works (Mock)

**Expected Behavior:**
- Update check command can be invoked
- Returns None when no update available
- Returns UpdateInfo when update exists

**Verification (requires running app):**
```rust
// In Rust tests or manual invocation
// When latest.json has same version: returns None
// When latest.json has newer version: returns Some(UpdateInfo)
```

### Test 7.6.2: Signing Key Generated

**Expected Behavior:**
- Update signing keypair can be generated
- Public key is a valid string
- Private key is stored securely

**Verification:**
```bash
# Generate test keypair (don't commit private key!)
npx tauri signer generate -w /tmp/test-key
# Expected: Outputs public key, creates private key file

# Check public key format
cat /tmp/test-key.pub
# Expected: Base64-encoded public key string
```

---

## Test Summary Checklist

| Test ID | Description | Pass Criteria |
|---------|-------------|---------------|
| 7.1.1 | tauri.conf.json settings | All config values correct |
| 7.1.2 | Icons exist | All required sizes present |
| 7.1.3 | Build compiles | No errors, artifacts created |
| 7.2.1 | Updater plugin | Dependency + registration found |
| 7.2.2 | Update commands | Functions defined + registered |
| 7.2.3 | Update store | State + actions exported |
| 7.2.4 | UpdateChecker UI | Component renders, TS compiles |
| 7.3.1 | GitHub Actions | Valid workflow with triggers |
| 7.3.2 | Build script | Executable, runs build |
| 7.3.3 | Version script | Updates all version files |
| 7.4.1 | CHANGELOG | File exists with format |
| 7.4.2 | Issue templates | Both templates exist |
| 7.4.3 | Release checklist | Checklist with items |
| 7.5.1 | Full build | Valid .deb and .AppImage |
| 7.5.2 | App launches | Process runs from package |
| 7.5.3 | Rust tests | 227+ tests pass |
| 7.5.4 | Frontend build | TS compiles, exit 0 |
| 7.6.1 | Update check | Command invocable |
| 7.6.2 | Signing key | Keypair generates |

---

## Automated Test Script

```bash
#!/bin/bash
# scripts/test-phase7.sh - Run all Phase 7 verification tests

set -e
PASS=0
FAIL=0

check() {
    if eval "$2" > /dev/null 2>&1; then
        echo "✓ $1"
        ((PASS++))
    else
        echo "✗ $1"
        ((FAIL++))
    fi
}

echo "=== Phase 7 Verification Tests ==="
echo ""

# 7.1 Build Configuration
echo "--- 7.1 Build Configuration ---"
check "7.1.1 productName is RustCamp" \
    "[[ \$(cat apps/desktop/src-tauri/tauri.conf.json | jq -r '.productName') == 'RustCamp' ]]"
check "7.1.2 Icons exist" \
    "ls apps/desktop/src-tauri/icons/icon.png"
check "7.1.3 Backend compiles" \
    "cd apps/desktop/src-tauri && cargo check"

# 7.2 Auto-Update
echo ""
echo "--- 7.2 Auto-Update System ---"
check "7.2.1 Updater plugin in Cargo.toml" \
    "grep 'tauri-plugin-updater' apps/desktop/src-tauri/Cargo.toml"
check "7.2.2 Update commands exist" \
    "test -f apps/desktop/src-tauri/src/commands/update.rs"
check "7.2.3 Update store exists" \
    "test -f apps/desktop/src/stores/updateStore.ts"
check "7.2.4 UpdateChecker component exists" \
    "test -f apps/desktop/src/components/UpdateChecker.tsx"

# 7.3 Build Pipeline
echo ""
echo "--- 7.3 Build Pipeline ---"
check "7.3.1 GitHub Actions workflow exists" \
    "test -f .github/workflows/release.yml"
check "7.3.2 Build script exists" \
    "test -f scripts/build-release.sh"
check "7.3.3 Version bump script exists" \
    "test -f scripts/bump-version.sh"

# 7.4 Release Preparation
echo ""
echo "--- 7.4 Release Preparation ---"
check "7.4.1 CHANGELOG exists" \
    "test -f CHANGELOG.md"
check "7.4.2 Issue templates exist" \
    "test -d .github/ISSUE_TEMPLATE"
check "7.4.3 Release checklist exists" \
    "test -f apps/desktop/docs/RELEASE_CHECKLIST.md"

# 7.5 Integration
echo ""
echo "--- 7.5 Integration Tests ---"
check "7.5.3 Rust tests pass" \
    "cargo test --workspace"
check "7.5.4 Frontend builds" \
    "cd apps/desktop && npm run build"

echo ""
echo "=== Results ==="
echo "Passed: $PASS"
echo "Failed: $FAIL"

if [ $FAIL -gt 0 ]; then
    exit 1
fi
```
