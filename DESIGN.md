# Abax — design

## Identity

Abax should feel like a real instrument on a desk, not a math game.
Calm, physical, quiet. No neon, no confetti, no mascots.

### Palette

| Token | Light | Dark | Use |
|-------|-------|------|-----|
| paper | #F4EDE0 | #1C1A17 | app background |
| paper-2 | #EADFCC | #26221E | panels, cards |
| ink | #2A2320 | #EDE6DA | text |
| ink-soft | #6E6259 | #A69C90 | secondary text |
| frame | #3B2A1E | #2C1F16 | soroban frame (lacquered walnut) |
| frame-light | #6B4A2E | #4A3424 | frame bevel highlight |
| rod | #B08D5B | #8C6F47 | bamboo rods |
| beam | #C9A66B | #A8894F | reckoning bar |
| bead | #EFE6D6 | #E6DAC4 | bone beads |
| bead-shadow | #C8B89E | #A89675 | bead lower edge |
| bead-hilite | #FFFBF3 | #F7F0E2 | bead specular |
| accent | #C2452D | #D8563B | vermilion lacquer: the one accent (active bead, primary button, chart line) |
| accent-soft | #E9C4B8 | #5C2A20 | highlight fill for "move this bead" |
| ok | #4F7A4A | #7FAF78 | correct |
| warn | #B8862B | #D4A440 | hint / stalled |

Highlights for "move this bead" are never colour-only: a soft accent glow
behind the bead (pulsing with a 1.2 s period, steady with reduce motion) plus
a small rounded arrow in the direction of the move (research §4).

The frame, rods, beam and beads take their colours from the finish
(Settings > Appearance: walnut, ebony, maple; `theme::material`), not from
this table; the frame/rod/beam/bead tokens above are the walnut finish.

### Type

egui default fonts (Ubuntu / Noto). Number display uses the monospace face
so digits do not jitter while animating. Western digits everywhere.

### Motion

- Every bead move settles on a critically damped spring curve (starts from
  rest, decelerates long and soft, never passes its stop).
- A bead move the user makes: 90 ms. Beads on the same rod that move
  together are staggered by 15 ms so the group reads as a sweep; the moving
  beads brighten briefly (150 ms), in time with the optional click.
- A demonstrated move (lesson, hint): 400 ms, repeated after a
  600 ms pause until the user performs it.
- Reset: tilt metaphor — all beads leave the beam in 220 ms with a 12 ms
  stagger from the right rod.
- Anzan fade: beads turn to glass over the configured span: the body goes
  translucent first, highlights stay until late, everything vanishes over
  the last quarter; the frame stays.

### Sound

Off by default. One synthesised click (a 2 ms noise burst into a short
decaying 1.8 kHz resonance, 30 ms total) per bead group move, slightly
lower pitch for heaven beads. No music.

### Layout

Phone-first: the soroban fills the width, the keypad sits under the thumb,
the question or lesson text sits above. On desktop the same layout grows;
the keypad becomes optional because the keyboard works.

Rod count defaults: 13 on phones, 17 on desktop, choices 9/13/17/21/23/27.

## Interaction model

- Tap an earth bead: it and the beads between it and the beam move together
  (toward the beam if it is away, away if it is touching). Tap the heaven
  bead: toggles. The hit area of a bead is the full rod width and the bead's
  row height, so fingers never need precision.
- Drag: beads follow the finger vertically with the same group rule; release
  snaps to the nearer state.
- Keyboard: digits set the active rod and advance right; Backspace clears
  the active rod and moves left; arrows move the active rod; Space/Enter
  submits; Escape clears the soroban; `c` clears.
- The learner is never told "wrong" without being shown the correct move.
  On a wrong answer the correct bead sequence replays on the soroban.

## Curriculum

Skills form a small graph (prerequisites in brackets). A skill unlocks when
every prerequisite has mastery estimate >= 0.80; it is mastered when the
scheduler's gate passes (see research/trainer-design.md §3.5).

| Id | Skill | Taught as |
|----|-------|-----------|
| S0 | Read and set numbers | Rods, beam, heaven = 5, earth = 1, unit dots; set 1..9, then multi-digit |
| S1 | Add without complements | Thumb pushes earth beads up; index pulls heaven down |
| S2 | Subtract without complements | Index pushes earth beads down and heaven up |
| S3 | Add with five-complement (small friends) | +4 on 1: set 5, take 1 (4 = 5 − 1); pairs 1+4, 2+3 |
| S4 | Subtract with five-complement | −4 on 5: take 5, set 1 |
| S5 | Add with ten-complement (big friends) | +7 on 4: take 3, set 1 on the next rod left (7 = 10 − 3) |
| S6 | Subtract with ten-complement | −7 on 11: take 1 on the left, set 3 |
| S7 | Combined complements | +6 on 7: take 4 — but 4 needs the five-complement: set 1, take 5; then carry 1 |
| S8 | Multi-digit addition chains | Left to right, one rod at a time |
| S9 | Multi-digit mixed chains | Add and subtract in one run |
| S10 | Multiplication | Standard method: multiplier left, multiplicand right, partial products placed by rod count |
| S11 | Division | Standard method: quotient left of the dividend, revise when the trial quotient is too large |
| A1 | Anzan: visualised chains | Beads fade progressively while adding 1-digit numbers |
| A2 | Flash anzan, one digit | Numbers flash; learner answers the sum; display time ladder 1.5 s to 0.4 s |
| A3 | Flash anzan, two+ digits | Same, more digits and more numbers |

Each lesson: a short explanation in plain words, a demonstration with
highlighted beads and the finger to use, three guided items where the
trainer shows the move when the learner stalls for 4 s, then the drill.

Drill types: set this number; read this soroban; add/subtract chains;
multiplication; division; mixed; timed sprint (60 s, as many as you can,
ends on time not on error).

## Stats

Momentum, not streaks: a 14-day rolling count of items and minutes, a
"carried over" indicator that decays gently instead of resetting to zero,
per-skill mastery bars, accuracy and speed curves per skill, and a session
log. Nothing in the app shames a missed day.
