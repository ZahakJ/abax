# Getting started

## Install

The commands below use v0.3.0; the newest version is on the
[releases page](https://github.com/ZahakJ/abax/releases/latest). Check a
download with `sha256sum -c SHA256SUMS.txt --ignore-missing`.

| Platform | File |
|----------|------|
| Linux x86_64 | `abax-v0.3.0-linux-x86_64.tar.gz` |
| Windows x86_64 | `abax-v0.3.0-windows-x86_64.zip` |
| macOS, Apple silicon | `abax-v0.3.0-macos-aarch64.dmg` |
| macOS, Intel | `abax-v0.3.0-macos-x86_64.dmg` |
| Android, 64-bit ARM | `abax-v0.3.0-android-arm64-v8a.apk` |
| Android, universal (adds 32-bit ARM and x86_64) | `abax-v0.3.0-android-universal.apk` |

### Linux

```sh
curl -L https://github.com/ZahakJ/abax/releases/download/v0.3.0/abax-v0.3.0-linux-x86_64.tar.gz | tar xz
./abax-v0.3.0-linux-x86_64/install.sh
```

`install.sh` needs no root. It copies the binary to `~/.local/bin/abax`
(or `$XDG_BIN_HOME`), the desktop entry to `~/.local/share/applications/`
and the icon to `~/.local/share/icons/`. The binary also runs in place,
`./abax-v0.3.0-linux-x86_64/abax`. It needs a Vulkan or OpenGL driver.

### Windows

In PowerShell:

```powershell
Invoke-WebRequest https://github.com/ZahakJ/abax/releases/download/v0.3.0/abax-v0.3.0-windows-x86_64.zip -OutFile abax.zip; Expand-Archive abax.zip -DestinationPath abax
```

Then run `abax\abax.exe`. The binary is not code-signed, so SmartScreen may
stop the first launch; choose "More info", then "Run anyway".

### macOS

Download the dmg for your Mac (Apple menu > About This Mac: "Apple M1" or
later means Apple silicon, "Intel" means Intel), open it and drag
`Abax.app` to Applications.

The app is ad-hoc signed but not notarized, so the first launch says
"Apple could not verify Abax".
Right-click the app > Open > Open; on macOS 15 and later, open it once, then
click Open Anyway in System Settings > Privacy & Security. Or run once in a
terminal:

```sh
xattr -cr /Applications/Abax.app
```

### Android

On the phone, open the
[arm64-v8a APK](https://github.com/ZahakJ/abax/releases/download/v0.3.0/abax-v0.3.0-android-arm64-v8a.apk)
link and install it, allowing installs from the browser when asked.
Abax needs Android 7.0 (API 24) or newer. The
[universal APK](https://github.com/ZahakJ/abax/releases/download/v0.3.0/abax-v0.3.0-android-universal.apk)
also contains 32-bit ARM and x86_64 code; use it on an old 32-bit phone or
an emulator. There is no sound on Android.

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
   is recorded. A wrong answer replays the correct bead moves, in order.
5. **Back home.** The card now offers the suggested next lesson as soon as
   its prerequisites are known well enough, or a planned session: a short
   warm-up, a focus drill on your weakest skill and a cool-down. The
   [scheduler chapter](scheduler.md) explains how it chooses.

Nothing is locked. Open any lesson in any order, and tap **I know this
already** on the ones you can skip
([how that works](curriculum.md#skipping-what-you-know)). The home screen
has eight entries:

| Entry | What it does |
|-------|--------------|
| Practice | A free soroban. Tap or drag beads, or type digits. |
| Lessons | Every lesson, all open, with the suggested next one marked. |
| Drills | Pick a skill and a difficulty step yourself. |
| Anzan | A drill where the beads fade and you finish in your head. |
| Flash | Flash anzan with your own digits, count and speed. |
| Sprint | Sixty seconds, as many items as you can. |
| Stats | Momentum, per-skill mastery, accuracy and speed. |
| Settings | Rods, input, theme, finish, sound, data. |

## Working the soroban

**Touch and mouse.** Tap an earth bead and it moves together with the beads
between it and the beam. Tap the heaven bead to toggle it. Dragged beads
snap to the nearer position on release. A bead's hit area is the full width
of its rod.

**Keyboard.** Digits set the active rod and move one rod to the right;
Backspace clears the active rod and moves left; the Left and Right arrows
move the active rod in free practice; Enter or Space checks the answer;
Escape or Delete clears the soroban.

**Keypad.** An on-screen keypad sits under the soroban. Settings can turn
it off or move it to the left.
