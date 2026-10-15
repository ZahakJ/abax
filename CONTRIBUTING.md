# Contributing

Open an issue before a large change. Small fixes can go straight to a pull
request.

```sh
./verify                     # fmt check, clippy -D warnings, tests, release build
cargo run --release -p abax
```

CI runs `./verify` on every push. Android: see
[docs/manual/building.md](docs/manual/building.md). Layout of the code:
[docs/manual/architecture.md](docs/manual/architecture.md). Palette, motion
and interaction: [docs/internals.md](docs/internals.md).

## Pull requests

- One topic per pull request.
- `./verify` must pass.
- New logic in `abax-core` needs unit tests; bead-move changes must keep the
  full addition and subtraction tables green.
- Western digits in every string.
- No captures outside `docs/`, no keystores.

## Bugs

Use the bug report template: platform, version (Settings > About), steps. If
the save file is involved, say which version wrote it; the format is in
[docs/manual/save-file.md](docs/manual/save-file.md).

## Curriculum corrections

Cite the source (Kojima, Heffelfinger, [research/](research/)) and the rule
or table entry that is wrong. A failing test with the correct move list is
the best report.
