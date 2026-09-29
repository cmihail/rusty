// Library interface for the screenshot application
// This allows integration tests to access the screenshot and recording functionality

pub mod notify;
pub mod recording;
pub mod screenshot;

// Re-export commonly used functions and types for easier access
pub use screenshot::{
    build_all_monitors_command, build_focused_monitor_command,
    build_focused_monitor_command_with_monitor, build_focused_window_command,
    build_focused_window_command_with_window, build_select_area_command,
    execute_screenshot_command, get_active_window, get_focused_monitor, show_notification,
    take_all_monitors_screenshot, take_focused_monitor_screenshot,
    take_focused_monitor_screenshot_with_monitor, take_focused_window_screenshot,
    take_focused_window_screenshot_with_window, take_select_area_screenshot, ActiveWindow, Monitor,
};

pub use notify::{parse_notify_args, spawn_notification, NotificationKind};

pub use recording::{
    build_focused_monitor_recording_command, build_focused_monitor_recording_command_with_monitor,
    build_focused_window_recording_command, build_focused_window_recording_command_with_window,
    build_select_area_recording_command, is_recording_active, record_focused_monitor,
    record_focused_window, record_select_area, show_recording_started_notification,
    show_recording_stopped_notification, start_recording, stop_recording,
};

// Re-export test-only functions when testing
#[cfg(test)]
pub use screenshot::{get_output_file_custom, get_output_file_with_timestamp};

#[cfg(test)]
pub use recording::{get_video_output_file_custom, get_video_output_file_with_timestamp};
