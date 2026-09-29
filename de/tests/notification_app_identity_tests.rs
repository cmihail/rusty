// Chromium-based clients (Slack, Discord) send an empty app_name and identify
// themselves only through the desktop-entry hint.

use rusty_de::service::desktop_entry;
use rusty_de::service::notifications::{Notification, NotificationAction, Urgency};

fn notification(app_name: &str, desktop_entry: Option<&str>) -> Notification {
    Notification {
        id: 1,
        app_name: app_name.to_string(),
        app_icon: String::new(),
        summary: String::new(),
        body: String::new(),
        actions: vec![],
        urgency: Urgency::Normal,
        time: 0,
        expire_timeout: -1,
        transient: false,
        category: None,
        desktop_entry: desktop_entry.map(|s| s.to_string()),
    }
}

fn action(id: &str, label: &str) -> NotificationAction {
    NotificationAction {
        id: id.to_string(),
        label: label.to_string(),
    }
}

#[test]
fn test_parse_desktop_entry_name_and_wm_class() {
    let entry = desktop_entry::parse(concat!(
        "[Desktop Entry]\n",
        "X-SnapInstanceName=slack\n",
        "Name=Slack\n",
        "StartupWMClass=Slack\n",
        "Type=Application\n",
    ));

    assert_eq!(entry.name.as_deref(), Some("Slack"));
    assert_eq!(entry.startup_wm_class.as_deref(), Some("Slack"));
}

#[test]
fn test_parse_desktop_entry_prefers_unlocalized_name() {
    let entry = desktop_entry::parse("[Desktop Entry]\nName[de]=Slack DE\nName=Slack\n");

    assert_eq!(entry.name.as_deref(), Some("Slack"));
}

#[test]
fn test_parse_desktop_entry_ignores_other_sections() {
    let entry = desktop_entry::parse(concat!(
        "[Desktop Entry]\n",
        "Name=Slack\n",
        "\n",
        "[Desktop Action New]\n",
        "Name=New Window\n",
        "StartupWMClass=Other\n",
    ));

    assert_eq!(entry.name.as_deref(), Some("Slack"));
    assert_eq!(entry.startup_wm_class, None);
}

#[test]
fn test_parse_desktop_entry_skips_empty_values() {
    let entry = desktop_entry::parse("[Desktop Entry]\nName=\nStartupWMClass=Slack\n");

    assert_eq!(entry.name, None);
    assert_eq!(entry.startup_wm_class.as_deref(), Some("Slack"));
}

#[test]
fn test_lookup_empty_entry_id() {
    assert_eq!(desktop_entry::lookup(""), None);
}

#[test]
fn test_display_app_name_prefers_app_name() {
    let notification = notification("Slack", Some("slack_slack"));

    assert_eq!(notification.display_app_name(), "Slack");
}

#[test]
fn test_display_app_name_trims_session_suffix() {
    let notification = notification("Claude Code: session-id", None);

    assert_eq!(notification.display_app_name(), "Claude Code");
}

#[test]
fn test_display_app_name_falls_back_to_desktop_entry_id() {
    let notification = notification("", Some("not-an-installed-app"));

    assert_eq!(notification.display_app_name(), "not-an-installed-app");
}

#[test]
fn test_display_app_name_empty_without_desktop_entry() {
    let notification = notification("", None);

    assert_eq!(notification.display_app_name(), "");
}

#[test]
fn test_display_app_name_ignores_whitespace_only_app_name() {
    let notification = notification("   ", Some("not-an-installed-app"));

    assert_eq!(notification.display_app_name(), "not-an-installed-app");
}

#[test]
fn test_window_classes_includes_snap_app_part() {
    let notification = notification("", Some("not-installed_someapp"));
    let classes = notification.window_classes();

    assert!(classes.contains(&"not-installed_someapp".to_string()));
    assert!(classes.contains(&"someapp".to_string()));
}

#[test]
fn test_window_classes_deduplicates() {
    let notification = notification("Someapp", Some("someapp"));
    let mut sorted = notification.window_classes();
    sorted.sort();
    let deduped_len = {
        let mut copy = sorted.clone();
        copy.dedup();
        copy.len()
    };

    assert_eq!(sorted.len(), deduped_len);
}

#[test]
fn test_window_classes_empty_without_identity() {
    let notification = notification("", None);

    assert!(notification.window_classes().is_empty());
}

#[test]
fn test_click_action_prefers_default() {
    let mut notification = notification("", Some("slack_slack"));
    notification.actions = vec![action("other", "Other"), action("default", "")];

    assert_eq!(
        notification.click_action().map(|a| a.id.as_str()),
        Some("default")
    );
}

#[test]
fn test_click_action_falls_back_to_first() {
    let mut notification = notification("Claude Code", None);
    notification.actions = vec![action("focus", "Focus Terminal")];

    assert_eq!(
        notification.click_action().map(|a| a.id.as_str()),
        Some("focus")
    );
}

#[test]
fn test_click_action_without_actions() {
    let notification = notification("Test", None);

    assert!(notification.click_action().is_none());
}

#[test]
fn test_button_actions_exclude_default() {
    let mut notification = notification("", Some("slack_slack"));
    notification.actions = vec![action("default", ""), action("reply", "Reply")];

    let labels: Vec<&str> = notification
        .button_actions()
        .map(|a| a.id.as_str())
        .collect();

    assert_eq!(labels, vec!["reply"]);
}

#[test]
fn test_button_actions_keep_custom_actions() {
    let mut notification = notification("Claude Code", None);
    notification.actions = vec![action("focus", "Focus Terminal")];

    assert_eq!(notification.button_actions().count(), 1);
}
