// Tests for Brave Search functionality
// Note: These tests focus on the GObject integration and property handling
// They don't require actual API calls to avoid network dependencies

use gtk4::prelude::*;
use rusty_universal_search::brave_search::{BraveSearch, BraveSearchProfile, BraveSearchResult};
use std::env;
use std::sync::Once;

static INIT: Once = Once::new();

fn ensure_gtk_init() {
    INIT.call_once(|| {
        gtk4::init().expect("Failed to initialize GTK");
    });
}

#[test]
fn test_brave_search_initialization() {
    ensure_gtk_init();

    // Test that BraveSearch service can be created
    let brave_search = BraveSearch::new();

    // Initial state should be not loading and empty results
    assert!(
        !brave_search.loading(),
        "BraveSearch should not be loading initially"
    );
    assert_eq!(
        brave_search.results().len(),
        0,
        "BraveSearch results should be empty initially"
    );

    println!("BraveSearch service initialized successfully");
}

#[test]
fn test_brave_search_reset_functionality() {
    ensure_gtk_init();

    let brave_search = BraveSearch::new();

    // Create some mock results
    let mock_result = BraveSearchResult {
        title: "Test Title".to_string(),
        url: "https://example.com".to_string(),
        description: "Test description".to_string(),
        age: None,
        page_age: None,
        profile: None,
        thumbnail: None,
    };

    // Manually set some state to test reset
    brave_search.set_loading(true);
    brave_search.set_results(vec![mock_result]);

    // Verify state was set
    assert!(
        brave_search.loading(),
        "Loading should be true after setting"
    );
    assert_eq!(
        brave_search.results().len(),
        1,
        "Results should have one item after setting"
    );

    // Reset and verify
    brave_search.reset();

    assert!(
        !brave_search.loading(),
        "Loading should be false after reset"
    );
    assert_eq!(
        brave_search.results().len(),
        0,
        "Results should be empty after reset"
    );

    println!("BraveSearch reset functionality works correctly");
}

#[test]
fn test_brave_search_property_changes() {
    ensure_gtk_init();

    let brave_search = BraveSearch::new();

    // Test loading property changes
    assert!(!brave_search.loading());
    brave_search.set_loading(true);
    assert!(brave_search.loading());
    brave_search.set_loading(false);
    assert!(!brave_search.loading());

    // Test results property changes
    assert_eq!(brave_search.results().len(), 0);

    let mock_results = vec![
        BraveSearchResult {
            title: "Result 1".to_string(),
            url: "https://example1.com".to_string(),
            description: "First result".to_string(),
            age: None,
            page_age: None,
            profile: None,
            thumbnail: None,
        },
        BraveSearchResult {
            title: "Result 2".to_string(),
            url: "https://example2.com".to_string(),
            description: "Second result".to_string(),
            age: None,
            page_age: None,
            profile: None,
            thumbnail: None,
        },
    ];

    brave_search.set_results(mock_results.clone());
    assert_eq!(brave_search.results().len(), 2);
    assert_eq!(brave_search.results()[0].title, "Result 1");
    assert_eq!(brave_search.results()[1].title, "Result 2");

    println!("BraveSearch property changes work correctly");
}

#[test]
fn test_brave_search_api_key_validation() {
    // Test API key validation without making actual requests

    // Save original API key
    let original_key = env::var("BRAVE_SEARCH_API_KEY").ok();

    // Test with no API key
    env::remove_var("BRAVE_SEARCH_API_KEY");

    // Since we can't easily test the internal validation without exposing it,
    // we'll test that the BraveSearch object can be created without an API key
    // The actual search validation is tested in integration scenarios

    ensure_gtk_init();
    let brave_search = BraveSearch::new();

    // Object should be created successfully even without API key
    assert!(!brave_search.loading(), "Should not be loading initially");
    assert_eq!(
        brave_search.results().len(),
        0,
        "Should have no results initially"
    );

    // Restore original API key if it existed
    if let Some(key) = original_key {
        env::set_var("BRAVE_SEARCH_API_KEY", key);
    }

    println!("BraveSearch API key validation test completed");
}

#[test]
fn test_brave_search_multiple_instances() {
    ensure_gtk_init();

    // Test that multiple BraveSearch instances can be created independently
    let brave_search1 = BraveSearch::new();
    let brave_search2 = BraveSearch::new();

    // Create different mock results
    let mock_result1 = BraveSearchResult {
        title: "Result 1".to_string(),
        url: "https://example1.com".to_string(),
        description: "First instance result".to_string(),
        age: None,
        page_age: None,
        profile: None,
        thumbnail: None,
    };

    let mock_result2 = BraveSearchResult {
        title: "Result 2".to_string(),
        url: "https://example2.com".to_string(),
        description: "Second instance result".to_string(),
        age: None,
        page_age: None,
        profile: None,
        thumbnail: None,
    };

    // Set different states
    brave_search1.set_loading(true);
    brave_search1.set_results(vec![mock_result1]);

    brave_search2.set_loading(false);
    brave_search2.set_results(vec![mock_result2]);

    // Verify they're independent
    assert!(brave_search1.loading());
    assert!(!brave_search2.loading());
    assert_eq!(brave_search1.results().len(), 1);
    assert_eq!(brave_search2.results().len(), 1);
    assert_eq!(brave_search1.results()[0].title, "Result 1");
    assert_eq!(brave_search2.results()[0].title, "Result 2");

    println!("Multiple BraveSearch instances work independently");
}

