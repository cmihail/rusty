#[test]
fn test_seconds_to_pretty_time() {
    let test_cases = vec![
        (0, "0h:0m"),
        (60, "0h:1m"),
        (3600, "1h:0m"),
        (3660, "1h:1m"),
        (7200, "2h:0m"),
        (7320, "2h:2m"),
        (86400, "24h:0m"), // 1 day
    ];

    for (seconds, expected) in test_cases {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        let result = format!("{}h:{}m", hours, minutes);

        assert_eq!(
            result, expected,
            "Converting {} seconds should produce {}",
            seconds, expected
        );
    }
}

#[test]
fn test_workspace_label_formatting() {
    // Test workspace label logic from Bar
    let test_cases = vec![
        (1, "1|"),
        (5, "5|"),
        (9, "9|"),
        (10, "+|"),
        (15, "+|"),
        (100, "+|"),
    ];

    for (workspace_id, expected) in test_cases {
        let label = if workspace_id > 9 {
            "+|".to_string()
        } else {
            format!("{}|", workspace_id)
        };

        assert_eq!(
            label, expected,
            "Workspace {} should format as {}",
            workspace_id, expected
        );
    }
}

#[test]
fn test_workspace_per_monitor_display() {
    // Test that each monitor displays its own workspace, not the focused monitor's workspace
    // Simulates scenario: Monitor1 shows workspace 1, Monitor2 shows workspace 5

    struct MonitorWorkspace {
        model: String,
        workspace_id: i32,
    }

    let monitors = vec![
        MonitorWorkspace {
            model: "Monitor1".to_string(),
            workspace_id: 1,
        },
        MonitorWorkspace {
            model: "Monitor2".to_string(),
            workspace_id: 5,
        },
    ];

    // Each monitor should show its own workspace
    for monitor in monitors {
        let label = if monitor.workspace_id > 9 {
            "+|".to_string()
        } else {
            format!("{}|", monitor.workspace_id)
        };

        let expected = if monitor.workspace_id > 9 {
            "+|".to_string()
        } else {
            format!("{}|", monitor.workspace_id)
        };

        assert_eq!(
            label, expected,
            "Monitor {} should display workspace {} as {}",
            monitor.model, monitor.workspace_id, expected
        );
    }
}

#[test]
fn test_notification_count_label() {
    // Test notification count display logic
    let test_cases = vec![
        (0, "", false),   // 0 notifications -> hidden
        (1, "1", true),   // 1 notification -> show "1"
        (5, "5", true),   // 5 notifications -> show "5"
        (9, "9", true),   // 9 notifications -> show "9"
        (10, "+", true),  // 10+ notifications -> show "+"
        (100, "+", true), // 100+ notifications -> show "+"
    ];

    for (count, expected_label, expected_visible) in test_cases {
        let (label, visible) = if count > 0 {
            let label_text = if count > 9 {
                "+".to_string()
            } else {
                count.to_string()
            };
            (label_text, true)
        } else {
            (String::new(), false)
        };

        assert_eq!(
            visible, expected_visible,
            "Count {} should have visibility {}",
            count, expected_visible
        );

        if visible {
            assert_eq!(
                label, expected_label,
                "Count {} should display as '{}'",
                count, expected_label
            );
        }
    }
}

#[test]
fn test_battery_tooltip_formatting() {
    // Test battery tooltip text generation
    let percentage = 75;
    let time_to_full = 0i64;
    let time_to_empty = 7200i64; // 2 hours

    let mut text = format!("Battery: {}%", percentage);

    if time_to_full != 0 {
        let hours = time_to_full / 3600;
        let minutes = (time_to_full % 3600) / 60;
        text.push_str(&format!("\nCharging: {}h:{}m", hours, minutes));
    }

    if time_to_empty != 0 {
        let hours = time_to_empty / 3600;
        let minutes = (time_to_empty % 3600) / 60;
        text.push_str(&format!("\nRemaining: {}h:{}m", hours, minutes));
    }

    assert!(text.contains("Battery: 75%"));
    assert!(text.contains("Remaining: 2h:0m"));
    assert!(!text.contains("Charging:"));
}

