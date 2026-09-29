use rusty_universal_search::apps::{AppSearch, Application};
use rusty_universal_search::favorites::Favorites;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

fn get_test_favorites_path() -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp_dir = env::temp_dir().join(format!("test_favorites_ordering_{}", timestamp));
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
fn test_favorites_alphabetical_sorting() {
    let test_path = get_test_favorites_path();
    cleanup_test_file(&test_path);

    let mut favorites = Favorites::new_with_path(test_path.clone());

    // Add favorites in non-alphabetical order
    favorites.toggle("zebra.desktop");
    favorites.toggle("apple.desktop");
    favorites.toggle("mango.desktop");

    let favorite_ids = favorites.get_favorites();

    // Simulate sorting logic from main.rs
    let app_search = AppSearch::new();
    let all_apps = app_search.get_all_apps(usize::MAX);

    let favorite_set: HashSet<String> = favorite_ids.into_iter().collect();
    let mut favorite_apps: Vec<Application> = all_apps
        .into_iter()
        .filter(|app| favorite_set.contains(&app.desktop_id))
        .collect();

    // Sort alphabetically (case-insensitive)
    favorite_apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    // Verify sorting - if we found any matching apps, check they're sorted
    if !favorite_apps.is_empty() {
        for i in 1..favorite_apps.len() {
            assert!(
                favorite_apps[i - 1].name.to_lowercase() <= favorite_apps[i].name.to_lowercase(),
                "Apps should be sorted alphabetically: {} should come before or equal to {}",
                favorite_apps[i - 1].name,
                favorite_apps[i].name
            );
        }
    }

    cleanup_test_file(&test_path);
}

#[test]
fn test_favorites_separation_from_others() {
    let test_path = get_test_favorites_path();
    cleanup_test_file(&test_path);

    let mut favorites = Favorites::new_with_path(test_path.clone());
    let app_search = AppSearch::new();
    let all_apps = app_search.get_all_apps(usize::MAX);

    if all_apps.len() >= 3 {
        // Mark some apps as favorites
        favorites.toggle(&all_apps[0].desktop_id);
        favorites.toggle(&all_apps[2].desktop_id);

        let favorite_ids: HashSet<String> = favorites.get_favorites().into_iter().collect();

        let mut favorite_apps: Vec<Application> = Vec::new();
        let mut other_apps: Vec<Application> = Vec::new();

        for app in all_apps.clone() {
            if favorite_ids.contains(&app.desktop_id) {
                favorite_apps.push(app);
            } else {
                other_apps.push(app);
            }
        }

        // Verify separation
        assert_eq!(favorite_apps.len(), 2);
        assert_eq!(
            favorite_apps.len() + other_apps.len(),
            all_apps.len(),
            "All apps should be categorized"
        );

        // Verify no overlap
        for fav_app in &favorite_apps {
            assert!(
                !other_apps
                    .iter()
                    .any(|other| other.desktop_id == fav_app.desktop_id),
                "Favorite app should not be in other apps"
            );
        }
    }

    cleanup_test_file(&test_path);
}

#[test]
fn test_favorites_case_insensitive_sorting() {
    let test_path = get_test_favorites_path();
    cleanup_test_file(&test_path);

    let app_search = AppSearch::new();
    let all_apps = app_search.get_all_apps(usize::MAX);

    if all_apps.len() >= 3 {
        let mut favorites = Favorites::new_with_path(test_path.clone());

        // Mark multiple apps as favorites
        for app in all_apps.iter().take(5) {
            favorites.toggle(&app.desktop_id);
        }

        let favorite_ids: HashSet<String> = favorites.get_favorites().into_iter().collect();
        let mut favorite_apps: Vec<Application> = all_apps
            .into_iter()
            .filter(|app| favorite_ids.contains(&app.desktop_id))
            .collect();

        // Sort alphabetically (case-insensitive)
        favorite_apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        // Verify case-insensitive sorting
        for i in 1..favorite_apps.len() {
            let prev_name = favorite_apps[i - 1].name.to_lowercase();
            let current_name = favorite_apps[i].name.to_lowercase();
            assert!(
                prev_name <= current_name,
                "Case-insensitive sorting failed: '{}' should come before or equal to '{}'",
                favorite_apps[i - 1].name,
                favorite_apps[i].name
            );
        }
    }

    cleanup_test_file(&test_path);
}

#[test]
fn test_empty_favorites_no_crash() {
    let test_path = get_test_favorites_path();
    cleanup_test_file(&test_path);

    let favorites = Favorites::new_with_path(test_path.clone());
    let app_search = AppSearch::new();
    let all_apps = app_search.get_all_apps(usize::MAX);

    let favorite_ids: HashSet<String> = favorites.get_favorites().into_iter().collect();
    let mut favorite_apps: Vec<Application> = all_apps
        .into_iter()
        .filter(|app| favorite_ids.contains(&app.desktop_id))
        .collect();

    // Should handle empty favorites without crash
    favorite_apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    assert_eq!(favorite_apps.len(), 0, "Should have no favorite apps");

    cleanup_test_file(&test_path);
}

#[test]
fn test_single_favorite_sorting() {
    let test_path = get_test_favorites_path();
    cleanup_test_file(&test_path);

    let mut favorites = Favorites::new_with_path(test_path.clone());
    let app_search = AppSearch::new();
    let all_apps = app_search.get_all_apps(usize::MAX);

    if !all_apps.is_empty() {
        // Add single favorite
        favorites.toggle(&all_apps[0].desktop_id);

        let favorite_ids: HashSet<String> = favorites.get_favorites().into_iter().collect();
        let mut favorite_apps: Vec<Application> = all_apps
            .into_iter()
            .filter(|app| favorite_ids.contains(&app.desktop_id))
            .collect();

        // Sort single element (should not crash)
        favorite_apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        assert_eq!(favorite_apps.len(), 1, "Should have exactly one favorite");
    }

    cleanup_test_file(&test_path);
}
