#!/usr/bin/env bash
# Rebuild assets/icon/AppIcon.icns and size PNGs from assets/icon/devtools-1024.png.
# On macOS: uses iconutil (+ sips). On Linux: ImageMagick + `pip install icnsutil`.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ICON="$ROOT/assets/icon"
SRC="$ICON/devtools-1024.png"
test -f "$SRC"

ICONSET="$ICON/AppIcon.iconset"
rm -rf "$ICONSET"
mkdir -p "$ICONSET"

resize() {
  local size="$1" out="$2"
  if command -v sips >/dev/null 2>&1; then
    sips -z "$size" "$size" "$SRC" --out "$out" >/dev/null
  elif command -v magick >/dev/null 2>&1; then
    magick "$SRC" -resize "${size}x${size}" "$out"
  else
    echo "need sips or ImageMagick magick" >&2
    exit 1
  fi
}

resize 16   "$ICONSET/icon_16x16.png"
resize 32   "$ICONSET/icon_16x16@2x.png"
resize 32   "$ICONSET/icon_32x32.png"
resize 64   "$ICONSET/icon_32x32@2x.png"
resize 128  "$ICONSET/icon_128x128.png"
resize 256  "$ICONSET/icon_128x128@2x.png"
resize 256  "$ICONSET/icon_256x256.png"
resize 512  "$ICONSET/icon_256x256@2x.png"
resize 512  "$ICONSET/icon_512x512.png"
resize 1024 "$ICONSET/icon_512x512@2x.png"

for s in 16 32 64 128 256 512 1024; do
  resize "$s" "$ICON/devtools-${s}.png"
done

if command -v iconutil >/dev/null 2>&1; then
  iconutil -c icns "$ICONSET" -o "$ICON/AppIcon.icns"
  echo "Wrote $ICON/AppIcon.icns (iconutil)"
elif python3 -c "import icnsutil" 2>/dev/null; then
  ICONSET="$ICONSET" OUT="$ICON/AppIcon.icns" python3 - <<'PY'
import os
from pathlib import Path
import icnsutil
iconset = Path(os.environ["ICONSET"])
out = Path(os.environ["OUT"])
icns = icnsutil.IcnsFile()
for name in [
    "icon_16x16.png", "icon_16x16@2x.png",
    "icon_32x32.png", "icon_32x32@2x.png",
    "icon_128x128.png", "icon_128x128@2x.png",
    "icon_256x256.png", "icon_256x256@2x.png",
    "icon_512x512.png", "icon_512x512@2x.png",
]:
    icns.add_media(file=str(iconset / name))
icns.write(str(out))
print(f"Wrote {out} (icnsutil)")
PY
else
  echo "iconutil/icnsutil not found; iconset PNGs updated, keep existing AppIcon.icns" >&2
fi
