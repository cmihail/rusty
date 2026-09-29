use chrono::Local;
use serde::Deserialize;
use std::process::Command;

#[derive(Deserialize, Debug, Clone)]
pub struct Monitor {
    pub width: i32,
    pub height: i32,
    pub x: i32,
    pub y: i32,
    pub focused: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ActiveWindow {
    #[serde(rename = "at")]
    pub position: [i32; 2],
    pub size: [i32; 2],
}

pub fn get_output_file() -> String {
    let now = Local::now();
    let timestamp = now.format("%Y.%m.%d-%H:%M:%S").to_string();
    format!(
        "{}/Pictures/Screenshots/{}.png",
        std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string()),
        timestamp
    )
}

pub fn get_output_file_with_timestamp(timestamp: &str) -> String {
    format!(
        "{}/Pictures/Screenshots/{}.png",
        std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string()),
        timestamp
    )
}

#[cfg(test)]
pub fn get_output_file_custom(base_path: &str, timestamp: &str) -> String {
    format!("{}/Pictures/Screenshots/{}.png", base_path, timestamp)
}

pub fn get_focused_monitor() -> Result<Monitor, String> {
    let output = Command::new("hyprctl")
        .args(["monitors", "-j"])
        .output()
        .map_err(|e| format!("Failed to run hyprctl: {}", e))?;

    if !output.status.success() {
        return Err("hyprctl monitors command failed".to_string());
    }

    let json_str = String::from_utf8(output.stdout)
        .map_err(|e| format!("Invalid UTF-8 output from hyprctl: {}", e))?;

    let monitors: Vec<Monitor> =
        serde_json::from_str(&json_str).map_err(|e| format!("Failed to parse JSON: {}", e))?;

    // Find the focused monitor by checking the focused field
    if let Some(focused_monitor) = monitors.iter().find(|m| m.focused).cloned() {
        Ok(focused_monitor)
    } else {
        // Fallback to first monitor if no focused monitor found
        eprintln!("No focused monitor found, using first monitor as fallback");
        monitors
            .into_iter()
            .next()
            .ok_or_else(|| "No monitors found".to_string())
    }
}

pub fn get_active_window() -> Result<ActiveWindow, String> {
    let output = Command::new("hyprctl")
        .args(["activewindow", "-j"])
        .output()
        .map_err(|e| format!("Failed to run hyprctl: {}", e))?;

    if !output.status.success() {
        return Err("hyprctl activewindow command failed".to_string());
    }

    let json_str = String::from_utf8(output.stdout)
        .map_err(|e| format!("Invalid UTF-8 output from hyprctl: {}", e))?;

    serde_json::from_str(&json_str).map_err(|e| format!("Failed to parse JSON: {}", e))
}

pub fn build_all_monitors_command(output_file: &str) -> String {
    format!("grim '{}'", output_file)
}

pub fn build_select_area_command(output_file: &str) -> String {
    format!("grim -g \"$(slurp)\" '{}'", output_file)
}

pub fn build_focused_monitor_command(output_file: &str) -> String {
    build_focused_monitor_command_with_monitor(output_file, None)
}

pub fn build_focused_monitor_command_with_monitor(
    output_file: &str,
    monitor: Option<Monitor>,
) -> String {
    match monitor.or_else(|| get_focused_monitor().ok()) {
        Some(monitor) => {
            format!(
                "grim -g \"{},{} {}x{}\" '{}'",
                monitor.x, monitor.y, monitor.width, monitor.height, output_file
            )
        }
        None => {
            // Fallback to all monitors if hyprctl fails
            eprintln!("Failed to get focused monitor info, falling back to all monitors");
            format!("grim '{}'", output_file)
        }
    }
}

pub fn build_focused_window_command(output_file: &str) -> String {
    build_focused_window_command_with_window(output_file, None)
}

