#!/usr/bin/env bash
# Bundle macOS .app + zip + unsigned .dmg
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

VERSION="${VERSION:-$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)}"
TARGET="${TARGET:-}"
ARCH_LABEL="${ARCH_LABEL:-aarch64}"
DIST="${DIST:-dist}"
APP_NAME="Devtools"
BUNDLE="$DIST/${APP_NAME}.app"

if [[ -n "$TARGET" && -f "target/${TARGET}/release/devtools" ]]; then
  BIN="target/${TARGET}/release/devtools"
else
  BIN="target/release/devtools"
fi
test -x "$BIN"

rm -rf "$BUNDLE"
mkdir -p "$BUNDLE/Contents/MacOS" "$BUNDLE/Contents/Resources"
install -m755 "$BIN" "$BUNDLE/Contents/MacOS/devtools"
cat > "$BUNDLE/Contents/Info.plist" << PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleExecutable</key>
  <string>devtools</string>
  <key>CFBundleIdentifier</key>
  <string>dev.left56.devtools</string>
  <key>CFBundleName</key>
  <string>${APP_NAME}</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>${VERSION}</string>
  <key>CFBundleVersion</key>
  <string>${VERSION}</string>
  <key>LSMinimumSystemVersion</key>
  <string>13.0</string>
  <key>NSHighResolutionCapable</key>
  <true/>
</dict>
</plist>
PLIST

ZIP="devtools-${VERSION}-macos-${ARCH_LABEL}.zip"
DMG="devtools-${VERSION}-macos-${ARCH_LABEL}.dmg"
mkdir -p "$DIST"
(
  cd "$DIST"
  rm -f "$ZIP" "$DMG"
  ditto -c -k --sequesterRsrc --keepParent "${APP_NAME}.app" "$ZIP"
)
echo "Wrote $DIST/$ZIP"

# Unsigned DMG (Gatekeeper will warn until notarized).
STAGE="$DIST/dmg-stage"
rm -rf "$STAGE"
mkdir -p "$STAGE"
cp -R "$BUNDLE" "$STAGE/"
ln -sf /Applications "$STAGE/Applications"
hdiutil create -volname "$APP_NAME" -srcfolder "$STAGE" -ov -format UDZO "$DIST/$DMG"
rm -rf "$STAGE"
echo "Wrote $DIST/$DMG"
