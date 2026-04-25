#!/bin/sh
# Bundle the mhrv-f-ui binary into a macOS .app.
# Usage: build-app.sh <ui-binary> <version> <output-dir>
set -eu

BIN="$1"
VER="$2"
OUT_DIR="$3"

APP="$OUT_DIR/mhrv-f.app"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS"
mkdir -p "$APP/Contents/Resources"

cp "$BIN" "$APP/Contents/MacOS/mhrv-f-ui"
chmod +x "$APP/Contents/MacOS/mhrv-f-ui"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
sed "s/__VERSION__/$VER/g" "$SCRIPT_DIR/Info.plist" > "$APP/Contents/Info.plist"

# Best-effort icon generation.
#
# We intentionally do not commit binary `.icns` blobs to this repository.
# If the host has the required tools, we generate `AppIcon.icns` during bundling.
LOGO_SVG="$SCRIPT_DIR/../logo/frankestein-mark.svg"
ICONSET_DIR="$APP/Contents/Resources/AppIcon.iconset"
ICNS_OUT="$APP/Contents/Resources/AppIcon.icns"

if [ -f "$LOGO_SVG" ]; then
  rm -rf "$ICONSET_DIR"
  mkdir -p "$ICONSET_DIR"

  if command -v rsvg-convert >/dev/null 2>&1; then
    # Linux/macOS with librsvg installed.
    for size in 16 32 64 128 256 512; do
      rsvg-convert -w "$size" -h "$size" "$LOGO_SVG" > "$ICONSET_DIR/icon_${size}x${size}.png"
      rsvg-convert -w "$((size*2))" -h "$((size*2))" "$LOGO_SVG" > "$ICONSET_DIR/icon_${size}x${size}@2x.png"
    done
  elif command -v qlmanage >/dev/null 2>&1; then
    # macOS preview renderer (works on many machines; not guaranteed).
    for size in 16 32 64 128 256 512; do
      qlmanage -t -s "$size" -o "$ICONSET_DIR" "$LOGO_SVG" >/dev/null 2>&1 || true
      # qlmanage outputs with the original filename; rename if present.
      if [ -f "$ICONSET_DIR/$(basename "$LOGO_SVG").png" ]; then
        mv "$ICONSET_DIR/$(basename "$LOGO_SVG").png" "$ICONSET_DIR/icon_${size}x${size}.png"
      fi
    done
    # If we only got 1x assets, iconutil can still build an icns (lower quality on retina).
  fi

  if command -v iconutil >/dev/null 2>&1 && [ -d "$ICONSET_DIR" ]; then
    # Only attempt icns build if we actually produced at least one PNG.
    if ls "$ICONSET_DIR"/*.png >/dev/null 2>&1; then
      iconutil -c icns "$ICONSET_DIR" -o "$ICNS_OUT" >/dev/null 2>&1 || true
    fi
  fi
fi

echo "Built $APP"