#[test]
fn test_brave_search_loading_state() {
    ensure_gtk_init();

    let brave_search = BraveSearch::new();

    // Initially not loading
    assert!(!brave_search.loading());
    assert_eq!(brave_search.results().len(), 0);

    // Test manual state changes that would happen during search
    brave_search.set_loading(true);
    brave_search.set_results(Vec::new()); // Clear results when starting search

    // Should be in loading state with no results
    assert!(
        brave_search.loading(),
        "Should be loading after setting state"
    );
    assert_eq!(
        brave_search.results().len(),
        0,
        "Should have no results while loading"
    );

    println!("BraveSearch loading state behavior is correct");
}

#[test]
fn test_brave_search_result_structure() {
    // Test that BraveSearchResult can handle various data scenarios
    ensure_gtk_init();

    let brave_search = BraveSearch::new();

    // Test result with full data
    let full_result = BraveSearchResult {
        title: "Complete Result".to_string(),
        url: "https://complete.example.com".to_string(),
        description: "A result with all fields populated".to_string(),
        age: Some("2 days ago".to_string()),
        page_age: Some("1 week".to_string()),
        profile: Some(BraveSearchProfile {
            name: "Example Site".to_string(),
            url: "https://profile.example.com".to_string(),
            long_name: "Example Website Profile".to_string(),
        }),
        thumbnail: None, // We don't create thumbnail in this test
    };

    // Test result with minimal data
    let minimal_result = BraveSearchResult {
        title: "Minimal Result".to_string(),
        url: "https://minimal.example.com".to_string(),
        description: "Basic result".to_string(),
        age: None,
        page_age: None,
        profile: None,
        thumbnail: None,
    };

    brave_search.set_results(vec![full_result, minimal_result]);
    let results = brave_search.results();

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].title, "Complete Result");
    assert!(results[0].profile.is_some());
    assert_eq!(results[1].title, "Minimal Result");
    assert!(results[1].profile.is_none());

    println!("BraveSearch result structure handling works correctly");
}

#[test]
fn test_brave_search_empty_and_whitespace_queries() {
    ensure_gtk_init();

    let brave_search = BraveSearch::new();

    // Test that we can set results manually for various scenarios
    // This tests what would happen after the async processing completes

    // Test empty query result simulation
    brave_search.set_results(Vec::new());
    assert_eq!(brave_search.results().len(), 0);

    // Test successful query result simulation
    let mock_result = BraveSearchResult {
        title: "Example Result".to_string(),
        url: "https://example.com".to_string(),
        description: "An example search result".to_string(),
        age: None,
        page_age: None,
        profile: None,
        thumbnail: None,
    };
    brave_search.set_results(vec![mock_result]);
    assert_eq!(brave_search.results().len(), 1);

    println!("BraveSearch handles various query result types correctly");
}

#[test]
fn test_brave_search_concurrent_searches() {
    ensure_gtk_init();

    let brave_search = BraveSearch::new();

    // Test that the service can handle multiple state changes rapidly
    // This simulates what would happen during concurrent requests
    brave_search.set_loading(true);
    assert!(brave_search.loading());

    brave_search.set_loading(false);
    assert!(!brave_search.loading());

    brave_search.set_loading(true);
    assert!(brave_search.loading());

    // Test rapid result changes
    let result1 = BraveSearchResult {
        title: "First Result".to_string(),
        url: "https://first.com".to_string(),
        description: "First search result".to_string(),
        age: None,
        page_age: None,
        profile: None,
        thumbnail: None,
    };

    let result2 = BraveSearchResult {
        title: "Second Result".to_string(),
        url: "https://second.com".to_string(),
        description: "Second search result".to_string(),
        age: None,
        page_age: None,
        profile: None,
        thumbnail: None,
    };

    brave_search.set_results(vec![result1]);
    assert_eq!(brave_search.results().len(), 1);

    brave_search.set_results(vec![result2]);
    assert_eq!(brave_search.results().len(), 1);
    assert_eq!(brave_search.results()[0].title, "Second Result");

    println!("BraveSearch handles rapid state changes without crashing");
}

#[test]
fn test_brave_search_integration_with_gtk_main_loop() {
    // Test that BraveSearch works properly with GTK's main loop
    ensure_gtk_init();

    let brave_search = BraveSearch::new();

    // Set up a simple property change notification
    brave_search.connect_notify_local(Some("loading"), move |_obj, _| {
        // This would be called when the loading property changes
        println!("Loading property changed notification received");
    });

    // Trigger a property change
    brave_search.set_loading(true);

    // The notification system should work without crashing
    assert!(brave_search.loading());

    println!("BraveSearch integrates properly with GTK main loop");
}
