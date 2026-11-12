use rusty_de::service::desktop_entry::{self, DesktopEntry};
use rusty_de::service::hyprland::{Client, Workspace};
use rusty_de::window::dock::{dock_entries, in_dock_area, is_notified, notifications_of};
use rusty_de::window::dock::{handle_color, handle_curve, launcher_icon, track_urgent};
use rusty_de::window::dock::{workspace_badge, MINIMIZED_WORKSPACE};
use std::collections::HashSet;

#[test]
fn test_handle_curve_is_slim_at_the_edges_and_full_height_in_the_middle() {
    let [start, first, second, end] = handle_curve(300.0, 8.0);
    let at_half = |a: f64, b: f64, c: f64, d: f64| (a + 3.0 * b + 3.0 * c + d) / 8.0;

    assert_eq!(start, (0.0, 7.0));
    assert_eq!(end, (300.0, 7.0));
    assert_eq!(at_half(start.0, first.0, second.0, end.0), 150.0);
    assert!(at_half(start.1, first.1, second.1, end.1).abs() < 1e-9);
}

#[test]
fn test_is_notified_matches_class_ignoring_case() {
    let classes = ["Slack".to_string(), "slack_slack".to_string()];

    assert!(is_notified("slack", &classes));
    assert!(!is_notified("Alacritty", &classes));
    assert!(!is_notified("slack", &[]));
}

#[test]
fn test_notifications_of_picks_the_ids_matching_the_class() {
    let notifications = vec![
        (1, vec!["Slack".to_string(), "slack_slack".to_string()]),
        (2, vec!["brave".to_string()]),
        (3, vec!["slack".to_string()]),
    ];

    assert_eq!(notifications_of("slack", &notifications), vec![1, 3]);
    assert!(notifications_of("Alacritty", &notifications).is_empty());
}

#[test]
fn test_handle_color_alerts_for_any_notification_or_urgent_window() {
    assert_eq!(handle_color(true, 0, 0), "theme_selected_bg_color");
    assert_eq!(handle_color(true, 1, 0), "warning_color");
    assert_eq!(handle_color(true, 0, 1), "warning_color");
}

#[test]
fn test_handle_color_prefers_offline_over_alerts() {
    assert_eq!(handle_color(false, 0, 0), "error_color");
    assert_eq!(handle_color(false, 3, 2), "error_color");
}

#[test]
fn test_launcher_icon_prefers_app_grid_then_search() {
    assert_eq!(launcher_icon(|_| true), "view-app-grid-symbolic");
    assert_eq!(
        launcher_icon(|name| name == "system-search-symbolic"),
        "system-search-symbolic"
    );
    assert_eq!(launcher_icon(|_| false), "application-x-executable");
}

const MONITOR: (i32, i32, i32, i32) = (0, 0, 1920, 1200);

fn workspace(id: i32, name: &str) -> Workspace {
    Workspace {
        id,
        name: name.to_string(),
    }
}

#[test]
fn test_workspace_badge() {
    let badge = |id, name| workspace_badge(&workspace(id, name));

    assert_eq!(badge(1, "1").as_deref(), Some("1"));
    assert_eq!(badge(9, "9").as_deref(), Some("9"));
    assert_eq!(badge(10, "10").as_deref(), Some("+"));
    assert_eq!(badge(-99, "special:scratch").as_deref(), Some("S"));
    assert_eq!(badge(-98, MINIMIZED_WORKSPACE), None);
}

#[test]
fn test_track_urgent_until_focused_or_closed() {
    let mut urgent = HashSet::new();

    assert!(track_urgent(&mut urgent, "urgent", "a1"));
    assert!(track_urgent(&mut urgent, "urgent", "b2"));
    assert!(!track_urgent(&mut urgent, "urgent", "a1"));
    assert!(!track_urgent(&mut urgent, "openwindow", "a1"));
    assert!(track_urgent(&mut urgent, "activewindowv2", "a1"));
    assert!(!track_urgent(&mut urgent, "activewindowv2", "a1"));
    assert!(track_urgent(&mut urgent, "closewindow", "b2"));
    assert!(urgent.is_empty());
}

#[test]
fn test_in_dock_area_covers_the_dock() {
    assert!(in_dock_area((1919, 600), MONITOR, (56, 600)));
    assert!(in_dock_area((1864, 301), MONITOR, (56, 600)));
    assert!(!in_dock_area((1863, 600), MONITOR, (56, 600)));
    assert!(!in_dock_area((1919, 900), MONITOR, (56, 600)));
}

#[test]
fn test_in_dock_area_covers_the_strip_beside_a_short_dock() {
    // A 150px dock is shorter than the 400px strip that opened it.
    assert!(in_dock_area((1919, 401), MONITOR, (56, 150)));
    assert!(in_dock_area((1919, 799), MONITOR, (56, 150)));
    assert!(!in_dock_area((1919, 399), MONITOR, (56, 150)));
    assert!(!in_dock_area((1919, 800), MONITOR, (56, 150)));
}

#[test]
fn test_in_dock_area_uses_the_monitor_offset() {
    let second = (1920, 100, 2560, 1440);

    assert!(in_dock_area((4479, 820), second, (56, 300)));
    assert!(!in_dock_area((1919, 820), second, (56, 300)));
    assert!(!in_dock_area((4479, 100), second, (56, 300)));
}

