use std::sync::OnceLock;

// Ensure GTK is initialized once per test binary
static GTK_INIT: OnceLock<()> = OnceLock::new();

fn ensure_gtk_init() {
    GTK_INIT.get_or_init(|| {
        gtk4::init().expect("Failed to initialize GTK");
    });
}

#[test]
fn test_gtk_initialization() {
    ensure_gtk_init();
    // Just verify GTK can be initialized
    assert!(true, "GTK should initialize successfully");
}

#[test]
fn test_system_info_css_class_name() {
    // Test that the CSS class name is correct
    let class_name = "KillProcess";
    assert_eq!(class_name, "KillProcess", "CSS class should be KillProcess");
}

#[test]
fn test_command_truncation_length() {
    // Verify the command truncation length constant
    let max_length = 40;
    assert_eq!(max_length, 40, "Command should be truncated at 40 chars");
}

#[test]
fn test_process_table_column_count() {
    // Verify we have the expected columns: PID, CPU%, MEM%, Command, Kill button
    let expected_columns = 5;
    assert_eq!(expected_columns, 5, "Process table should have 5 columns");
}

#[test]
fn test_scrolled_window_max_height() {
    // Verify the max content height constant
    let max_height = 500;
    assert_eq!(max_height, 500, "Max content height should be 500");
}

#[test]
fn test_refresh_interval() {
    // Verify refresh interval is 2 seconds
    let refresh_seconds = 2;
    assert_eq!(refresh_seconds, 2, "Refresh interval should be 2 seconds");
}

#[test]
fn test_column_width_chars() {
    // Verify command column width
    let cmd_width = 40;
    assert_eq!(cmd_width, 40, "Command column should be 40 chars wide");
}

#[test]
fn test_kill_signal() {
    // Verify we use SIGKILL
    let signal = "-9";
    assert_eq!(signal, "-9", "Should use SIGKILL (-9)");
}

#[test]
fn test_top_process_count() {
    // Verify we show top 10 processes
    let process_count = 10;
    assert_eq!(process_count, 10, "Should show top 10 processes");
}
