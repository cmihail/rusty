// Tests for recent usage functionality
use rusty_universal_search::recent_usage::RecentUsage;
use std::env;
use std::fs;
use std::path::PathBuf;

fn create_temp_paths() -> (PathBuf, PathBuf) {
    use std::thread;
    let thread_id = thread::current().id();
    let temp_dir = env::temp_dir().join(format!(
        "test_universal_search_recent_integration_{:?}",
        thread_id
    ));
    fs::create_dir_all(&temp_dir).ok();

    let apps_path = temp_dir.join("recent_apps_integration_test.txt");
    let files_path = temp_dir.join("recent_files_integration_test.txt");

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
fn test_recent_usage_integration_workflow() {
    let (apps_path, files_path) = create_temp_paths();
    let mut usage = RecentUsage::new_with_paths(apps_path.clone(), files_path.clone());

    // Test adding recent apps
    usage.add_recent_app("firefox.desktop", "Firefox", "firefox");
    usage.add_recent_app("terminal.desktop", "Terminal", "terminal");
    usage.add_recent_app("code.desktop", "Visual Studio Code", "code");

    let recent_apps = usage.get_recent_apps();
    assert_eq!(recent_apps.len(), 3);
    assert_eq!(recent_apps[0].desktop_id, "code.desktop"); // Most recent
    assert_eq!(recent_apps[1].desktop_id, "terminal.desktop");
    assert_eq!(recent_apps[2].desktop_id, "firefox.desktop");

    // Test adding recent files
    usage.add_recent_file("/home/user/document.txt");
    usage.add_recent_file("/home/user/project/");
    usage.add_recent_file("/home/user/notes.md");

    let recent_files = usage.get_recent_files();
    assert_eq!(recent_files.len(), 3);
    assert_eq!(recent_files[0].path, "/home/user/notes.md"); // Most recent
    assert_eq!(recent_files[1].path, "/home/user/project/");
    assert_eq!(recent_files[2].path, "/home/user/document.txt");

    cleanup_temp_paths(&apps_path, &files_path);
}

#[test]
fn test_recent_usage_mru_behavior() {
    let (apps_path, files_path) = create_temp_paths();
    let mut usage = RecentUsage::new_with_paths(apps_path.clone(), files_path.clone());

    // Add apps in order
    usage.add_recent_app("app1.desktop", "App 1", "app1");
    usage.add_recent_app("app2.desktop", "App 2", "app2");
    usage.add_recent_app("app3.desktop", "App 3", "app3");

    // Use app1 again - should move to front
    usage.add_recent_app("app1.desktop", "App 1", "app1");

    let recent_apps = usage.get_recent_apps();
    assert_eq!(recent_apps.len(), 3);
    assert_eq!(recent_apps[0].desktop_id, "app1.desktop"); // Now most recent
    assert_eq!(recent_apps[1].desktop_id, "app3.desktop");
    assert_eq!(recent_apps[2].desktop_id, "app2.desktop");

    // Same test for files
    usage.add_recent_file("/file1.txt");
    usage.add_recent_file("/file2.txt");
    usage.add_recent_file("/file3.txt");

    // Use file1 again - should move to front
    usage.add_recent_file("/file1.txt");

    let recent_files = usage.get_recent_files();
    assert_eq!(recent_files.len(), 3);
    assert_eq!(recent_files[0].path, "/file1.txt"); // Now most recent
    assert_eq!(recent_files[1].path, "/file3.txt");
    assert_eq!(recent_files[2].path, "/file2.txt");

    cleanup_temp_paths(&apps_path, &files_path);
}

#[test]
fn test_recent_usage_size_limits() {
    let (apps_path, files_path) = create_temp_paths();
    let mut usage = RecentUsage::new_with_paths(apps_path.clone(), files_path.clone());

    // Add more than 10 apps
    for i in 0..15 {
        usage.add_recent_app(
            &format!("app{}.desktop", i),
            &format!("App {}", i),
            "app-icon",
        );
    }

    let recent_apps = usage.get_recent_apps();
    assert_eq!(recent_apps.len(), 10); // Should be limited to 10
    assert_eq!(recent_apps[0].desktop_id, "app14.desktop"); // Most recent

    // Add more than 10 files
    for i in 0..15 {
        usage.add_recent_file(&format!("/file{}.txt", i));
    }

    let recent_files = usage.get_recent_files();
    assert_eq!(recent_files.len(), 10); // Should be limited to 10
    assert_eq!(recent_files[0].path, "/file14.txt"); // Most recent

    cleanup_temp_paths(&apps_path, &files_path);
}

#[test]
fn test_recent_usage_persistence_across_instances() {
    let (apps_path, files_path) = create_temp_paths();

    // First instance - add some items
    {
        let mut usage = RecentUsage::new_with_paths(apps_path.clone(), files_path.clone());
        usage.add_recent_app("persistent-app.desktop", "Persistent App", "persistent");
        usage.add_recent_file("/persistent/file.txt");
    }

    // Second instance - should load the items
    {
        let usage = RecentUsage::new_with_paths(apps_path.clone(), files_path.clone());
        let recent_apps = usage.get_recent_apps();
        let recent_files = usage.get_recent_files();

        assert_eq!(recent_apps.len(), 1);
        assert_eq!(recent_apps[0].desktop_id, "persistent-app.desktop");
        assert_eq!(recent_apps[0].name, "Persistent App");
        assert_eq!(recent_apps[0].icon_name, "persistent");

        assert_eq!(recent_files.len(), 1);
        assert_eq!(recent_files[0].path, "/persistent/file.txt");
        assert_eq!(recent_files[0].display_name, "file.txt");
    }

    cleanup_temp_paths(&apps_path, &files_path);
}

#[test]
fn test_recent_usage_file_display_names() {
    let (apps_path, files_path) = create_temp_paths();
    let mut usage = RecentUsage::new_with_paths(apps_path.clone(), files_path.clone());

    // Test various file path formats
    usage.add_recent_file("/home/user/documents/report.pdf");
    usage.add_recent_file("/usr/local/bin/");
    usage.add_recent_file("relative/path/file.txt");
    usage.add_recent_file("/single-file");

    let recent_files = usage.get_recent_files();
    assert_eq!(recent_files.len(), 4);

    // Check display names (should be just the filename/directory name)
    assert_eq!(recent_files[3].display_name, "report.pdf");
    assert_eq!(recent_files[2].display_name, "bin");
    assert_eq!(recent_files[1].display_name, "file.txt");
    assert_eq!(recent_files[0].display_name, "single-file");

    cleanup_temp_paths(&apps_path, &files_path);
}

#[test]
fn test_recent_usage_clear_functionality() {
    let (apps_path, files_path) = create_temp_paths();
    let mut usage = RecentUsage::new_with_paths(apps_path.clone(), files_path.clone());

    // Add some items
    usage.add_recent_app("test.desktop", "Test App", "test");
    usage.add_recent_file("/test/file.txt");

    assert_eq!(usage.get_recent_apps().len(), 1);
    assert_eq!(usage.get_recent_files().len(), 1);

    // Test clearing apps only
    usage.clear_recent_apps();
    assert_eq!(usage.get_recent_apps().len(), 0);
    assert_eq!(usage.get_recent_files().len(), 1);

    // Test clearing files only
    usage.clear_recent_files();
    assert_eq!(usage.get_recent_apps().len(), 0);
    assert_eq!(usage.get_recent_files().len(), 0);

    // Add items again
    usage.add_recent_app("test.desktop", "Test App", "test");
    usage.add_recent_file("/test/file.txt");

    // Test clearing all
    usage.clear_all();
    assert_eq!(usage.get_recent_apps().len(), 0);
    assert_eq!(usage.get_recent_files().len(), 0);

    cleanup_temp_paths(&apps_path, &files_path);
}

#[test]
fn test_recent_usage_default_constructor() {
    let (apps_path, files_path) = create_temp_paths();

    // Test that the constructor works with fresh paths
    let usage = RecentUsage::new_with_paths(apps_path.clone(), files_path.clone());

    // Should start with empty lists
    assert_eq!(usage.get_recent_apps().len(), 0);
    assert_eq!(usage.get_recent_files().len(), 0);

    cleanup_temp_paths(&apps_path, &files_path);
}

#[test]
fn test_recent_usage_concurrent_operations() {
    let (apps_path, files_path) = create_temp_paths();

    // Test that multiple operations don't interfere with each other
    let mut usage = RecentUsage::new_with_paths(apps_path.clone(), files_path.clone());

    // Interleave app and file additions
    usage.add_recent_app("app1.desktop", "App 1", "app1");
    usage.add_recent_file("/file1.txt");
    usage.add_recent_app("app2.desktop", "App 2", "app2");
    usage.add_recent_file("/file2.txt");
    usage.add_recent_app("app1.desktop", "App 1", "app1"); // Use app1 again

    let recent_apps = usage.get_recent_apps();
    let recent_files = usage.get_recent_files();

    assert_eq!(recent_apps.len(), 2);
    assert_eq!(recent_apps[0].desktop_id, "app1.desktop"); // Most recent
    assert_eq!(recent_apps[1].desktop_id, "app2.desktop");

    assert_eq!(recent_files.len(), 2);
    assert_eq!(recent_files[0].path, "/file2.txt"); // Most recent
    assert_eq!(recent_files[1].path, "/file1.txt");

    cleanup_temp_paths(&apps_path, &files_path);
}
