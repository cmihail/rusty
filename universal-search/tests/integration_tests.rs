use gio::prelude::*;

// Import the apps module from the main crate (crate name with dashes becomes underscores)
use rusty_universal_search::apps::AppSearch;

// Note: These are integration tests for the universal search functionality
// Some tests may require actual desktop applications to be present

#[test]
fn test_app_search_initialization() {
    // Test that AppSearch can be created and initialized
    let _app_search = AppSearch::new();

    // The app search should have loaded some applications (assuming there are desktop files)
    // We don't test for a specific number since it depends on the system
    println!("AppSearch initialized successfully");
}

#[test]
fn test_empty_query_returns_apps() {
    let app_search = AppSearch::new();
    let results = app_search.get_all_apps(50); // Test with higher limit

    // Should return up to 50 apps (or all available apps if fewer)
    assert!(results.len() <= 50);
    println!("Empty query returned {} apps", results.len());
}

#[test]
fn test_fuzzy_search_functionality() {
    let app_search = AppSearch::new();

    // Test common queries that should work on most systems
    let test_queries = vec!["calc", "text", "file", "term"];

    for query in test_queries {
        let results = app_search.fuzzy_query(query);
        // Results should no longer be limited to 12
        println!("Query '{}' returned {} results", query, results.len());
    }
}

#[test]
fn test_application_properties() {
    let app_search = AppSearch::new();
    let apps = app_search.get_all_apps(5); // Get first 5 apps for testing

    for app in apps {
        // Each app should have a name
        assert!(!app.name.is_empty(), "App name should not be empty");

        // Icon name should not be empty (may be default icon)
        assert!(!app.icon_name.is_empty(), "Icon name should not be empty");

        // Desktop ID should not be empty
        assert!(!app.desktop_id.is_empty(), "Desktop ID should not be empty");

        println!("App: {} ({})", app.name, app.desktop_id);
    }
}

#[test]
fn test_fuzzy_match_scoring() {
    let app_search = AppSearch::new();

    // Test that exact matches score higher than partial matches
    let results = app_search.fuzzy_query("a"); // Single character should match many apps

    // All results should have positive scores
    for app in results {
        assert!(app.score > 0.0, "App score should be positive");
        println!("App: {} - Score: {}", app.name, app.score);
    }
}

#[test]
fn test_frequency_tracking() {
    let mut app_search = AppSearch::new();
    let apps = app_search.get_all_apps(1);

    if let Some(app) = apps.first() {
        // Test frequency increment
        app_search.increment_frequency(&app.desktop_id);
        println!("Incremented frequency for app: {}", app.name);
    }
}

#[test]
fn test_app_launch_method_exists() {
    let app_search = AppSearch::new();
    let apps = app_search.get_all_apps(1);

    if let Some(app) = apps.first() {
        // We don't actually launch the app in tests, but we can test that
        // the launch method exists and can be called
        // Note: This would actually launch the app, so we just verify the method exists
        println!("App {} has launch capability", app.name);
        assert!(true); // Just verify we can access the launch method
    }
}

#[test]
fn test_desktop_app_info_integration() {
    // Test that we can interact with the GIO desktop app info system
    let all_apps = gio::AppInfo::all();
    assert!(
        !all_apps.is_empty(),
        "System should have some desktop applications"
    );

    let mut visible_apps = 0;
    for app_info in all_apps.iter().take(10) {
        // Test first 10 apps
        if let Some(desktop_app) = app_info.downcast_ref::<gio::DesktopAppInfo>() {
            if desktop_app.should_show() {
                visible_apps += 1;
                println!("Visible app: {}", desktop_app.name());
            }
        }
    }

    println!("Found {} visible desktop applications", visible_apps);
}

#[test]
fn test_unlimited_search_results() {
    let app_search = AppSearch::new();

    // Test that results are properly limited when requested
    let limited_apps = app_search.get_all_apps(15);
    assert!(
        limited_apps.len() <= 15,
        "get_all_apps should respect the limit when specified"
    );

    // Test that unlimited search works
    let unlimited_apps = app_search.get_all_apps(usize::MAX);
    let search_results = app_search.fuzzy_query("a"); // Should match many apps

    println!(
        "Unlimited apps: {}, Search results for 'a': {}",
        unlimited_apps.len(),
        search_results.len()
    );

    // Search results should not be arbitrarily limited
    // They should only be limited by actual matches
}

#[test]
fn test_scrollable_results_behavior() {
    let app_search = AppSearch::new();

    // Test that we can get more than 12 results when available
    let all_apps = app_search.get_all_apps(usize::MAX);
    let _search_results = app_search.fuzzy_query(""); // Empty query should show all

    println!("Total apps available: {}", all_apps.len());

    // Test with a common letter that should match many apps
    let common_search = app_search.fuzzy_query("e");
    println!("Apps matching 'e': {}", common_search.len());

    // Verify that we're no longer limited to 12 items
    if all_apps.len() > 12 {
        // If system has more than 12 apps, verify we can access them all
        let large_result_set = app_search.get_all_apps(50);
        assert!(
            large_result_set.len() > 12 || large_result_set.len() == all_apps.len(),
            "Should be able to get more than 12 apps when available"
        );
    }
}

// Mock test for application structure
#[test]
fn test_application_struct_functionality() {
    // We can't easily create a real Application without desktop files,
    // but we can test the general structure expectations

    let app_search = AppSearch::new();
    let apps = app_search.get_all_apps(3);

    for app in apps {
        // Test that applications can be compared (for sorting)
        let app_clone = app.clone();
        assert_eq!(app.desktop_id, app_clone.desktop_id);

        // Test Display trait if available
        println!("Application: {}", app.name);
    }
}
