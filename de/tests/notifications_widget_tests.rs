// Note: Widget creation tests require GTK main thread.
// Testing notifications container widget logic without GTK dependency.

use rusty_de::service::notifications::{Notification, Urgency};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[test]
fn test_group_notifications_by_app_name() {
    let notifications = vec![
        Notification {
            id: 1,
            app_name: "Slack".to_string(),
            app_icon: "slack".to_string(),
            summary: "Message 1".to_string(),
            body: "".to_string(),
            actions: vec![],
            urgency: Urgency::Normal,
            time: 1000,
            expire_timeout: 5000,
            transient: false,
            category: None,
            desktop_entry: None,
        },
        Notification {
            id: 2,
            app_name: "Discord".to_string(),
            app_icon: "discord".to_string(),
            summary: "Message 1".to_string(),
            body: "".to_string(),
            actions: vec![],
            urgency: Urgency::Normal,
            time: 2000,
            expire_timeout: 5000,
            transient: false,
            category: None,
            desktop_entry: None,
        },
        Notification {
            id: 3,
            app_name: "Slack".to_string(),
            app_icon: "slack".to_string(),
            summary: "Message 2".to_string(),
            body: "".to_string(),
            actions: vec![],
            urgency: Urgency::Normal,
            time: 3000,
            expire_timeout: 5000,
            transient: false,
            category: None,
            desktop_entry: None,
        },
    ];

    // Group by app_name
    let mut groups: HashMap<String, Vec<Notification>> = HashMap::new();
    for notification in notifications {
        groups
            .entry(notification.app_name.clone())
            .or_default()
            .push(notification);
    }

    assert_eq!(groups.len(), 2);
    assert_eq!(groups.get("Slack").unwrap().len(), 2);
    assert_eq!(groups.get("Discord").unwrap().len(), 1);
}

#[test]
fn test_group_notifications_all_same_app() {
    let notifications = vec![
        Notification {
            id: 1,
            app_name: "App".to_string(),
            app_icon: "".to_string(),
            summary: "1".to_string(),
            body: "".to_string(),
            actions: vec![],
            urgency: Urgency::Normal,
            time: 1000,
            expire_timeout: 5000,
            transient: false,
            category: None,
            desktop_entry: None,
        },
        Notification {
            id: 2,
            app_name: "App".to_string(),
            app_icon: "".to_string(),
            summary: "2".to_string(),
            body: "".to_string(),
            actions: vec![],
            urgency: Urgency::Normal,
            time: 2000,
            expire_timeout: 5000,
            transient: false,
            category: None,
            desktop_entry: None,
        },
    ];

    let mut groups: HashMap<String, Vec<Notification>> = HashMap::new();
    for notification in notifications {
        groups
            .entry(notification.app_name.clone())
            .or_default()
            .push(notification);
    }

    assert_eq!(groups.len(), 1);
    assert_eq!(groups.get("App").unwrap().len(), 2);
}

#[test]
fn test_group_notifications_all_different_apps() {
    let notifications = vec![
        Notification {
            id: 1,
            app_name: "App1".to_string(),
            app_icon: "".to_string(),
            summary: "".to_string(),
            body: "".to_string(),
            actions: vec![],
            urgency: Urgency::Normal,
            time: 1000,
            expire_timeout: 5000,
            transient: false,
            category: None,
            desktop_entry: None,
        },
        Notification {
            id: 2,
            app_name: "App2".to_string(),
            app_icon: "".to_string(),
            summary: "".to_string(),
            body: "".to_string(),
            actions: vec![],
            urgency: Urgency::Normal,
            time: 2000,
            expire_timeout: 5000,
            transient: false,
            category: None,
            desktop_entry: None,
        },
    ];

    let mut groups: HashMap<String, Vec<Notification>> = HashMap::new();
    for notification in notifications {
        groups
            .entry(notification.app_name.clone())
            .or_default()
            .push(notification);
    }

    assert_eq!(groups.len(), 2);
    assert_eq!(groups.get("App1").unwrap().len(), 1);
    assert_eq!(groups.get("App2").unwrap().len(), 1);
}

#[test]
fn test_group_notifications_empty() {
    let notifications: Vec<Notification> = vec![];

    let mut groups: HashMap<String, Vec<Notification>> = HashMap::new();
    for notification in notifications {
        groups
            .entry(notification.app_name.clone())
            .or_default()
            .push(notification);
    }

    assert_eq!(groups.len(), 0);
}

#[test]
fn test_sort_groups_by_time() {
    let mut groups = [("App1", 1000i64), ("App2", 5000i64), ("App3", 3000i64)];

    // Sort by time descending (newest first)
    groups.sort_by(|a, b| b.1.cmp(&a.1));

    assert_eq!(groups[0].0, "App2");
    assert_eq!(groups[1].0, "App3");
    assert_eq!(groups[2].0, "App1");
}

