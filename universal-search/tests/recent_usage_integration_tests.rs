// Integration tests for recent usage functionality
use rusty_universal_search::recent_usage::RecentUsage;
use std::env;
use std::fs;
use std::path::PathBuf;

fn create_temp_paths() -> (PathBuf, PathBuf) {
    use std::thread;
    let thread_id = thread::current().id();
    let temp_dir = env::temp_dir().join(format!(
        "test_universal_search_recent_integration_main_{:?}",
        thread_id
    ));
    fs::create_dir_all(&temp_dir).ok();

    let apps_path = temp_dir.join("recent_apps_integration_main_test.txt");
    let files_path = temp_dir.join("recent_files_integration_main_test.txt");

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
fn test_recent_usage_startup_initialization() {
    let (apps_path, files_path) = create_temp_paths();

    // Pre-populate with some data
    {
        let mut usage = RecentUsage::new_with_paths(apps_path.clone(), files_path.clone());
        usage.add_recent_app("firefox.desktop", "Firefox", "firefox");
        usage.add_recent_app("terminal.desktop", "Terminal", "terminal");
        usage.add_recent_file("/home/user/document.txt");
        usage.add_recent_file("/home/user/project/");
    }

    // Simulate startup by creating a new instance
    {
        let usage = RecentUsage::new_with_paths(apps_path.clone(), files_path.clone());
        let recent_apps = usage.get_recent_apps();
        let recent_files = usage.get_recent_files();

        // Verify data is loaded correctly on startup
        assert_eq!(recent_apps.len(), 2);
        assert_eq!(recent_apps[0].desktop_id, "terminal.desktop");
        assert_eq!(recent_apps[1].desktop_id, "firefox.desktop");

        assert_eq!(recent_files.len(), 2);
        assert_eq!(recent_files[0].path, "/home/user/project/");
        assert_eq!(recent_files[1].path, "/home/user/document.txt");
    }

    cleanup_temp_paths(&apps_path, &files_path);
}

#[test]
fn test_recent_usage_empty_startup() {
    let (apps_path, files_path) = create_temp_paths();

    // Create fresh instance with no existing data
    let usage = RecentUsage::new_with_paths(apps_path.clone(), files_path.clone());
    let recent_apps = usage.get_recent_apps();
    let recent_files = usage.get_recent_files();

    // Verify empty state
    assert_eq!(recent_apps.len(), 0);
    assert_eq!(recent_files.len(), 0);

    cleanup_temp_paths(&apps_path, &files_path);
}

#[test]
fn test_recent_usage_startup_with_corrupted_data() {
    let (apps_path, files_path) = create_temp_paths();

    // Write corrupted data to files
    fs::write(&apps_path, "invalid|data\npartial|only|two").ok(); // First line only has 2 parts, second has 3
    fs::write(&files_path, "invalid\nvalid|entry").ok(); // First line only has 1 part, second has 2

    // Should handle corrupted data gracefully
    let usage = RecentUsage::new_with_paths(apps_path.clone(), files_path.clone());
    let recent_apps = usage.get_recent_apps();
    let recent_files = usage.get_recent_files();

    // Should only load valid entries
    assert_eq!(recent_apps.len(), 1); // Only "partial|only|two" has exactly 3 parts
    assert_eq!(recent_files.len(), 1); // Only "valid|entry" has exactly 2 parts

    cleanup_temp_paths(&apps_path, &files_path);
}

#[test]
fn test_recent_usage_startup_performance() {
    let (apps_path, files_path) = create_temp_paths();

    // Pre-populate with maximum entries
    {
        let mut usage = RecentUsage::new_with_paths(apps_path.clone(), files_path.clone());
        for i in 0..10 {
            usage.add_recent_app(&format!("app{}.desktop", i), &format!("App {}", i), "icon");
            usage.add_recent_file(&format!("/test/file{}.txt", i));
        }
    }

    // Measure startup time
    let start = std::time::Instant::now();
    let _usage = RecentUsage::new_with_paths(apps_path.clone(), files_path.clone());
    let duration = start.elapsed();

    // Should initialize quickly (under 100ms)
    assert!(
        duration < std::time::Duration::from_millis(100),
        "Recent usage initialization took too long: {:?}",
        duration
    );

    cleanup_temp_paths(&apps_path, &files_path);
}
