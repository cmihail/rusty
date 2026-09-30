use std::env;
use std::path::Path;

// Import the screenshot and recording modules from the main crate
use rusty_screenshot::recording;
use rusty_screenshot::screenshot;

// Note: These are integration tests that would require actual screenshot tools
// In a real environment, you might want to mock these or use test doubles

#[test]
fn test_screenshot_directory_structure() {
    let home = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let screenshots_dir = format!("{}/Pictures/Screenshots", home);

    // This test assumes the Screenshots directory exists or can be created
    // In a real scenario, the app should create this directory if it doesn't exist
    if !Path::new(&screenshots_dir).exists() {
        // Skip test if directory doesn't exist (not a failure)
        println!("Screenshots directory doesn't exist: {}", screenshots_dir);
        return;
    }

    assert!(Path::new(&screenshots_dir).is_dir());
}

#[test]
fn test_environment_variables() {
    // Test that HOME environment variable handling works correctly
    let original_home = env::var("HOME").ok();

    // Test with HOME set
    if original_home.is_some() {
        let output = screenshot::get_output_file_with_timestamp("test-timestamp");
        assert!(output.contains("Pictures/Screenshots"));
        assert!(output.contains("test-timestamp.png"));
    }

    // Test with HOME unset (should fall back to /tmp)
    env::remove_var("HOME");
    let output = screenshot::get_output_file_with_timestamp("test-timestamp");
    assert!(output.starts_with("/tmp/Pictures/Screenshots/"));

    // Restore HOME if it was set
    if let Some(home) = original_home {
        env::set_var("HOME", home);
    }
}

#[test]
fn test_command_building() {
    // Test that screenshot commands are built correctly
    let test_file = "/tmp/test-screenshot.png";

    let all_monitors_cmd = screenshot::build_all_monitors_command(test_file);
    assert_eq!(all_monitors_cmd, "grim '/tmp/test-screenshot.png'");

    let select_area_cmd = screenshot::build_select_area_command(test_file);
    assert_eq!(
        select_area_cmd,
        "grim -g \"$(slurp)\" '/tmp/test-screenshot.png'"
    );
}

#[test]
fn test_file_path_safety() {
    // Test that file paths are properly escaped
    let dangerous_path = "/tmp/test; rm -rf /";
    let cmd = screenshot::build_all_monitors_command(dangerous_path);

    // The command should be properly quoted to prevent injection
    assert!(cmd.contains("'/tmp/test; rm -rf /'"));
}

#[test]
fn test_file_path_with_single_quote() {
    let cmd = screenshot::build_all_monitors_command("/tmp/it's'; rm -rf /; '.png");
    assert_eq!(cmd, "grim '/tmp/it'\\''s'\\''; rm -rf /; '\\''.png'");
}

// Mock test for command execution (doesn't actually run screenshot commands)
#[test]
fn test_command_structure() {
    // Test various file paths and ensure commands are well-formed
    let test_cases = vec![
        "/home/user/Pictures/Screenshots/test.png",
        "/tmp/screenshot.png",
        "/path/with spaces/screenshot.png",
    ];

    for file_path in test_cases {
        let cmd = screenshot::build_all_monitors_command(file_path);
        assert!(cmd.starts_with("grim '"));
        assert!(cmd.ends_with("'"));
        assert!(cmd.contains(file_path));
    }
}

// Recording tests

#[test]
fn test_recording_directory_structure() {
    let home = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let recordings_dir = format!("{}/Videos/Recordings", home);

    if !Path::new(&recordings_dir).exists() {
        println!("Recordings directory doesn't exist: {}", recordings_dir);
        return;
    }

    assert!(Path::new(&recordings_dir).is_dir());
}

#[test]
fn test_recording_environment_variables() {
    let original_home = env::var("HOME").ok();

    if original_home.is_some() {
        let output = recording::get_video_output_file_with_timestamp("test-timestamp");
        assert!(output.contains("Videos/Recordings"));
        assert!(output.contains("test-timestamp.mp4"));
    }

    env::remove_var("HOME");
    let output = recording::get_video_output_file_with_timestamp("test-timestamp");
    assert!(output.starts_with("/tmp/Videos/Recordings/"));

    if let Some(home) = original_home {
        env::set_var("HOME", home);
    }
}

#[test]
fn test_recording_command_building() {
    let test_file = "/tmp/test-recording.mp4";

    let select_area_cmd = recording::build_select_area_recording_command(test_file, false);
    assert_eq!(
        select_area_cmd,
        "wf-recorder -g \"$(slurp)\" -f '/tmp/test-recording.mp4'"
    );

    let select_area_audio_cmd = recording::build_select_area_recording_command(test_file, true);
    assert_eq!(
        select_area_audio_cmd,
        "wf-recorder -g \"$(slurp)\" --audio -f '/tmp/test-recording.mp4'"
    );
}

#[test]
fn test_recording_file_path_safety() {
    let dangerous_path = "/tmp/test; rm -rf /";
    let cmd = recording::build_select_area_recording_command(dangerous_path, false);

    assert!(cmd.contains("'/tmp/test; rm -rf /'"));
}

#[test]
fn test_recording_command_structure() {
    let test_cases = vec![
        "/home/user/Videos/Recordings/test.mp4",
        "/tmp/recording.mp4",
        "/path/with spaces/recording.mp4",
    ];

    for file_path in test_cases {
        let cmd = recording::build_select_area_recording_command(file_path, false);
        assert!(cmd.starts_with("wf-recorder"));
        assert!(cmd.contains("-f '"));
        assert!(cmd.contains(file_path));
        assert!(cmd.ends_with("'"));
    }
}

#[test]
fn test_recording_audio_flag_consistency() {
    let test_file = "/tmp/test.mp4";

    let no_audio_cmd = recording::build_select_area_recording_command(test_file, false);
    assert!(!no_audio_cmd.contains("--audio"));

    let with_audio_cmd = recording::build_select_area_recording_command(test_file, true);
    assert!(with_audio_cmd.contains("--audio"));
}
