//! Versioned local application preferences, independent of documents and GUI widgets.
use crate::Result;
use serde::{Deserialize, Serialize};
use std::{
    ffi::OsString,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

const FORMAT: &str = "breakfree.docs.settings";
const VERSION: u32 = 1;
const MAX_BYTES: u64 = 64 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuickCommand {
    New,
    Open,
    Save,
    Undo,
    Redo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Placement {
    Above,
    Below,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Toolbar {
    pub commands: Vec<QuickCommand>,
    pub placement: Placement,
}
impl Default for Toolbar {
    fn default() -> Self {
        Self {
            commands: vec![QuickCommand::Save, QuickCommand::Undo, QuickCommand::Redo],
            placement: Placement::Above,
        }
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Settings {
    format: String,
    settings_version: u32,
    quick_access: Toolbar,
}

fn validate(toolbar: &Toolbar) -> Result<()> {
    if toolbar.commands.len() > 5
        || toolbar
            .commands
            .iter()
            .enumerate()
            .any(|(i, c)| toolbar.commands[..i].contains(c))
    {
        return Err("Toolbar commands must be unique and supported.".into());
    }
    Ok(())
}
fn decode(bytes: &[u8]) -> Result<Toolbar> {
    let settings: Settings =
        serde_json::from_slice(bytes).map_err(|e| format!("Invalid toolbar settings: {e}"))?;
    if settings.format != FORMAT {
        return Err("This is not a BreakFree Docs settings file.".into());
    }
    if settings.settings_version != VERSION {
        return Err(format!(
            "Unsupported settings version {}.",
            settings.settings_version
        ));
    }
    validate(&settings.quick_access)?;
    Ok(settings.quick_access)
}
fn encode(toolbar: &Toolbar) -> Result<Vec<u8>> {
    validate(toolbar)?;
    let mut bytes = serde_json::to_vec_pretty(&Settings {
        format: FORMAT.into(),
        settings_version: VERSION,
        quick_access: toolbar.clone(),
    })
    .map_err(|e| e.to_string())?;
    bytes.push(b'\n');
    Ok(bytes)
}

/// Resolving a path does not create directories. Invalid overrides never use cwd.
pub fn default_path() -> Result<PathBuf> {
    resolve_path(std::env::consts::OS, |name| std::env::var_os(name))
}
fn resolve_path(os: &str, env: impl Fn(&str) -> Option<OsString>) -> Result<PathBuf> {
    let absolute = |value: Option<OsString>, name: &str| -> Result<PathBuf> {
        let path = value
            .map(PathBuf::from)
            .ok_or_else(|| format!("{name} is not set."))?;
        if !path.is_absolute() {
            return Err(format!("{name} must be an absolute directory."));
        }
        Ok(path)
    };
    if let Some(value) = env("BREAKFREE_DOCS_CONFIG_DIR") {
        return Ok(absolute(Some(value), "BREAKFREE_DOCS_CONFIG_DIR")?.join("settings.json"));
    }
    let dir = match os {
        "windows" => absolute(env("APPDATA"), "APPDATA")?.join("BreakFree/Docs"),
        "macos" => {
            absolute(env("HOME"), "HOME")?.join("Library/Application Support/BreakFree/Docs")
        }
        _ => {
            let base = env("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .filter(|p| p.is_absolute());
            match base {
                Some(p) => p,
                None => absolute(env("HOME"), "HOME")?.join(".config"),
            }
            .join("breakfree-docs")
        }
    };
    Ok(dir.join("settings.json"))
}
fn read_current(path: &Path) -> Result<Option<Vec<u8>>> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("Cannot inspect settings: {e}")),
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err("Settings must be a regular file, not a link or directory.".into());
    }
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|e| format!("Cannot open settings: {e}"))?
        .take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("Cannot read settings: {e}"))?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err("Settings exceed 64 KiB; the original file was left unchanged.".into());
    }
    Ok(Some(bytes))
}

