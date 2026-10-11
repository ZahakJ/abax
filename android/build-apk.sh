#!/usr/bin/env bash
# Build a signed Abax APK without Gradle: cargo-ndk for the native library,
# then aapt2 / zipalign / apksigner from the Android build-tools.
#
# Usage: android/build-apk.sh [--abis "arm64-v8a armeabi-v7a x86_64"] [--out path.apk] [--debug]
# Env:   ANDROID_HOME (SDK root), ANDROID_NDK_HOME (NDK root), JAVA_HOME (for apksigner)
#        ABAX_BUILD_TOOLS (default 35.0.0), ABAX_PLATFORM (default 35)
#        ABAX_KEYSTORE, ABAX_KEYSTORE_PASSWORD, ABAX_KEY_ALIAS (release signing)
#        When ABAX_KEYSTORE is unset a throwaway debug keystore is generated.
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/.." && pwd)"
abis="arm64-v8a"
out="$root/dist/abax.apk"
profile="--release"
while [ $# -gt 0 ]; do
  case "$1" in
    --abis) abis="$2"; shift 2 ;;
    --out) out="$2"; shift 2 ;;
    --debug) profile=""; shift ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
done

: "${ANDROID_HOME:?set ANDROID_HOME}"
: "${ANDROID_NDK_HOME:?set ANDROID_NDK_HOME}"
# Pinned SDK components (the 0.1.0 release picked whatever sorted last on the
# runner, which was a preview API 37 platform). Override with the env vars.
bt_version="${ABAX_BUILD_TOOLS:-35.0.0}"
platform_version="${ABAX_PLATFORM:-35}"
bt="$ANDROID_HOME/build-tools/$bt_version"
android_jar="$ANDROID_HOME/platforms/android-$platform_version/android.jar"
[ -x "$bt/aapt2" ] || { echo "build-tools $bt_version not found under $ANDROID_HOME/build-tools" >&2; exit 1; }
[ -f "$android_jar" ] || { echo "platform android-$platform_version not found under $ANDROID_HOME/platforms" >&2; exit 1; }
java_bin="${JAVA_HOME:+$JAVA_HOME/bin/}"

version_name="$(grep -m1 '^version' "$root/Cargo.toml" | sed 's/.*"\(.*\)".*/\1/')"
# version code: major*10000 + minor*100 + patch
IFS=. read -r vmaj vmin vpat <<<"$version_name"
version_code=$((vmaj * 10000 + vmin * 100 + vpat))

build="$root/target/android"
rm -rf "$build/stage" "$build/res.zip"
mkdir -p "$build/stage" "$(dirname "$out")"

echo "==> cargo ndk ($abis)"
targets=()
for abi in $abis; do targets+=(-t "$abi"); done
(cd "$root" && cargo ndk "${targets[@]}" --platform 24 -o "$build/stage/lib" build -p abax $profile)

echo "==> aapt2 compile/link"
"$bt/aapt2" compile --dir "$here/res" -o "$build/res.zip"
"$bt/aapt2" link -o "$build/unaligned.apk" -I "$android_jar" \
  --manifest "$here/AndroidManifest.xml" \
  --min-sdk-version 24 --target-sdk-version 35 \
  --version-code "$version_code" --version-name "$version_name" \
  --auto-add-overlay "$build/res.zip"

echo "==> add native libraries (stored, uncompressed)"
python3 - "$build/unaligned.apk" "$build/stage" <<'PY'
import os, sys, zipfile
apk, stage = sys.argv[1], sys.argv[2]
with zipfile.ZipFile(apk, "a", compression=zipfile.ZIP_STORED) as z:
    for dirpath, _, files in os.walk(os.path.join(stage, "lib")):
        for f in files:
            full = os.path.join(dirpath, f)
            arc = os.path.relpath(full, stage)
            z.write(full, arc, compress_type=zipfile.ZIP_STORED)
            print("   ", arc, os.path.getsize(full) // 1024, "KiB")
PY

echo "==> zipalign (4-byte, 16 KiB pages for .so)"
"$bt/zipalign" -P 16 -f 4 "$build/unaligned.apk" "$build/aligned.apk"

if [ -z "${ABAX_KEYSTORE:-}" ]; then
  echo "==> no ABAX_KEYSTORE: signing with a throwaway debug key"
  ABAX_KEYSTORE="$build/debug.keystore"
  ABAX_KEYSTORE_PASSWORD="android"
  ABAX_KEY_ALIAS="androiddebugkey"
  if [ ! -f "$ABAX_KEYSTORE" ]; then
    "${java_bin}keytool" -genkeypair -keystore "$ABAX_KEYSTORE" -storetype PKCS12 -alias "$ABAX_KEY_ALIAS" \
      -keyalg RSA -keysize 2048 -validity 10000 -storepass android -keypass android \
      -dname "CN=Abax Debug" >/dev/null 2>&1
  fi
fi
: "${ABAX_KEYSTORE_PASSWORD:?set ABAX_KEYSTORE_PASSWORD}"
: "${ABAX_KEY_ALIAS:?set ABAX_KEY_ALIAS}"

echo "==> apksigner"
"$bt/apksigner" sign --ks "$ABAX_KEYSTORE" --ks-pass "pass:$ABAX_KEYSTORE_PASSWORD" \
  --ks-key-alias "$ABAX_KEY_ALIAS" --key-pass "pass:$ABAX_KEYSTORE_PASSWORD" \
  --min-sdk-version 24 --out "$out" "$build/aligned.apk"
"$bt/apksigner" verify --verbose --print-certs "$out" | head -8
"$bt/aapt" dump badging "$out" | grep -E "^(package|sdkVersion|targetSdkVersion|application-label|launchable-activity|native-code)"
ls -la "$out"
