# Contributing to Abax

Thank you for taking the time. Abax is small and deliberately so; the easiest contributions to land are bug reports with a reproduction, corrections to the curriculum against the cited sources, and focused pull requests.

## Before you start

- Read [DESIGN.md](DESIGN.md) (the identity and interaction model) and [PLAN.md](PLAN.md) (why things are built the way they are). The research notes under [research/](research/) are the authority for bead moves, finger order and the scheduler.
- Open an issue before a large change so we can agree on the direction. Small fixes can go straight to a pull request.

## Building

You need a stable Rust toolchain.

```sh
./verify                     # fmt check, clippy -D warnings, tests, release build
cargo run --release -p abax  # run the desktop app
```

`./verify` is the release gate; CI runs the same script on every push. The Android build is documented in the manual (Building from source).

## Pull requests

- One topic per pull request, with a short description of the why.
- `./verify` must pass. Clippy warnings are errors.
- New logic in `abax-core` needs unit tests; bead-move changes must keep the full addition and subtraction tables green.
- Keep the identity: calm, physical, no neon, no gamification. Western digits in every string, including Arabic ones.
- Do not commit captures outside `docs/`, keystores, or personal paths.

## Reporting bugs

Use the bug report template. Include the platform, the Abax version (Settings > About), and the steps. If the save file is involved, say which version wrote it; the format is documented in the manual.

## Curriculum corrections

Cite the source (Kojima, Heffelfinger, the research notes) and say which rule or table entry is wrong. A failing test that encodes the correct move list is the best possible report.
