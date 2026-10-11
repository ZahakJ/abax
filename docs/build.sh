#!/usr/bin/env bash
# Build the Pages site: landing page at the root, the mdBook manual under
# /manual/, screenshots under /screenshots/. Usage: docs/build.sh <out-dir>
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
out="$(mkdir -p "${1:?out dir}" && cd "$1" && pwd)"
rm -rf "$out"/*
mdbook build "$here" -d "$out/manual"
cp -r "$here/site/." "$out/"
cp -r "$here/screenshots" "$out/screenshots"
touch "$out/.nojekyll"
echo "site built in $out"
find "$out" -maxdepth 2 -type d | sort
