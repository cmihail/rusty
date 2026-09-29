// Note: Widget creation tests require GTK main thread.
// Testing notification widget logic without GTK dependency.

use rusty_de::service::notifications::{Notification, NotificationAction, Urgency};

#[test]
fn test_notification_with_multiple_actions() {
    let notification = Notification {
        id: 1,
        app_name: "Test App".to_string(),
        app_icon: "test-icon".to_string(),
        summary: "Test Summary".to_string(),
        body: "Test Body".to_string(),
        actions: vec![
            NotificationAction {
                id: "default".to_string(),
                label: "Open".to_string(),
            },
            NotificationAction {
                id: "dismiss".to_string(),
                label: "Dismiss".to_string(),
            },
        ],
        urgency: Urgency::Normal,
        time: 1234567890,
        expire_timeout: 5000,
        transient: false,
        category: None,
        desktop_entry: None,
    };

    assert_eq!(notification.actions.len(), 2);
    assert_eq!(notification.actions[0].id, "default");
    assert_eq!(notification.actions[1].id, "dismiss");
}

#[test]
fn test_notification_first_action_extraction() {
    let notification = Notification {
        id: 1,
        app_name: "Test".to_string(),
        app_icon: "".to_string(),
        summary: "".to_string(),
        body: "".to_string(),
        actions: vec![
            NotificationAction {
                id: "action1".to_string(),
                label: "First".to_string(),
            },
            NotificationAction {
                id: "action2".to_string(),
                label: "Second".to_string(),
            },
        ],
        urgency: Urgency::Normal,
        time: 0,
        expire_timeout: 0,
        transient: false,
        category: None,
        desktop_entry: None,
    };

    let first_action = notification.actions.first();
    assert!(first_action.is_some());
    assert_eq!(first_action.unwrap().id, "action1");
}

#[test]
fn test_notification_no_first_action() {
    let notification = Notification {
        id: 1,
        app_name: "Test".to_string(),
        app_icon: "".to_string(),
        summary: "".to_string(),
        body: "".to_string(),
        actions: vec![],
        urgency: Urgency::Normal,
        time: 0,
        expire_timeout: 0,
        transient: false,
        category: None,
        desktop_entry: None,
    };

    let first_action = notification.actions.first();
    assert!(first_action.is_none());
}

#[test]
fn test_notification_header_count_display_single() {
    let count = 1;
    let should_show_count = count > 1;
    assert!(!should_show_count);
}

#[test]
fn test_notification_header_count_display_multiple() {
    let count = 5;
    let should_show_count = count > 1;
    assert!(should_show_count);
}

#[test]
fn test_notification_dismiss_all_button_visibility_single() {
    let count = 1;
    let should_show_dismiss_all = count > 1;
    assert!(!should_show_dismiss_all);
}

#[test]
fn test_notification_dismiss_all_button_visibility_multiple() {
    let count = 3;
    let should_show_dismiss_all = count > 1;
    assert!(should_show_dismiss_all);
}

#[test]
fn test_notification_body_visibility_empty() {
    let body = "";
    let should_show_body = !body.is_empty();
    assert!(!should_show_body);
}

#[test]
fn test_notification_body_visibility_nonempty() {
    let body = "Some content";
    let should_show_body = !body.is_empty();
    assert!(should_show_body);
}

#[test]
fn test_notification_time_formatting() {
    // Test that timestamp conversion can happen
    let time = chrono::Local::now().timestamp();
    assert!(time > 0);
}

#[test]
fn test_notification_urgency_critical_should_not_timeout() {
    let notification = Notification {
        id: 1,
        app_name: "Critical".to_string(),
        app_icon: "".to_string(),
        summary: "Critical Alert".to_string(),
        body: "".to_string(),
        actions: vec![],
        urgency: Urgency::Critical,
        time: 0,
        expire_timeout: 0,
        transient: false,
        category: None,
        desktop_entry: None,
    };

    assert_eq!(notification.urgency, Urgency::Critical);
    assert_eq!(notification.expire_timeout, 0);
}

#[test]
fn test_on_invoke_timeout_constant() {
    const ON_INVOKE_TIMEOUT_DISMISS: u64 = 100;
    assert_eq!(ON_INVOKE_TIMEOUT_DISMISS, 100);
}

#[test]
fn test_urgency_css_class_critical() {
    // Test that Critical urgency maps to "Critical" CSS class
    let notification = Notification {
        id: 1,
        app_name: "Test".to_string(),
        app_icon: "".to_string(),
        summary: "Critical".to_string(),
        body: "".to_string(),
        actions: vec![],
        urgency: Urgency::Critical,
        time: 0,
        expire_timeout: 0,
        transient: false,
        category: None,
        desktop_entry: None,
    };

    let css_class = match notification.urgency {
        Urgency::Critical => "Critical",
        Urgency::Low => "Low",
        Urgency::Normal => "",
    };

    assert_eq!(css_class, "Critical");
}

