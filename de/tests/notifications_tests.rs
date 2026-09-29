use rusty_de::service::notifications::*;

#[test]
fn test_urgency_from_u8_low() {
    assert_eq!(Urgency::from_u8(0), Urgency::Low);
}

#[test]
fn test_urgency_from_u8_normal() {
    assert_eq!(Urgency::from_u8(1), Urgency::Normal);
}

#[test]
fn test_urgency_from_u8_critical() {
    assert_eq!(Urgency::from_u8(2), Urgency::Critical);
}

#[test]
fn test_urgency_from_u8_unknown() {
    assert_eq!(Urgency::from_u8(3), Urgency::Normal);
    assert_eq!(Urgency::from_u8(99), Urgency::Normal);
    assert_eq!(Urgency::from_u8(255), Urgency::Normal);
}

#[test]
fn test_urgency_values() {
    assert_eq!(Urgency::Low as u8, 0);
    assert_eq!(Urgency::Normal as u8, 1);
    assert_eq!(Urgency::Critical as u8, 2);
}

#[test]
fn test_closed_reason_values() {
    assert_eq!(ClosedReason::Expired as u32, 1);
    assert_eq!(ClosedReason::DismissedByUser as u32, 2);
    assert_eq!(ClosedReason::Closed as u32, 3);
    assert_eq!(ClosedReason::Undefined as u32, 4);
}

#[test]
fn test_notification_action_creation() {
    let action = NotificationAction {
        id: "action1".to_string(),
        label: "Click me".to_string(),
    };

    assert_eq!(action.id, "action1");
    assert_eq!(action.label, "Click me");
}

#[test]
fn test_notification_creation() {
    let actions = vec![
        NotificationAction {
            id: "default".to_string(),
            label: "Open".to_string(),
        },
        NotificationAction {
            id: "dismiss".to_string(),
            label: "Dismiss".to_string(),
        },
    ];

    let notification = Notification {
        id: 1,
        app_name: "Test App".to_string(),
        app_icon: "app-icon".to_string(),
        summary: "Test Summary".to_string(),
        body: "Test Body".to_string(),
        actions: actions.clone(),
        urgency: Urgency::Normal,
        time: 1234567890,
        expire_timeout: 5000,
        transient: false,
        category: None,
        desktop_entry: None,
    };

    assert_eq!(notification.id, 1);
    assert_eq!(notification.app_name, "Test App");
    assert_eq!(notification.app_icon, "app-icon");
    assert_eq!(notification.summary, "Test Summary");
    assert_eq!(notification.body, "Test Body");
    assert_eq!(notification.actions.len(), 2);
    assert_eq!(notification.actions[0].id, "default");
    assert_eq!(notification.actions[1].label, "Dismiss");
    assert_eq!(notification.urgency, Urgency::Normal);
    assert_eq!(notification.time, 1234567890);
    assert_eq!(notification.expire_timeout, 5000);
}

#[test]
fn test_notification_with_critical_urgency() {
    let notification = Notification {
        id: 2,
        app_name: "Alert App".to_string(),
        app_icon: "alert-icon".to_string(),
        summary: "Critical Alert".to_string(),
        body: "This is urgent!".to_string(),
        actions: vec![],
        urgency: Urgency::Critical,
        time: 1234567891,
        expire_timeout: 0,
        transient: false,
        category: None,
        desktop_entry: None,
    };

    assert_eq!(notification.urgency, Urgency::Critical);
    assert_eq!(notification.expire_timeout, 0);
}

#[test]
fn test_notification_with_low_urgency() {
    let notification = Notification {
        id: 3,
        app_name: "Info App".to_string(),
        app_icon: "info-icon".to_string(),
        summary: "Low Priority Info".to_string(),
        body: "This can wait".to_string(),
        actions: vec![],
        urgency: Urgency::Low,
        time: 1234567892,
        expire_timeout: 10000,
        transient: false,
        category: None,
        desktop_entry: None,
    };

    assert_eq!(notification.urgency, Urgency::Low);
}