pub struct Store {
    path: Result<PathBuf>,
    // None means the original could not be read, not that the file was missing.
    observed: Option<Option<Vec<u8>>>,
}
struct Lock(PathBuf, Option<fs::File>);
impl Drop for Lock {
    fn drop(&mut self) {
        self.1.take(); // Close before unlinking on Windows.
        let _ = fs::remove_file(&self.0);
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Stage {
    Write,
    Sync,
    Backup,
    Replace,
}

impl Store {
    /// Read only. Defaults are usable after an error, but bad input is never repaired silently.
    pub fn open(path: Result<PathBuf>) -> (Self, Toolbar, Option<String>) {
        let read = path
            .as_ref()
            .map_err(Clone::clone)
            .and_then(|p| read_current(p));
        let observed = read.as_ref().ok().cloned();
        let loaded = read.and_then(|bytes| {
            bytes
                .map(|b| decode(&b))
                .unwrap_or_else(|| Ok(Toolbar::default()))
        });
        let (toolbar, warning) = match loaded {
            Ok(t) => (t, None),
            Err(e) => (Toolbar::default(), Some(e)),
        };
        (Self { path, observed }, toolbar, warning)
    }
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref().ok()
    }
    pub fn save(&mut self, toolbar: &Toolbar) -> Result<()> {
        self.write(toolbar, false, |_| Ok(())).map(|_| ())
    }
    /// Explicit reset preserves any readable original in a unique sibling backup first.
    pub fn reset(&mut self) -> Result<Option<PathBuf>> {
        self.write(&Toolbar::default(), true, |_| Ok(()))
    }

    fn write(
        &mut self,
        toolbar: &Toolbar,
        reset: bool,
        mut check: impl FnMut(Stage) -> Result<()>,
    ) -> Result<Option<PathBuf>> {
        let data = encode(toolbar)?;
        let path = self.path.as_ref().map_err(Clone::clone)?;
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .ok_or("Settings path needs a parent directory.")?;
        if !path.is_absolute() {
            return Err("Settings path must be absolute.".into());
        }
        fs::create_dir_all(parent).map_err(|e| format!("Cannot create settings directory: {e}"))?;
        let lock_path = parent.join(".toolbar-settings.lock");
        let lock = fs::OpenOptions::new().write(true).create_new(true).open(&lock_path).map_err(|e| format!("Cannot lock toolbar settings: {e}. Close other Docs windows and retry. A lock left after a crash must be removed manually while all Docs windows are closed."))?;
        let _lock = Lock(lock_path, Some(lock));
        let current = read_current(path)?;
        if !reset {
            if self.observed.as_ref() != Some(&current) {
                return Err("Toolbar settings changed outside this window. Restart Docs to load them, or Reset Toolbar to preserve a backup and use defaults.".into());
            }
            if let Some(bytes) = &current {
                decode(bytes)?;
            }
        }
        let permissions = fs::metadata(path).ok().map(|m| m.permissions());
        if permissions.as_ref().is_some_and(|p| p.readonly()) {
            return Err("Toolbar settings are read-only.".into());
        }
        let mut temp = tempfile::Builder::new()
            .prefix(".toolbar-")
            .suffix(".tmp")
            .tempfile_in(parent)
            .map_err(|e| format!("Cannot prepare settings: {e}"))?;
        let half = data.len() / 2;
        temp.write_all(&data[..half])
            .map_err(|e| format!("Cannot write settings: {e}"))?;
        check(Stage::Write)?;
        temp.write_all(&data[half..])
            .and_then(|_| temp.flush())
            .map_err(|e| format!("Cannot write settings: {e}"))?;
        if let Some(p) = permissions {
            temp.as_file()
                .set_permissions(p)
                .map_err(|e| format!("Cannot preserve settings permissions: {e}"))?;
        }
        check(Stage::Sync)?;
        temp.as_file()
            .sync_all()
            .map_err(|e| format!("Cannot flush settings: {e}"))?;
        let backup = if reset && let Some(bytes) = &current {
            check(Stage::Backup)?;
            let mut backup = tempfile::Builder::new()
                .prefix("toolbar-settings-backup-")
                .suffix(".json")
                .tempfile_in(parent)
                .map_err(|e| format!("Cannot prepare settings backup: {e}"))?;
            backup
                .write_all(bytes)
                .and_then(|_| backup.flush())
                .and_then(|_| backup.as_file().sync_all())
                .map_err(|e| format!("Cannot write settings backup: {e}"))?;
            let (file, saved) = backup
                .keep()
                .map_err(|e| format!("Cannot keep settings backup: {e}"))?;
            drop(file);
            Some(saved)
        } else {
            None
        };
        check(Stage::Replace)?;
        temp.persist(path)
            .map_err(|e| format!("Cannot replace settings: {}", e.error))?;
        self.observed = Some(Some(data));
        Ok(backup)
    }
}

#[cfg(test)]
#[path = "preferences_tests.rs"]
mod tests;
