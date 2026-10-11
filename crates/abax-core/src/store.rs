//! The save file: settings, scheduler state and history in one versioned
//! JSON document. Unknown fields are ignored and missing fields take their
//! defaults, so older saves keep loading as the format grows.

use serde::{Deserialize, Serialize};

use crate::scheduler::Scheduler;
use crate::stats::History;

/// Format version written by this build.
pub const CURRENT_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputMode {
    Tap,
    Drag,
    #[default]
    Both,
}

/// User preferences. Every field falls back to its default when missing.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub rod_count: usize,
    pub theme: Theme,
    pub sound: bool,
    pub volume: f32,
    pub left_handed: bool,
    pub show_numbers: bool,
    pub show_keypad: bool,
    pub input_mode: InputMode,
    pub reduce_motion: bool,
    pub large_text: bool,
    pub hint_delay_seconds: f32,
    pub flash_digits: u8,
    pub flash_count: u8,
    pub flash_ms: u16,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            rod_count: 13,
            theme: Theme::System,
            sound: false,
            volume: 0.6,
            left_handed: false,
            show_numbers: true,
            show_keypad: true,
            input_mode: InputMode::Both,
            reduce_motion: false,
            large_text: false,
            hint_delay_seconds: 4.0,
            flash_digits: 1,
            flash_count: 5,
            flash_ms: 1000,
        }
    }
}

/// Everything persisted for one learner.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SaveFile {
    /// Missing version reads as 0, which is rejected as unsupported.
    #[serde(default)]
    pub version: u32,
    pub settings: Settings,
    pub scheduler: Scheduler,
    pub history: History,
    /// Unix seconds the profile was created.
    pub created: u64,
    /// Unix seconds of the last save.
    pub updated: u64,
}

impl Default for SaveFile {
    fn default() -> Self {
        SaveFile {
            version: CURRENT_VERSION,
            settings: Settings::default(),
            scheduler: Scheduler::default(),
            history: History::default(),
            created: 0,
            updated: 0,
        }
    }
}

/// Why a save could not be loaded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreError {
    /// The text is not a valid save document.
    Parse(String),
    /// The document claims a version this build cannot read.
    UnsupportedVersion(u32),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::Parse(msg) => write!(f, "cannot parse save file: {msg}"),
            StoreError::UnsupportedVersion(v) => {
                write!(
                    f,
                    "unsupported save file version {v} (this build reads up to {CURRENT_VERSION})"
                )
            }
        }
    }
}

impl std::error::Error for StoreError {}

impl SaveFile {
    /// A fresh profile created at `now`.
    pub fn new(now: u64) -> Self {
        SaveFile {
            created: now,
            updated: now,
            ..SaveFile::default()
        }
    }

    pub fn to_json(&self) -> String {
        // Serialising plain data cannot fail.
        serde_json::to_string(self).unwrap_or_default()
    }

    /// Parse and migrate a save document.
    pub fn from_json(text: &str) -> Result<Self, StoreError> {
        let save: SaveFile =
            serde_json::from_str(text).map_err(|e| StoreError::Parse(e.to_string()))?;
        if save.version == 0 || save.version > CURRENT_VERSION {
            return Err(StoreError::UnsupportedVersion(save.version));
        }
        Ok(save.migrate())
    }

    /// Bring an older document up to the current format. Version 1 only
    /// needs the curriculum filled in; later versions chain their steps here.
    pub fn migrate(mut self) -> Self {
        if self.version <= 1 {
            self.scheduler.fill_missing();
        }
        self.version = CURRENT_VERSION;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::curriculum::SkillId;
    use crate::stats::{Attempt, DrillKind};

    #[test]
    fn round_trip() {
        let mut save = SaveFile::new(1_700_000_000);
        save.settings.left_handed = true;
        save.settings.volume = 0.25;
        save.scheduler
            .record(SkillId::Setting, true, 2.5, 1_700_000_100);
        save.history.push(Attempt {
            ts: 1_700_000_100,
            skill: SkillId::Setting,
            level: 0,
            kind: DrillKind::Set,
            correct: true,
            seconds: 2.5,
        });
        save.updated = 1_700_000_100;
        let json = save.to_json();
        let back = SaveFile::from_json(&json).unwrap();
        assert_eq!(back, save);
        assert_eq!(back.version, CURRENT_VERSION);
    }

    #[test]
    fn unknown_fields_are_ignored() {
        let json =
            r#"{"version":1,"settings":{"rod_count":9,"future_toggle":true},"mystery":[1,2,3]}"#;
        let save = SaveFile::from_json(json).unwrap();
        assert_eq!(save.settings.rod_count, 9);
        assert_eq!(save.settings.theme, Theme::System);
    }

    #[test]
    fn missing_fields_take_defaults() {
        let save = SaveFile::from_json(r#"{"version":1}"#).unwrap();
        assert_eq!(save.settings, Settings::default());
        assert_eq!(save.scheduler, Scheduler::default());
        assert!(save.history.is_empty());
        assert_eq!(save.created, 0);

        let settings: Settings = serde_json::from_str(r#"{"sound":true}"#).unwrap();
        assert!(settings.sound);
        assert_eq!(settings.rod_count, 13);
        assert_eq!(settings.input_mode, InputMode::Both);
        assert_eq!(settings.flash_ms, 1000);
    }

    #[test]
    fn partial_scheduler_is_filled_in() {
        let json = r#"{"version":1,"scheduler":{"skills":{"Setting":{"p_known":0.9,"n":3}}}}"#;
        let save = SaveFile::from_json(json).unwrap();
        let st = save.scheduler.state(SkillId::Setting);
        assert_eq!(st.n, 3);
        assert!((st.p_known - 0.9).abs() < 1e-12);
        assert_eq!(save.scheduler.states().count(), SkillId::ALL.len());
        assert!(save.scheduler.is_unlocked(SkillId::AddSimple));
    }

    #[test]
    fn bad_input_is_an_error_not_a_panic() {
        assert!(matches!(
            SaveFile::from_json("{\"version\":0}"),
            Err(StoreError::UnsupportedVersion(0))
        ));
        assert!(matches!(
            SaveFile::from_json("{}"),
            Err(StoreError::UnsupportedVersion(0))
        ));
        assert!(matches!(
            SaveFile::from_json("{\"version\":99}"),
            Err(StoreError::UnsupportedVersion(99))
        ));
        assert!(matches!(
            SaveFile::from_json("not json"),
            Err(StoreError::Parse(_))
        ));
        assert!(matches!(SaveFile::from_json(""), Err(StoreError::Parse(_))));
        assert!(matches!(
            SaveFile::from_json(r#"{"version":1,"settings":{"rod_count":"thirteen"}}"#),
            Err(StoreError::Parse(_))
        ));
        let err = SaveFile::from_json("{}").unwrap_err();
        assert!(err.to_string().contains("version 0"));
    }
}
