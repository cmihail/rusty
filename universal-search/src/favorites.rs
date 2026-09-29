use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

const FAVORITES_FILE: &str = ".config/universal-search/favorites.txt";

pub struct Favorites {
    entries: HashSet<String>,
    file_path: PathBuf,
}

impl Default for Favorites {
    fn default() -> Self {
        Self::new()
    }
}

impl Favorites {
    pub fn new() -> Self {
        let mut home_dir = dirs::home_dir().unwrap();
        home_dir.push(FAVORITES_FILE);
        let file_path = home_dir;

        let entries = if file_path.exists() {
            Self::load_entries(&file_path).unwrap_or_else(|_| HashSet::new())
        } else {
            HashSet::new()
        };

        Self { entries, file_path }
    }

    pub fn new_with_path(path: PathBuf) -> Self {
        let entries = if path.exists() {
            Self::load_entries(&path).unwrap_or_else(|_| HashSet::new())
        } else {
            HashSet::new()
        };

        Self {
            entries,
            file_path: path,
        }
    }

    pub fn toggle(&mut self, desktop_id: &str) -> bool {
        let desktop_id = desktop_id.to_string();
        let is_favorited = if self.entries.contains(&desktop_id) {
            self.entries.remove(&desktop_id);
            false
        } else {
            self.entries.insert(desktop_id);
            true
        };
        self.save().ok();
        is_favorited
    }

    pub fn is_favorited(&self, desktop_id: &str) -> bool {
        self.entries.contains(desktop_id)
    }

    pub fn get_favorites(&self) -> Vec<String> {
        self.entries.iter().cloned().collect()
    }

    fn load_entries(path: &PathBuf) -> io::Result<HashSet<String>> {
        let file = File::open(path)?;
        let reader = io::BufReader::new(file);
        reader.lines().collect()
    }

    fn save(&self) -> io::Result<()> {
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.file_path)?;

        for entry in &self.entries {
            writeln!(file, "{}", entry)?;
        }

        Ok(())
    }
}
