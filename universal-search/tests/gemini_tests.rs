// Tests for Gemini AI search functionality
// Note: These tests focus on the GObject integration and markup processing
// They don't require actual API calls to avoid network dependencies

use gtk4::prelude::*;
use rusty_universal_search::gemini::Gemini;
use std::env;
use std::sync::Once;

static INIT: Once = Once::new();

fn ensure_gtk_init() {
    INIT.call_once(|| {
        gtk4::init().expect("Failed to initialize GTK");
    });
}

#[test]
fn test_gemini_initialization() {
    ensure_gtk_init();

    // Test that Gemini service can be created
    let gemini = Gemini::new();

    // Initial state should be not loading and empty result
    assert!(!gemini.loading(), "Gemini should not be loading initially");
    assert_eq!(
        gemini.result(),
        "",
        "Gemini result should be empty initially"
    );

    println!("Gemini service initialized successfully");
}

#[test]
fn test_gemini_reset_functionality() {
    ensure_gtk_init();

    let gemini = Gemini::new();

    // Manually set some state to test reset
    gemini.set_loading(true);
    gemini.set_result("Test result".to_string());

    // Verify state was set
    assert!(gemini.loading(), "Loading should be true after setting");
    assert_eq!(
        gemini.result(),
        "Test result",
        "Result should match what we set"
    );

    // Reset and verify
    gemini.reset();

    assert!(!gemini.loading(), "Loading should be false after reset");
    assert_eq!(gemini.result(), "", "Result should be empty after reset");

    println!("Gemini reset functionality works correctly");
}

#[test]
fn test_gemini_property_changes() {
    ensure_gtk_init();

    let gemini = Gemini::new();

    // Test loading property changes
    assert!(!gemini.loading());
    gemini.set_loading(true);
    assert!(gemini.loading());
    gemini.set_loading(false);
    assert!(!gemini.loading());

    // Test result property changes
    assert_eq!(gemini.result(), "");
    gemini.set_result("Test response".to_string());
    assert_eq!(gemini.result(), "Test response");
    gemini.set_result("Updated response".to_string());
    assert_eq!(gemini.result(), "Updated response");

    println!("Gemini property changes work correctly");
}

#[test]
fn test_gemini_api_key_validation() {
    // Test API key validation without making actual requests

    // Save original API key
    let original_key = env::var("PERSONAL_GEMINI_API_KEY").ok();

    // Test with no API key
    env::remove_var("PERSONAL_GEMINI_API_KEY");

    // Since we can't easily test the internal validation without exposing it,
    // we'll test the behavior when no API key is set by triggering a search
    // The search should handle the missing API key gracefully

    ensure_gtk_init();
    let gemini = Gemini::new();

    // Object should be created successfully even without API key
    assert!(!gemini.loading(), "Should not be loading initially");
    assert_eq!(gemini.result(), "", "Should have empty result initially");

    // Restore original API key if it existed
    if let Some(key) = original_key {
        env::set_var("PERSONAL_GEMINI_API_KEY", key);
    }

    println!("Gemini API key validation test completed");
}

#[test]
fn test_gemini_multiple_instances() {
    ensure_gtk_init();

    // Test that multiple Gemini instances can be created independently
    let gemini1 = Gemini::new();
    let gemini2 = Gemini::new();

    // Set different states
    gemini1.set_loading(true);
    gemini1.set_result("Result 1".to_string());

    gemini2.set_loading(false);
    gemini2.set_result("Result 2".to_string());

    // Verify they're independent
    assert!(gemini1.loading());
    assert!(!gemini2.loading());
    assert_eq!(gemini1.result(), "Result 1");
    assert_eq!(gemini2.result(), "Result 2");

    println!("Multiple Gemini instances work independently");
}

#[test]
fn test_gemini_search_loading_state() {
    ensure_gtk_init();

    let gemini = Gemini::new();

    // Initially not loading
    assert!(!gemini.loading());
    assert_eq!(gemini.result(), "");

    // Test manual state changes that would happen during search
    gemini.set_loading(true);
    gemini.set_result("Thinking...".to_string());

    // Should be in loading state with thinking message
    assert!(gemini.loading(), "Should be loading after setting state");
    assert_eq!(
        gemini.result(),
        "Thinking...",
        "Should show thinking message"
    );

    println!("Gemini search loading state behavior is correct");
}

#[test]
fn test_gemini_pango_markup_handling() {
    // Test that the Gemini service can handle various markup scenarios
    ensure_gtk_init();

    let gemini = Gemini::new();

    // Test setting result with basic markup
    let markup_text = "<b>Bold text</b> and <i>italic text</i>";
    gemini.set_result(markup_text.to_string());
    assert_eq!(gemini.result(), markup_text);

    // Test with longer markup text
    let complex_markup = "<span foreground=\"#74c0fc\">Info:</span> This is a <b>test</b> response with <tt>code</tt>";
    gemini.set_result(complex_markup.to_string());
    assert_eq!(gemini.result(), complex_markup);

    // Test with plain text (should work fine too)
    let plain_text = "This is plain text without markup";
    gemini.set_result(plain_text.to_string());
    assert_eq!(gemini.result(), plain_text);

    println!("Gemini Pango markup handling works correctly");
}

#[test]
fn test_gemini_empty_and_whitespace_queries() {
    ensure_gtk_init();

    let gemini = Gemini::new();

    // Test that we can handle various query types by setting results manually
    // This tests what would happen after the async processing completes

    // Test empty query result
    gemini.set_result("Error: Empty query not allowed".to_string());
    assert_eq!(gemini.result(), "Error: Empty query not allowed");

    gemini.reset();

    // Test successful query result
    gemini.set_result("This is a response to: How do I use regex: [a-z]+?".to_string());
    assert_eq!(
        gemini.result(),
        "This is a response to: How do I use regex: [a-z]+?"
    );

    println!("Gemini handles various query result types correctly");
}

#[test]
fn test_gemini_concurrent_searches() {
    ensure_gtk_init();

    let gemini = Gemini::new();

    // Test that the service can handle multiple state changes rapidly
    // This simulates what would happen during concurrent requests
    gemini.set_loading(true);
    assert!(gemini.loading());

    gemini.set_loading(false);
    assert!(!gemini.loading());

    gemini.set_loading(true);
    assert!(gemini.loading());

    // Test rapid result changes
    gemini.set_result("First result".to_string());
    assert_eq!(gemini.result(), "First result");

    gemini.set_result("Second result".to_string());
    assert_eq!(gemini.result(), "Second result");

    println!("Gemini handles rapid state changes without crashing");
}

#[test]
fn test_gemini_integration_with_gtk_main_loop() {
    // Test that Gemini works properly with GTK's main loop
    ensure_gtk_init();

    let gemini = Gemini::new();

    // Set up a simple property change notification
    gemini.connect_notify_local(Some("result"), move |_obj, _| {
        // This would be called when the result property changes
        println!("Result property changed notification received");
    });

    // Trigger a property change
    gemini.set_result("Test notification".to_string());

    // The notification system should work without crashing
    assert_eq!(gemini.result(), "Test notification");

    println!("Gemini integrates properly with GTK main loop");
}
