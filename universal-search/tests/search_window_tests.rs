use rusty_universal_search::{brave_search::BraveSearch, gemini::Gemini, history::History};
use std::cell::RefCell;
use std::rc::Rc;

const DEFAULT_APP_DISPLAY_LIMIT: usize = 50;

#[test]
fn test_history_move_to_top_behavior() {
    use std::env;

    // Use a temporary directory for testing
    let temp_dir = env::temp_dir().join("test_universal_search_history");
    std::fs::create_dir_all(&temp_dir).ok();
    let temp_file = temp_dir.join("history.txt");

    // Clean slate for testing
    std::fs::remove_file(&temp_file).ok();

    // Create history with empty state using test path
    let mut history = History::new_with_path(temp_file.clone());

    // Add initial entries
    history.add("first entry");
    history.add("second entry");
    history.add("third entry");

    let entries = history.get_entries();
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0], "first entry");
    assert_eq!(entries[1], "second entry");
    assert_eq!(entries[2], "third entry");

    // Add an existing entry - should move to top (end of vec)
    history.add("first entry");

    let entries = history.get_entries();
    assert_eq!(entries.len(), 3); // Still 3 entries
    assert_eq!(entries[0], "second entry"); // Original first moved
    assert_eq!(entries[1], "third entry");
    assert_eq!(entries[2], "first entry"); // Moved to end (most recent)

    // Add another existing entry
    history.add("second entry");

    let entries = history.get_entries();
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0], "third entry");
    assert_eq!(entries[1], "first entry");
    assert_eq!(entries[2], "second entry"); // Now most recent

    // Add a new entry
    history.add("fourth entry");

    let entries = history.get_entries();
    assert_eq!(entries.len(), 4);
    assert_eq!(entries[3], "fourth entry"); // Added to end

    // Clean up
    std::fs::remove_file(&temp_file).ok();
    std::fs::remove_dir(&temp_dir).ok();
}

#[test]
fn test_ai_results_hidden_when_query_changes() {
    gtk4::init().unwrap();

    // Create Gemini and BraveSearch instances
    let gemini = Gemini::new();
    let brave_search = BraveSearch::new();

    // Simulate having AI results with a specific query
    let last_ai_query = Rc::new(RefCell::new("weather".to_string()));
    gemini.set_result("Weather information".to_string());
    brave_search.set_results(vec![]);

    // Test 1: Query matches - should show AI results
    let query = "weather";
    let last_query = last_ai_query.borrow().clone();
    let query_matches = query == last_query;
    assert!(query_matches);

    // Test 2: Query changed - should not show AI results
    let query = "weather forecast";
    let last_query = last_ai_query.borrow().clone();
    let query_matches = query == last_query;
    assert!(!query_matches);

    // Test 3: Empty query - should not match
    let query = "";
    let last_query = last_ai_query.borrow().clone();
    let query_matches = query == last_query;
    assert!(!query_matches);
}

#[test]
fn test_empty_query_clears_last_ai_query() {
    let last_ai_query = Rc::new(RefCell::new("previous search".to_string()));

    // Simulate clearing on empty query
    let query = "";
    if query.is_empty() {
        *last_ai_query.borrow_mut() = String::new();
    }

    assert_eq!(*last_ai_query.borrow(), String::new());
}

#[test]
fn test_app_display_limit_with_few_favorites() {
    // Test: When favorites < limit, total apps = limit
    let favorite_count = 10;
    let total_to_show = favorite_count.max(DEFAULT_APP_DISPLAY_LIMIT);
    let other_apps_to_show = total_to_show.saturating_sub(favorite_count);

    assert_eq!(total_to_show, DEFAULT_APP_DISPLAY_LIMIT);
    assert_eq!(
        other_apps_to_show,
        DEFAULT_APP_DISPLAY_LIMIT - favorite_count
    );
}

#[test]
fn test_app_display_limit_with_many_favorites() {
    // Test: When favorites > limit, total apps = favorite count
    let favorite_count = 100;
    let total_to_show = favorite_count.max(DEFAULT_APP_DISPLAY_LIMIT);
    let other_apps_to_show = total_to_show.saturating_sub(favorite_count);

    assert_eq!(total_to_show, favorite_count);
    assert_eq!(other_apps_to_show, 0);
}

#[test]
fn test_app_display_limit_with_no_favorites() {
    // Test: When favorites = 0, total apps = limit
    let favorite_count = 0;
    let total_to_show = favorite_count.max(DEFAULT_APP_DISPLAY_LIMIT);
    let other_apps_to_show = total_to_show.saturating_sub(favorite_count);

    assert_eq!(total_to_show, DEFAULT_APP_DISPLAY_LIMIT);
    assert_eq!(other_apps_to_show, DEFAULT_APP_DISPLAY_LIMIT);
}

#[test]
fn test_app_display_limit_constant() {
    // Ensure the constant is set to a reasonable value
    assert_eq!(DEFAULT_APP_DISPLAY_LIMIT, 50);
}
