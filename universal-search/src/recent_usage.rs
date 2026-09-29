use std::collections::VecDeque;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

const MAX_RECENT_APPS: usize = 10;
const MAX_RECENT_FILES: usize = 10;

#[derive(Debug, Clone)]
pub struct RecentApp {
    pub desktop_id: String,
    pub name: String,
    pub icon_name: String,
}

#[derive(Debug, Clone)]
pub struct RecentFile {
    pub path: String,
    pub display_name: String,
}

pub struct RecentUsage {
    recent_apps: VecDeque<RecentApp>,
    recent_files: VecDeque<RecentFile>,
    apps_file_path: PathBuf,
    files_file_path: PathBuf,
}

impl RecentUsage {
    pub fn new() -> Self {
        let mut home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        home_dir.push(".config/universal-search");

        // Create directory if it doesn't exist
        fs::create_dir_all(&home_dir).ok();

        let apps_file_path = home_dir.join("recent_apps.txt");
        let files_file_path = home_dir.join("recent_files.txt");

        let mut usage = Self {
            recent_apps: VecDeque::new(),
            recent_files: VecDeque::new(),
            apps_file_path,
            files_file_path,
        };

        usage.load_from_files();
        usage
    }

    pub fn new_with_paths(apps_path: PathBuf, files_path: PathBuf) -> Self {
        let mut usage = Self {
            recent_apps: VecDeque::new(),
            recent_files: VecDeque::new(),
            apps_file_path: apps_path,
            files_file_path: files_path,
        };

        usage.load_from_files();
        usage
    }

    fn load_from_files(&mut self) {
        self.load_recent_apps();
        self.load_recent_files();
    }

    fn load_recent_apps(&mut self) {
        if let Ok(file) = fs::File::open(&self.apps_file_path) {
            let reader = BufReader::new(file);
            for line in reader.lines().take(MAX_RECENT_APPS).flatten() {
                let parts: Vec<&str> = line.splitn(3, '|').collect();
                if parts.len() == 3 {
                    let recent_app = RecentApp {
                        desktop_id: parts[0].to_string(),
                        name: parts[1].to_string(),
                        icon_name: parts[2].to_string(),
                    };
                    self.recent_apps.push_back(recent_app);
                }
            }
        }
    }

    fn load_recent_files(&mut self) {
        if let Ok(file) = fs::File::open(&self.files_file_path) {
            let reader = BufReader::new(file);
            for line in reader.lines().take(MAX_RECENT_FILES).flatten() {
                let parts: Vec<&str> = line.splitn(2, '|').collect();
                if parts.len() == 2 {
                    let recent_file = RecentFile {
                        path: parts[0].to_string(),
                        display_name: parts[1].to_string(),
                    };
                    self.recent_files.push_back(recent_file);
                }
            }
        }
    }

    fn save_recent_apps(&self) {
        if let Ok(mut file) = fs::File::create(&self.apps_file_path) {
            for app in &self.recent_apps {
                writeln!(file, "{}|{}|{}", app.desktop_id, app.name, app.icon_name).ok();
            }
        }
    }

    fn save_recent_files(&self) {
        if let Ok(mut file) = fs::File::create(&self.files_file_path) {
            for file_entry in &self.recent_files {
                writeln!(file, "{}|{}", file_entry.path, file_entry.display_name).ok();
            }
        }
    }

    pub fn add_recent_app(&mut self, desktop_id: &str, name: &str, icon_name: &str) {
        let new_app = RecentApp {
            desktop_id: desktop_id.to_string(),
            name: name.to_string(),
            icon_name: icon_name.to_string(),
        };

        // Remove existing entry if it exists (move to front)
        self.recent_apps.retain(|app| app.desktop_id != desktop_id);

        // Add to front
        self.recent_apps.push_front(new_app);

        // Keep only the most recent MAX_RECENT_APPS
        while self.recent_apps.len() > MAX_RECENT_APPS {
            self.recent_apps.pop_back();
        }

        self.save_recent_apps();
    }

    pub fn add_recent_file(&mut self, path: &str) {
        let display_name = Self::get_file_display_name(path);

        let new_file = RecentFile {
            path: path.to_string(),
            display_name,
        };

        // Remove existing entry if it exists (move to front)
        self.recent_files.retain(|file| file.path != path);

        // Add to front
        self.recent_files.push_front(new_file);

        // Keep only the most recent MAX_RECENT_FILES
        while self.recent_files.len() > MAX_RECENT_FILES {
            self.recent_files.pop_back();
        }

        self.save_recent_files();
    }

    fn get_file_display_name(path: &str) -> String {
        // Extract the filename or last directory component
        if let Some(name) = std::path::Path::new(path).file_name() {
            name.to_string_lossy().to_string()
        } else {
            path.to_string()
        }
    }

    pub fn get_recent_apps(&self) -> Vec<RecentApp> {
        self.recent_apps.iter().cloned().collect()
    }

    pub fn get_recent_files(&self) -> Vec<RecentFile> {
        self.recent_files.iter().cloned().collect()
    }

    pub fn clear_recent_apps(&mut self) {
        self.recent_apps.clear();
        self.save_recent_apps();
    }

    pub fn clear_recent_files(&mut self) {
        self.recent_files.clear();
        self.save_recent_files();
    }

    pub fn clear_all(&mut self) {
        self.clear_recent_apps();
        self.clear_recent_files();
    }
}

