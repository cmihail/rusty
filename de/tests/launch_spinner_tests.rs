use rusty_de::service::desktop_entry::DesktopEntry;
use rusty_de::window::launch_spinner::{hides_on, launch_label};

#[test]
fn test_hides_on_open_window_only() {
    assert!(hides_on("openwindow"));
    assert!(!hides_on("closewindow"));
    assert!(!hides_on("activewindow"));
}

#[test]
fn test_launch_label_prefers_the_entry_name() {
    let entry = DesktopEntry {
        name: Some("Firefox".to_string()),
        ..Default::default()
    };
    assert_eq!(launch_label("firefox.desktop", Some(&entry)), "Firefox");
}

#[test]
fn test_launch_label_falls_back_to_the_desktop_id() {
    assert_eq!(
        launch_label("org.gnome.Nautilus.desktop", None),
        "org.gnome.Nautilus"
    );
    assert_eq!(
        launch_label("kitty", Some(&DesktopEntry::default())),
        "kitty"
    );
}
