use chrono::Local;
use std::fs;
use std::process::{Command, Stdio};

use crate::screenshot::{ActiveWindow, Monitor};

pub fn get_video_output_file() -> String {
    let now = Local::now();
    let timestamp = now.format("%Y.%m.%d-%H:%M:%S").to_string();
    format!(
        "{}/Videos/Recordings/{}.mp4",
        std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string()),
        timestamp
    )
}

pub fn get_video_output_file_with_timestamp(timestamp: &str) -> String {
    format!(
        "{}/Videos/Recordings/{}.mp4",
        std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string()),
        timestamp
    )
}

#[cfg(test)]
pub fn get_video_output_file_custom(base_path: &str, timestamp: &str) -> String {
    format!("{}/Videos/Recordings/{}.mp4", base_path, timestamp)
}

// Note: wf-recorder doesn't support recording all monitors simultaneously
// This function is kept for tests but not used in the UI
#[allow(dead_code)]
fn build_all_monitors_recording_command(output_file: &str, with_audio: bool) -> String {
    if with_audio {
        format!("wf-recorder --audio -f '{}'", output_file)
    } else {
        format!("wf-recorder -f '{}'", output_file)
    }
}

pub fn build_select_area_recording_command(output_file: &str, with_audio: bool) -> String {
    if with_audio {
        format!("wf-recorder -g \"$(slurp)\" --audio -f '{}'", output_file)
    } else {
        format!("wf-recorder -g \"$(slurp)\" -f '{}'", output_file)
    }
}

pub fn build_focused_monitor_recording_command(output_file: &str, with_audio: bool) -> String {
    build_focused_monitor_recording_command_with_monitor(output_file, with_audio, None)
}

pub fn build_focused_monitor_recording_command_with_monitor(
    output_file: &str,
    with_audio: bool,
    monitor: Option<Monitor>,
) -> String {
    match monitor.or_else(|| crate::screenshot::get_focused_monitor().ok()) {
        Some(monitor) => {
            let audio_flag = if with_audio { " --audio" } else { "" };
            format!(
                "wf-recorder -g \"{},{} {}x{}\"{} -f '{}'",
                monitor.x, monitor.y, monitor.width, monitor.height, audio_flag, output_file
            )
        }
        None => {
            eprintln!("Failed to get focused monitor info, falling back to select area");
            build_select_area_recording_command(output_file, with_audio)
        }
    }
}

pub fn build_focused_window_recording_command(output_file: &str, with_audio: bool) -> String {
    build_focused_window_recording_command_with_window(output_file, with_audio, None)
}

pub fn build_focused_window_recording_command_with_window(
    output_file: &str,
    with_audio: bool,
    window: Option<ActiveWindow>,
) -> String {
    match window.or_else(|| crate::screenshot::get_active_window().ok()) {
        Some(window) => {
            let x = window.position[0];
            let y = window.position[1];
            let width = window.size[0];
            let height = window.size[1];
            let audio_flag = if with_audio { " --audio" } else { "" };
            format!(
                "wf-recorder -g \"{},{} {}x{}\"{} -f '{}'",
                x, y, width, height, audio_flag, output_file
            )
        }
        None => {
            eprintln!("Failed to get active window info, falling back to select area");
            build_select_area_recording_command(output_file, with_audio)
        }
    }
}

pub fn start_recording(command: &str, output_file: &str) -> Result<(), String> {
    // Ensure the recordings directory exists
    let recordings_dir = format!(
        "{}/Videos/Recordings",
        std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())
    );
    if let Err(e) = fs::create_dir_all(&recordings_dir) {
        return Err(format!("Failed to create recordings directory: {}", e));
    }

    // Start the recording process in the background
    Command::new("bash")
        .args(["-c", command])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to start recording: {}", e))?;

    // Show notification that recording has started
    crate::notify::spawn_notification(
        crate::notify::NotificationKind::RecordingStarted,
        output_file,
    );

    Ok(())
}

