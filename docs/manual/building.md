# Building from source

## Desktop

You need a stable Rust toolchain (install it with [rustup](https://rustup.rs)).
On Linux you also need the development headers for ALSA (sound) and for
X11/Wayland windowing. On Debian or Ubuntu:

```sh
sudo apt-get install libasound2-dev libxkbcommon-dev libwayland-dev \
  libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev
```

Windows and macOS need nothing beyond Rust (and the Xcode command-line
tools on macOS).

```sh
git clone https://github.com/ZahakJ/abax
cd abax
cargo run --release -p abax
```

The release profile uses link-time optimisation, strips symbols and aborts
on panic; the Linux binary is about 14 MB.

## The release gate: `./verify`

```sh
./verify
```

runs, in order, and stops at the first failure:

1. `cargo fmt --all -- --check`: formatting.
2. `cargo clippy --workspace --all-targets -- -D warnings`: lints, with
   every warning an error.
3. `cargo test --workspace`: all unit tests.
4. `cargo build --release -p abax`: the desktop binary.

CI runs the same script on every push, so a green `./verify` locally means a
green check on GitHub.

## Android, without Gradle

Abax on Android is a pure native app: the manifest says `hasCode="false"`
and the activity is Android's built-in `NativeActivity`, which loads
`libabax.so` and calls its `android_main`. There is no Java or Kotlin, so
there is nothing for Gradle to do. `android/build-apk.sh` builds the APK with
the SDK's own command-line tools, which is faster and easier to reproduce
than a Gradle project. The one thing `NativeActivity` does worse than the
newer `GameActivity` is soft-keyboard input; Abax has its own keypad, so it
does not need it.

### What you need

- The Android SDK with `build-tools;35.0.0` and `platforms;android-35`.
- The NDK (CI uses 27.2.12479018).
- A JDK, for `apksigner` and `keytool`.
- `cargo-ndk` and the Rust Android targets:

```sh
cargo install cargo-ndk
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
export ANDROID_HOME=/path/to/android-sdk
export ANDROID_NDK_HOME=$ANDROID_HOME/ndk/27.2.12479018
export JAVA_HOME=/path/to/jdk
```

### Build

```sh
android/build-apk.sh --abis arm64-v8a --out dist/abax.apk
android/build-apk.sh --abis "arm64-v8a armeabi-v7a x86_64" --out dist/abax-universal.apk
```

Add `--debug` for an unoptimised build.

### What the script does, step by step

1. **Native library.** `cargo ndk -t <abi> … --platform 24 build -p abax
   --release` compiles the `abax` crate as `libabax.so` for each ABI, against
   API level 24 (Android 7.0). `.cargo/config.toml` adds
   `-z max-page-size=16384` to the link, which Android 15 devices with 16 KB
   memory pages require.
2. **Resources.** `aapt2 compile` turns `android/res/` (icon, label, theme)
   into compiled resources; `aapt2 link` combines them with
   `android/AndroidManifest.xml` and the platform's `android.jar` into an
   unsigned APK. The version name comes from the workspace `Cargo.toml`
   (0.2.0) and the version code is major × 10000 + minor × 100 + patch
   (200).
3. **Libraries in.** A few lines of Python add `lib/<abi>/libabax.so` to
   the APK **uncompressed**, so Android can map them straight from the APK
   instead of extracting them.
4. **Align.** `zipalign -P 16 -f 4` aligns the uncompressed libraries to
   16 KB pages and everything else to 4 bytes.
5. **Sign.** `apksigner sign` signs with APK signature schemes v2 and v3,
   then `apksigner verify` and `aapt dump badging` print what was built:
   the package, `minSdkVersion 24`, `targetSdkVersion 35`, the launchable
   `android.app.NativeActivity` and the native ABIs.

### Pinned SDK components

The script uses fixed versions instead of whatever is newest in your SDK.
(Picking the last directory in sort order once picked a preview platform.)

| Variable | Default | Selects |
|----------|---------|---------|
| `ABAX_BUILD_TOOLS` | `35.0.0` | `$ANDROID_HOME/build-tools/<version>` |
| `ABAX_PLATFORM` | `35` | `$ANDROID_HOME/platforms/android-<version>/android.jar` |

It stops with a clear message if either is missing.

### Signing

Without `ABAX_KEYSTORE` the script generates a throwaway debug key under
`target/android/` and signs with it. That is fine for trying a build on your
own phone, but Android refuses to update an app signed with a different key,
so a release needs a stable one:

| Variable | Meaning |
|----------|---------|
| `ABAX_KEYSTORE` | path to the keystore |
| `ABAX_KEYSTORE_PASSWORD` | its password (also used as the key password) |
| `ABAX_KEY_ALIAS` | the key's alias |

In the release workflow the keystore arrives base64-encoded in the
`ABAX_KEYSTORE_B64` secret; the job refuses to run without the three
secrets rather than publish a debug-signed APK.

## Releases

Pushing a tag `vX.Y.Z` runs `.github/workflows/release.yml`: it builds Linux
(tarball with an installer), Windows (zip), macOS for Apple silicon and Intel
(dmg, cross-compiled on one Apple silicon runner) and both Android APKs,
writes `SHA256SUMS.txt`, and publishes a GitHub release whose text is the
tag's section of `CHANGELOG.md`.

## The website

The landing page and this manual are built by `docs/build.sh` and published
to GitHub Pages by `.github/workflows/pages.yml` on every push to `main` that
touches `docs/`. To build them locally you need
[mdBook](https://github.com/rust-lang/mdBook) 0.5.4:

```sh
cargo install mdbook --version 0.5.4 --locked
docs/build.sh site-out
python3 -m http.server -d site-out 8000   # landing page at /, manual at /manual/
```

On GitHub Pages the site lives under `/abax/`; all links are relative, so it
works at either root.