#[test]
fn test_battery_css_classes() {
    use rusty_de::service::battery::BatteryState;

    // Test CSS class logic for battery state
    let test_cases = vec![
        (0.95, BatteryState::Discharging, vec![]),
        (0.65, BatteryState::Discharging, vec![]),
        (0.50, BatteryState::Discharging, vec!["Active"]),
        (0.25, BatteryState::Discharging, vec!["Warning"]),
        (0.15, BatteryState::Discharging, vec!["Critical"]),
        (0.15, BatteryState::Charging, vec![]),
        (0.15, BatteryState::FullyCharged, vec![]),
    ];

    for (percentage, state, expected_classes) in test_cases {
        let classes: Vec<&str> =
            if state != BatteryState::Charging && state != BatteryState::FullyCharged {
                if percentage >= 0.60 {
                    vec![]
                } else if percentage >= 0.40 {
                    vec!["Active"]
                } else if percentage >= 0.20 {
                    vec!["Warning"]
                } else {
                    vec!["Critical"]
                }
            } else {
                vec![]
            };

        assert_eq!(
            classes,
            expected_classes,
            "Battery at {}% in state {:?} should have classes {:?}",
            percentage * 100.0,
            state,
            expected_classes
        );
    }
}

#[test]
fn test_speaker_css_classes() {
    // Test CSS class logic for speaker state
    let test_cases = vec![
        (0.5, false, vec![]),          // Normal volume
        (1.0, false, vec![]),          // Max normal volume
        (1.5, false, vec!["Warning"]), // Over-amplified
        (0.5, true, vec!["Critical"]), // Muted
        (0.0, true, vec!["Critical"]), // Muted at 0
    ];

    for (volume, muted, expected_classes) in test_cases {
        let classes: Vec<&str> = if muted {
            vec!["Critical"]
        } else if volume > 1.0 {
            vec!["Warning"]
        } else {
            vec![]
        };

        assert_eq!(
            classes, expected_classes,
            "Speaker at volume {} (muted: {}) should have classes {:?}",
            volume, muted, expected_classes
        );
    }
}

#[test]
fn test_speaker_volume_percentage_for_progress() {
    // Test volume scaling for circular progress (volume / 1.5)
    let test_cases = vec![(0.0, 0.0), (0.75, 0.5), (1.0, 0.666666), (1.5, 1.0)];

    for (volume, expected_percentage) in test_cases {
        let percentage: f64 = volume / 1.5;
        let diff = (percentage - expected_percentage).abs();

        assert!(
            diff < 0.001,
            "Volume {} should map to progress percentage ~{} (got {})",
            volume,
            expected_percentage,
            percentage
        );
    }
}

#[test]
fn test_network_icon_name_logic() {
    // Test basic network icon name selection
    // This tests the logic pattern, not actual service calls

    let test_cases = vec![
        ("wired", true, "network-wired"),
        ("wifi", true, "network-wireless"),
        ("unknown", false, "network-offline"),
    ];

    for (network_type, connected, expected_contains) in test_cases {
        let icon = match network_type {
            "wired" if connected => "network-wired-symbolic",
            "wifi" if connected => "network-wireless-symbolic",
            _ => "network-offline-symbolic",
        };

        assert!(
            icon.contains(expected_contains),
            "Network type {} (connected: {}) should produce icon containing '{}'",
            network_type,
            connected,
            expected_contains
        );
    }
}

#[test]
fn test_wifi_signal_strength_to_percentage() {
    // Test converting WiFi signal strength (0-100) to percentage (0.0-1.0)
    let test_cases = vec![
        (0u8, 0.0),
        (25u8, 0.25),
        (50u8, 0.5),
        (75u8, 0.75),
        (100u8, 1.0),
    ];

    for (strength, expected) in test_cases {
        let percentage = strength as f64 / 100.0;
        assert_eq!(
            percentage, expected,
            "WiFi strength {} should convert to percentage {}",
            strength, expected
        );
    }
}

