use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DesktopEntry {
    pub name: Option<String>,
    pub startup_wm_class: Option<String>,
}

// A Wayland session started outside a display manager inherits an XDG_DATA_DIRS that
// omits the snap and flatpak export dirs, so those are searched unconditionally.
const EXTRA_APPLICATION_DIRS: &[&str] = &[
    "/var/lib/snapd/desktop/applications",
    "/var/lib/flatpak/exports/share/applications",
    "/usr/local/share/applications",
    "/usr/share/applications",
];

fn application_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Some(data_home) = std::env::var_os("XDG_DATA_HOME") {
        dirs.push(PathBuf::from(data_home).join("applications"));
    } else if let Some(home) = std::env::var_os("HOME") {
        dirs.push(PathBuf::from(home).join(".local/share/applications"));
    }

    if let Ok(data_dirs) = std::env::var("XDG_DATA_DIRS") {
        for dir in data_dirs.split(':').filter(|d| !d.is_empty()) {
            dirs.push(PathBuf::from(dir).join("applications"));
        }
    }

    for dir in EXTRA_APPLICATION_DIRS {
        dirs.push(PathBuf::from(dir));
    }

    dirs.dedup();
    dirs
}

pub fn parse(contents: &str) -> DesktopEntry {
    let mut entry = DesktopEntry::default();
    let mut in_desktop_entry = false;

    for line in contents.lines() {
        let line = line.trim();

        if line.starts_with('[') {
            in_desktop_entry = line == "[Desktop Entry]";
            continue;
        }

        if !in_desktop_entry {
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim();
        if value.is_empty() {
            continue;
        }

        // Localized keys ("Name[de]") are ignored in favour of the unlocalized one.
        match key.trim() {
            "Name" if entry.name.is_none() => entry.name = Some(value.to_string()),
            "StartupWMClass" if entry.startup_wm_class.is_none() => {
                entry.startup_wm_class = Some(value.to_string())
            }
            _ => {}
        }
    }

    entry
}

fn read(entry_id: &str) -> Option<DesktopEntry> {
    let file_name = if entry_id.ends_with(".desktop") {
        entry_id.to_string()
    } else {
        format!("{}.desktop", entry_id)
    };

    for dir in application_dirs() {
        let path = dir.join(&file_name);
        if let Ok(contents) = std::fs::read_to_string(&path) {
            return Some(parse(&contents));
        }
    }

    None
}

/// Resolve a `desktop-entry` notification hint to its desktop file.
pub fn lookup(entry_id: &str) -> Option<DesktopEntry> {
    if entry_id.is_empty() {
        return None;
    }

    static CACHE: OnceLock<Mutex<HashMap<String, DesktopEntry>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    if let Ok(cached) = cache.lock() {
        if let Some(entry) = cached.get(entry_id) {
            return Some(entry.clone());
        }
    }

    let entry = read(entry_id)?;

    if let Ok(mut cached) = cache.lock() {
        cached.insert(entry_id.to_string(), entry.clone());
    }

    Some(entry)
}
