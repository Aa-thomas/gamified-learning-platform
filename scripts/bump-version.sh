#!/bin/bash
# Bump version in all project files
set -e

if [ -z "$1" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 0.2.0"
    exit 1
fi

VERSION="$1"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "=== Bumping version to $VERSION ==="

# Update package.json
PACKAGE_JSON="$PROJECT_ROOT/apps/desktop/package.json"
if [ -f "$PACKAGE_JSON" ]; then
    echo "Updating $PACKAGE_JSON"
    sed -i "s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/" "$PACKAGE_JSON"
fi

# Update tauri.conf.json
TAURI_CONF="$PROJECT_ROOT/apps/desktop/src-tauri/tauri.conf.json"
if [ -f "$TAURI_CONF" ]; then
    echo "Updating $TAURI_CONF"
    sed -i "s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/" "$TAURI_CONF"
fi

# Update apps/desktop/src-tauri/Cargo.toml
DESKTOP_CARGO="$PROJECT_ROOT/apps/desktop/src-tauri/Cargo.toml"
if [ -f "$DESKTOP_CARGO" ]; then
    echo "Updating $DESKTOP_CARGO"
    sed -i "s/^version = \"[^\"]*\"/version = \"$VERSION\"/" "$DESKTOP_CARGO"
fi

# Update root Cargo.toml (workspace version)
ROOT_CARGO="$PROJECT_ROOT/Cargo.toml"
if [ -f "$ROOT_CARGO" ]; then
    # Only update the workspace.package.version line
    echo "Updating $ROOT_CARGO (workspace version)"
    sed -i "/\[workspace.package\]/,/^\[/ s/^version = \"[^\"]*\"/version = \"$VERSION\"/" "$ROOT_CARGO"
fi

echo ""
echo "=== Version updated to $VERSION ==="
echo ""
echo "Files modified:"
echo "  - apps/desktop/package.json"
echo "  - apps/desktop/src-tauri/tauri.conf.json"
echo "  - apps/desktop/src-tauri/Cargo.toml"
echo "  - Cargo.toml (workspace)"
echo ""
echo "Next steps:"
echo "  1. Update CHANGELOG.md with release notes"
echo "  2. Commit: git add -A && git commit -m 'chore: bump version to $VERSION'"
echo "  3. Tag: git tag v$VERSION"
echo "  4. Push: git push origin main --tags"
