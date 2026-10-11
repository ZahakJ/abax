# Architecture for contributors

## Two crates

```text
crates/abax-core/     pure logic: no UI, no I/O, no randomness of its own
  soroban.rs          rods and the soroban: beads <-> digits <-> numbers
  moves.rs            the bead-move engine: the exact move list for adding or
                      subtracting a digit (complements, carry, borrow, finger order)
  arith.rs            multi-digit add/sub; multiplication and division as
                      narrated step sequences (Kojima's standard methods)
  curriculum.rs       the 15 skills, prerequisites, ladders, lesson text and demos
  generator.rs        drill items per skill and ladder step
  scheduler.rs        BKT, EWMAs, Elo, ladder rules, mastery, review, session plan
  stats.rs            history, momentum, chart series, session log
  store.rs            the versioned save document and its migration
crates/abax/          the eframe app: a binary on the desktop, a cdylib on Android
  lib.rs              App: screens, settings, autosave, capture and bench modes,
                      android_main
  main.rs             desktop entry point (window size, bench options)
  soroban_view.rs     the interactive soroban: hit testing, tap and drag,
                      spring animation, hints, building render instances
  render/             the wgpu paint callback (mod.rs) and its shader (soroban.wgsl)
  screens/            home, practice, lesson, drill, flash, picker, stats, settings
  keypad.rs           on-screen keypad and keyboard mapping
  theme.rs, ui.rs     palette, finishes, fonts; shared widgets
  charts.rs           the stats charts
  audio.rs            the synthesised click (desktop, cpal)
  storage.rs          data directory, atomic save, load with backup
android/              manifest, resources, build-apk.sh
packaging/            .desktop file, icon, install.sh, Info.plist, make-app.sh
docs/                 this manual, the landing page, the screenshots
research/             the cited notes the design is built on
verify                the release gate
```

`abax-core` is where the rules live, and it is the part with most of the
tests. Its randomness comes from the caller: generators take a `dice`
closure, the scheduler a small seeded xorshift generator, so every test is
deterministic.

## Data flow

1. **Input.** A tap, drag or key goes to `SorobanView`, which updates its
   `Soroban` model and animates the beads towards it.
2. **Checking.** The active screen compares the model with the current
   `Item` (from `generator::generate`). Chain items follow the running total
   operand by operand; typed items compare the keypad buffer.
3. **Recording.** `DrillState::submit` calls `Scheduler::record(skill,
   correct, seconds, now)`, which updates that skill's state and returns
   whether it stepped up, stepped down or became mastered, and appends an
   `Attempt` to `History`.
4. **Saving.** The app marks itself dirty; within 5 seconds, and on exit,
   `storage::save` writes the whole `SaveFile` atomically.
5. **Planning.** The home screen asks `Scheduler::next_skill_to_learn` for
   an untried lesson, otherwise `Scheduler::plan_session` for warm-up, focus
   and cool-down drills.

## Adding a skill

1. `curriculum.rs`: add a `SkillId` variant in teaching order and to
   `SkillId::ALL`; give it a `code`, `name`, `short_name`,
   `prerequisites`, `base_difficulty`, `benchmark_seconds` and a `ladder`;
   add its `Lesson` to `LESSONS`.
2. `generator.rs`: generate its items in `generate`, and if it is a bead
   skill, give it a `Constraint` in `constraint_for` saying which rules it
   allows and which it must contain.
3. The app: `drill_kind_for` in `screens/drill.rs`, the groups in
   `screens/picker.rs`, and the session start in `App::start_pending_or`
   if it needs a special mode.
4. Tests already guard the shape: prerequisites must form a graph in
   teaching order with one root, and every skill needs a lesson and a ladder
   of at least three steps that starts untimed.

Old save files keep loading: `Scheduler::fill_missing` gives a new skill a
fresh state. **Do not rename existing variants**: their names are the keys in
the save file.

## Adding a drill type

1. `generator.rs`: an `Item` variant, with `answer`, `prompt`,
   `operand_count` and `solution_steps` (the moves a wrong-answer replay
   shows).
2. `stats.rs`: a `DrillKind` variant, if the stats should tell it apart.
   Append it; variant names are stored.
3. `screens/drill.rs`: how the item is set up on the soroban
   (`setup_item`), whether the answer is typed or worked on the beads
   (`typed_answer`), how many rods it needs (`rods_needed`) and what the
   replay shows (`replay_steps_for`).

## Tests

There are 115: 92 in `abax-core` and 23 in the app. They include:

- the complete 10 × 9 addition and 10 × 9 subtraction tables from the
  research notes: for every digit on the rod and every digit added or
  subtracted, the exact move list, rule and order;
- worked multiplication and division examples from Kojima, including
  division with revised quotient digits;
- generators: every item for a skill uses that skill's rule and nothing
  beyond it;
- the scheduler checklist from `research/trainer-design.md` section 3.8:
  the BKT sequence, the EWMA sequences, U(n), Elo monotonicity, the
  mastery gate rejecting four lucky answers and slow answers, forgetting
  and the session planner never picking a locked skill;
- save files: round trip, unknown fields ignored, missing fields defaulted,
  bad input an error rather than a crash; atomic write and backup on a damaged file.

Run them with `cargo test --workspace`, or the whole gate with `./verify`.
Bead-move changes must keep the full tables green.

## Conventions

- Clippy warnings are errors; `cargo fmt` decides formatting.
- Western digits (0 to 9) in every string.
- The identity in `DESIGN.md`: calm, physical, no gamification, one
  vermilion accent.
- Captures live only in `docs/screenshots/`.

See [`CONTRIBUTING.md`](https://github.com/ZahakJ/abax/blob/main/CONTRIBUTING.md)
for how to propose a change.
