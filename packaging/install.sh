#!/usr/bin/env bash
# Install Abax for the current user (no root): binary to ~/.local/bin,
# desktop entry and icon to ~/.local/share.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
bin_dir="${XDG_BIN_HOME:-$HOME/.local/bin}"
share="${XDG_DATA_HOME:-$HOME/.local/share}"
mkdir -p "$bin_dir" "$share/applications" "$share/icons/hicolor/256x256/apps"
install -m 755 "$here/abax" "$bin_dir/abax"
install -m 644 "$here/abax.desktop" "$share/applications/abax.desktop"
install -m 644 "$here/icon-256.png" "$share/icons/hicolor/256x256/apps/abax.png"
command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$share/applications" || true
echo "Installed abax to $bin_dir/abax"
case ":$PATH:" in
  *":$bin_dir:"*) ;;
  *) echo "Note: $bin_dir is not on your PATH; run it as $bin_dir/abax or add the directory to PATH." ;;
esac
