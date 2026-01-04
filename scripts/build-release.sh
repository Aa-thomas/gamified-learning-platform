#!/bin/bash
# Build release artifacts for Linux
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "=== RustCamp Release Build ==="
echo "Project root: $PROJECT_ROOT"

# Check for signing key
if [ -n "$TAURI_SIGNING_PRIVATE_KEY" ]; then
    echo "✓ Signing key found"
else
    echo "⚠ No signing key found (TAURI_SIGNING_PRIVATE_KEY)"
    echo "  Builds will not be signed for auto-update"
fi

# Navigate to desktop app
cd "$PROJECT_ROOT/apps/desktop"

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
    echo "Installing npm dependencies..."
    npm ci
fi

# Build the application
echo "Building Tauri application..."
npm run tauri build

# Show results
echo ""
echo "=== Build Complete ==="
echo "Artifacts:"
ls -la src-tauri/target/release/bundle/deb/ 2>/dev/null || echo "  No .deb files"
ls -la src-tauri/target/release/bundle/appimage/ 2>/dev/null || echo "  No AppImage files"

echo ""
echo "To test the AppImage:"
echo "  chmod +x src-tauri/target/release/bundle/appimage/*.AppImage"
echo "  ./src-tauri/target/release/bundle/appimage/*.AppImage"
