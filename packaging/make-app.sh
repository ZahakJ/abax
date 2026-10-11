#!/usr/bin/env bash
# Build Abax.app from a compiled binary: packaging/make-app.sh <binary> <out-dir>
set -euo pipefail
bin="$1"; out="$2"
here="$(cd "$(dirname "$0")" && pwd)"
version="$(grep -m1 '^version' "$here/../Cargo.toml" | sed 's/.*"\(.*\)".*/\1/')"
app="$out/Abax.app"
rm -rf "$app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
cp "$bin" "$app/Contents/MacOS/abax"
chmod +x "$app/Contents/MacOS/abax"
sed "s/__VERSION__/$version/g" "$here/Info.plist" > "$app/Contents/Info.plist"
# Icon: build an .icns from the PNG when the macOS tools are present.
if command -v iconutil >/dev/null 2>&1 && command -v sips >/dev/null 2>&1; then
  iconset="$out/abax.iconset"
  rm -rf "$iconset"; mkdir -p "$iconset"
  for s in 16 32 64 128 256; do
    sips -z "$s" "$s" "$here/icon-256.png" --out "$iconset/icon_${s}x${s}.png" >/dev/null
  done
  cp "$iconset/icon_32x32.png" "$iconset/icon_16x16@2x.png"
  cp "$iconset/icon_64x64.png" "$iconset/icon_32x32@2x.png"
  cp "$iconset/icon_256x256.png" "$iconset/icon_128x128@2x.png"
  iconutil -c icns "$iconset" -o "$app/Contents/Resources/abax.icns" || true
  rm -rf "$iconset"
fi
echo "built $app"
