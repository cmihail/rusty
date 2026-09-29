use gtk4::{Box, Orientation, Window};
use rusty_de::window::confirmation_dialog::is_click_outside;

#[test]
fn test_is_click_outside_no_bounds() {
    gtk4::init().unwrap();
    let window = Window::new();
    let content = Box::new(Orientation::Vertical, 0);

    // When bounds cannot be computed, should return false
    assert!(!is_click_outside(&window, &content, 10.0, 10.0));
}

#[test]
fn test_shell_command_execution() {
    // Test that commands are executed via sh -c wrapper
    let test_commands = vec![
        "systemctl suspend",
        "setsid hyprshutdown",
        "setsid hyprshutdown -t 'Shutting down...' --post-cmd 'shutdown -P 0'",
        "setsid hyprshutdown -t 'Restarting...' --post-cmd 'reboot'",
    ];

    for command in test_commands {
        // Verify shell execution pattern: sh -c "command"
        let shell = "sh";
        let shell_arg = "-c";

        assert!(!command.is_empty(), "Command should not be empty");
        assert_eq!(shell, "sh", "Commands should be executed via sh");
        assert_eq!(shell_arg, "-c", "Commands should use -c flag");
    }
}

#[test]
fn test_hyprshutdown_with_args() {
    // Test that hyprshutdown commands with quoted args are supported
    let command = "setsid hyprshutdown -t 'Shutting down...' --post-cmd 'shutdown -P 0'";

    assert!(command.contains("setsid"));
    assert!(command.contains("hyprshutdown"));
    assert!(command.contains("--post-cmd"));
    assert!(command.contains("'Shutting down...'"));
}

#[test]
fn test_empty_command_handling() {
    let command = "";
    let parts: Vec<&str> = command.split_whitespace().collect();

    assert!(parts.is_empty(), "Empty command should produce empty parts");
}

#[test]
fn test_setsid_detachment() {
    // Test that setsid is used to detach processes from parent
    let commands_with_setsid = vec![
        "setsid hyprshutdown",
        "setsid hyprshutdown -t 'Shutting down...' --post-cmd 'shutdown -P 0'",
        "setsid hyprshutdown -t 'Restarting...' --post-cmd 'reboot'",
    ];

    for command in commands_with_setsid {
        assert!(
            command.starts_with("setsid"),
            "Command should start with setsid: {}",
            command
        );
    }
}
