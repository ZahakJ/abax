# Abax — build plan

Abax is a soroban (Japanese abacus) trainer: one native binary per desktop
platform plus an Android APK, written in Rust with egui/eframe on wgpu.

## Name

"Abax" (Greek ἄβαξ, the board the abacus takes its name from). Short,
classical, a usable command. Known namesake: ABAX AS, a Norwegian fleet
telematics company (abax.com). It is a B2B vehicle-tracking service, not a
consumer app or developer tool, so there is no realistic confusion with a
free abacus trainer; the name is kept and the namesake recorded here.

## Framework decision (Phase 0, done)

egui/eframe 0.36 on wgpu, confirmed by a spike:

- Desktop: `cargo build -p abax` produces a binary that opens a window on the
  live Wayland session and finds the Vulkan adapter (exit 0 under `timeout`).
- Android: `cargo ndk -t arm64-v8a --platform 24 build -p abax --release`
  compiles eframe with the `android-native-activity` feature; the APK is
  packaged **without Gradle** by `android/build-apk.sh` using only the SDK
  build-tools (`aapt2 compile/link`, native libs stored uncompressed, `zipalign
  -P 16`, `apksigner`). `apksigner verify` reports v2 and v3 signatures; `aapt
  dump badging` shows `android.app.NativeActivity`, minSdk 24, targetSdk 35,
  `native-code: arm64-v8a`. The manifest has `hasCode="false"`, so no Java,
  no Dex, no Gradle plugin download: this is faster and far more reproducible
  in CI than a Gradle wrapper project.
- NativeActivity rather than GameActivity: GameActivity needs the
  `androidx.games` AAR, hence Gradle. The one thing NativeActivity does worse
  is soft-keyboard text input; Abax ships its own on-screen keypad, so it is
  not needed.
- Fallbacks (Tauri 2 + canvas, Bevy) were not needed and are not pursued.

## Crate layout

```
Cargo.toml                 workspace, release profile (lto, strip, panic=abort)
crates/abax-core/          pure logic, no UI dependencies, heavily unit tested
  src/soroban.rs           Rod/Soroban model: bead states <-> digits <-> numbers
  src/moves.rs             the bead-move engine: add/subtract a digit to a rod,
                           producing the exact move list (complement rules,
                           carry/borrow propagation) a teacher would make
  src/arith.rs             multi-digit add/sub, multiplication and division
                           procedures as step sequences for display
  src/curriculum.rs        skills, prerequisites, lessons (text + demo), ladders
  src/generator.rs         drill item generators per skill and difficulty step
  src/scheduler.rs         per-skill mastery (BKT + EWMAs + Elo), session planner
  src/stats.rs             history, momentum stats, chart series
  src/store.rs             serde model of everything saved, versioned
crates/abax/               eframe app (bin for desktop, cdylib for Android)
  src/lib.rs               App, android_main
  src/main.rs              desktop entry
  src/theme.rs             palette, fonts, light/dark visuals
  src/soroban_view.rs      the interactive abacus widget (hit testing, animation)
  src/keypad.rs            on-screen keypad + keyboard handling
  src/screens/             home, practice, lessons, drills, anzan, flash, stats, settings
  src/audio.rs             synthesised bead click (desktop: cpal; Android: AAudio
                           via the ndk crate, or silently disabled if unavailable)
  src/storage.rs           platform data dir, atomic save, Android internal path
android/                   manifest, resources, build-apk.sh
packaging/                 .desktop file, icons, AppImage/dmg helpers
.github/workflows/         ci.yml, release.yml
research/                  cited notes the design is built on
verify                     the release gate
```

Why no SQLite: the data is one user's history (a few thousand attempts). A
versioned JSON file written atomically (write temp, fsync, rename) is
simpler, cross-compiles to Android with zero C code, and is trivially
inspectable. Load time is negligible.

## Phases

