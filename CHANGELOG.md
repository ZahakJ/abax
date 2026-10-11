# Changelog

All notable changes to Abax. The format follows Keep a Changelog; versions follow semantic versioning.

## Unreleased

## 0.2.0 - 2026-10-11

### Rendering

- The soroban is drawn by one wgpu paint callback: one pipeline, one instanced draw, shaded per pixel in WGSL. Beads are solids of revolution with a key light, specular highlight, rim darkening, contact occlusion and soft shadows; rods are shaded cylinders; the beam is bevelled lacquer with inset unit dots; the frame is grained, bevelled wood around a vignetted well.
- Three finishes under Settings > Appearance, each tuned for light and dark: Walnut (bone beads, bamboo rods, the default), Ebony (bone beads, brass rods and beam) and Maple (slate beads, steel rods). Saves from 0.1.0 load as Walnut.
- The anzan fade turns beads to glass: the body goes first, the highlights last.
- CPU time per frame with 27 rods animating halves (0.38 to 0.18 ms average on the reference machine).
- The previous egui painter remains as a fallback when wgpu is unavailable.

### Interface

- Hints: a stronger vermilion halo and an outlined arrow with a shaft.
- Bead moves settle on a critically damped spring; beads you move brighten briefly.
- Flash anzan shows large numerals; the home screen carries a small live soroban.
- Shared header, card, section and button styles across all screens.

### Docs and site

- A manual (mdBook) and a landing page with downloads for the latest release, published on GitHub Pages at https://zahakj.github.io/abax/.
- The README is short; the long material moved to the manual.
- New captures of every screen in light and dark, the three finishes, and a looping hero clip.
- Screenshot mode can record a frame sequence (`ABAX_SHOT=<screen>:<dir>/`) and has `glass` and `flash-run` targets.
- Contributing guide, code of conduct, security policy, issue and pull request templates.

### Build and release

- The Android build pins build-tools 35.0.0 and platform 35 (`ABAX_BUILD_TOOLS`, `ABAX_PLATFORM`) instead of taking whatever sorts last in the SDK.
- Release notes are this file's section for the tag.
- The APK carries the workspace version: 0.1.0 shipped with versionCode 1 because aapt2 keeps manifest values unless told to replace them; 0.2.0 is versionCode 200 and installs over 0.1.0.
- The Android build was checked on an API 35 emulator: the wgpu soroban renders and responds to taps.
- A Pages workflow builds the site with a pinned mdBook 0.5.4.

## 0.1.0 - 2026-10-10

First release.

- Soroban model and bead-move engine with every five- and ten-complement rule, carry and borrow, and Kojima's finger-order rules; multiplication and division as step sequences.
- Curriculum of 15 skills (S0-S11, A1-A3) with lessons (explanation, demonstration, guided practice), drills (set, read, chains, multiplication, division, mixed, 60 s sprint), anzan with fading beads and flash anzan.
- Adaptive scheduler: Bayesian Knowledge Tracing per skill, accuracy and speed EWMAs, Elo-style item difficulty, mastery gate, session planner (warm-up, focus, cool-down), 14-day momentum instead of streaks.
- egui/eframe app on wgpu: interactive soroban (tap, drag, keyboard), on-screen keypad, light and dark themes, stats charts, settings, synthesised bead click on desktop.
- Versioned JSON profile written atomically.
- Binaries for Linux, Windows and macOS; Android APKs (arm64 and universal) built without Gradle.
