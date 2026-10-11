//! Where the profile lives and how it is written (atomically).

use abax_core::store::SaveFile;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static OVERRIDE_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Set by `android_main` (the activity's internal data path) or by tests.
pub fn set_data_dir(p: PathBuf) {
    let _ = OVERRIDE_DIR.set(p);
}

pub fn data_dir() -> PathBuf {
    if let Some(p) = OVERRIDE_DIR.get() {
        return p.clone();
    }
    if let Ok(p) = std::env::var("ABAX_DATA_DIR") {
        return PathBuf::from(p);
    }
    #[cfg(not(target_os = "android"))]
    {
        if let Some(dirs) = directories::ProjectDirs::from("", "", "abax") {
            return dirs.data_dir().to_path_buf();
        }
    }
    std::env::temp_dir().join("abax")
}

pub fn profile_path() -> PathBuf {
    data_dir().join("profile.json")
}

pub fn load() -> SaveFile {
    load_from(&profile_path())
}

pub fn load_from(path: &Path) -> SaveFile {
    match std::fs::read_to_string(path) {
        Ok(text) => match SaveFile::from_json(&text) {
            Ok(s) => s,
            Err(e) => {
                log::warn!("profile unreadable ({e}); starting fresh and keeping a backup");
                let _ = std::fs::copy(path, path.with_extension("json.bak"));
                SaveFile::new(now_unix())
            }
        },
        Err(_) => SaveFile::new(now_unix()),
    }
}

pub fn save(file: &SaveFile) -> std::io::Result<()> {
    save_to(&profile_path(), file)
}

pub fn save_to(path: &Path, file: &SaveFile) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("json.tmp");
    {
        use std::io::Write;
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(file.to_json().as_bytes())?;
        f.sync_all()?;
    }
    std::fs::rename(&tmp, path)?;
    Ok(())
}

pub fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_through_disk() {
        let dir = std::env::temp_dir().join(format!("abax-test-{}", std::process::id()));
        let path = dir.join("profile.json");
        let mut f = SaveFile::new(1000);
        f.settings.rod_count = 21;
        save_to(&path, &f).unwrap();
        let back = load_from(&path);
        assert_eq!(back.settings.rod_count, 21);
        assert!(!path.with_extension("json.tmp").exists());
        std::fs::write(&path, "garbage").unwrap();
        let fresh = load_from(&path);
        assert_eq!(fresh.settings.rod_count, 13);
        assert!(path.with_extension("json.bak").exists());
        let _ = std::fs::remove_dir_all(dir);
    }
}