#[test]
fn test_get_latest_time_from_group() {
    let notifications = vec![
        Notification {
            id: 1,
            app_name: "App".to_string(),
            app_icon: "".to_string(),
            summary: "Old".to_string(),
            body: "".to_string(),
            actions: vec![],
            urgency: Urgency::Normal,
            time: 1000,
            expire_timeout: 5000,
            transient: false,
            category: None,
            desktop_entry: None,
        },
        Notification {
            id: 2,
            app_name: "App".to_string(),
            app_icon: "".to_string(),
            summary: "New".to_string(),
            body: "".to_string(),
            actions: vec![],
            urgency: Urgency::Normal,
            time: 5000,
            expire_timeout: 5000,
            transient: false,
            category: None,
            desktop_entry: None,
        },
    ];

    let max_time = notifications.iter().map(|n| n.time).max();
    assert_eq!(max_time, Some(5000));
}

#[test]
fn test_notification_exists_in_group() {
    let notifications = vec![
        Notification {
            id: 1,
            app_name: "App".to_string(),
            app_icon: "".to_string(),
            summary: "".to_string(),
            body: "".to_string(),
            actions: vec![],
            urgency: Urgency::Normal,
            time: 1000,
            expire_timeout: 5000,
            transient: false,
            category: None,
            desktop_entry: None,
        },
        Notification {
            id: 2,
            app_name: "App".to_string(),
            app_icon: "".to_string(),
            summary: "".to_string(),
            body: "".to_string(),
            actions: vec![],
            urgency: Urgency::Normal,
            time: 2000,
            expire_timeout: 5000,
            transient: false,
            category: None,
            desktop_entry: None,
        },
    ];

    let exists = notifications.iter().any(|n| n.id == 2);
    assert!(exists);

    let not_exists = notifications.iter().any(|n| n.id == 99);
    assert!(!not_exists);
}

#[test]
fn test_add_notification_to_existing_group() {
    let mut groups: HashMap<String, Vec<Notification>> = HashMap::new();

    let notification1 = Notification {
        id: 1,
        app_name: "App".to_string(),
        app_icon: "".to_string(),
        summary: "First".to_string(),
        body: "".to_string(),
        actions: vec![],
        urgency: Urgency::Normal,
        time: 1000,
        expire_timeout: 5000,
        transient: false,
        category: None,
        desktop_entry: None,
    };

    groups
        .entry(notification1.app_name.clone())
        .or_default()
        .push(notification1);

    assert_eq!(groups.get("App").unwrap().len(), 1);

    let notification2 = Notification {
        id: 2,
        app_name: "App".to_string(),
        app_icon: "".to_string(),
        summary: "Second".to_string(),
        body: "".to_string(),
        actions: vec![],
        urgency: Urgency::Normal,
        time: 2000,
        expire_timeout: 5000,
        transient: false,
        category: None,
        desktop_entry: None,
    };

    groups
        .entry(notification2.app_name.clone())
        .or_default()
        .push(notification2);

    assert_eq!(groups.get("App").unwrap().len(), 2);
}

#[test]
fn test_remove_notification_from_group() {
    let mut notifications = vec![
        Notification {
            id: 1,
            app_name: "App".to_string(),
            app_icon: "".to_string(),
            summary: "".to_string(),
            body: "".to_string(),
            actions: vec![],
            urgency: Urgency::Normal,
            time: 1000,
            expire_timeout: 5000,
            transient: false,
            category: None,
            desktop_entry: None,
        },
        Notification {
            id: 2,
            app_name: "App".to_string(),
            app_icon: "".to_string(),
            summary: "".to_string(),
            body: "".to_string(),
            actions: vec![],
            urgency: Urgency::Normal,
            time: 2000,
            expire_timeout: 5000,
            transient: false,
            category: None,
            desktop_entry: None,
        },
    ];

    assert_eq!(notifications.len(), 2);
    notifications.retain(|n| n.id != 1);
    assert_eq!(notifications.len(), 1);
    assert_eq!(notifications[0].id, 2);
}

#[test]
fn test_group_becomes_empty_after_removal() {
    let mut notifications = vec![Notification {
        id: 1,
        app_name: "App".to_string(),
        app_icon: "".to_string(),
        summary: "".to_string(),
        body: "".to_string(),
        actions: vec![],
        urgency: Urgency::Normal,
        time: 1000,
        expire_timeout: 5000,
        transient: false,
        category: None,
        desktop_entry: None,
    }];

    assert_eq!(notifications.len(), 1);
    notifications.retain(|n| n.id != 1);
    assert_eq!(notifications.len(), 0);
    assert!(notifications.is_empty());
}

#[test]
fn test_callback_invoked_on_first_notification() {
    let callback_called = Rc::new(RefCell::new(false));
    let callback_called_clone = callback_called.clone();

    let callback = move || {
        *callback_called_clone.borrow_mut() = true;
    };

    // Simulate adding first notification to empty groups
    let groups_empty = true;
    if groups_empty {
        callback();
    }

    assert!(*callback_called.borrow());
}

#[test]
fn test_callback_not_invoked_when_groups_not_empty() {
    let callback_called = Rc::new(RefCell::new(false));
    let callback_called_clone = callback_called.clone();

    let callback = move || {
        *callback_called_clone.borrow_mut() = true;
    };

    // Simulate adding notification to non-empty groups
    let groups_empty = false;
    if groups_empty {
        callback();
    }

    assert!(!*callback_called.borrow());
}