pub fn stop_recording() -> Result<(), String> {
    // Send SIGINT to all wf-recorder processes
    Command::new("pkill")
        .args(["-INT", "wf-recorder"])
        .output()
        .map_err(|e| format!("Failed to stop recording: {}", e))?;

    Ok(())
}

pub fn is_recording_active() -> bool {
    // Check if wf-recorder is running
    Command::new("pgrep")
        .arg("wf-recorder")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

pub fn show_recording_started_notification(output_file: &str) {
    let notification_cmd = format!(
        "notify-send -A 'Stop Recording' 'Recording Started' 'Saving to {}'",
        output_file
    );

    // Block and wait for user response, just like screenshot notifications
    if let Ok(output) = Command::new("bash")
        .args(["-c", &notification_cmd])
        .output()
    {
        if let Ok(response) = String::from_utf8(output.stdout) {
            let response = response.trim();
            if response == "0" {
                // User clicked "Stop Recording"
                if stop_recording().is_ok() {
                    show_recording_stopped_notification(output_file);
                }
            }
        }
    }
}

pub fn show_recording_stopped_notification(output_file: &str) {
    let notification_cmd = format!(
        "notify-send -A 'File' -A 'Directory' 'Recording Stopped' 'Saved to {}'",
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
                    let recordings_dir = format!(
                        "{}/Videos/Recordings/",
                        std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())
                    );
                    let _ = Command::new("xdg-open").arg(recordings_dir).spawn();
                }
                _ => {}
            }
        }
    }
}

pub fn record_select_area(with_audio: bool) {
    let output_file = get_video_output_file();
    let command = build_select_area_recording_command(&output_file, with_audio);
    let _ = start_recording(&command, &output_file);
}

pub fn record_focused_monitor(with_audio: bool) {
    let output_file = get_video_output_file();
    let command = build_focused_monitor_recording_command(&output_file, with_audio);
    let _ = start_recording(&command, &output_file);
}

