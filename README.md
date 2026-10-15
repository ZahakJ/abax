<p align="center"><img src="docs/media/banner.png" alt="Abax" width="820"></p>

# Abax

Soroban (Japanese abacus) trainer for Linux, Windows, macOS and Android.
Lessons show each bead move with the finger that makes it, then adaptive
drills, anzan and flash anzan.

<https://zahakj.github.io/abax/>

![Practice](docs/screenshots/hero.png)

## Install

Linux (x86_64, installs to `~/.local/bin`):

```sh
curl -L https://github.com/ZahakJ/abax/releases/download/v0.3.0/abax-v0.3.0-linux-x86_64.tar.gz | tar xz && ./abax-v0.3.0-linux-x86_64/install.sh
```

Windows, macOS and Android (7.0 or newer) builds are on the
[releases page](https://github.com/ZahakJ/abax/releases/latest), with
`SHA256SUMS.txt`. The macOS app is not notarized: right-click > Open on first
launch, or run `xattr -cr /Applications/Abax.app`
([details](docs/macos-gatekeeper.md)).

From source:

```sh
cargo run --release -p abax
```

## Usage

| Skill | |
| --- | --- |
| S0 | read and set numbers |
| S1, S2 | add, subtract without complements |
| S3, S4 | add, subtract with the five-complement |
| S5, S6 | add, subtract with the ten-complement |
| S7 | combined complements |
| S8, S9 | multi-digit addition, mixed chains |
| S10, S11 | multiplication, division |
| A1 | anzan with fading beads |
| A2, A3 | flash anzan |

Every skill is open; prerequisites only suggest an order. *Continue* on the
home screen runs a short session chosen by the scheduler. Beads take taps,
drags or the keyboard.

Bead moves and finger order follow Kojima, *The Japanese Abacus: Its Use and
Theory* (1954). The scheduler is Bayesian Knowledge Tracing with an Elo-style
difficulty estimate. Manual: <https://zahakj.github.io/abax/manual/>, source
in [docs/manual/](docs/manual/). Sources and derivations:
[research/](research/).

## Build

```sh
./verify                 # fmt, clippy -D warnings, tests, release build
android/build-apk.sh     # APK without Gradle
```

See [docs/manual/building.md](docs/manual/building.md).

## License

MIT
