# Getting started

## Install

Every release on GitHub carries the same set of files. The commands below
use v0.2.1; the [releases page](https://github.com/ZahakJ/abax/releases/latest)
always has the newest version, and `SHA256SUMS.txt` next to the downloads
lets you check what you fetched (`sha256sum -c SHA256SUMS.txt --ignore-missing`).

| Platform | File |
|----------|------|
| Linux x86_64 | `abax-v0.2.1-linux-x86_64.tar.gz` |
| Windows x86_64 | `abax-v0.2.1-windows-x86_64.zip` |
| macOS, Apple silicon | `abax-v0.2.1-macos-aarch64.dmg` |
| macOS, Intel | `abax-v0.2.1-macos-x86_64.dmg` |
| Android, 64-bit ARM (almost every current phone) | `abax-v0.2.1-android-arm64-v8a.apk` |
| Android, universal (adds 32-bit ARM and x86_64) | `abax-v0.2.1-android-universal.apk` |

### Linux

```sh
curl -L https://github.com/ZahakJ/abax/releases/download/v0.2.1/abax-v0.2.1-linux-x86_64.tar.gz | tar xz
./abax-v0.2.1-linux-x86_64/install.sh
```

`install.sh` needs no root. It copies the binary to `~/.local/bin/abax`
(or `$XDG_BIN_HOME`), the desktop entry to `~/.local/share/applications/`
and the icon to `~/.local/share/icons/`, so Abax appears in your launcher.
You can also skip the installer and run `./abax-v0.2.1-linux-x86_64/abax`
directly. The binary needs a Vulkan or OpenGL driver, which every desktop
Linux has.

### Windows

In PowerShell:

```powershell
Invoke-WebRequest https://github.com/ZahakJ/abax/releases/download/v0.2.1/abax-v0.2.1-windows-x86_64.zip -OutFile abax.zip; Expand-Archive abax.zip -DestinationPath abax
```

Then run `abax\abax.exe`. The binary is not code-signed, so SmartScreen may
stop the first launch; choose "More info", then "Run anyway".

### macOS

Download the dmg for your Mac (Apple menu > About This Mac: "Apple M1" or
later means Apple silicon, "Intel" means Intel), open it and drag
`Abax.app` to Applications.

The app is ad-hoc signed but not notarized (there is no paid Apple
certificate), so the first launch says "Apple could not verify Abax".
Right-click the app > Open > Open; on macOS 15 and later, open it once, then
click Open Anyway in System Settings > Privacy & Security. Or run once in a
terminal:

```sh
xattr -cr /Applications/Abax.app
```

Version 0.2.0 was not signed as a bundle and macOS called it "damaged";
0.2.1 fixes that ([details](https://github.com/ZahakJ/abax/blob/main/docs/macos-gatekeeper.md)).

### Android

On the phone, open the
[arm64-v8a APK](https://github.com/ZahakJ/abax/releases/download/v0.2.1/abax-v0.2.1-android-arm64-v8a.apk)
link and install it. Android asks you to allow installing apps from your
browser the first time. Abax needs Android 7.0 (API 24) or newer. The
[universal APK](https://github.com/ZahakJ/abax/releases/download/v0.2.1/abax-v0.2.1-android-universal.apk)
also contains 32-bit ARM and x86_64 code; use it on an old 32-bit phone or
an emulator. This version has no sound on Android.

## Your first session

1. **Home.** The card at the top says what comes next. On a new profile that
   is the first lesson, "The rods, the beam and the beads". Press
   **Continue**.
2. **The lesson.** A few short paragraphs explain one idea. Below them the
   soroban plays demonstrations: the bead to move is ringed in vermilion,
   an arrow shows the direction, and a line of text names the finger
   ("Index finger: push the heaven bead down"). Use **Next move** to step
   through a demonstration by hand, **Replay** to see it again, and **Next
   demo** for the next example.
3. **Guided practice.** Press **Try it with hints**. You get three items to
   work on the beads yourself. If you pause for 1.5 s, the next move is
   shown. Nothing here counts towards your record.
4. **The drill.** Press **Start the drill** for ten items. Now the hint
   waits for the delay set in Settings (4 s by default) and every answer
   is recorded. When you are wrong, Abax does not just say so: the correct
   bead moves replay on the soroban, in order.
5. **Back home.** The card now offers the next lesson as soon as its
   prerequisites are known well enough, or a planned session: a short
   warm-up, a focus drill on your weakest skill and a cool-down. The
   [scheduler chapter](scheduler.md) explains how it chooses.

You can also leave the path at any time. The home screen has eight entries:

| Entry | What it does |
|-------|--------------|
| Practice | A free soroban. Tap or drag beads, or type digits. |
| Lessons | Every lesson, locked ones marked, to revisit any of them. |
| Drills | Pick a skill and a difficulty step yourself. |
| Anzan | A drill where the beads fade and you finish in your head. |
| Flash | Flash anzan with your own digits, count and speed. |
| Sprint | Sixty seconds, as many items as you can. |
| Stats | Momentum, per-skill mastery, accuracy and speed. |
| Settings | Rods, input, theme, finish, sound, data. |

## Working the soroban

**Touch and mouse.** Tap an earth bead and it moves together with the beads
between it and the beam, the way a finger pushes a group. Tap the heaven
bead to toggle it. You can also drag beads; on release they snap to the
nearer position. A bead's hit area is the full width of its rod, so you do
not need to aim at the bead itself.

**Keyboard.** Digits set the active rod and move one rod to the right;
Backspace clears the active rod and moves left; the Left and Right arrows
move the active rod in free practice; Enter or Space checks the answer;
Escape or Delete clears the soroban.

**Keypad.** Phones get an on-screen keypad under the thumb. It can be
turned off in Settings, and moved to the left for left-handed use.