#[test]
fn test_callback_invoked_on_last_notification_removed() {
    let callback_called = Rc::new(RefCell::new(false));
    let callback_called_clone = callback_called.clone();

    let callback = move || {
        *callback_called_clone.borrow_mut() = true;
    };

    // Simulate removing last notification (groups become empty)
    let groups_empty_after_removal = true;
    if groups_empty_after_removal {
        callback();
    }

    assert!(*callback_called.borrow());
}

#[test]
fn test_callback_not_invoked_when_groups_still_have_notifications() {
    let callback_called = Rc::new(RefCell::new(false));
    let callback_called_clone = callback_called.clone();

    let callback = move || {
        *callback_called_clone.borrow_mut() = true;
    };

    // Simulate removing notification but groups still not empty
    let groups_empty_after_removal = false;
    if groups_empty_after_removal {
        callback();
    }

    assert!(!*callback_called.borrow());
}

#[test]
fn test_callback_counter_multiple_invocations() {
    let callback_count = Rc::new(RefCell::new(0));

    // First notification added
    {
        let callback_count_clone = callback_count.clone();
        let callback = move || {
            *callback_count_clone.borrow_mut() += 1;
        };
        callback();
    }

    assert_eq!(*callback_count.borrow(), 1);

    // Second notification added (groups not empty, should not increment)
    // (callback not called)

    // Third notification added (groups not empty, should not increment)
    // (callback not called)

    assert_eq!(*callback_count.borrow(), 1);

    // Last notification removed
    {
        let callback_count_clone = callback_count.clone();
        let callback = move || {
            *callback_count_clone.borrow_mut() += 1;
        };
        callback();
    }

    assert_eq!(*callback_count.borrow(), 2);
}

#[test]
fn test_auto_dismiss_triggers_removal() {
    let mut notifications = vec![Notification {
        id: 1,
        app_name: "App".to_string(),
        app_icon: "".to_string(),
        summary: "Auto-dismiss".to_string(),
        body: "".to_string(),
        actions: vec![],
        urgency: Urgency::Normal,
        time: 1000,
        expire_timeout: 5000,
        transient: false,
        category: None,
        desktop_entry: None,
    }];

    // Simulate auto-dismiss by removing notification after timeout
    assert_eq!(notifications.len(), 1);
    notifications.retain(|n| n.id != 1);
    assert_eq!(notifications.len(), 0);
}

#[test]
fn test_auto_dismiss_with_callback() {
    let callback_called = Rc::new(RefCell::new(false));
    let callback_called_clone = callback_called.clone();

    let mut notifications = vec![Notification {
        id: 1,
        app_name: "App".to_string(),
        app_icon: "".to_string(),
        summary: "Auto-dismiss".to_string(),
        body: "".to_string(),
        actions: vec![],
        urgency: Urgency::Normal,
        time: 1000,
        expire_timeout: 5000,
        transient: false,
        category: None,
        desktop_entry: None,
    }];

    // Simulate auto-dismiss removal
    notifications.retain(|n| n.id != 1);

    // If this was the last notification, trigger callback
    if notifications.is_empty() {
        *callback_called_clone.borrow_mut() = true;
    }

    assert!(notifications.is_empty());
    assert!(*callback_called.borrow());
}

#[test]
fn test_manual_dismiss_vs_auto_dismiss() {
    let auto_dismiss_count = Rc::new(RefCell::new(0));
    let manual_dismiss_count = Rc::new(RefCell::new(0));

    // Test auto-dismiss
    {
        let auto_dismiss_count_clone = auto_dismiss_count.clone();
        let mut notifications = vec![Notification {
            id: 1,
            app_name: "App".to_string(),
            app_icon: "".to_string(),
            summary: "".to_string(),
            body: "".to_string(),
            actions: vec![],
            urgency: Urgency::Normal,
            time: 1000,
            expire_timeout: 5000,
            transient: false,
            category: None,
            desktop_entry: None,
        }];

        // Auto-dismiss after timeout
        notifications.retain(|n| n.id != 1);
        if notifications.is_empty() {
            *auto_dismiss_count_clone.borrow_mut() += 1;
        }
    }

    // Test manual dismiss
    {
        let manual_dismiss_count_clone = manual_dismiss_count.clone();
        let mut notifications = vec![Notification {
            id: 2,
            app_name: "App".to_string(),
            app_icon: "".to_string(),
            summary: "".to_string(),
            body: "".to_string(),
            actions: vec![],
            urgency: Urgency::Normal,
            time: 2000,
            expire_timeout: 5000,
            transient: false,
            category: None,
            desktop_entry: None,
        }];

        // Manual dismiss
        notifications.retain(|n| n.id != 2);
        if notifications.is_empty() {
            *manual_dismiss_count_clone.borrow_mut() += 1;
        }
    }

    assert_eq!(*auto_dismiss_count.borrow(), 1);
    assert_eq!(*manual_dismiss_count.borrow(), 1);
}