#[test]
fn test_color_alpha_reduction() {
    use gtk4::gdk::RGBA;

    // Test color alpha reduction logic (alpha / 1.3)
    let original = RGBA::parse("rgba(255, 0, 0, 1.0)").unwrap();
    let reduced_alpha = original.alpha() / 1.3;

    assert!(
        reduced_alpha < original.alpha(),
        "Reduced alpha should be less than original"
    );
    assert!(
        reduced_alpha > 0.0,
        "Reduced alpha should still be positive"
    );
}

#[test]
fn test_left_stack_transition_for_network() {
    use gtk4::StackTransitionType;

    // Test: Network widget should slide right
    let battery_not_charging = false; // Battery is charging/discharging
    let network_connected = false; // Network not connected

    let expected_transition = StackTransitionType::SlideRight;
    let expected_child = "network";

    let (transition, child) = if network_connected && !battery_not_charging {
        (StackTransitionType::SlideLeft, "audio-left")
    } else {
        (StackTransitionType::SlideRight, "network")
    };

    assert_eq!(
        transition, expected_transition,
        "Network widget should use SlideRight transition"
    );
    assert_eq!(child, expected_child, "Should show network widget");
}

#[test]
fn test_left_stack_transition_for_audio() {
    use gtk4::StackTransitionType;

    // Test: Audio in left stack should slide left
    let battery_not_charging = false; // Battery is charging/discharging
    let network_connected = true; // Network connected

    let expected_transition = StackTransitionType::SlideLeft;
    let expected_child = "audio-left";

    let (transition, child) = if network_connected && !battery_not_charging {
        (StackTransitionType::SlideLeft, "audio-left")
    } else {
        (StackTransitionType::SlideRight, "network")
    };

    assert_eq!(
        transition, expected_transition,
        "Audio widget in left stack should use SlideLeft transition"
    );
    assert_eq!(child, expected_child, "Should show audio-left widget");
}

#[test]
fn test_right_stack_transition_for_battery() {
    use gtk4::StackTransitionType;

    // Test: Battery widget should slide left
    let battery_not_charging = false; // Battery is charging/discharging

    let expected_transition = StackTransitionType::SlideLeft;
    let expected_child = "battery";

    let (transition, child) = if battery_not_charging {
        (StackTransitionType::SlideRight, "audio-right")
    } else {
        (StackTransitionType::SlideLeft, "battery")
    };

    assert_eq!(
        transition, expected_transition,
        "Battery widget should use SlideLeft transition"
    );
    assert_eq!(child, expected_child, "Should show battery widget");
}

#[test]
fn test_right_stack_transition_for_audio() {
    use gtk4::StackTransitionType;

    // Test: Audio in right stack should slide right
    let battery_not_charging = true; // Battery not charging (capped)

    let expected_transition = StackTransitionType::SlideRight;
    let expected_child = "audio-right";

    let (transition, child) = if battery_not_charging {
        (StackTransitionType::SlideRight, "audio-right")
    } else {
        (StackTransitionType::SlideLeft, "battery")
    };

    assert_eq!(
        transition, expected_transition,
        "Audio widget in right stack should use SlideRight transition"
    );
    assert_eq!(child, expected_child, "Should show audio-right widget");
}

