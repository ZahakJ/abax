# Abax

Abax is a soroban trainer. The soroban is the Japanese abacus: a frame of
rods, each carrying one bead worth 5 above a beam and four beads worth 1
below it. Abax teaches it the way soroban schools do. First you learn which
finger moves which bead, and in what order, for every kind of addition and
subtraction. Then you drill those moves until they are quick. Then the beads
fade, and you keep calculating on a soroban you only picture.

<picture>
  <source srcset="../screenshots/lesson-dark.png" media="(prefers-color-scheme: dark)">
  <img src="../screenshots/lesson-light.png" alt="A lesson in Abax: the heaven bead to move is ringed in vermilion, with an arrow and the instruction 'Index finger: push the heaven bead down'.">
</picture>

It runs as one native program on Linux, Windows and macOS, and as an APK on
Android 7.0 or newer. It is free, open source (MIT), works offline, needs no
account, and never makes a network request.

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
Elo-style ability estimate. Two research notes in the repository collect the
sources and the reasoning:

- [`research/soroban-mechanics-and-curriculum.md`](https://github.com/ZahakJ/abax/blob/main/research/soroban-mechanics-and-curriculum.md)
- [`research/trainer-design.md`](https://github.com/ZahakJ/abax/blob/main/research/trainer-design.md)

The project lives at <https://github.com/ZahakJ/abax>; the landing page is
<https://zahakj.github.io/abax/>.
