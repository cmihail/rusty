use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DesktopEntry {
    pub name: Option<String>,
    pub startup_wm_class: Option<String>,
    pub icon: Option<String>,
    pub exec: Option<String>,
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
            "Icon" if entry.icon.is_none() => entry.icon = Some(value.to_string()),
            "Exec" if entry.exec.is_none() => entry.exec = Some(value.to_string()),
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

/// Pick the entry for a window class: `StartupWMClass` first, then a desktop id equal to the
/// class, both case-insensitive. `entries` pairs desktop ids (without `.desktop`) with entries.
pub fn match_window_class(class: &str, entries: &[(String, DesktopEntry)]) -> Option<DesktopEntry> {
    if class.is_empty() {
        return None;
    }

    let by_wm_class = entries.iter().find(|(_, entry)| {
        entry
            .startup_wm_class
            .as_deref()
            .is_some_and(|wm_class| wm_class.eq_ignore_ascii_case(class))
    });
    let by_id = || {
        entries
            .iter()
            .find(|(id, _)| id.eq_ignore_ascii_case(class))
    };

    by_wm_class.or_else(by_id).map(|(_, entry)| entry.clone())
}

fn all_entries() -> Vec<(String, DesktopEntry)> {
    let mut entries = Vec::new();

    for dir in application_dirs() {
        let Ok(read_dir) = std::fs::read_dir(&dir) else {
            continue;
        };

        for file in read_dir.flatten() {
            let path = file.path();
            let Some(id) = path
                .file_name()
                .and_then(|name| name.to_str())
                .and_then(|name| name.strip_suffix(".desktop"))
            else {
                continue;
            };

            if let Ok(contents) = std::fs::read_to_string(&path) {
                entries.push((id.to_string(), parse(&contents)));
            }
        }
    }

    entries
}

/// The program name of an `Exec` value: its first word without quotes or directory.
pub fn exec_program(exec: &str) -> Option<&str> {
    let exec = exec.trim_start();
    let command = match exec.strip_prefix('"') {
        Some(quoted) => quoted.split('"').next()?,
        None => exec.split_whitespace().next()?,
    };
    command.rsplit('/').next().filter(|name| !name.is_empty())
}

/// Pick the entry whose `Exec` runs `program`, for windows whose class matches no entry.
pub fn match_executable(program: &str, entries: &[(String, DesktopEntry)]) -> Option<DesktopEntry> {
    if program.is_empty() {
        return None;
    }

    entries
        .iter()
        .find(|(_, entry)| entry.exec.as_deref().and_then(exec_program) == Some(program))
        .map(|(_, entry)| entry.clone())
}

type LookupCache = OnceLock<Mutex<HashMap<String, Option<DesktopEntry>>>>;
type Matcher = fn(&str, &[(String, DesktopEntry)]) -> Option<DesktopEntry>;

fn cached_lookup(cache: &LookupCache, key: &str, find: Matcher) -> Option<DesktopEntry> {
    let cache = cache.get_or_init(|| Mutex::new(HashMap::new()));

    if let Ok(cached) = cache.lock() {
        if let Some(entry) = cached.get(key) {
            return entry.clone();
        }
    }

    let entry = find(key, &all_entries());

    if let Ok(mut cached) = cache.lock() {
        cached.insert(key.to_string(), entry.clone());
    }

    entry
}

/// Resolve a Hyprland window class to its desktop file, cached per class.
pub fn lookup_by_window_class(class: &str) -> Option<DesktopEntry> {
    static CACHE: LookupCache = OnceLock::new();
    cached_lookup(&CACHE, class, match_window_class)
}

/// Resolve a program name (as in `/proc/<pid>/exe`) to its desktop file, cached per name.
pub fn lookup_by_executable(program: &str) -> Option<DesktopEntry> {
    static CACHE: LookupCache = OnceLock::new();
    cached_lookup(&CACHE, program, match_executable)
}
