# The save file

Settings, scheduler state and history are one JSON file, `profile.json`.

## Where it is

| Platform | Directory |
|----------|-----------|
| Linux | `~/.local/share/abax/` (or `$XDG_DATA_HOME/abax/`) |
| macOS | `~/Library/Application Support/abax/` |
| Windows | `%APPDATA%\abax\data\` |
| Android | the app's internal storage (not visible without root) |

Settings > Data shows the exact path. Set the environment variable
`ABAX_DATA_DIR` to use another directory on the desktop:

```sh
ABAX_DATA_DIR=~/abax-test abax     # a separate, throwaway profile
```

To move your progress to another computer, copy `profile.json` into the same
place there while Abax is closed.

## How it is written

Abax writes the whole document to `profile.json.tmp`, flushes it to disk
(`fsync`), and renames it over `profile.json`. The rename is atomic: the
file is either the old version or the new one.

If the file cannot be read (damaged, or written by a newer Abax), Abax
copies it to `profile.json.bak`, starts a fresh profile and logs a warning.

## Versioning

The document carries a `version` field. This build writes version 1 and
reads versions up to 1; a missing or newer version is rejected. Two rules
let the format grow:

- **Unknown fields are ignored**, so an older Abax can open a file a newer
  one wrote, as long as the version number did not change.
- **Missing fields take their defaults**, so a newer Abax can open an older
  file. The `finish` setting was added in 0.2.0; a 0.1.0 profile has no such
  field and loads as walnut. 0.3.0 added `marked_known` to each skill and
  `builds_on_noted` at the top level; a 0.2 profile loads with every skill
  unmarked and no notes shown. Both changes are additive, so the version
  stays 1 and 0.2.x can still open a 0.3.0 file (it ignores the two fields).

When the curriculum gains a skill, loading fills in a fresh state for it.
When a change needs real conversion, the version number goes up and
`SaveFile::migrate` in `crates/abax-core/src/store.rs` gets a step for it.

## The schema, annotated

The file is written on one line. Formatted and commented, after one
answered item, with one of the fifteen skill entries:

```jsonc
{
  "version": 1,                       // format version
  "settings": {
    "rod_count": 13,                  // 9, 13, 17, 21, 23 or 27
    "theme": "System",                // "System" | "Light" | "Dark"
    "finish": "Walnut",               // "Walnut" | "Ebony" | "Maple" (0.2.0)
    "sound": false,
    "volume": 0.6,                    // 0..1
    "left_handed": false,
    "show_numbers": true,
    "show_keypad": true,
    "input_mode": "Both",             // "Tap" | "Drag" | "Both"
    "reduce_motion": false,
    "large_text": false,
    "hint_delay_seconds": 4.0,
    "flash_digits": 1,                // free flash anzan defaults
    "flash_count": 5,
    "flash_ms": 1000
  },
  "scheduler": {
    "skills": {                       // one entry per skill, keyed by name
      "Setting": {                    // S0; others: AddSimple, SubSimple, AddFive,
                                      // SubFive, AddTen, SubTen, Combined, MultiAdd,
                                      // MultiMixed, Multiply, Divide, Anzan,
                                      // FlashOne, FlashMulti
        "p_known": 0.4,               // BKT probability the skill is known
        "acc_ewma": 0.6,              // accuracy average
        "log_t_ewma": 1.8007,         // average ln(seconds per operand)
        "theta": 0.1824,              // Elo ability on this skill
        "n": 1,                       // answers so far
        "level": 0,                   // current ladder step, 0-based
        "last_practised": 1760000060, // unix seconds; 0 = never
        "mastered_at": null,          // unix seconds, or null
        "review_interval_days": 1.0,
        "consecutive_errors": 0,      // on the current step
        "items_at_level": 1,          // since the last step change
        "budget_seconds": 0.0,        // time budget per operand on a timed step
        "marked_known": false         // "I know this already" (0.3.0)
      }
    }
  },
  "history": {
    "attempts": [                     // oldest first, at most 50,000 kept
      {
        "ts": 1760000060,             // unix seconds
        "skill": "Setting",
        "level": 0,                   // ladder step the item came from
        "kind": "Set",                // Set | Read | AddSub | Multiply | Divide |
                                      // Mixed | Sprint | Anzan | Flash | Lesson
        "correct": true,
        "seconds": 2.4                // time for the whole item
      }
    ]
  },
  "created": 1760000000,              // unix seconds the profile was created
  "updated": 1760000060,              // unix seconds of the last save
  "builds_on_noted": []               // lessons whose "builds on" note was
                                      // shown once, e.g. ["Divide"] (0.3.0)
}
```

The [scheduler chapter](scheduler.md) explains every field under
`scheduler`. Everything on the stats screen (momentum, mastery bars,
accuracy and speed curves, the session log) is computed from `history` when
it is shown; none of it is stored separately.
