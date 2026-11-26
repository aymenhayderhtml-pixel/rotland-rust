use std::path::{Component, Path, PathBuf};
use serde::{Deserialize, Serialize};

use crate::error::SaveError;
use crate::file::{SaveFile, FORMAT_VERSION};

/// Three save slots per Godot's SaveSlots.Count = 3.
pub const DEFAULT_SLOT_COUNT: usize = 3;

/// Slot state for slot picker / info.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SlotState {
    Empty,
    Damaged,
    Filled,
}

/// Summary of a save slot for the picker.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SlotInfo {
    pub slot: usize,
    pub state: SlotState,
    pub seed: i32,
    pub day: i32,
    pub play_time_s: f32,
    pub difficulty: String,
    pub saved_at: String,
}

impl SlotInfo {
    pub fn empty(slot: usize) -> Self {
        Self {
            slot,
            state: SlotState::Empty,
            seed: 0,
            day: 0,
            play_time_s: 0.0,
            difficulty: String::new(),
            saved_at: String::new(),
        }
    }

    pub fn damaged(slot: usize) -> Self {
        Self {
            slot,
            state: SlotState::Damaged,
            seed: 0,
            day: 0,
            play_time_s: 0.0,
            difficulty: String::new(),
            saved_at: String::new(),
        }
    }
}

/// Normalizes a path by removing CurDir (.) and resolving ParentDir (..).
pub fn normalize_path(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for comp in path.components() {
        match comp {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            c => out.push(c.as_os_str()),
        }
    }
    out
}

/// Resolves symlinks in a path as far as possible (even for paths whose final file does not exist yet).
pub fn resolve_symlinks(path: &Path) -> PathBuf {
    if let Ok(c) = path.canonicalize() {
        return c;
    }
    let mut stack = Vec::new();
    let mut curr = path;
    while !curr.exists() {
        if let Some(name) = curr.file_name() {
            stack.push(name);
        }
        if let Some(parent) = curr.parent() {
            curr = parent;
        } else {
            break;
        }
    }
    let mut resolved = curr.canonicalize().unwrap_or_else(|_| normalize_path(curr));
    for comp in stack.into_iter().rev() {
        resolved.push(comp);
    }
    normalize_path(&resolved)
}

/// Returns true if `path` is equal to or inside `base`.
pub fn is_subpath_or_same(path: &Path, base: &Path) -> bool {
    let canon_path = resolve_symlinks(path);
    let canon_base = resolve_symlinks(base);
    if canon_path.starts_with(&canon_base) {
        return true;
    }
    let norm_path = normalize_path(path);
    let norm_base = normalize_path(base);
    if norm_path.starts_with(&norm_base) {
        return true;
    }
    let p_str = norm_path.to_string_lossy();
    let b_str = norm_base.to_string_lossy();
    if !b_str.is_empty() && (p_str == b_str || p_str.starts_with(&format!("{b_str}/"))) {
        return true;
    }
    false
}

/// The Rust game's own save directory `<XDG_DATA_HOME or ~/.local/share>/rotland-rust/saves`.
pub fn default_base_dir() -> PathBuf {
    if let Some(xdg) = std::env::var_os("XDG_DATA_HOME").filter(|s| !s.is_empty()) {
        PathBuf::from(xdg).join("rotland-rust").join("saves")
    } else if let Some(home) = std::env::var_os("HOME").filter(|s| !s.is_empty()) {
        PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("rotland-rust")
            .join("saves")
    } else {
        PathBuf::from(".local")
            .join("share")
            .join("rotland-rust")
            .join("saves")
    }
}

/// The Godot game's save directory `~/.local/share/godot/app_userdata/Rotland/saves`.
pub fn godot_save_dir() -> PathBuf {
    if let Some(home) = std::env::var_os("HOME").filter(|s| !s.is_empty()) {
        PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("godot")
            .join("app_userdata")
            .join("Rotland")
            .join("saves")
    } else {
        PathBuf::from(".local")
            .join("share")
            .join("godot")
            .join("app_userdata")
            .join("Rotland")
            .join("saves")
    }
}

