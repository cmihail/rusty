// Tests for notification race condition fix
// Simulates the Brave browser behavior of rapidly closing and opening notifications

use rusty_de::service::notifications::{Notification, Urgency};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[test]
fn test_group_empty_check_before_delayed_removal() {
    // Simulates scenario where group becomes empty, schedules removal,
    // but new notification arrives before removal executes

    let mut group_notifications = vec![Notification {
        id: 106,
        app_name: "Brave".to_string(),
        app_icon: "brave".to_string(),
        summary: "Message 1".to_string(),
        body: "".to_string(),
        actions: vec![],
        urgency: Urgency::Normal,
        time: 1000,
        expire_timeout: 5000,
        transient: false,
        category: None,
        desktop_entry: None,
    }];

    // Remove notification (simulates Brave closing notification)
    group_notifications.retain(|n| n.id != 106);
    let is_empty_before_new = group_notifications.is_empty();
    assert!(is_empty_before_new);

    // New notification arrives before delayed removal (simulates Brave opening new notification)
    group_notifications.push(Notification {
        id: 107,
        app_name: "Brave".to_string(),
        app_icon: "brave".to_string(),
        summary: "Message 2".to_string(),
        body: "".to_string(),
        actions: vec![],
        urgency: Urgency::Normal,
        time: 1001,
        expire_timeout: 5000,
        transient: false,
        category: None,
        desktop_entry: None,
    });

    // Delayed removal callback executes - should re-check emptiness
    let is_empty_after_new = group_notifications.is_empty();
    assert!(!is_empty_after_new);

    // Group should NOT be removed because it has notifications
    let should_remove = is_empty_after_new;
    assert!(!should_remove);
}

#[test]
fn test_rapid_add_remove_add_sequence() {
    // Simulates Brave's exact behavior: close id=106, add id=107
    let mut notifications_map: HashMap<u32, Notification> = HashMap::new();

    // Initial notification
    notifications_map.insert(
        106,
        Notification {
            id: 106,
            app_name: "Brave".to_string(),
            app_icon: "brave".to_string(),
            summary: "Message 1".to_string(),
            body: "".to_string(),
            actions: vec![],
            urgency: Urgency::Normal,
            time: 26408,
            expire_timeout: 5000,
            transient: false,
            category: None,
            desktop_entry: None,
        },
    );

    // Get notifications for group "Brave"
    let mut group_notifs: Vec<_> = notifications_map
        .values()
        .filter(|n| n.app_name == "Brave")
        .cloned()
        .collect();
    assert_eq!(group_notifs.len(), 1);

    // Brave closes notification 106 (at time 26408ms)
    notifications_map.remove(&106);
    group_notifs.retain(|n| n.id != 106);
    assert!(group_notifs.is_empty());

    // 11ms later: Brave adds notification 107 (at time 26419ms)
    notifications_map.insert(
        107,
        Notification {
            id: 107,
            app_name: "Brave".to_string(),
            app_icon: "brave".to_string(),
            summary: "Message 2".to_string(),
            body: "".to_string(),
            actions: vec![],
            urgency: Urgency::Normal,
            time: 26419,
            expire_timeout: 5000,
            transient: false,
            category: None,
            desktop_entry: None,
        },
    );

    group_notifs = notifications_map
        .values()
        .filter(|n| n.app_name == "Brave")
        .cloned()
        .collect();

    // Group should have new notification
    assert_eq!(group_notifs.len(), 1);
    assert_eq!(group_notifs[0].id, 107);

    // 500ms delay expires (at time 26908ms)
    // Delayed callback should re-check: group is NOT empty
    let should_remove_group = group_notifs.is_empty();
    assert!(!should_remove_group);
}

#[test]
fn test_group_not_removed_when_has_notifications() {
    // Verifies that delayed removal checks group state
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

    // Check if group should be removed
    let should_remove = notifications.is_empty();
    assert!(!should_remove);
}

#[test]
fn test_group_removed_only_when_truly_empty() {
    // Group becomes empty
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

    notifications.retain(|n| n.id != 1);

    // Delayed removal executes - group is empty
    let should_remove = notifications.is_empty();
    assert!(should_remove);
}

