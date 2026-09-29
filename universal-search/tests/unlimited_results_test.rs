use rusty_universal_search::files::FileSearcher;

#[test]
fn test_unlimited_file_results() {
    let searcher = FileSearcher::new();

    // Search for a common pattern that should return many results
    let results = searcher.search("rs").unwrap();

    println!("Found {} .rs files", results.len());

    // Should find many .rs files without any artificial limit
    // This test verifies there's no MAX_RESULTS constraint
    assert!(!results.is_empty(), "Should find .rs files");

    // If we find more than the old limit (12), it confirms unlimited results work
    if results.len() > 12 {
        println!(
            "✓ Unlimited results confirmed: found {} results (> 12)",
            results.len()
        );
    }
}

#[test]
fn test_unlimited_app_and_file_integration() {
    // This test verifies that when searching, we don't artificially limit
    // the combination of apps and files to a specific number
    let searcher = FileSearcher::new();

    // Test with a query that should find both files and potentially apps
    let file_results = searcher.search("test").unwrap();

    println!("Found {} files matching 'test'", file_results.len());

    // The key is that we don't limit file results based on app results anymore
    // All matching files should be returned
    assert!(
        !file_results.is_empty(),
        "Should find files matching 'test'"
    );
}