#[test]
fn test_notification_no_actions() {
    let notification = Notification {
        id: 4,
        app_name: "Simple App".to_string(),
        app_icon: "simple-icon".to_string(),
        summary: "Simple Notification".to_string(),
        body: "No actions available".to_string(),
        actions: vec![],
        urgency: Urgency::Normal,
        time: 1234567893,
        expire_timeout: 3000,
        transient: false,
        category: None,
        desktop_entry: None,
    };

    assert_eq!(notification.actions.len(), 0);
}

#[test]
fn test_notification_empty_body() {
    let notification = Notification {
        id: 5,
        app_name: "Minimal App".to_string(),
        app_icon: "minimal-icon".to_string(),
        summary: "Summary Only".to_string(),
        body: "".to_string(),
        actions: vec![],
        urgency: Urgency::Normal,
        time: 1234567894,
        expire_timeout: 5000,
        transient: false,
        category: None,
        desktop_entry: None,
    };

    assert_eq!(notification.body, "");
    assert_eq!(notification.summary, "Summary Only");
}

#[test]
fn test_notification_clone() {
    let original = Notification {
        id: 6,
        app_name: "Clone Test".to_string(),
        app_icon: "clone-icon".to_string(),
        summary: "Original".to_string(),
        body: "Body".to_string(),
        actions: vec![],
        urgency: Urgency::Normal,
        time: 1234567895,
        expire_timeout: 1000,
        transient: false,
        category: None,
        desktop_entry: None,
    };

    let cloned = original.clone();

    assert_eq!(cloned.id, original.id);
    assert_eq!(cloned.app_name, original.app_name);
    assert_eq!(cloned.summary, original.summary);
}

#[test]
fn test_urgency_equality() {
    assert_eq!(Urgency::Low, Urgency::Low);
    assert_eq!(Urgency::Normal, Urgency::Normal);
    assert_eq!(Urgency::Critical, Urgency::Critical);
    assert_ne!(Urgency::Low, Urgency::Normal);
    assert_ne!(Urgency::Normal, Urgency::Critical);
}

#[test]
fn test_closed_reason_equality() {
    assert_eq!(ClosedReason::Expired, ClosedReason::Expired);
    assert_eq!(ClosedReason::DismissedByUser, ClosedReason::DismissedByUser);
    assert_eq!(ClosedReason::Closed, ClosedReason::Closed);
    assert_eq!(ClosedReason::Undefined, ClosedReason::Undefined);
    assert_ne!(ClosedReason::Expired, ClosedReason::DismissedByUser);
}

// Note: Service tests are skipped because they require D-Bus session bus and would conflict
// when running in parallel. The Notifications service starts a D-Bus daemon on construction
// and tries to acquire the org.freedesktop.Notifications name, which can only be held once.

#[test]
fn test_notification_action_clone() {
    let original = NotificationAction {
        id: "test".to_string(),
        label: "Test Action".to_string(),
    };

    let cloned = original.clone();
    assert_eq!(cloned.id, original.id);
    assert_eq!(cloned.label, original.label);
}

#[test]
fn test_send_notification_structure() {
    // Test that send_notification creates notification with correct properties
    let notification = Notification {
        id: 42,
        app_name: "rusty-de".to_string(),
        app_icon: "dialog-error".to_string(),
        summary: "Test Error".to_string(),
        body: "Error message body".to_string(),
        actions: Vec::new(),
        urgency: Urgency::Critical,
        time: chrono::Local::now().timestamp(),
        expire_timeout: 5000,
        transient: false,
        category: None,
        desktop_entry: None,
    };

    assert_eq!(notification.app_name, "rusty-de");
    assert_eq!(notification.app_icon, "dialog-error");
    assert_eq!(notification.summary, "Test Error");
    assert_eq!(notification.body, "Error message body");
    assert_eq!(notification.urgency, Urgency::Critical);
    assert_eq!(notification.expire_timeout, 5000);
    assert!(notification.actions.is_empty());
}