impl Default for RecentUsage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;

    fn create_temp_paths() -> (PathBuf, PathBuf) {
        use std::thread;
        let thread_id = thread::current().id();
        let temp_dir =
            env::temp_dir().join(format!("test_universal_search_recent_{:?}", thread_id));
        fs::create_dir_all(&temp_dir).ok();

        let apps_path = temp_dir.join("recent_apps_test.txt");
        let files_path = temp_dir.join("recent_files_test.txt");

        // Clean up any existing files
        fs::remove_file(&apps_path).ok();
        fs::remove_file(&files_path).ok();

        (apps_path, files_path)
    }

    fn cleanup_temp_paths(apps_path: &PathBuf, files_path: &PathBuf) {
        fs::remove_file(apps_path).ok();
        fs::remove_file(files_path).ok();
        if let Some(parent) = apps_path.parent() {
            fs::remove_dir(parent).ok();
        }
    }

    #[test]
    fn test_recent_usage_creation() {
        let (apps_path, files_path) = create_temp_paths();
        let usage = RecentUsage::new_with_paths(apps_path.clone(), files_path.clone());

        assert_eq!(usage.get_recent_apps().len(), 0);
        assert_eq!(usage.get_recent_files().len(), 0);

        cleanup_temp_paths(&apps_path, &files_path);
    }

    #[test]
    fn test_add_recent_app() {
        let (apps_path, files_path) = create_temp_paths();
        let mut usage = RecentUsage::new_with_paths(apps_path.clone(), files_path.clone());

        usage.add_recent_app("test.desktop", "Test App", "test-icon");

        let recent_apps = usage.get_recent_apps();
        assert_eq!(recent_apps.len(), 1);
        assert_eq!(recent_apps[0].desktop_id, "test.desktop");
        assert_eq!(recent_apps[0].name, "Test App");
        assert_eq!(recent_apps[0].icon_name, "test-icon");

        cleanup_temp_paths(&apps_path, &files_path);
    }

    #[test]
    fn test_add_recent_file() {
        let (apps_path, files_path) = create_temp_paths();
        let mut usage = RecentUsage::new_with_paths(apps_path.clone(), files_path.clone());

        usage.add_recent_file("/home/user/document.txt");

        let recent_files = usage.get_recent_files();
        assert_eq!(recent_files.len(), 1);
        assert_eq!(recent_files[0].path, "/home/user/document.txt");
        assert_eq!(recent_files[0].display_name, "document.txt");

        cleanup_temp_paths(&apps_path, &files_path);
    }

    #[test]
    fn test_recent_app_move_to_front() {
        let (apps_path, files_path) = create_temp_paths();
        let mut usage = RecentUsage::new_with_paths(apps_path.clone(), files_path.clone());

        usage.add_recent_app("app1.desktop", "App 1", "icon1");
        usage.add_recent_app("app2.desktop", "App 2", "icon2");
        usage.add_recent_app("app1.desktop", "App 1", "icon1"); // Add same app again

        let recent_apps = usage.get_recent_apps();
        assert_eq!(recent_apps.len(), 2);
        assert_eq!(recent_apps[0].desktop_id, "app1.desktop"); // Should be first now
        assert_eq!(recent_apps[1].desktop_id, "app2.desktop");

        cleanup_temp_paths(&apps_path, &files_path);
    }

    #[test]
    fn test_max_recent_apps_limit() {
        let (apps_path, files_path) = create_temp_paths();
        let mut usage = RecentUsage::new_with_paths(apps_path.clone(), files_path.clone());

        // Add more than MAX_RECENT_APPS
        for i in 0..15 {
            usage.add_recent_app(&format!("app{}.desktop", i), &format!("App {}", i), "icon");
        }

        let recent_apps = usage.get_recent_apps();
        assert_eq!(recent_apps.len(), MAX_RECENT_APPS);
        assert_eq!(recent_apps[0].desktop_id, "app14.desktop"); // Most recent

        cleanup_temp_paths(&apps_path, &files_path);
    }

    #[test]
    fn test_persistence() {
        let (apps_path, files_path) = create_temp_paths();

        // Create first instance and add data
        {
            let mut usage = RecentUsage::new_with_paths(apps_path.clone(), files_path.clone());
            usage.add_recent_app("test.desktop", "Test App", "test-icon");
            usage.add_recent_file("/home/user/test.txt");
        }

        // Create second instance and verify data is loaded
        {
            let usage = RecentUsage::new_with_paths(apps_path.clone(), files_path.clone());
            let recent_apps = usage.get_recent_apps();
            let recent_files = usage.get_recent_files();

            assert_eq!(recent_apps.len(), 1);
            assert_eq!(recent_apps[0].desktop_id, "test.desktop");
            assert_eq!(recent_files.len(), 1);
            assert_eq!(recent_files[0].path, "/home/user/test.txt");
        }

        cleanup_temp_paths(&apps_path, &files_path);
    }

    #[test]
    fn test_clear_functionality() {
        let (apps_path, files_path) = create_temp_paths();
        let mut usage = RecentUsage::new_with_paths(apps_path.clone(), files_path.clone());

        usage.add_recent_app("test.desktop", "Test App", "test-icon");
        usage.add_recent_file("/home/user/test.txt");

        usage.clear_recent_apps();
        assert_eq!(usage.get_recent_apps().len(), 0);
        assert_eq!(usage.get_recent_files().len(), 1);

        usage.clear_recent_files();
        assert_eq!(usage.get_recent_files().len(), 0);

        cleanup_temp_paths(&apps_path, &files_path);
    }
}
