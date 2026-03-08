#!/usr/bin/env bash
set -euo pipefail

# Build ChessMentor for macOS App Store submission.
#
# Prerequisites:
#   1. Apple Developer account with App Store Connect access
#   2. Certificates installed in Keychain:
#      - "3rd Party Mac Developer Application: <Team>" (for app signing)
#      - "3rd Party Mac Developer Installer: <Team>" (for .pkg signing)
#   3. Provisioning profile installed in ~/Library/MobileDevice/Provisioning Profiles/
#   4. Stockfish sidecar downloaded: ./scripts/fetch-stockfish.sh
#
# Usage:
#   ./scripts/build-appstore.sh [--identity <signing-identity>] [--installer-identity <installer-identity>]
#
# Environment variables (alternative to flags):
#   APPLE_SIGNING_IDENTITY    - Code signing identity (e.g., "3rd Party Mac Developer Application: Your Team (TEAMID)")
#   APPLE_INSTALLER_IDENTITY  - Installer signing identity (e.g., "3rd Party Mac Developer Installer: Your Team (TEAMID)")

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --identity)
            APPLE_SIGNING_IDENTITY="$2"
            shift 2
            ;;
        --installer-identity)
            APPLE_INSTALLER_IDENTITY="$2"
            shift 2
            ;;
        --help|-h)
            head -20 "$0" | tail -15
            exit 0
            ;;
        *)
            echo "Unknown option: $1" >&2
            exit 1
            ;;
    esac
done

# Validate required variables
if [ -z "${APPLE_SIGNING_IDENTITY:-}" ]; then
    echo "Error: APPLE_SIGNING_IDENTITY not set." >&2
    echo "Use --identity flag or export APPLE_SIGNING_IDENTITY." >&2
    exit 1
fi

if [ -z "${APPLE_INSTALLER_IDENTITY:-}" ]; then
    echo "Error: APPLE_INSTALLER_IDENTITY not set." >&2
    echo "Use --installer-identity flag or export APPLE_INSTALLER_IDENTITY." >&2
    exit 1
fi

ENTITLEMENTS_APP="$PROJECT_ROOT/src-tauri/entitlements/ChessMentor.entitlements"
ENTITLEMENTS_SIDECAR="$PROJECT_ROOT/src-tauri/entitlements/Stockfish.entitlements"

# Detect architecture
ARCH="$(uname -m)"
case "$ARCH" in
    arm64)  TARGET="aarch64-apple-darwin" ;;
    x86_64) TARGET="x86_64-apple-darwin" ;;
    *)      echo "Unsupported architecture: $ARCH" >&2; exit 1 ;;
esac

echo "==> Building ChessMentor for App Store"
echo "    Target: $TARGET"
echo "    Signing: $APPLE_SIGNING_IDENTITY"
echo ""

# Step 1: Ensure Stockfish sidecar exists
if ! ls "$PROJECT_ROOT/src-tauri/binaries/stockfish-"* &>/dev/null; then
    echo "==> Fetching Stockfish sidecar..."
    "$SCRIPT_DIR/fetch-stockfish.sh"
fi

# Step 2: Install npm dependencies
echo "==> Installing npm dependencies..."
cd "$PROJECT_ROOT"
npm install

# Step 3: Build with Tauri
echo "==> Building Tauri app..."
npm run tauri build -- --target "$TARGET"

# Step 4: Find the built .app
APP_PATH=$(find "$PROJECT_ROOT/src-tauri/target/$TARGET/release/bundle/macos" -name "*.app" -maxdepth 1 | head -1)
if [ -z "$APP_PATH" ]; then
    echo "Error: Could not find built .app bundle" >&2
    exit 1
fi
echo "==> Found app: $APP_PATH"

# Step 5: Sign the Stockfish sidecar
SIDECAR=$(find "$APP_PATH" -name "stockfish-*" -type f | head -1)
if [ -n "$SIDECAR" ]; then
    echo "==> Signing sidecar: $(basename "$SIDECAR")"
    codesign --force --options runtime \
        --entitlements "$ENTITLEMENTS_SIDECAR" \
        --sign "$APPLE_SIGNING_IDENTITY" \
        "$SIDECAR"
fi

# Step 6: Sign the main app bundle
echo "==> Signing app bundle..."
codesign --force --deep --options runtime \
    --entitlements "$ENTITLEMENTS_APP" \
    --sign "$APPLE_SIGNING_IDENTITY" \
    "$APP_PATH"

# Step 7: Verify signature
echo "==> Verifying signature..."
codesign --verify --deep --strict --verbose=2 "$APP_PATH"

# Step 8: Build the .pkg for App Store upload
PKG_PATH="$PROJECT_ROOT/src-tauri/target/$TARGET/release/bundle/ChessMentor.pkg"
echo "==> Creating App Store package..."
productbuild \
    --sign "$APPLE_INSTALLER_IDENTITY" \
    --component "$APP_PATH" /Applications \
    "$PKG_PATH"

echo ""
echo "==> Build complete!"
echo "    App: $APP_PATH"
echo "    Package: $PKG_PATH"
echo ""
echo "Next steps:"
echo "  1. Validate: xcrun altool --validate-app -f '$PKG_PATH' -t macos --apiKey <KEY> --apiIssuer <ISSUER>"
echo "  2. Upload:   xcrun altool --upload-app -f '$PKG_PATH' -t macos --apiKey <KEY> --apiIssuer <ISSUER>"
echo "  Or use Transporter.app to upload the .pkg to App Store Connect."
