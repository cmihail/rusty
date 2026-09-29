use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

const HISTORY_FILE: &str = ".config/universal-search/history.txt";

pub struct History {
    entries: Vec<String>,
    file_path: PathBuf,
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}

impl History {
    pub fn new() -> Self {
        let mut home_dir = dirs::home_dir().unwrap();
        home_dir.push(HISTORY_FILE);
        let file_path = home_dir;

        let entries = if file_path.exists() {
            Self::load_entries(&file_path).unwrap_or_else(|_| Vec::new())
        } else {
            Vec::new()
        };

        Self { entries, file_path }
    }

    pub fn new_with_path(path: PathBuf) -> Self {
        let entries = if path.exists() {
            Self::load_entries(&path).unwrap_or_else(|_| Vec::new())
        } else {
            Vec::new()
        };

        Self {
            entries,
            file_path: path,
        }
    }

    pub fn add(&mut self, entry: &str) {
        if !entry.trim().is_empty() {
            let entry_string = entry.to_string();

            // Remove the entry if it already exists
            if let Some(pos) = self.entries.iter().position(|x| *x == entry_string) {
                self.entries.remove(pos);
            }

            // Add the entry to the end (most recent)
            self.entries.push(entry_string);
            self.save().ok();
        }
    }

    pub fn get_entries(&self) -> &Vec<String> {
        &self.entries
    }

    fn load_entries(path: &PathBuf) -> io::Result<Vec<String>> {
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