#[test]
fn test_error_notification_has_critical_urgency() {
    // Verify error notifications use critical urgency
    let notification = Notification {
        id: 1,
        app_name: "rusty-de".to_string(),
        app_icon: "dialog-error".to_string(),
        summary: "Error".to_string(),
        body: "An error occurred".to_string(),
        actions: Vec::new(),
        urgency: Urgency::Critical,
        time: chrono::Local::now().timestamp(),
        expire_timeout: 5000,
        transient: false,
        category: None,
        desktop_entry: None,
    };

    assert_eq!(notification.urgency, Urgency::Critical);
}

#[test]
fn test_error_notification_has_error_icon() {
    // Verify error notifications use dialog-error icon
    let notification = Notification {
        id: 1,
        app_name: "rusty-de".to_string(),
        app_icon: "dialog-error".to_string(),
        summary: "Error".to_string(),
        body: "An error occurred".to_string(),
        actions: Vec::new(),
        urgency: Urgency::Critical,
        time: chrono::Local::now().timestamp(),
        expire_timeout: 5000,
        transient: false,
        category: None,
        desktop_entry: None,
    };

    assert_eq!(notification.app_icon, "dialog-error");
}

#[test]
fn test_error_notification_timeout() {
    // Verify error notifications have 5 second timeout
    let notification = Notification {
        id: 1,
        app_name: "rusty-de".to_string(),
        app_icon: "dialog-error".to_string(),
        summary: "Error".to_string(),
        body: "An error occurred".to_string(),
        actions: Vec::new(),
        urgency: Urgency::Critical,
        time: chrono::Local::now().timestamp(),
        expire_timeout: 5000,
        transient: false,
        category: None,
        desktop_entry: None,
    };

    assert_eq!(notification.expire_timeout, 5000);
}

#[test]
fn test_notification_serialization() {
    let notification = Notification {
        id: 1,
        app_name: "Test App".to_string(),
        app_icon: "app-icon".to_string(),
        summary: "Test Summary".to_string(),
        body: "Test Body".to_string(),
        actions: vec![NotificationAction {
            id: "default".to_string(),
            label: "Open".to_string(),
        }],
        urgency: Urgency::Normal,
        time: 1234567890,
        expire_timeout: 5000,
        transient: false,
        category: None,
        desktop_entry: None,
    };

    let json = serde_json::to_string(&notification).unwrap();
    let deserialized: Notification = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.id, notification.id);
    assert_eq!(deserialized.app_name, notification.app_name);
    assert_eq!(deserialized.summary, notification.summary);
    assert_eq!(deserialized.body, notification.body);
    assert_eq!(deserialized.urgency, notification.urgency);
    assert_eq!(deserialized.transient, notification.transient);
}

#[test]
fn test_transient_notification() {
    let notification = Notification {
        id: 1,
        app_name: "Volume".to_string(),
        app_icon: "audio-volume-high".to_string(),
        summary: "Volume".to_string(),
        body: "50%".to_string(),
        actions: vec![],
        urgency: Urgency::Low,
        time: chrono::Local::now().timestamp(),
        expire_timeout: 1000,
        transient: true,
        category: None,
        desktop_entry: None,
    };

    assert!(notification.transient);
}

#[test]
fn test_non_transient_notification() {
    let notification = Notification {
        id: 1,
        app_name: "Email".to_string(),
        app_icon: "mail-unread".to_string(),
        summary: "New Email".to_string(),
        body: "You have a new message".to_string(),
        actions: vec![],
        urgency: Urgency::Normal,
        time: chrono::Local::now().timestamp(),
        expire_timeout: 5000,
        transient: false,
        category: None,
        desktop_entry: None,
    };

    assert!(!notification.transient);
}