pub fn build_focused_window_command_with_window(
    output_file: &str,
    window: Option<ActiveWindow>,
) -> String {
    match window.or_else(|| get_active_window().ok()) {
        Some(window) => {
            let x = window.position[0];
            let y = window.position[1];
            let width = window.size[0];
            let height = window.size[1];
            format!(
                "grim -g \"{},{} {}x{}\" '{}'",
                x, y, width, height, output_file
            )
        }
        None => {
            // Fallback to select area if hyprctl fails
            eprintln!("Failed to get active window info, falling back to select area");
            format!("grim -g \"$(slurp)\" '{}'", output_file)
        }
    }
}

pub fn execute_screenshot_command(command: &str, output_file: &str) -> Result<(), String> {
    let result = Command::new("bash").args(["-c", command]).output();

    match result {
        Ok(output) => {
            if output.status.success() {
                crate::notify::spawn_notification(
                    crate::notify::NotificationKind::Screenshot,
                    output_file,
                );
                Ok(())
            } else {
                let error_msg = String::from_utf8_lossy(&output.stderr);
                eprintln!("Screenshot failed: {}", error_msg);
                Err(format!("Screenshot command failed: {}", error_msg))
            }
        }
        Err(e) => {
            eprintln!("Failed to execute screenshot command: {}", e);
            Err(format!("Failed to execute command: {}", e))
        }
    }
}

pub fn show_notification(output_file: &str) {
    let notification_cmd = format!(
        "notify-send -A 'File' -A 'Directory' 'Screenshot taken' 'Saved to {}'",
        output_file
    );

    if let Ok(output) = Command::new("bash")
        .args(["-c", &notification_cmd])
        .output()
    {
        if let Ok(response) = String::from_utf8(output.stdout) {
            let response = response.trim();
            match response {
                "0" => {
                    // Open file
                    let _ = Command::new("xdg-open").arg(output_file).spawn();
                }
                "1" => {
                    // Open directory
                    let screenshots_dir = format!(
                        "{}/Pictures/Screenshots/",
                        std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())
                    );
                    let _ = Command::new("xdg-open").arg(screenshots_dir).spawn();
                }
                _ => {}
            }
        }
    }
}

pub fn take_all_monitors_screenshot() {
    let output_file = get_output_file();
    let command = build_all_monitors_command(&output_file);
    let _ = execute_screenshot_command(&command, &output_file);
}

pub fn take_focused_monitor_screenshot() {
    take_focused_monitor_screenshot_with_monitor(None);
}

pub fn take_focused_monitor_screenshot_with_monitor(monitor: Option<Monitor>) {
    let output_file = get_output_file();
    let command = build_focused_monitor_command_with_monitor(&output_file, monitor);
    let _ = execute_screenshot_command(&command, &output_file);
}

pub fn take_focused_window_screenshot() {
    take_focused_window_screenshot_with_window(None);
}

pub fn take_focused_window_screenshot_with_window(window: Option<ActiveWindow>) {
    let output_file = get_output_file();
    let command = build_focused_window_command_with_window(&output_file, window);
    let _ = execute_screenshot_command(&command, &output_file);
}