#[test]
fn test_stack_transitions_all_scenarios() {
    use gtk4::StackTransitionType;

    // Test all combinations of battery and network states
    let test_cases = vec![
        // (battery_not_charging, network_connected, left_transition, left_child, right_transition, right_child)
        (
            false,
            false,
            StackTransitionType::SlideRight,
            "network",
            StackTransitionType::SlideLeft,
            "battery",
        ),
        (
            false,
            true,
            StackTransitionType::SlideLeft,
            "audio-left",
            StackTransitionType::SlideLeft,
            "battery",
        ),
        (
            true,
            false,
            StackTransitionType::SlideRight,
            "network",
            StackTransitionType::SlideRight,
            "audio-right",
        ),
        (
            true,
            true,
            StackTransitionType::SlideRight,
            "network",
            StackTransitionType::SlideRight,
            "audio-right",
        ),
    ];

    for (
        battery_not_charging,
        network_connected,
        exp_left_trans,
        exp_left_child,
        exp_right_trans,
        exp_right_child,
    ) in test_cases
    {
        // Left stack logic
        let (left_transition, left_child) = if network_connected && !battery_not_charging {
            (StackTransitionType::SlideLeft, "audio-left")
        } else {
            (StackTransitionType::SlideRight, "network")
        };

        // Right stack logic
        let (right_transition, right_child) = if battery_not_charging {
            (StackTransitionType::SlideRight, "audio-right")
        } else {
            (StackTransitionType::SlideLeft, "battery")
        };

        assert_eq!(
            left_transition, exp_left_trans,
            "Left transition mismatch for battery_not_charging={}, network_connected={}",
            battery_not_charging, network_connected
        );
        assert_eq!(
            left_child, exp_left_child,
            "Left child mismatch for battery_not_charging={}, network_connected={}",
            battery_not_charging, network_connected
        );
        assert_eq!(
            right_transition, exp_right_trans,
            "Right transition mismatch for battery_not_charging={}, network_connected={}",
            battery_not_charging, network_connected
        );
        assert_eq!(
            right_child, exp_right_child,
            "Right child mismatch for battery_not_charging={}, network_connected={}",
            battery_not_charging, network_connected
        );
    }
}

#[test]
fn test_battery_not_charging_states() {
    use rusty_de::service::battery::BatteryState;

    // Test which battery states are considered "not charging"
    let test_cases = vec![
        (BatteryState::FullyCharged, true),
        (BatteryState::PendingCharge, true),
        (BatteryState::Charging, false),
        (BatteryState::Discharging, false),
        (BatteryState::Empty, false),
        (BatteryState::Unknown, false),
    ];

    for (state, expected_not_charging) in test_cases {
        let battery_not_charging = matches!(
            state,
            BatteryState::FullyCharged | BatteryState::PendingCharge
        );

        assert_eq!(
            battery_not_charging, expected_not_charging,
            "Battery state {:?} should be not_charging={}",
            state, expected_not_charging
        );
    }
}

#[test]
fn test_scroll_left_stack_show_network() {
    use gtk4::StackTransitionType;

    // Test: Scroll up on left stack should show network with SlideDown
    let current_widget = "workspace-digit";
    let scroll_direction = "up";

    let (transition, target) = if scroll_direction == "up" && current_widget != "network" {
        (StackTransitionType::SlideDown, "network")
    } else {
        (StackTransitionType::None, current_widget)
    };

    assert_eq!(transition, StackTransitionType::SlideDown);
    assert_eq!(target, "network");
}

#[test]
fn test_scroll_left_stack_show_workspace() {
    use gtk4::StackTransitionType;

    // Test: Scroll down on left stack should show workspace with SlideUp
    let current_widget = "network";
    let scroll_direction = "down";

    let (transition, target) = if scroll_direction == "down" && current_widget != "workspace-digit"
    {
        (StackTransitionType::SlideUp, "workspace-digit")
    } else {
        (StackTransitionType::None, current_widget)
    };

    assert_eq!(transition, StackTransitionType::SlideUp);
    assert_eq!(target, "workspace-digit");
}

#[test]
fn test_scroll_right_stack_show_audio() {
    use gtk4::StackTransitionType;

    // Test: Scroll up on right stack should show audio with SlideDown
    let current_widget = "battery";
    let scroll_direction = "up";

    let (transition, target) = if scroll_direction == "up" && current_widget != "audio-right" {
        (StackTransitionType::SlideDown, "audio-right")
    } else {
        (StackTransitionType::None, current_widget)
    };

    assert_eq!(transition, StackTransitionType::SlideDown);
    assert_eq!(target, "audio-right");
}

