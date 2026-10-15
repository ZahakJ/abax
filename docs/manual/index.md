# Abax

Abax is a soroban trainer. The soroban is the Japanese abacus: a frame of
rods, each carrying one bead worth 5 above a beam and four beads worth 1
below it. Abax teaches which finger moves which bead, and in what order,
for every kind of addition and subtraction; drills those moves; then fades
the beads for mental calculation.

<picture>
  <source srcset="../screenshots/lesson-dark.png" media="(prefers-color-scheme: dark)">
  <img src="../screenshots/lesson-light.png" alt="A lesson in Abax: the heaven bead to move is ringed in vermilion, with an arrow and the instruction 'Index finger: push the heaven bead down'.">
</picture>

Linux, Windows, macOS, and Android 7.0 or newer. MIT licence.

## What is in this manual

- **Using Abax**: installing it, how the soroban works, the finger rules, the
  fifteen skills of the curriculum, the drills, anzan, how the scheduler
  decides what you practise next, every setting, and the save file.
- **Under the hood**: building from source (including the Android APK
  without Gradle), how the soroban is drawn on the GPU, and where things live
  in the code.

## Where the content comes from

The bead moves, the finger order and the multiplication and division
methods follow Takashi Kojima, *The Japanese Abacus: Its Use and Theory*
(Tuttle, 1954). The grading ladders and the flash anzan speeds were checked
against the Japanese federations' published exam tables. The scheduler is
built on Bayesian Knowledge Tracing (Corbett and Anderson, 1995) and an
Elo-style ability estimate. Sources and the full bead-move tables:
[`research/soroban-mechanics-and-curriculum.md`](https://github.com/ZahakJ/abax/blob/main/research/soroban-mechanics-and-curriculum.md).

Source: <https://github.com/ZahakJ/abax>.