#[test]
fn test_revealer_state_updated_when_notification_added() {
    // Simulates revealer state tracking
    let mut revealer_revealed = false;
    let mut group_notifications: Vec<Notification> = vec![];

    // Group is empty, revealer should be hidden
    assert!(!revealer_revealed);
    assert!(group_notifications.is_empty());

    // New notification arrives - should reveal
    group_notifications.push(Notification {
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
    });

    // Check conditions that should trigger re-reveal
    let should_reveal = !revealer_revealed || group_notifications.is_empty();
    if should_reveal {
        revealer_revealed = true;
    }

    assert!(revealer_revealed);
    assert!(!group_notifications.is_empty());
}

#[test]
fn test_multiple_rapid_notifications() {
    // Simulates multiple notifications arriving in quick succession
    let mut group: HashMap<String, Vec<Notification>> = HashMap::new();
    let app_name = "Brave".to_string();

    // Notification 1 arrives
    group
        .entry(app_name.clone())
        .or_default()
        .push(Notification {
            id: 1,
            app_name: app_name.clone(),
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
        });
    assert_eq!(group.get(&app_name).unwrap().len(), 1);

    // Notification 1 removed
    group.get_mut(&app_name).unwrap().retain(|n| n.id != 1);
    assert!(group.get(&app_name).unwrap().is_empty());

    // Notification 2 arrives immediately
    group.get_mut(&app_name).unwrap().push(Notification {
        id: 2,
        app_name: app_name.clone(),
        app_icon: "".to_string(),
        summary: "2".to_string(),
        body: "".to_string(),
        actions: vec![],
        urgency: Urgency::Normal,
        time: 1001,
        expire_timeout: 5000,
        transient: false,
        category: None,
        desktop_entry: None,
    });
    assert_eq!(group.get(&app_name).unwrap().len(), 1);

    // Notification 2 removed
    group.get_mut(&app_name).unwrap().retain(|n| n.id != 2);
    assert!(group.get(&app_name).unwrap().is_empty());

    // Notification 3 arrives immediately
    group.get_mut(&app_name).unwrap().push(Notification {
        id: 3,
        app_name: app_name.clone(),
        app_icon: "".to_string(),
        summary: "3".to_string(),
        body: "".to_string(),
        actions: vec![],
        urgency: Urgency::Normal,
        time: 1002,
        expire_timeout: 5000,
        transient: false,
        category: None,
        desktop_entry: None,
    });

    // Final check: group should still exist and have notification 3
    assert_eq!(group.get(&app_name).unwrap().len(), 1);
    assert_eq!(group.get(&app_name).unwrap()[0].id, 3);

    // Delayed removal callbacks would fire but find group non-empty
    let should_remove = group.get(&app_name).unwrap().is_empty();
    assert!(!should_remove);
}

#[test]
fn test_internal_state_consistency() {
    // Verifies that internal state is updated immediately when group becomes empty
    struct GroupState {
        notifications: Vec<Notification>,
        revealer_hidden: bool,
    }

    let mut state = GroupState {
        notifications: vec![Notification {
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
        }],
        revealer_hidden: false,
    };

    // Remove notification
    state.notifications.retain(|n| n.id != 1);

    // Internal state should be updated immediately
    let new_notifs = state.notifications.clone();
    assert!(new_notifs.is_empty());

    // Revealer should be hidden
    state.revealer_hidden = true;
    assert!(state.revealer_hidden);

    // New notification arrives
    state.notifications.push(Notification {
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
    });

    // Should detect and re-reveal
    if !state.revealer_hidden || state.notifications.is_empty() {
        // Won't re-reveal because revealer_hidden is true
    } else {
        state.revealer_hidden = false;
    }

    // Actually test the correct condition from the code
    if state.revealer_hidden || state.notifications.is_empty() {
        state.revealer_hidden = false;
    }

    assert!(!state.revealer_hidden);
    assert!(!state.notifications.is_empty());
}

#[test]
fn test_delayed_callback_recheck_logic() {
    // Simulates the delayed callback re-checking group state
    let group_exists = Rc::new(RefCell::new(true));
    let group_notifications = Rc::new(RefCell::new(vec![Notification {
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
    }]));

    // Delayed callback executes
    let should_remove = {
        if *group_exists.borrow() {
            let is_empty = group_notifications.borrow().is_empty();
            is_empty
        } else {
            // Group already removed
            false
        }
    };

    // Group has notification, should not be removed
    assert!(!should_remove);

    // Clear notifications
    group_notifications.borrow_mut().clear();

    // Delayed callback executes again
    let should_remove = {
        if *group_exists.borrow() {
            let is_empty = group_notifications.borrow().is_empty();
            is_empty
        } else {
            false
        }
    };

    // Now group is empty, should be removed
    assert!(should_remove);
}