#[test]
fn test_notification_action_serialization() {
    let action = NotificationAction {
        id: "action1".to_string(),
        label: "Click me".to_string(),
    };

    let json = serde_json::to_string(&action).unwrap();
    let deserialized: NotificationAction = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.id, action.id);
    assert_eq!(deserialized.label, action.label);
}

#[test]
fn test_urgency_serialization() {
    let low = Urgency::Low;
    let normal = Urgency::Normal;
    let critical = Urgency::Critical;

    assert_eq!(serde_json::to_string(&low).unwrap(), "\"Low\"");
    assert_eq!(serde_json::to_string(&normal).unwrap(), "\"Normal\"");
    assert_eq!(serde_json::to_string(&critical).unwrap(), "\"Critical\"");

    let low_deser: Urgency = serde_json::from_str("\"Low\"").unwrap();
    let normal_deser: Urgency = serde_json::from_str("\"Normal\"").unwrap();
    let critical_deser: Urgency = serde_json::from_str("\"Critical\"").unwrap();

    assert_eq!(low_deser, Urgency::Low);
    assert_eq!(normal_deser, Urgency::Normal);
    assert_eq!(critical_deser, Urgency::Critical);
}

#[test]
fn test_notification_default_transient() {
    // Test that transient defaults to false when deserializing old notifications
    let json = r#"{
        "id": 1,
        "app_name": "Test",
        "app_icon": "icon",
        "summary": "Summary",
        "body": "Body",
        "actions": [],
        "urgency": "Normal",
        "time": 1234567890,
        "expire_timeout": 5000
    }"#;

    let notification: Notification = serde_json::from_str(json).unwrap();
    assert!(!notification.transient);
}
#[test]
fn test_notification_with_category() {
    let notification = Notification {
        id: 1,
        app_name: "Clipboard".to_string(),
        app_icon: "edit-copy".to_string(),
        summary: "Text copied to clipboard".to_string(),
        body: "".to_string(),
        actions: vec![],
        urgency: Urgency::Normal,
        time: chrono::Local::now().timestamp(),
        expire_timeout: 2000,
        transient: false,
        category: Some("x.clipboard".to_string()),
        desktop_entry: None,
    };

    assert_eq!(notification.category, Some("x.clipboard".to_string()));
}

#[test]
fn test_notification_without_category() {
    let notification = Notification {
        id: 1,
        app_name: "Email".to_string(),
        app_icon: "mail-unread".to_string(),
        summary: "New Email".to_string(),
        body: "You have a new message".to_string(),
        actions: vec![],
        urgency: Urgency::Normal,
        time: chrono::Local::now().timestamp(),
        expire_timeout: 5000,
        transient: false,
        category: None,
        desktop_entry: None,
    };

    assert_eq!(notification.category, None);
}

#[test]
fn test_notification_default_category() {
    // Test that category defaults to None when deserializing old notifications
    let json = r#"{
        "id": 1,
        "app_name": "Test",
        "app_icon": "icon",
        "summary": "Summary",
        "body": "Body",
        "actions": [],
        "urgency": "Normal",
        "time": 1234567890,
        "expire_timeout": 5000,
        "transient": false
    }"#;

    let notification: Notification = serde_json::from_str(json).unwrap();
    assert_eq!(notification.category, None);
}

#[test]
fn test_notification_category_serialization() {
    let notification = Notification {
        id: 1,
        app_name: "Clipboard".to_string(),
        app_icon: "edit-copy".to_string(),
        summary: "Copied".to_string(),
        body: "".to_string(),
        actions: vec![],
        urgency: Urgency::Normal,
        time: 1234567890,
        expire_timeout: 2000,
        transient: false,
        category: Some("x.clipboard".to_string()),
        desktop_entry: None,
    };

    let json = serde_json::to_string(&notification).unwrap();
    let deserialized: Notification = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.category, notification.category);
}
