# Internals

Palette, motion and interaction. Code layout is in
[manual/architecture.md](manual/architecture.md), the renderer in
[manual/rendering.md](manual/rendering.md), the curriculum in
[manual/curriculum.md](manual/curriculum.md).

## Palette

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
| accent | #C2452D | #D8563B | vermilion: active bead, primary button, chart line |
| accent-soft | #E9C4B8 | #5C2A20 | "move this bead" fill |
| ok | #4F7A4A | #7FAF78 | correct |
| warn | #B8862B | #D4A440 | hint, stalled |

The frame, rod, beam and bead tokens are the walnut finish. Each finish
(walnut, ebony, maple; `theme::material`) has its own. Vermilion is the only
accent in every finish.

A "move this bead" hint is never colour alone: an accent glow behind the bead
(1.2 s pulse, steady with reduce motion) and an arrow in the direction of the
move.

## Type

egui default fonts. Numbers use the monospace face so digits do not jitter
while animating. Western digits everywhere.

## Motion

- Bead moves settle on a critically damped spring and never overshoot.
- A move the user makes: 90 ms. Beads on one rod moving together are
  staggered by 15 ms; moving beads brighten for 150 ms, in time with the
  click.
- A demonstrated move (lesson, hint): 400 ms, repeated after a 600 ms pause
  until the user makes it.
- Reset: all beads leave the beam in 220 ms, 12 ms stagger from the right.
- Anzan fade: the bead body turns translucent first, highlights last,
  everything gone over the final quarter of the span; the frame stays.

## Sound

Off by default. One synthesised click per bead group: a 2 ms noise burst into
a decaying 1.8 kHz resonance, 30 ms in all, lower for heaven beads.

## Layout

The soroban fills the width, the keypad sits under it, the question or lesson
text above. On the desktop the keypad is optional. Default rod count: 13 on
phones, 17 on the desktop; choices 9, 13, 17, 21, 23, 27.

## Interaction

- Tapping an earth bead moves it and every bead between it and the beam
  (toward the beam if away, away if touching). Tapping the heaven bead
  toggles it. The hit area is the full rod width and the bead's row height.
- Dragging moves beads with the same group rule; release snaps to the nearer
  state.
- Keyboard: digits set the active rod and advance right; Backspace clears it
  and moves left; arrows move the active rod; Space or Enter submits; Escape
  or Delete clears.
- A wrong answer always replays the correct bead sequence.

## Lessons and stats

A lesson is a short explanation, a demonstration with the bead and finger
highlighted, three guided items (the move is shown after a 4 s stall), then
the drill.

Stats: a 14-day rolling count of items and minutes, a carried-over figure
that halves every three idle days, per-skill
mastery bars, accuracy and speed per skill, and a session log.