/// Guard: returns true if the path is protected against write/delete.
/// Refuses any path under `godot_save_dir()`; in `cfg(test)` also refuses `default_base_dir()`.
pub fn is_protected_path(path: &Path) -> bool {
    let godot = godot_save_dir();
    if is_subpath_or_same(path, &godot) {
        return true;
    }
    #[cfg(test)]
    {
        let default_dir = default_base_dir();
        if is_subpath_or_same(path, &default_dir) {
            return true;
        }
    }
    false
}

/// Manages save slots in a directory.
#[derive(Clone, Debug, PartialEq)]
pub struct Slots {
    pub base_dir: PathBuf,
}

impl Default for Slots {
    fn default() -> Self {
        Self {
            base_dir: default_base_dir(),
        }
    }
}

impl Slots {
    pub const COUNT: usize = DEFAULT_SLOT_COUNT;

    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
        }
    }

    pub fn path_for(&self, slot: usize) -> Result<PathBuf, SaveError> {
        if !(1..=Self::COUNT).contains(&slot) {
            return Err(SaveError::InvalidSlot(slot));
        }
        Ok(self.base_dir.join(format!("slot{slot}.json")))
    }

    fn check_guard(&self, path: &Path) -> Result<(), SaveError> {
        if is_protected_path(path) {
            return Err(SaveError::ProtectedPath(path.to_path_buf()));
        }
        Ok(())
    }

    /// Slot summary info matching `SaveSlots.Info`.
    pub fn info(&self, slot: usize) -> SlotInfo {
        let path = match self.path_for(slot) {
            Ok(p) => p,
            Err(_) => return SlotInfo::damaged(slot),
        };
        if !path.is_file() {
            return SlotInfo::empty(slot);
        }
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return SlotInfo::damaged(slot),
        };
        let save = match SaveFile::from_json_str(&content) {
            Ok(s) => s,
            Err(_) => return SlotInfo::damaged(slot),
        };
        if save.version != FORMAT_VERSION {
            return SlotInfo::damaged(slot);
        }

        let seed = save.systems.world.as_ref().map(|w| w.seed).unwrap_or(0);
        let (day, play_time_s) = save
            .systems
            .stats
            .as_ref()
            .map(|s| (s.highest_day, s.play_time as f32))
            .unwrap_or((0, 0.0));
        let difficulty = save
            .systems
            .difficulty
            .as_ref()
            .map(|d| d.label())
            .unwrap_or("Normal")
            .to_string();
        let saved_at = save.saved_at_utc;

        SlotInfo {
            slot,
            state: SlotState::Filled,
            seed,
            day,
            play_time_s,
            difficulty,
            saved_at,
        }
    }

    /// Reads and parses the save file at `slot`.
    pub fn read(&self, slot: usize) -> Result<SaveFile, SaveError> {
        let path = self.path_for(slot)?;
        if !path.is_file() {
            return Err(SaveError::SlotNotFound(slot));
        }
        let content = std::fs::read_to_string(&path)?;
        SaveFile::from_json_str(&content)
    }

    /// Writes `save` to `slot` using a temp file then rename (matching `SaveFile.Write`).
    pub fn write(&self, slot: usize, save: &SaveFile) -> Result<(), SaveError> {
        let path = self.path_for(slot)?;
        self.check_guard(&path)?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let temp_path = PathBuf::from(format!("{}.tmp", path.display()));
        let json = save.to_json_pretty()?;

        if let Err(e) = std::fs::write(&temp_path, &json) {
            let _ = std::fs::remove_file(&temp_path);
            return Err(SaveError::Io(e));
        }

        if let Err(e) = std::fs::rename(&temp_path, &path) {
            let _ = std::fs::remove_file(&temp_path);
            return Err(SaveError::Io(e));
        }

        Ok(())
    }

    /// Deletes the save file for `slot`.
    pub fn delete(&self, slot: usize) -> Result<(), SaveError> {
        let path = self.path_for(slot)?;
        self.check_guard(&path)?;

        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        let temp_path = PathBuf::from(format!("{}.tmp", path.display()));
        if temp_path.exists() {
            let _ = std::fs::remove_file(&temp_path);
        }
        Ok(())
    }

    /// Imports a Godot save slot into this Rust slot by READING the Godot file only.
    pub fn import_godot(&self, slot: usize) -> Result<SaveFile, SaveError> {
        import_godot(slot, self)
    }

    /// Imports from a custom Godot directory (useful for testing).
    pub fn import_godot_from(&self, slot: usize, godot_dir: &Path) -> Result<SaveFile, SaveError> {
        import_godot_from(slot, godot_dir, self)
    }
}

