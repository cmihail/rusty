use rusty_universal_search::favorites::Favorites;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

fn get_test_favorites_path() -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp_dir = env::temp_dir().join(format!("test_universal_search_favorites_{}", timestamp));
    fs::create_dir_all(&temp_dir).ok();
    temp_dir.join("favorites.txt")
}

fn cleanup_test_file(path: &PathBuf) {
    fs::remove_file(path).ok();
    if let Some(parent) = path.parent() {
        fs::remove_dir(parent).ok();
    }
}

#[test]
fn test_favorites_creation() {
    let test_path = get_test_favorites_path();
    cleanup_test_file(&test_path);

    let favorites = Favorites::new_with_path(test_path.clone());
    assert_eq!(favorites.get_favorites().len(), 0);

    cleanup_test_file(&test_path);
}

#[test]
fn test_toggle_add_favorite() {
    let test_path = get_test_favorites_path();
    cleanup_test_file(&test_path);

    let mut favorites = Favorites::new_with_path(test_path.clone());

    let is_favorited = favorites.toggle("firefox.desktop");
    assert!(is_favorited);
    assert!(favorites.is_favorited("firefox.desktop"));

    cleanup_test_file(&test_path);
}

#[test]
fn test_toggle_remove_favorite() {
    let test_path = get_test_favorites_path();
    cleanup_test_file(&test_path);

    let mut favorites = Favorites::new_with_path(test_path.clone());

    favorites.toggle("firefox.desktop");
    assert!(favorites.is_favorited("firefox.desktop"));

    let is_favorited = favorites.toggle("firefox.desktop");
    assert!(!is_favorited);
    assert!(!favorites.is_favorited("firefox.desktop"));

    cleanup_test_file(&test_path);
}

#[test]
fn test_multiple_favorites() {
    let test_path = get_test_favorites_path();
    cleanup_test_file(&test_path);

    let mut favorites = Favorites::new_with_path(test_path.clone());

    favorites.toggle("firefox.desktop");
    favorites.toggle("chrome.desktop");
    favorites.toggle("code.desktop");

    assert!(favorites.is_favorited("firefox.desktop"));
    assert!(favorites.is_favorited("chrome.desktop"));
    assert!(favorites.is_favorited("code.desktop"));
    assert_eq!(favorites.get_favorites().len(), 3);

    cleanup_test_file(&test_path);
}

#[test]
fn test_favorites_persistence() {
    let test_path = get_test_favorites_path();
    cleanup_test_file(&test_path);

    {
        let mut favorites = Favorites::new_with_path(test_path.clone());
        favorites.toggle("firefox.desktop");
        favorites.toggle("chrome.desktop");
    }

    {
        let favorites = Favorites::new_with_path(test_path.clone());
        assert!(favorites.is_favorited("firefox.desktop"));
        assert!(favorites.is_favorited("chrome.desktop"));
        assert_eq!(favorites.get_favorites().len(), 2);
    }

    cleanup_test_file(&test_path);
}

#[test]
fn test_is_favorited_nonexistent() {
    let test_path = get_test_favorites_path();
    cleanup_test_file(&test_path);

    let favorites = Favorites::new_with_path(test_path.clone());
    assert!(!favorites.is_favorited("nonexistent.desktop"));

    cleanup_test_file(&test_path);
}

#[test]
fn test_get_favorites_returns_all() {
    let test_path = get_test_favorites_path();
    cleanup_test_file(&test_path);

    let mut favorites = Favorites::new_with_path(test_path.clone());

    favorites.toggle("firefox.desktop");
    favorites.toggle("chrome.desktop");
    favorites.toggle("code.desktop");

    let all_favorites = favorites.get_favorites();
    assert_eq!(all_favorites.len(), 3);
    assert!(all_favorites.contains(&"firefox.desktop".to_string()));
    assert!(all_favorites.contains(&"chrome.desktop".to_string()));
    assert!(all_favorites.contains(&"code.desktop".to_string()));

    cleanup_test_file(&test_path);
}

#[test]
fn test_favorites_no_duplicates() {
    let test_path = get_test_favorites_path();
    cleanup_test_file(&test_path);

    let mut favorites = Favorites::new_with_path(test_path.clone());

    favorites.toggle("firefox.desktop");
    favorites.toggle("firefox.desktop");
    favorites.toggle("firefox.desktop");

    // After 3 toggles (add, remove, add), should be favorited
    assert!(favorites.is_favorited("firefox.desktop"));
    assert_eq!(favorites.get_favorites().len(), 1);

    cleanup_test_file(&test_path);
}

#[test]
fn test_favorites_load_from_existing_file() {
    let test_path = get_test_favorites_path();
    cleanup_test_file(&test_path);

    fs::create_dir_all(test_path.parent().unwrap()).unwrap();
    fs::write(
        &test_path,
        "firefox.desktop\nchrome.desktop\ncode.desktop\n",
    )
    .unwrap();

    let favorites = Favorites::new_with_path(test_path.clone());
    assert_eq!(favorites.get_favorites().len(), 3);
    assert!(favorites.is_favorited("firefox.desktop"));
    assert!(favorites.is_favorited("chrome.desktop"));
    assert!(favorites.is_favorited("code.desktop"));

    cleanup_test_file(&test_path);
}

#[test]
fn test_favorites_corrupted_file_handling() {
    let test_path = get_test_favorites_path();
    cleanup_test_file(&test_path);

    fs::create_dir_all(test_path.parent().unwrap()).unwrap();
    fs::write(&test_path, "firefox.desktop\n\n\nchrome.desktop\n").unwrap();

    let favorites = Favorites::new_with_path(test_path.clone());
    assert!(favorites.is_favorited("firefox.desktop"));
    assert!(favorites.is_favorited("chrome.desktop"));
    assert!(favorites.is_favorited(""));

    cleanup_test_file(&test_path);
}

#[test]
fn test_favorites_empty_desktop_id() {
    let test_path = get_test_favorites_path();
    cleanup_test_file(&test_path);

    let mut favorites = Favorites::new_with_path(test_path.clone());

    favorites.toggle("");
    assert!(favorites.is_favorited(""));

    cleanup_test_file(&test_path);
}