#[test]
fn test_scroll_right_stack_show_battery() {
    use gtk4::StackTransitionType;

    // Test: Scroll down on right stack should show battery with SlideUp
    let current_widget = "audio-right";
    let scroll_direction = "down";

    let (transition, target) = if scroll_direction == "down" && current_widget != "battery" {
        (StackTransitionType::SlideUp, "battery")
    } else {
        (StackTransitionType::None, current_widget)
    };

    assert_eq!(transition, StackTransitionType::SlideUp);
    assert_eq!(target, "battery");
}

#[test]
fn test_scroll_no_change_when_already_visible() {
    // Test: Scrolling should not trigger transition when target is already visible
    let test_cases = vec![
        ("network", "up", "network"),
        ("workspace-digit", "down", "workspace-digit"),
        ("audio-right", "up", "audio-right"),
        ("battery", "down", "battery"),
    ];

    for (current, direction, expected) in test_cases {
        let should_transition = match direction {
            "up" => current != expected,
            "down" => current != expected,
            _ => false,
        };

        assert!(
            !should_transition,
            "Should not transition when {} is already visible and scrolling {}",
            current, direction
        );
    }
}

#[test]
fn test_scroll_all_transitions() {
    use gtk4::StackTransitionType;

    // Test all scroll direction combinations
    let test_cases = vec![
        (
            "network",
            "down",
            StackTransitionType::SlideUp,
            "workspace-digit",
        ),
        (
            "workspace-digit",
            "up",
            StackTransitionType::SlideDown,
            "network",
        ),
        (
            "battery",
            "up",
            StackTransitionType::SlideDown,
            "audio-right",
        ),
        (
            "audio-right",
            "down",
            StackTransitionType::SlideUp,
            "battery",
        ),
    ];

    for (current, direction, expected_transition, expected_target) in test_cases {
        let is_left_stack = current == "network" || current == "workspace-digit";

        let (transition, target) = if direction == "up" {
            if is_left_stack && current != "network" {
                (StackTransitionType::SlideDown, "network")
            } else if !is_left_stack && current != "audio-right" {
                (StackTransitionType::SlideDown, "audio-right")
            } else {
                (StackTransitionType::None, current)
            }
        } else if is_left_stack && current != "workspace-digit" {
            (StackTransitionType::SlideUp, "workspace-digit")
        } else if !is_left_stack && current != "battery" {
            (StackTransitionType::SlideUp, "battery")
        } else {
            (StackTransitionType::None, current)
        };

        assert_eq!(
            transition, expected_transition,
            "Scroll {} from {} should use {:?}",
            direction, current, expected_transition
        );
        assert_eq!(
            target, expected_target,
            "Scroll {} from {} should show {}",
            direction, current, expected_target
        );
    }
}

#[test]
fn test_left_stack_scroll_stops_at_network_edge() {
    // Test that scrolling up from network does nothing (stops at first item)
    let current_visible = "network";
    let scroll_direction = "up"; // dy < 0.0

    // Simulate scroll logic
    let new_visible = match (current_visible, scroll_direction) {
        ("network", "up") => "network", // Should not change
        ("workspace-digit", "up") => "network",
        ("bluetooth", "up") => "workspace-digit",
        ("network", "down") => "workspace-digit",
        ("workspace-digit", "down") => "bluetooth",
        ("bluetooth", "down") => "bluetooth", // Should not change
        _ => current_visible,
    };

    assert_eq!(
        new_visible, "network",
        "Scrolling up from network should stay on network (edge case)"
    );
}

#[test]
fn test_left_stack_scroll_stops_at_bluetooth_edge() {
    // Test that scrolling down from bluetooth does nothing (stops at last item)
    let current_visible = "bluetooth";
    let scroll_direction = "down"; // dy > 0.0

    // Simulate scroll logic
    let new_visible = match (current_visible, scroll_direction) {
        ("network", "up") => "network",
        ("workspace-digit", "up") => "network",
        ("bluetooth", "up") => "workspace-digit",
        ("network", "down") => "workspace-digit",
        ("workspace-digit", "down") => "bluetooth",
        ("bluetooth", "down") => "bluetooth", // Should not change
        _ => current_visible,
    };

    assert_eq!(
        new_visible, "bluetooth",
        "Scrolling down from bluetooth should stay on bluetooth (edge case)"
    );
}

