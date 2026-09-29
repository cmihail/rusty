use std::process::{Command, Stdio};

/// Internal flag: the GUI hands blocking notifications to a detached copy of
/// itself, so the process can exit while the notification stays clickable.
pub const NOTIFY_FLAG: &str = "--notify";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationKind {
    Screenshot,
    RecordingStarted,
    RecordingStopped,
}

impl NotificationKind {
    pub fn as_arg(self) -> &'static str {
        match self {
            NotificationKind::Screenshot => "screenshot",
            NotificationKind::RecordingStarted => "recording-started",
            NotificationKind::RecordingStopped => "recording-stopped",
        }
    }

    pub fn from_arg(arg: &str) -> Option<Self> {
        match arg {
            "screenshot" => Some(NotificationKind::Screenshot),
            "recording-started" => Some(NotificationKind::RecordingStarted),
            "recording-stopped" => Some(NotificationKind::RecordingStopped),
            _ => None,
        }
    }
}

/// Parsed out of argv by main() before GTK starts, so GApplication never sees
/// the flag. Expects argv[0] to already be stripped.
pub fn parse_notify_args<I: IntoIterator<Item = String>>(
    args: I,
) -> Option<(NotificationKind, String)> {
    let args: Vec<String> = args.into_iter().collect();
    match args.as_slice() {
        [flag, kind, output_file] if flag == NOTIFY_FLAG => {
            NotificationKind::from_arg(kind).map(|kind| (kind, output_file.clone()))
        }
        _ => None,
    }
}

/// Fire-and-forget: re-exec this binary to own the blocking notify-send.
pub fn spawn_notification(kind: NotificationKind, output_file: &str) {
    let spawned = std::env::current_exe().and_then(|exe| {
        Command::new(exe)
            .arg(NOTIFY_FLAG)
            .arg(kind.as_arg())
            .arg(output_file)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
    });

    if spawned.is_err() {
        run_notification(kind, output_file);
    }
}

/// Runs in the child process; blocks until the notification is answered or
/// dismissed, then performs the chosen action.
pub fn run_notification(kind: NotificationKind, output_file: &str) {
    match kind {
        NotificationKind::Screenshot => crate::screenshot::show_notification(output_file),
        NotificationKind::RecordingStarted => {
            crate::recording::show_recording_started_notification(output_file)
        }
        NotificationKind::RecordingStopped => {
            crate::recording::show_recording_stopped_notification(output_file)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn test_notification_kind_arg_roundtrip() {
        let kinds = [
            NotificationKind::Screenshot,
            NotificationKind::RecordingStarted,
            NotificationKind::RecordingStopped,
        ];

        for kind in kinds {
            assert_eq!(NotificationKind::from_arg(kind.as_arg()), Some(kind));
        }
    }

    #[test]
    fn test_notification_kind_arg_values() {
        assert_eq!(NotificationKind::Screenshot.as_arg(), "screenshot");
        assert_eq!(
            NotificationKind::RecordingStarted.as_arg(),
            "recording-started"
        );
        assert_eq!(
            NotificationKind::RecordingStopped.as_arg(),
            "recording-stopped"
        );
    }

    #[test]
    fn test_notification_kind_from_unknown_arg() {
        assert_eq!(NotificationKind::from_arg("nonsense"), None);
        assert_eq!(NotificationKind::from_arg(""), None);
    }

    #[test]
    fn test_parse_notify_args_recognizes_each_kind() {
        assert_eq!(
            parse_notify_args(args(&["--notify", "screenshot", "/tmp/a.png"])),
            Some((NotificationKind::Screenshot, "/tmp/a.png".to_string()))
        );
        assert_eq!(
            parse_notify_args(args(&["--notify", "recording-started", "/tmp/a.mp4"])),
            Some((NotificationKind::RecordingStarted, "/tmp/a.mp4".to_string()))
        );
        assert_eq!(
            parse_notify_args(args(&["--notify", "recording-stopped", "/tmp/a.mp4"])),
            Some((NotificationKind::RecordingStopped, "/tmp/a.mp4".to_string()))
        );
    }

    #[test]
    fn test_parse_notify_args_without_arguments() {
        assert_eq!(parse_notify_args(args(&[])), None);
    }

    #[test]
    fn test_parse_notify_args_unknown_kind() {
        assert_eq!(
            parse_notify_args(args(&["--notify", "bogus", "/tmp/a"])),
            None
        );
    }

    #[test]
    fn test_parse_notify_args_missing_file() {
        assert_eq!(parse_notify_args(args(&["--notify", "screenshot"])), None);
        assert_eq!(parse_notify_args(args(&["--notify"])), None);
    }

    #[test]
    fn test_parse_notify_args_ignores_other_arguments() {
        assert_eq!(parse_notify_args(args(&["--help"])), None);
        assert_eq!(
            parse_notify_args(args(&["--notify", "screenshot", "/tmp/a.png", "extra"])),
            None
        );
    }

    #[test]
    fn test_parse_notify_args_path_with_spaces() {
        let path = "/home/user name/test file.png";
        assert_eq!(
            parse_notify_args(args(&["--notify", "screenshot", path])),
            Some((NotificationKind::Screenshot, path.to_string()))
        );
    }

    #[test]
    fn test_current_exe_is_available_for_spawning() {
        // spawn_notification itself is not called here: under `cargo test` the
        // re-exec target is the test binary, not the app.
        assert!(std::env::current_exe().is_ok());
    }
}