| # | Phase | Verification |
|---|-------|--------------|
| 0 | Spike: eframe desktop + Android APK | done (see above) |
| 1 | `abax-core`: soroban model, move engine, every complement rule, carry/borrow | unit tests incl. the full 10x9 add table and 10x9 subtract table from research |
| 2 | `abax-core`: multiplication and division procedures as steps | tests against worked examples from Kojima/Heffelfinger |
| 3 | `abax-core`: curriculum, generators, scheduler, stats, store | tests from research/trainer-design.md §3.8; property tests on generators |
| 4 | UI: theme, soroban widget (tap/drag, animation), keypad, free practice | manual launch under `timeout`; frame-time log with 27 rods |
| 5 | UI: lessons, drills, anzan, flash anzan, stats charts, settings, sound | launch; screenshots |
| 6 | Storage on desktop + Android; `verify` script; README; perf numbers | `./verify` green; APK rebuilt and checked |
| 7 | GitHub: private repo, CI, release workflow, keystore secrets, tag v0.1.0 | `gh run watch`, assets for Linux/Windows/macOS/Android attached |

## Risks and mitigations

- **egui 0.36 API churn** (App::ui instead of update, panels take `&mut Ui`):
  read the vendored source, not memory. Spike already adapted.
- **Android touch input through NativeActivity**: egui-winit maps touch to
  pointer events; the widget must use full-height rod columns as hit areas
  (research §4) so small beads are not a precision problem.
- **Audio on Android**: cpal needs oboe (C++). Plan: desktop-only cpal with a
  feature flag; Android uses the `ndk` crate's AAudio bindings if they build
  cleanly, otherwise sound is unavailable on Android in v0.1 and documented.
- **macOS CI cost**: build once, after local Linux and Android checks pass;
  one tag only.
- **Memory on this machine**: a sibling build is running. `CARGO_BUILD_JOBS=6`,
  no emulator unless `free -g` shows > 8 GB available.
- **16 KB page size** on Android 15: linker flag in `.cargo/config.toml` plus
  `zipalign -P 16`.

## Performance targets (to measure and record in README)

- Cold start to first frame on the desktop: < 300 ms (release).
- Frame time with 27 rods and all beads animating: < 4 ms (release, 60 Hz headroom).
- Binary size: Linux release < 15 MB; APK < 12 MB per ABI.

## v0.2.0: rendering and public presence

### Rendering decision

The soroban is drawn by a custom `egui_wgpu::Callback` (one pipeline, one
instanced draw, WGSL fragment shading) instead of layered egui polygons.
Why:

- Per-pixel lighting is what makes a bead read as a solid object: a key
  light, a specular lobe, rim darkening, ambient occlusion where two beads
  touch and a contact shadow on the board. Faking those with stacked
  convex polygons and vertex colours gives bands, not surfaces, and needs
  a dozen shapes per bead (135 beads at 27 rods).
- Wood grain, lacquer sheen, bevels and a vignette are a few lines of
  procedural math in a fragment shader; as textures they would need an
  atlas per theme and per pixel density.
- Cost is one draw call per soroban with ~250 instances; the fragment work
  is cheap SDF math, well inside the budget at 27 rods animating. The
  benchmark mode (`ABAX_BENCH=frames=N`) measures it; numbers before and
  after are in the README.
- eframe is already built on wgpu on every target (desktop and Android
  NativeActivity), so the callback adds no dependency; `eframe::egui_wgpu`
  is re-exported. The shader is WGSL, translated by naga to Vulkan, Metal,
  DX12 and GLES, so one shader covers the four platforms.
- Fallback: if `wgpu_render_state` is unavailable the widget uses the
  previous painter path, so the app never shows an empty board.

Everything that is not the instrument itself (text, hint arrows, digits,
keypad, charts) stays in egui so it keeps egui's accessibility and
theming.

### Materials

Three finishes under Settings > Appearance, each tuned for light and dark:

| Finish | Frame | Rods and beam | Beads |
|--------|-------|---------------|-------|
| Walnut (default) | lacquered walnut, warm grain | bamboo | bone |
| Ebony | near-black ebony, fine grain | brass | bone |
| Maple | pale maple, fine grain | steel | slate |

Vermilion stays the single accent in every finish.

### Public presence

- README: short; everything longer moves to the manual.
- Manual: mdBook under `docs/` (pinned binary downloaded in the Pages
  workflow, custom theme in the Abax palette), published at
  `/abax/manual/`.
- Landing page: hand-written static HTML/CSS under `docs/site/`, published
  at `/abax/`. It reads the latest release from the GitHub API for the
  download block.
- Both are built by `.github/workflows/pages.yml` on push to `main`.
- Captures live only under `docs/screenshots/` (used by the README, the
  landing page and the manual).