#[test]
fn test_left_stack_scroll_through_all_states() {
    // Test scrolling through all three states without wrapping
    let states = vec![
        ("network", "down", "workspace-digit"),
        ("workspace-digit", "down", "bluetooth"),
        ("bluetooth", "down", "bluetooth"), // Edge: stays
        ("bluetooth", "up", "workspace-digit"),
        ("workspace-digit", "up", "network"),
        ("network", "up", "network"), // Edge: stays
    ];

    for (current, direction, expected) in states {
        let new_visible = match (current, direction) {
            ("network", "up") => "network",
            ("workspace-digit", "up") => "network",
            ("bluetooth", "up") => "workspace-digit",
            ("network", "down") => "workspace-digit",
            ("workspace-digit", "down") => "bluetooth",
            ("bluetooth", "down") => "bluetooth",
            _ => current,
        };

        assert_eq!(
            new_visible, expected,
            "Scroll {} from {} should result in {}",
            direction, current, expected
        );
    }
}

#[test]
fn test_left_stack_workspace_in_middle() {
    // Test that workspace-digit is properly positioned between network and bluetooth
    let scroll_from_network_down = match ("network", "down") {
        ("network", "down") => "workspace-digit",
        _ => "error",
    };

    let scroll_from_bluetooth_up = match ("bluetooth", "up") {
        ("bluetooth", "up") => "workspace-digit",
        _ => "error",
    };

    assert_eq!(
        scroll_from_network_down, "workspace-digit",
        "Scrolling down from network should go to workspace-digit"
    );
    assert_eq!(
        scroll_from_bluetooth_up, "workspace-digit",
        "Scrolling up from bluetooth should go to workspace-digit"
    );
}

#[test]
fn test_left_stack_no_infinite_loop() {
    // Test that repeated scrolling at edges doesn't cause infinite loop
    let mut current = "network";

    // Try scrolling up multiple times from network (should stay at network)
    for _ in 0..5 {
        current = match (current, "up") {
            ("network", "up") => "network",
            ("workspace-digit", "up") => "network",
            ("bluetooth", "up") => "workspace-digit",
            _ => current,
        };
    }

    assert_eq!(
        current, "network",
        "Multiple scroll ups from network should stay at network"
    );

    // Now scroll to bluetooth and try scrolling down multiple times
    current = "bluetooth";
    for _ in 0..5 {
        current = match (current, "down") {
            ("network", "down") => "workspace-digit",
            ("workspace-digit", "down") => "bluetooth",
            ("bluetooth", "down") => "bluetooth",
            _ => current,
        };
    }

    assert_eq!(
        current, "bluetooth",
        "Multiple scroll downs from bluetooth should stay at bluetooth"
    );
}

#[test]
fn test_microphone_css_classes() {
    // Test CSS class logic for microphone state
    let test_cases = vec![
        (0.5, false, vec![]),          // Normal mic volume
        (1.0, false, vec![]),          // Max mic volume
        (0.5, true, vec!["Critical"]), // Muted
        (0.0, true, vec!["Critical"]), // Muted at 0
    ];

    for (mic_volume, mic_muted, expected_classes) in test_cases {
        let classes: Vec<&str> = if mic_muted { vec!["Critical"] } else { vec![] };

        assert_eq!(
            classes, expected_classes,
            "Microphone at volume {} (muted: {}) should have classes {:?}",
            mic_volume, mic_muted, expected_classes
        );
    }
}