#[test]
fn test_in_dock_area_before_the_dock_has_a_size() {
    assert!(in_dock_area((1919, 600), MONITOR, (0, 0)));
    assert!(!in_dock_area((1918, 600), MONITOR, (0, 0)));
}

fn client(address: &str, workspace_id: i32, workspace_name: &str, x: i32) -> Client {
    Client {
        address: address.to_string(),
        x,
        y: 0,
        width: 100,
        height: 100,
        title: address.to_string(),
        class: "app".to_string(),
        workspace: Workspace {
            id: workspace_id,
            name: workspace_name.to_string(),
        },
        monitor: 0,
        pid: 0,
    }
}

fn addresses(clients: Vec<Client>) -> Vec<String> {
    dock_entries(clients)
        .into_iter()
        .map(|entry| entry.client.address)
        .collect()
}

#[test]
fn test_dock_entries_sort_by_workspace_then_position() {
    let clients = vec![
        client("b", 2, "2", 0),
        client("a2", 1, "1", 500),
        client("a1", 1, "1", 0),
    ];

    assert_eq!(addresses(clients), vec!["a1", "a2", "b"]);
}

#[test]
fn test_dock_entries_put_minimized_last_and_mark_them() {
    let clients = vec![
        client("min", -98, MINIMIZED_WORKSPACE, 0),
        client("scratch", -99, "special:scratch", 0),
        client("normal", 3, "3", 0),
    ];

    let entries = dock_entries(clients);
    let order: Vec<&str> = entries.iter().map(|e| e.client.address.as_str()).collect();

    assert_eq!(order, vec!["normal", "scratch", "min"]);
    assert_eq!(
        entries.iter().map(|e| e.minimized).collect::<Vec<_>>(),
        vec![false, false, true]
    );
}

#[test]
fn test_parse_desktop_entry_icon() {
    let entry = desktop_entry::parse("[Desktop Entry]\nName=Calc\nIcon=org.gnome.Calculator\n");

    assert_eq!(entry.icon.as_deref(), Some("org.gnome.Calculator"));
}

fn entry(wm_class: Option<&str>, icon: &str) -> DesktopEntry {
    DesktopEntry {
        name: None,
        startup_wm_class: wm_class.map(str::to_string),
        icon: Some(icon.to_string()),
        exec: None,
    }
}

fn exec_entry(exec: &str, icon: &str) -> DesktopEntry {
    DesktopEntry {
        exec: Some(exec.to_string()),
        ..entry(None, icon)
    }
}

#[test]
fn test_parse_desktop_entry_exec_ignores_actions() {
    let entry = desktop_entry::parse(
        "[Desktop Entry]\nExec=alacritty\n[Desktop Action New]\nExec=alacritty --new\n",
    );

    assert_eq!(entry.exec.as_deref(), Some("alacritty"));
}

#[test]
fn test_exec_program_strips_path_quotes_and_args() {
    assert_eq!(
        desktop_entry::exec_program("/snap/bin/brave %U"),
        Some("brave")
    );
    assert_eq!(
        desktop_entry::exec_program("\"/opt/My App/app\" %f"),
        Some("app")
    );
    assert_eq!(
        desktop_entry::exec_program("\"/usr/bin/foo\" %f"),
        Some("foo")
    );
    assert_eq!(desktop_entry::exec_program("alacritty"), Some("alacritty"));
    assert_eq!(desktop_entry::exec_program(""), None);
}

#[test]
fn test_match_executable_by_exec_program() {
    let entries = vec![
        (
            "Alacritty".to_string(),
            exec_entry("alacritty", "Alacritty"),
        ),
        (
            "brave_brave".to_string(),
            exec_entry("/snap/bin/brave %U", "brave-logo"),
        ),
    ];

    let icon = |program| desktop_entry::match_executable(program, &entries).and_then(|e| e.icon);

    assert_eq!(icon("alacritty").as_deref(), Some("Alacritty"));
    assert_eq!(icon("brave").as_deref(), Some("brave-logo"));
    assert_eq!(icon("kitty"), None);
    assert_eq!(icon(""), None);
}

#[test]
fn test_match_window_class_prefers_startup_wm_class() {
    let entries = vec![
        ("1password".to_string(), entry(None, "by-id")),
        (
            "1password_1password".to_string(),
            entry(Some("1Password"), "by-wm-class"),
        ),
    ];

    let matched = desktop_entry::match_window_class("1password", &entries);

    assert_eq!(matched.and_then(|e| e.icon).as_deref(), Some("by-wm-class"));
}

#[test]
fn test_match_window_class_falls_back_to_desktop_id() {
    let entries = vec![("org.gnome.Calculator".to_string(), entry(None, "calc"))];

    let matched = desktop_entry::match_window_class("org.gnome.Calculator", &entries);

    assert_eq!(matched.and_then(|e| e.icon).as_deref(), Some("calc"));
}

#[test]
fn test_match_window_class_no_match() {
    let entries = vec![("slack".to_string(), entry(Some("Slack"), "slack"))];

    assert_eq!(desktop_entry::match_window_class("brave", &entries), None);
    assert_eq!(desktop_entry::match_window_class("", &entries), None);
}
