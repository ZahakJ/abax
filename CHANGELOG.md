# Changelog

All notable changes to Abax. The format follows Keep a Changelog; versions follow semantic versioning.

## Unreleased

## 0.2.0 - 2026-10-11

(filled in before the tag)

## 0.1.0 - 2026-10-10

First release.

- Soroban model and bead-move engine with every five- and ten-complement rule, carry and borrow, and Kojima's finger-order rules; multiplication and division as step sequences.
- Curriculum of 15 skills (S0-S11, A1-A3) with lessons (explanation, demonstration, guided practice), drills (set, read, chains, multiplication, division, mixed, 60 s sprint), anzan with fading beads and flash anzan.
- Adaptive scheduler: Bayesian Knowledge Tracing per skill, accuracy and speed EWMAs, Elo-style item difficulty, mastery gate, session planner (warm-up, focus, cool-down), 14-day momentum instead of streaks.
- egui/eframe app on wgpu: interactive soroban (tap, drag, keyboard), on-screen keypad, light and dark themes, stats charts, settings, synthesised bead click on desktop.
- Versioned JSON profile written atomically.
- Binaries for Linux, Windows and macOS; Android APKs (arm64 and universal) built without Gradle.