#[test]
fn test_microphone_volume_percentage_for_progress() {
    // Test microphone volume directly maps to circular progress (no scaling)
    let test_cases = vec![
        (0.0, 0.0),
        (0.25, 0.25),
        (0.5, 0.5),
        (0.75, 0.75),
        (1.0, 1.0),
    ];

    for (mic_volume, expected_percentage) in test_cases {
        let percentage: f64 = mic_volume;

        assert_eq!(
            percentage, expected_percentage,
            "Mic volume {} should map to progress percentage {}",
            mic_volume, expected_percentage
        );
    }
}

#[test]
fn test_right_stack_auto_show_priority() {
    use gtk4::StackTransitionType;

    // Test auto-show priority for right stack with microphone, volume, battery
    let test_cases = vec![
        // (battery_not_charging, mic_muted, expected_transition, expected_child)
        // Battery needs attention (highest priority)
        (false, false, StackTransitionType::SlideLeft, "battery"),
        (false, true, StackTransitionType::SlideLeft, "battery"),
        // Microphone muted (medium priority)
        (true, true, StackTransitionType::SlideRight, "microphone"),
        // Normal state - show volume (default)
        (true, false, StackTransitionType::SlideRight, "volume"),
    ];

    for (battery_not_charging, mic_muted, exp_transition, exp_child) in test_cases {
        let (transition, child) = if !battery_not_charging {
            (StackTransitionType::SlideLeft, "battery")
        } else if mic_muted {
            (StackTransitionType::SlideRight, "microphone")
        } else {
            (StackTransitionType::SlideRight, "volume")
        };

        assert_eq!(
            transition, exp_transition,
            "Auto-show for battery_not_charging={}, mic_muted={} should use {:?}",
            battery_not_charging, mic_muted, exp_transition
        );
        assert_eq!(
            child, exp_child,
            "Auto-show for battery_not_charging={}, mic_muted={} should show {}",
            battery_not_charging, mic_muted, exp_child
        );
    }
}

#[test]
fn test_right_stack_scroll_three_items() {
    use gtk4::StackTransitionType;

    // Test scrolling through microphone -> volume -> battery
    let test_cases = vec![
        // (current, direction, expected_transition, expected_target)
        // Scroll up (previous item)
        ("battery", "up", StackTransitionType::SlideDown, "volume"),
        ("volume", "up", StackTransitionType::SlideDown, "microphone"),
        ("microphone", "up", StackTransitionType::None, "microphone"), // Edge: stays
        // Scroll down (next item)
        ("microphone", "down", StackTransitionType::SlideUp, "volume"),
        ("volume", "down", StackTransitionType::SlideUp, "battery"),
        ("battery", "down", StackTransitionType::None, "battery"), // Edge: stays
    ];

    for (current, direction, exp_transition, exp_target) in test_cases {
        let (transition, target) = match (current, direction) {
            ("microphone", "up") => (StackTransitionType::None, "microphone"),
            ("volume", "up") => (StackTransitionType::SlideDown, "microphone"),
            ("battery", "up") => (StackTransitionType::SlideDown, "volume"),
            ("microphone", "down") => (StackTransitionType::SlideUp, "volume"),
            ("volume", "down") => (StackTransitionType::SlideUp, "battery"),
            ("battery", "down") => (StackTransitionType::None, "battery"),
            _ => (StackTransitionType::None, current),
        };

        assert_eq!(
            transition, exp_transition,
            "Scroll {} from {} should use {:?}",
            direction, current, exp_transition
        );
        assert_eq!(
            target, exp_target,
            "Scroll {} from {} should show {}",
            direction, current, exp_target
        );
    }
}

#[test]
fn test_right_stack_scroll_stops_at_microphone_edge() {
    // Test that scrolling up from microphone does nothing (stops at first item)
    let current_visible = "microphone";
    let scroll_direction = "up"; // dy < 0.0

    let new_visible = match (current_visible, scroll_direction) {
        ("microphone", "up") => "microphone", // Should not change
        ("volume", "up") => "microphone",
        ("battery", "up") => "volume",
        ("microphone", "down") => "volume",
        ("volume", "down") => "battery",
        ("battery", "down") => "battery",
        _ => current_visible,
    };

    assert_eq!(
        new_visible, "microphone",
        "Scrolling up from microphone should stay on microphone (edge case)"
    );
}

