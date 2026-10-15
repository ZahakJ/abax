# Drill types

A drill is a short run of items, 8 to 15 in a planned session, 3 to 30 when
you choose one yourself under **Drills**. Every item has exactly one focus
skill, and the generator builds it so that the bead moves it needs use that
skill's rule and nothing you have not met yet. A "small friends" drill never
asks for a carry; a "combined" drill always contains at least one combined
move.

## The item kinds

| Kind | You see | You do | Skills |
|------|---------|--------|--------|
| Set this number | "Set 4071" and a clear soroban | Move the beads until they show it | S0 |
| Read this soroban | A number on the beads, digits hidden | Type the number | S0 |
| Chain | "6 + 5 + 6", the start already on the beads | Work each operation on the beads | S1 to S9 |
| Multiplication | "34 × 6" | Type the product; the worked procedure is available | S10 |
| Division | "391 ÷ 17" | Type the quotient; the worked procedure is available | S11 |
| Anzan chain | A chain whose start fades from the beads | Finish in your head, type the total | A1 |
| Flash | Numbers shown one at a time | Type the sum | A2, A3 |

**Working on the beads.** For "set" and chain items you do not type the
answer. The current operand is highlighted in the problem line; Abax follows
the running total on the beads and accepts the item once the beads have
shown the final answer for about half a second. You can also press Enter to
check early (the **Check** button).

**Hints.** If you make no move for the hint delay (4 s by default, see
[Settings](settings.md); 1.5 s in a lesson's guided practice), the next move
is shown on the soroban: the beads to move are haloed in vermilion, with an
arrow, for one rod at a time.

**When you are wrong.** Abax shows the correct answer and replays the
correct bead moves on the soroban in order, from where the item started. For
multiplication and division the step-by-step procedure takes that role.

## Where the drills come from

- **A planned session** (the Continue button on the home screen): warm-up,
  focus and cool-down drills chosen by the [scheduler](scheduler.md). About
  30 % of the focus drill's items are drawn from other skills you have
  already practised, each at its own current step.
- **Drills** on the home screen: choose a skill (grouped as set and read,
  add and subtract, longer numbers, multiply, divide, anzan), a step on its
  ladder (it starts at your current step; you can move it either way) and
  the number of items. Every skill can be chosen, whatever you have learned
  so far; it starts on the furthest skill you have met.
- **Sprint**: choose a skill and a step; you get as many items as you can
  answer in 60 seconds. Mistakes do not end it.
- **Lessons** end in three guided items (not recorded), then a 10-item drill.

## Difficulty and time pressure

Each skill's ladder (see [the curriculum](curriculum.md#ladders)) raises one
thing at a time: more operands, then more digits, then time pressure. Timed
steps have a time budget per operand: 2.0 times your typical time on a
"gentle" step, 1.3 times on a "tight" one, never below 1.2 s. The budget is
not a countdown on the screen; the scheduler uses it to decide when you are
fluent enough to go further: on a timed step you step up only when your
typical time is at most 80 % of the budget.

Every answer and its time goes into the history and the scheduler, except
in guided lesson practice.