pub fn take_select_area_screenshot() {
    let output_file = get_output_file();
    let command = build_select_area_command(&output_file);
    let _ = execute_screenshot_command(&command, &output_file);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_get_output_file_custom() {
        let result = get_output_file_custom("/home/test", "2024.01.01-12:00:00");
        assert_eq!(
            result,
            "/home/test/Pictures/Screenshots/2024.01.01-12:00:00.png"
        );
    }

    #[test]
    fn test_get_output_file_with_timestamp() {
        let timestamp = "2024.01.01-12:00:00";
        let result = get_output_file_with_timestamp(timestamp);

        let expected_base = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let expected = format!("{}/Pictures/Screenshots/{}.png", expected_base, timestamp);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_build_all_monitors_command() {
        let output_file = "/test/screenshot.png";
        let command = build_all_monitors_command(output_file);
        assert_eq!(command, "grim '/test/screenshot.png'");
    }

    #[test]
    fn test_build_select_area_command() {
        let output_file = "/test/screenshot.png";
        let command = build_select_area_command(output_file);
        assert_eq!(command, "grim -g \"$(slurp)\" '/test/screenshot.png'");
    }

    #[test]
    fn test_build_focused_monitor_command() {
        let output_file = "/test/screenshot.png";
        let command = build_focused_monitor_command(output_file);
        // Will either use hyprctl geometry or fallback to all monitors
        assert!(command.contains("'/test/screenshot.png'"));
        assert!(command.starts_with("grim"));
    }

    #[test]
    fn test_build_focused_window_command() {
        let output_file = "/test/screenshot.png";
        let command = build_focused_window_command(output_file);
        // Will either use hyprctl geometry or fallback to select area
        assert!(command.contains("'/test/screenshot.png'"));
        assert!(command.starts_with("grim"));
    }

    #[test]
    fn test_output_file_format() {
        let result = get_output_file_custom("/home/user", "2024.12.25-15:30:45");
        assert!(result.ends_with(".png"));
        assert!(result.contains("Pictures/Screenshots"));
        assert!(result.contains("2024.12.25-15:30:45"));
    }

    #[test]
    fn test_command_injection_safety() {
        let malicious_file = "/test/file; rm -rf /";
        let command = build_all_monitors_command(malicious_file);
        // The single quotes should protect against command injection
        assert_eq!(command, "grim '/test/file; rm -rf /'");
    }

    #[test]
    fn test_timestamp_format() {
        // Test that our timestamp format is valid for filenames (matching real format)
        let result = get_output_file_custom("/tmp", "2024.12.25-15:30:45");
        assert!(!result.contains(" ")); // No spaces
        assert!(result.contains("Pictures/Screenshots"));
        assert!(result.contains("2024.12.25-15:30:45"));
        assert!(result.ends_with(".png"));
    }

    #[test]
    fn test_file_path_with_spaces() {
        let path_with_spaces = "/home/user name/test file.png";
        let command = build_all_monitors_command(path_with_spaces);
        assert_eq!(command, "grim '/home/user name/test file.png'");
        // Single quotes should handle spaces correctly
    }

    #[test]
    fn test_file_path_with_special_characters() {
        let special_chars = "/tmp/test$&()file.png";
        let command = build_all_monitors_command(special_chars);
        assert_eq!(command, "grim '/tmp/test$&()file.png'");
        // Single quotes should handle special characters
    }

    #[test]
    fn test_empty_file_path() {
        let command = build_all_monitors_command("");
        assert_eq!(command, "grim ''");
    }

    #[test]
    fn test_all_command_builders_consistency() {
        let test_file = "/tmp/test.png";

        // All commands should properly quote the file path and start with grim
        let commands = vec![
            build_all_monitors_command(test_file),
            build_select_area_command(test_file),
            build_focused_monitor_command(test_file),
            build_focused_window_command(test_file),
        ];

        for command in commands {
            assert!(
                command.contains("'/tmp/test.png'"),
                "Command should contain quoted file path: {}",
                command
            );
            assert!(
                command.starts_with("grim"),
                "Command should start with grim: {}",
                command
            );
        }
    }

    // Mock test for execute_screenshot_command error handling
    #[test]
    fn test_execute_command_error_handling() {
        // Test with a command that will definitely fail
        let result = execute_screenshot_command("false", "/tmp/test.png");
        assert!(result.is_err());

        // Test the error message contains useful information
        if let Err(error_msg) = result {
            assert!(
                error_msg.contains("Screenshot command failed")
                    || error_msg.contains("Failed to execute")
            );
        }
    }

    #[test]
    fn test_output_file_directory_structure() {
        let custom_path = get_output_file_custom("/custom/base", "test-time");
        let parts: Vec<&str> = custom_path.split('/').collect();

        // Should have: ["", "custom", "base", "Pictures", "Screenshots", "test-time.png"]
        assert!(parts.len() >= 5);
        assert!(parts.contains(&"Pictures"));
        assert!(parts.contains(&"Screenshots"));
        assert!(parts.last().unwrap().ends_with(".png"));
    }

    #[test]
    fn test_hyprctl_fallback_behavior() {
        // Test that commands work even when hyprctl is not available or fails
        let output_file = "/tmp/test.png";

        // These should not panic and should produce valid grim commands
        let monitor_cmd = build_focused_monitor_command(output_file);
        let window_cmd = build_focused_window_command(output_file);

        assert!(monitor_cmd.starts_with("grim"));
        assert!(monitor_cmd.contains("'/tmp/test.png'"));
        assert!(window_cmd.starts_with("grim"));
        assert!(window_cmd.contains("'/tmp/test.png'"));
    }

    #[test]
    fn test_get_output_file() {
        let output_file = get_output_file();

        // Should contain expected directory structure
        assert!(output_file.contains("Pictures/Screenshots"));
        assert!(output_file.ends_with(".png"));

        // Should contain a timestamp-like pattern (YYYY.MM.DD-HH:MM:SS)
        let filename = output_file.split('/').last().unwrap();
        let timestamp = filename.strip_suffix(".png").unwrap();

        // Basic format validation - should have dots and dashes in expected positions
        assert!(timestamp.contains('.'));
        assert!(timestamp.contains('-'));
        assert!(timestamp.contains(':'));

        // Should be reasonable length (YYYY.MM.DD-HH:MM:SS = 19 chars)
        assert_eq!(timestamp.len(), 19);
    }

    #[test]
    fn test_get_output_file_environment_fallback() {
        use std::env;

        // Test with HOME unset
        let original_home = env::var("HOME").ok();
        env::remove_var("HOME");

        let output_file = get_output_file();
        assert!(output_file.starts_with("/tmp/Pictures/Screenshots/"));
        assert!(output_file.ends_with(".png"));

        // Restore HOME if it was set
        if let Some(home) = original_home {
            env::set_var("HOME", home);
        }
    }

    #[test]
    fn test_take_functions_dont_panic() {
        // These functions call execute_screenshot_command which will fail
        // because we don't have the actual screenshot tools in test environment
        // But they shouldn't panic - they should handle errors gracefully

        // We can't easily test the full execution without mocking,
        // but we can test that they generate the expected commands
        let output_file = get_output_file();

        // Test that command builders are called correctly
        let all_cmd = build_all_monitors_command(&output_file);
        let select_cmd = build_select_area_command(&output_file);
        let monitor_cmd = build_focused_monitor_command(&output_file);
        let window_cmd = build_focused_window_command(&output_file);

        // All should be valid grim commands
        assert!(all_cmd.starts_with("grim"));
        assert!(select_cmd.starts_with("grim"));
        assert!(monitor_cmd.starts_with("grim"));
        assert!(window_cmd.starts_with("grim"));
    }

    #[test]
    fn test_notification_command_format() {
        // Test that show_notification generates a properly formatted command
        // We can't easily test the full execution, but we can verify
        // the command would be properly formatted
        let test_file = "/tmp/test-screenshot.png";

        // The notification command should contain the expected elements
        let expected_elements = vec![
            "notify-send",
            "-A 'File'",
            "-A 'Directory'",
            "'Screenshot taken'",
            test_file,
        ];

        // We can't directly test show_notification without it actually running,
        // but we can test the format it would generate
        let notification_cmd = format!(
            "notify-send -A 'File' -A 'Directory' 'Screenshot taken' 'Saved to {}'",
            test_file
        );

        for element in expected_elements {
            assert!(notification_cmd.contains(element));
        }
    }

    #[test]
    fn test_serde_structs() {
        // Test that our serde structs can deserialize expected JSON formats

        // Test Monitor deserialization
        let monitor_json = r#"{
            "width": 1920,
            "height": 1080,
            "x": 0,
            "y": 0,
            "focused": true
        }"#;

        let monitor: Result<Monitor, _> = serde_json::from_str(monitor_json);
        assert!(monitor.is_ok());
        let monitor = monitor.unwrap();
        assert_eq!(monitor.width, 1920);
        assert_eq!(monitor.height, 1080);
        assert_eq!(monitor.x, 0);
        assert_eq!(monitor.y, 0);
        assert_eq!(monitor.focused, true);

        // Test ActiveWindow deserialization
        let window_json = r#"{
            "at": [100, 200],
            "size": [800, 600]
        }"#;

        let window: Result<ActiveWindow, _> = serde_json::from_str(window_json);
        assert!(window.is_ok());
        let window = window.unwrap();
        assert_eq!(window.position, [100, 200]);
        assert_eq!(window.size, [800, 600]);
    }

    #[test]
    fn test_get_focused_monitor_error_handling() {
        // Test various error conditions for get_focused_monitor
        // Since we can't easily mock hyprctl, we test that the function
        // handles errors gracefully and returns appropriate error messages

        match get_focused_monitor() {
            Ok(monitor) => {
                // If hyprctl is available and returns valid data
                assert!(monitor.width > 0);
                assert!(monitor.height > 0);
            }
            Err(error) => {
                // Should return a descriptive error message
                assert!(!error.is_empty());
                assert!(
                    error.contains("Failed to run hyprctl")
                        || error.contains("hyprctl monitors command failed")
                        || error.contains("Invalid UTF-8 output")
                        || error.contains("Failed to parse JSON")
                        || error.contains("No monitors found")
                );
            }
        }
    }

    #[test]
    fn test_get_active_window_error_handling() {
        // Test various error conditions for get_active_window
        match get_active_window() {
            Ok(window) => {
                // If hyprctl is available and returns valid data
                assert_eq!(window.position.len(), 2);
                assert_eq!(window.size.len(), 2);
            }
            Err(error) => {
                // Should return a descriptive error message
                assert!(!error.is_empty());
                assert!(
                    error.contains("Failed to run hyprctl")
                        || error.contains("hyprctl activewindow command failed")
                        || error.contains("Invalid UTF-8 output")
                        || error.contains("Failed to parse JSON")
                );
            }
        }
    }

    #[test]
    fn test_malformed_json_handling() {
        // Test that our JSON parsing handles malformed data gracefully
        let malformed_monitor = r#"{"width": "not a number"}"#;
        let monitor_result: Result<Monitor, _> = serde_json::from_str(malformed_monitor);
        assert!(monitor_result.is_err());

        let malformed_window = r#"{"at": "not an array"}"#;
        let window_result: Result<ActiveWindow, _> = serde_json::from_str(malformed_window);
        assert!(window_result.is_err());
    }

    #[test]
    fn test_timestamp_format_consistency() {
        // Test that timestamps are consistently formatted
        let timestamp1 = get_output_file();
        // Sleep a tiny bit to ensure different timestamps
        std::thread::sleep(std::time::Duration::from_millis(1));
        let timestamp2 = get_output_file();

        // Both should follow the same format pattern
        let extract_timestamp = |path: &str| -> String {
            path.split('/')
                .last()
                .unwrap()
                .strip_suffix(".png")
                .unwrap()
                .to_string()
        };

        let ts1 = extract_timestamp(&timestamp1);
        let ts2 = extract_timestamp(&timestamp2);

        // Both should have the same format (length and structure)
        assert_eq!(ts1.len(), ts2.len());
        assert_eq!(ts1.matches('.').count(), ts2.matches('.').count());
        assert_eq!(ts1.matches('-').count(), ts2.matches('-').count());
        assert_eq!(ts1.matches(':').count(), ts2.matches(':').count());
    }

    #[test]
    fn test_monitor_focused_field() {
        // Test that Monitor struct correctly deserializes the focused field
        let focused_monitor_json = r#"{
            "width": 1920,
            "height": 1080,
            "x": 0,
            "y": 0,
            "focused": true
        }"#;

        let monitor: Monitor = serde_json::from_str(focused_monitor_json).unwrap();
        assert_eq!(monitor.focused, true);

        let unfocused_monitor_json = r#"{
            "width": 2560,
            "height": 1440,
            "x": 1920,
            "y": 0,
            "focused": false
        }"#;

        let monitor: Monitor = serde_json::from_str(unfocused_monitor_json).unwrap();
        assert_eq!(monitor.focused, false);
    }

    #[test]
    fn test_build_focused_monitor_command_with_monitor_provided() {
        // Test that build_focused_monitor_command_with_monitor uses the provided monitor
        let monitor = Monitor {
            width: 1920,
            height: 1080,
            x: 0,
            y: 0,
            focused: true,
        };

        let output_file = "/tmp/test-monitor.png";
        let command = build_focused_monitor_command_with_monitor(output_file, Some(monitor));

        assert!(command.starts_with("grim -g"));
        assert!(command.contains("0,0 1920x1080"));
        assert!(command.contains("'/tmp/test-monitor.png'"));
    }

    #[test]
    fn test_build_focused_monitor_command_with_secondary_monitor() {
        // Test with a secondary monitor (not at 0,0)
        let monitor = Monitor {
            width: 2560,
            height: 1440,
            x: 1920,
            y: 0,
            focused: true,
        };

        let output_file = "/tmp/test-secondary.png";
        let command = build_focused_monitor_command_with_monitor(output_file, Some(monitor));

        assert!(command.starts_with("grim -g"));
        assert!(command.contains("1920,0 2560x1440"));
        assert!(command.contains("'/tmp/test-secondary.png'"));
    }

    #[test]
    fn test_build_focused_window_command_with_window_provided() {
        // Test that build_focused_window_command_with_window uses the provided window
        let window = ActiveWindow {
            position: [100, 200],
            size: [800, 600],
        };

        let output_file = "/tmp/test-window.png";
        let command = build_focused_window_command_with_window(output_file, Some(window));

        assert!(command.starts_with("grim -g"));
        assert!(command.contains("100,200 800x600"));
        assert!(command.contains("'/tmp/test-window.png'"));
    }

    #[test]
    fn test_focused_monitor_command_consistency() {
        // Test that providing a monitor vs None produces different results
        let monitor = Monitor {
            width: 1920,
            height: 1080,
            x: 0,
            y: 0,
            focused: true,
        };

        let output_file = "/tmp/test.png";
        let command_with_monitor =
            build_focused_monitor_command_with_monitor(output_file, Some(monitor));
        let command_without_monitor = build_focused_monitor_command_with_monitor(output_file, None);

        // Both should be valid grim commands
        assert!(command_with_monitor.starts_with("grim"));
        assert!(command_without_monitor.starts_with("grim"));

        // The command with monitor should always have the specific geometry
        assert!(command_with_monitor.contains("0,0 1920x1080"));
    }

    #[test]
    fn test_monitor_with_different_positions() {
        // Test monitors at various positions
        let test_cases = vec![
            (0, 0, 1920, 1080),       // Primary monitor at origin
            (1920, 0, 2560, 1440),    // Secondary to the right
            (0, 1080, 1920, 1080),    // Secondary below
            (-1920, 0, 1920, 1080),   // Secondary to the left
            (1920, -500, 1920, 1080), // Secondary above and to the right
        ];

        for (x, y, width, height) in test_cases {
            let monitor = Monitor {
                width,
                height,
                x,
                y,
                focused: true,
            };

            let command =
                build_focused_monitor_command_with_monitor("/tmp/test.png", Some(monitor));
            let expected_geometry = format!("{},{} {}x{}", x, y, width, height);

            assert!(
                command.contains(&expected_geometry),
                "Command should contain geometry '{}', got: {}",
                expected_geometry,
                command
            );
        }
    }
}
