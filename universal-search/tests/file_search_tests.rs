use rusty_universal_search::files::{FileSearcher, FileType};

#[test]
fn test_file_search_basic() {
    let searcher = FileSearcher::new();

    // Test with a common filename that should exist
    let results = searcher.search("Cargo").unwrap();

    // Should find at least the Cargo.toml in the current project
    assert!(!results.is_empty(), "Should find files containing 'Cargo'");

    // Check that we have both files and/or directories
    let has_file = results.iter().any(|r| r.file_type == FileType::File);
    assert!(has_file, "Should find at least one file");
}

#[test]
fn test_file_search_empty_query() {
    let searcher = FileSearcher::new();

    let results = searcher.search("").unwrap();
    assert!(
        results.is_empty(),
        "Empty query should return empty results"
    );
}

#[test]
fn test_file_search_whitespace_query() {
    let searcher = FileSearcher::new();

    let results = searcher.search("   ").unwrap();
    assert!(
        results.is_empty(),
        "Whitespace-only query should return empty results"
    );
}

#[test]
fn test_file_search_nonexistent() {
    let searcher = FileSearcher::new();

    // Search for something very unlikely to exist
    let results = searcher.search("xyzabcdefghijk123456789").unwrap();
    // This might be empty or very small
    println!("Results for nonexistent file: {}", results.len());
}

#[test]
fn test_file_searcher_new() {
    let searcher = FileSearcher::new();

    // Test that we can create a searcher
    let results = searcher.search("src");
    assert!(results.is_ok(), "File searcher should work");
}

#[test]
fn test_file_search_directory() {
    let searcher = FileSearcher::new();

    // Should find src directory
    let results = searcher.search("src").unwrap();

    let has_directory = results
        .iter()
        .any(|r| r.file_type == FileType::Directory && r.name == "src");
    assert!(has_directory, "Should find src directory");
}