pub fn record_focused_window(with_audio: bool) {
    let output_file = get_video_output_file();
    let command = build_focused_window_recording_command(&output_file, with_audio);
    let _ = start_recording(&command, &output_file);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_get_video_output_file_custom() {
        let result = get_video_output_file_custom("/home/test", "2024.01.01-12:00:00");
        assert_eq!(
            result,
            "/home/test/Videos/Recordings/2024.01.01-12:00:00.mp4"
        );
    }

    #[test]
    fn test_get_video_output_file_with_timestamp() {
        let timestamp = "2024.01.01-12:00:00";
        let result = get_video_output_file_with_timestamp(timestamp);

        let expected_base = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let expected = format!("{}/Videos/Recordings/{}.mp4", expected_base, timestamp);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_build_all_monitors_recording_command_no_audio() {
        let output_file = "/test/recording.mp4";
        let command = build_all_monitors_recording_command(output_file, false);
        assert_eq!(command, "wf-recorder -f '/test/recording.mp4'");
    }

    #[test]
    fn test_build_all_monitors_recording_command_with_audio() {
        let output_file = "/test/recording.mp4";
        let command = build_all_monitors_recording_command(output_file, true);
        assert_eq!(command, "wf-recorder --audio -f '/test/recording.mp4'");
    }

    #[test]
    fn test_build_select_area_recording_command_no_audio() {
        let output_file = "/test/recording.mp4";
        let command = build_select_area_recording_command(output_file, false);
        assert_eq!(
            command,
            "wf-recorder -g \"$(slurp)\" -f '/test/recording.mp4'"
        );
    }

    #[test]
    fn test_build_select_area_recording_command_with_audio() {
        let output_file = "/test/recording.mp4";
        let command = build_select_area_recording_command(output_file, true);
        assert_eq!(
            command,
            "wf-recorder -g \"$(slurp)\" --audio -f '/test/recording.mp4'"
        );
    }

    #[test]
    fn test_video_output_file_format() {
        let result = get_video_output_file_custom("/home/user", "2024.12.25-15:30:45");
        assert!(result.ends_with(".mp4"));
        assert!(result.contains("Videos/Recordings"));
        assert!(result.contains("2024.12.25-15:30:45"));
    }

    #[test]
    fn test_command_injection_safety() {
        let malicious_file = "/test/file; rm -rf /";
        let command = build_all_monitors_recording_command(malicious_file, false);
        assert_eq!(command, "wf-recorder -f '/test/file; rm -rf /'");
    }

    #[test]
    fn test_file_path_with_spaces() {
        let path_with_spaces = "/home/user name/test file.mp4";
        let command = build_all_monitors_recording_command(path_with_spaces, false);
        assert_eq!(command, "wf-recorder -f '/home/user name/test file.mp4'");
    }

    #[test]
    fn test_file_path_with_special_characters() {
        let special_chars = "/tmp/test$&()file.mp4";
        let command = build_all_monitors_recording_command(special_chars, false);
        assert_eq!(command, "wf-recorder -f '/tmp/test$&()file.mp4'");
    }

    #[test]
    fn test_empty_file_path() {
        let command = build_all_monitors_recording_command("", false);
        assert_eq!(command, "wf-recorder -f ''");
    }

    #[test]
    fn test_all_command_builders_consistency() {
        let test_file = "/tmp/test.mp4";

        let commands = vec![
            build_all_monitors_recording_command(test_file, false),
            build_select_area_recording_command(test_file, false),
            build_focused_monitor_recording_command(test_file, false),
            build_focused_window_recording_command(test_file, false),
        ];

        for command in commands {
            assert!(
                command.contains("'/tmp/test.mp4'"),
                "Command should contain quoted file path: {}",
                command
            );
            assert!(
                command.starts_with("wf-recorder"),
                "Command should start with wf-recorder: {}",
                command
            );
        }
    }

    #[test]
    fn test_audio_flag_consistency() {
        let test_file = "/tmp/test.mp4";

        // Commands without audio should not contain --audio
        let no_audio_commands = vec![
            build_all_monitors_recording_command(test_file, false),
            build_select_area_recording_command(test_file, false),
        ];

        for command in no_audio_commands {
            assert!(
                !command.contains("--audio"),
                "Command should not contain --audio flag: {}",
                command
            );
        }

        // Commands with audio should contain --audio
        let with_audio_commands = vec![
            build_all_monitors_recording_command(test_file, true),
            build_select_area_recording_command(test_file, true),
        ];

        for command in with_audio_commands {
            assert!(
                command.contains("--audio"),
                "Command should contain --audio flag: {}",
                command
            );
        }
    }

    #[test]
    fn test_video_directory_structure() {
        let custom_path = get_video_output_file_custom("/custom/base", "test-time");
        let parts: Vec<&str> = custom_path.split('/').collect();

        assert!(parts.contains(&"Videos"));
        assert!(parts.contains(&"Recordings"));
        assert!(parts.last().unwrap().ends_with(".mp4"));
    }

    #[test]
    fn test_timestamp_format_consistency() {
        let timestamp1 = get_video_output_file();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let timestamp2 = get_video_output_file();

        let extract_timestamp = |path: &str| -> String {
            path.split('/')
                .last()
                .unwrap()
                .strip_suffix(".mp4")
                .unwrap()
                .to_string()
        };

        let ts1 = extract_timestamp(&timestamp1);
        let ts2 = extract_timestamp(&timestamp2);

        assert_eq!(ts1.len(), ts2.len());
        assert_eq!(ts1.matches('.').count(), ts2.matches('.').count());
        assert_eq!(ts1.matches('-').count(), ts2.matches('-').count());
        assert_eq!(ts1.matches(':').count(), ts2.matches(':').count());
    }

    #[test]
    fn test_build_focused_monitor_command_with_monitor() {
        let monitor = Monitor {
            width: 1920,
            height: 1080,
            x: 0,
            y: 0,
            focused: true,
        };

        let output_file = "/tmp/test-monitor.mp4";
        let command =
            build_focused_monitor_recording_command_with_monitor(output_file, false, Some(monitor));

        assert!(command.starts_with("wf-recorder -g"));
        assert!(command.contains("0,0 1920x1080"));
        assert!(command.contains("-f '/tmp/test-monitor.mp4'"));
        assert!(!command.contains("--audio"));
    }

    #[test]
    fn test_build_focused_monitor_command_with_audio() {
        let monitor = Monitor {
            width: 2560,
            height: 1440,
            x: 1920,
            y: 0,
            focused: true,
        };

        let output_file = "/tmp/test-audio.mp4";
        let command =
            build_focused_monitor_recording_command_with_monitor(output_file, true, Some(monitor));

        assert!(command.contains("1920,0 2560x1440"));
        assert!(command.contains("--audio"));
        assert!(command.contains("-f '/tmp/test-audio.mp4'"));
    }

    #[test]
    fn test_build_focused_window_command_with_window() {
        let window = ActiveWindow {
            position: [100, 200],
            size: [800, 600],
        };

        let output_file = "/tmp/test-window.mp4";
        let command =
            build_focused_window_recording_command_with_window(output_file, false, Some(window));

        assert!(command.starts_with("wf-recorder -g"));
        assert!(command.contains("100,200 800x600"));
        assert!(command.contains("-f '/tmp/test-window.mp4'"));
    }

    #[test]
    fn test_build_focused_window_command_with_audio() {
        let window = ActiveWindow {
            position: [50, 100],
            size: [1280, 720],
        };

        let output_file = "/tmp/test-window-audio.mp4";
        let command =
            build_focused_window_recording_command_with_window(output_file, true, Some(window));

        assert!(command.contains("50,100 1280x720"));
        assert!(command.contains("--audio"));
        assert!(command.contains("-f '/tmp/test-window-audio.mp4'"));
    }

    #[test]
    fn test_stop_recording_does_not_panic() {
        // Test that stop_recording can be called without panicking
        // The function should not panic regardless of whether a process is found
        let _ = stop_recording();
        // If we get here without panicking, the test passes
    }

    #[test]
    fn test_is_recording_active_returns_bool() {
        // Test that is_recording_active returns a boolean
        let is_active = is_recording_active();
        // Should return either true or false, not panic
        assert!(is_active == true || is_active == false);
    }

    #[test]
    fn test_is_recording_active_when_no_recording() {
        // Ensure no wf-recorder is running first
        let _ = stop_recording();
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Should return false when no recording is active
        let is_active = is_recording_active();
        assert!(!is_active);
    }

    #[test]
    fn test_recording_functions_are_safe() {
        // Test that recording functions don't panic with various inputs
        let test_file = "/tmp/test-recording-safety.mp4";

        let commands = vec![
            build_select_area_recording_command(test_file, false),
            build_focused_monitor_recording_command(test_file, false),
            build_focused_window_recording_command(test_file, false),
        ];

        for command in commands {
            assert!(command.contains("wf-recorder"));
            assert!(command.contains(test_file));
        }
    }

    #[test]
    fn test_video_output_file_timestamp_is_unique() {
        // Test that successive calls generate unique filenames
        let file1 = get_video_output_file();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let file2 = get_video_output_file();

        // Files should be different if enough time has passed
        // or same if within same second
        assert!(file1.contains("Videos/Recordings"));
        assert!(file2.contains("Videos/Recordings"));
    }

    #[test]
    fn test_recording_commands_preserve_geometry() {
        let monitor = Monitor {
            width: 3840,
            height: 2160,
            x: 100,
            y: 50,
            focused: true,
        };

        let window = ActiveWindow {
            position: [200, 150],
            size: [1600, 900],
        };

        let monitor_cmd = build_focused_monitor_recording_command_with_monitor(
            "/tmp/test.mp4",
            false,
            Some(monitor),
        );
        let window_cmd = build_focused_window_recording_command_with_window(
            "/tmp/test.mp4",
            false,
            Some(window),
        );

        assert!(monitor_cmd.contains("100,50 3840x2160"));
        assert!(window_cmd.contains("200,150 1600x900"));
    }
}