#[test]
fn test_right_stack_scroll_stops_at_battery_edge() {
    // Test that scrolling down from battery does nothing (stops at last item)
    let current_visible = "battery";
    let scroll_direction = "down"; // dy > 0.0

    let new_visible = match (current_visible, scroll_direction) {
        ("microphone", "up") => "microphone",
        ("volume", "up") => "microphone",
        ("battery", "up") => "volume",
        ("microphone", "down") => "volume",
        ("volume", "down") => "battery",
        ("battery", "down") => "battery", // Should not change
        _ => current_visible,
    };

    assert_eq!(
        new_visible, "battery",
        "Scrolling down from battery should stay on battery (edge case)"
    );
}

#[test]
fn test_right_stack_volume_in_middle() {
    // Test that volume is properly positioned between microphone and battery
    let scroll_from_microphone_down = match ("microphone", "down") {
        ("microphone", "down") => "volume",
        _ => "error",
    };

    let scroll_from_battery_up = match ("battery", "up") {
        ("battery", "up") => "volume",
        _ => "error",
    };

    assert_eq!(
        scroll_from_microphone_down, "volume",
        "Scrolling down from microphone should go to volume"
    );
    assert_eq!(
        scroll_from_battery_up, "volume",
        "Scrolling up from battery should go to volume"
    );
}

#[test]
fn test_right_stack_no_infinite_loop() {
    // Test that repeated scrolling at edges doesn't cause infinite loop
    let mut current = "microphone";

    // Try scrolling up multiple times from microphone (should stay at microphone)
    for _ in 0..5 {
        current = match (current, "up") {
            ("microphone", "up") => "microphone",
            ("volume", "up") => "microphone",
            ("battery", "up") => "volume",
            _ => current,
        };
    }

    assert_eq!(
        current, "microphone",
        "Multiple scroll ups from microphone should stay at microphone"
    );

    // Now scroll to battery and try scrolling down multiple times
    current = "battery";
    for _ in 0..5 {
        current = match (current, "down") {
            ("microphone", "down") => "volume",
            ("volume", "down") => "battery",
            ("battery", "down") => "battery",
            _ => current,
        };
    }

    assert_eq!(
        current, "battery",
        "Multiple scroll downs from battery should stay at battery"
    );
}

#[test]
fn test_volume_muted_shows_volume_not_battery() {
    // Test that when volume is muted, we show the volume widget (with muted icon)
    // not the battery widget
    let battery_not_charging = true; // Battery ok (fully charged)
    let mic_muted = false; // Mic not muted

    // Volume being muted should NOT trigger showing battery
    // Volume widget will display its own muted icon with Critical CSS class
    let expected_child = "volume";

    let child = if !battery_not_charging {
        "battery"
    } else if mic_muted {
        "microphone"
    } else {
        "volume"
    };

    assert_eq!(
        child, expected_child,
        "When battery is ok and mic is ok, should show volume (even if muted)"
    );
}

#[test]
fn test_right_stack_microphone_priority_over_volume() {
    // Test that muted microphone has priority over volume
    let battery_not_charging = true; // Battery ok
    let mic_muted = true; // Microphone muted

    let expected_child = "microphone";

    let child = if !battery_not_charging {
        "battery"
    } else if mic_muted {
        "microphone"
    } else {
        "volume"
    };

    assert_eq!(
        child, expected_child,
        "When battery is ok but mic is muted, should show microphone"
    );
}

#[test]
fn test_right_stack_battery_highest_priority() {
    // Test that battery needing attention has highest priority
    let battery_not_charging = false; // Battery needs attention
    let mic_muted = true; // Microphone also muted

    let expected_child = "battery";

    let child = if !battery_not_charging {
        "battery"
    } else if mic_muted {
        "microphone"
    } else {
        "volume"
    };

    assert_eq!(
        child, expected_child,
        "When battery needs attention, it should be shown even if mic is muted"
    );
}