/// Copies a Godot slot into a Rust slot by READING the Godot file only.
/// Never writes, renames or deletes anything under the Godot folder.
pub fn import_godot(slot: usize, dest: &Slots) -> Result<SaveFile, SaveError> {
    import_godot_from(slot, &godot_save_dir(), dest)
}

/// Copies a slot from `godot_dir` into `dest` by READING only.
pub fn import_godot_from(slot: usize, godot_dir: &Path, dest: &Slots) -> Result<SaveFile, SaveError> {
    if !(1..=Slots::COUNT).contains(&slot) {
        return Err(SaveError::InvalidSlot(slot));
    }
    let src_path = godot_dir.join(format!("slot{slot}.json"));
    if !src_path.is_file() {
        return Err(SaveError::SlotNotFound(slot));
    }
    let content = std::fs::read_to_string(&src_path)?;
    let save_file = SaveFile::from_json_str(&content)?;
    dest.write(slot, &save_file)?;
    Ok(save_file)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Unit tests must never write the Rust game's real save folder either.
    #[test]
    fn guard_refuses_real_rust_folder_in_tests() {
        let slots = Slots::new(default_base_dir());
        assert!(matches!(slots.write(1, &SaveFile::default()), Err(SaveError::ProtectedPath(_))));
        assert!(matches!(slots.delete(1), Err(SaveError::ProtectedPath(_))));
    }

    #[test]
    fn slot_4_and_invalid_slots_rejected() {
        let temp_dir = std::env::temp_dir().join("rotland_test_slot4_reject");
        let slots = Slots::new(&temp_dir);
        assert!(matches!(slots.path_for(0), Err(SaveError::InvalidSlot(0))));
        assert!(matches!(slots.path_for(4), Err(SaveError::InvalidSlot(4))));
        assert!(matches!(slots.path_for(99), Err(SaveError::InvalidSlot(99))));
        assert!(matches!(slots.read(4), Err(SaveError::InvalidSlot(4))));
        assert!(matches!(slots.write(4, &SaveFile::default()), Err(SaveError::InvalidSlot(4))));
        assert!(matches!(slots.delete(4), Err(SaveError::InvalidSlot(4))));
        assert_eq!(slots.info(4).state, SlotState::Damaged);
    }

    #[test]
    fn guard_refuses_symlink_to_protected_dir() {
        let temp_base = std::env::temp_dir().join(format!("rotland_symlink_test_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_base);
        let link_path = temp_base.join("symlink_to_godot");
        let _ = std::fs::remove_file(&link_path);
        #[cfg(unix)]
        {
            if std::os::unix::fs::symlink(godot_save_dir(), &link_path).is_ok() {
                let slots = Slots::new(&link_path);
                assert!(matches!(slots.write(1, &SaveFile::default()), Err(SaveError::ProtectedPath(_))));
                assert!(matches!(slots.delete(1), Err(SaveError::ProtectedPath(_))));
                let _ = std::fs::remove_file(&link_path);
            }
        }
        let _ = std::fs::remove_dir_all(&temp_base);
    }
}