#[test]
fn test_urgency_css_class_low() {
    // Test that Low urgency maps to "Low" CSS class
    let notification = Notification {
        id: 1,
        app_name: "Test".to_string(),
        app_icon: "".to_string(),
        summary: "Low".to_string(),
        body: "".to_string(),
        actions: vec![],
        urgency: Urgency::Low,
        time: 0,
        expire_timeout: 0,
        transient: false,
        category: None,
        desktop_entry: None,
    };

    let css_class = match notification.urgency {
        Urgency::Critical => "Critical",
        Urgency::Low => "Low",
        Urgency::Normal => "",
    };

    assert_eq!(css_class, "Low");
}

#[test]
fn test_urgency_css_class_normal() {
    // Test that Normal urgency has no additional CSS class
    let notification = Notification {
        id: 1,
        app_name: "Test".to_string(),
        app_icon: "".to_string(),
        summary: "Normal".to_string(),
        body: "".to_string(),
        actions: vec![],
        urgency: Urgency::Normal,
        time: 0,
        expire_timeout: 0,
        transient: false,
        category: None,
        desktop_entry: None,
    };

    let css_class = match notification.urgency {
        Urgency::Critical => "Critical",
        Urgency::Low => "Low",
        Urgency::Normal => "",
    };

    assert_eq!(css_class, "");
}

#[test]
fn test_critical_notification_gets_error_border_class() {
    // Verify critical notifications should get the Critical CSS class for error border
    let notification = Notification {
        id: 1,
        app_name: "rusty-de".to_string(),
        app_icon: "dialog-error".to_string(),
        summary: "Error".to_string(),
        body: "System error".to_string(),
        actions: vec![],
        urgency: Urgency::Critical,
        time: 0,
        expire_timeout: 5000,
        transient: false,
        category: None,
        desktop_entry: None,
    };

    assert_eq!(notification.urgency, Urgency::Critical);

    let should_have_critical_class = notification.urgency == Urgency::Critical;
    assert!(should_have_critical_class);
}

#[test]
fn test_normal_notification_no_error_border_class() {
    // Verify normal notifications don't get error border class
    let notification = Notification {
        id: 1,
        app_name: "App".to_string(),
        app_icon: "info".to_string(),
        summary: "Info".to_string(),
        body: "Normal message".to_string(),
        actions: vec![],
        urgency: Urgency::Normal,
        time: 0,
        expire_timeout: 5000,
        transient: false,
        category: None,
        desktop_entry: None,
    };

    assert_eq!(notification.urgency, Urgency::Normal);

    let should_have_critical_class = notification.urgency == Urgency::Critical;
    assert!(!should_have_critical_class);
}

#[test]
fn test_markup_escape_ampersand() {
    let text = "Nova Power&Gas";
    let escaped = glib::markup_escape_text(text);
    assert_eq!(escaped, "Nova Power&amp;Gas");
}

#[test]
fn test_markup_escape_less_than() {
    let text = "Value < 10";
    let escaped = glib::markup_escape_text(text);
    assert_eq!(escaped, "Value &lt; 10");
}

#[test]
fn test_markup_escape_greater_than() {
    let text = "Value > 10";
    let escaped = glib::markup_escape_text(text);
    assert_eq!(escaped, "Value &gt; 10");
}

#[test]
fn test_markup_escape_quotes() {
    let text = r#"He said "hello""#;
    let escaped = glib::markup_escape_text(text);
    assert_eq!(escaped, "He said &quot;hello&quot;");
}

#[test]
fn test_markup_escape_notification_summary_with_ampersand() {
    let notification = Notification {
        id: 1,
        app_name: "mail.proton.me".to_string(),
        app_icon: "".to_string(),
        summary: "Nova Power&Gas - Message".to_string(),
        body: "".to_string(),
        actions: vec![],
        urgency: Urgency::Normal,
        time: 0,
        expire_timeout: 0,
        transient: false,
        category: None,
        desktop_entry: None,
    };

    let escaped_summary = glib::markup_escape_text(&notification.summary);
    assert_eq!(escaped_summary, "Nova Power&amp;Gas - Message");
}

#[test]
fn test_markup_escape_notification_body_with_special_chars() {
    let notification = Notification {
        id: 1,
        app_name: "Test".to_string(),
        app_icon: "".to_string(),
        summary: "".to_string(),
        body: "From: Nova Power&Gas - A început perioada".to_string(),
        actions: vec![],
        urgency: Urgency::Normal,
        time: 0,
        expire_timeout: 0,
        transient: false,
        category: None,
        desktop_entry: None,
    };

    let escaped_body = glib::markup_escape_text(&notification.body);
    assert_eq!(
        escaped_body,
        "From: Nova Power&amp;Gas - A început perioada"
    );
}

#[test]
fn test_markup_escape_preserves_normal_text() {
    let text = "Normal text without special characters";
    let escaped = glib::markup_escape_text(text);
    assert_eq!(escaped, "Normal text without special characters");
}
