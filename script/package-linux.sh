#!/usr/bin/env bash
# Build Linux release archives: tar.gz + simple .deb (no AppImage yet).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

VERSION="${VERSION:-$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)}"
TARGET="${TARGET:-x86_64-unknown-linux-gnu}"
DIST="${DIST:-dist}"
NAME="devtools-${VERSION}-linux-x86_64"
ICON_SRC="$ROOT/assets/icon/devtools-256.png"
if [[ ! -f "$ICON_SRC" ]]; then
  ICON_SRC="$ROOT/assets/icon/devtools-512.png"
fi

mkdir -p "$DIST" "$DIST/stage/$NAME"

BIN="target/${TARGET}/release/devtools"
if [[ ! -f "$BIN" ]]; then
  BIN="target/release/devtools"
fi
test -x "$BIN"

install -Dm755 "$BIN" "$DIST/stage/$NAME/devtools"
cp README.md "$DIST/stage/$NAME/"
cp docs/packaging.md "$DIST/stage/$NAME/" 2>/dev/null || true
if [[ -f "$ICON_SRC" ]]; then
  mkdir -p "$DIST/stage/$NAME/icons"
  cp "$ICON_SRC" "$DIST/stage/$NAME/icons/devtools.png"
fi

tar -C "$DIST/stage" -czf "$DIST/${NAME}.tar.gz" "$NAME"
echo "Wrote $DIST/${NAME}.tar.gz"

# Minimal .deb (amd64)
DEB_ROOT="$DIST/deb"
rm -rf "$DEB_ROOT"
mkdir -p "$DEB_ROOT/DEBIAN" \
  "$DEB_ROOT/usr/bin" \
  "$DEB_ROOT/usr/share/doc/devtools" \
  "$DEB_ROOT/usr/share/applications" \
  "$DEB_ROOT/usr/share/icons/hicolor/256x256/apps"
install -Dm755 "$BIN" "$DEB_ROOT/usr/bin/devtools"
cp README.md "$DEB_ROOT/usr/share/doc/devtools/"
if [[ -f "$ICON_SRC" ]]; then
  install -Dm644 "$ICON_SRC" "$DEB_ROOT/usr/share/icons/hicolor/256x256/apps/devtools.png"
fi
cat > "$DEB_ROOT/usr/share/applications/devtools.desktop" << DESKTOP
[Desktop Entry]
Type=Application
Name=Devtools
Comment=Native offline developer toolbox
Exec=devtools
Icon=devtools
Categories=Development;Utility;
Terminal=false
StartupNotify=true
DESKTOP
cat > "$DEB_ROOT/DEBIAN/control" << CTRL
Package: devtools
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: amd64
Maintainer: Devtools contributors <noreply@users.noreply.github.com>
Depends: libxkbcommon0, libxkbcommon-x11-0, libxcb1, libfontconfig1, libfreetype6, libvulkan1
Description: Native offline developer toolbox (GPUI)
 Offline-first developer utilities built with GPUI (X11).
CTRL
dpkg-deb --build --root-owner-group "$DEB_ROOT" "$DIST/devtools_${VERSION}_amd64.deb"
echo "Wrote $DIST/devtools_${VERSION}_amd64.deb"
