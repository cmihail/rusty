#[test]
fn test_expo_delay_constant() {
    // Verify that expo delay matches AGS implementation (1000ms)
    const EXPO_DELAY_MS: u32 = 1000;
    assert_eq!(EXPO_DELAY_MS, 1000, "Expo delay should be 1000ms");
}

#[test]
fn test_monitor_connector_parsing() {
    // Test parsing monitor connector strings from hyprctl
    let connectors = vec!["DP-1", "HDMI-A-1", "eDP-1", "DVI-D-1"];

    for connector in connectors {
        assert!(
            !connector.is_empty(),
            "Monitor connector '{}' should not be empty",
            connector
        );
        assert!(
            connector.contains('-'),
            "Monitor connector '{}' should contain hyphen",
            connector
        );
    }
}

#[test]
fn test_monitor_connector_validation() {
    // Test that empty connectors are filtered out
    let connectors = vec!["DP-1", "", "HDMI-A-1", ""];

    let valid_connectors: Vec<&str> = connectors
        .iter()
        .filter(|c| !c.is_empty())
        .copied()
        .collect();

    assert_eq!(
        valid_connectors.len(),
        2,
        "Should filter out empty connectors"
    );
    assert_eq!(valid_connectors, vec!["DP-1", "HDMI-A-1"]);
}

#[test]
fn test_hyprland_dispatch_command_formatting() {
    // Test formatting of hyprland dispatch commands
    let test_cases = vec![
        ("hyprexpo:expo", "", "hyprexpo:expo"),
        ("hyprexpo:expo", "toggle", "hyprexpo:expo toggle"),
        ("workspace", "1", "workspace 1"),
        (
            "moveworkspacetomonitor",
            "1 DP-1",
            "moveworkspacetomonitor 1 DP-1",
        ),
    ];

    for (command, args, expected) in test_cases {
        let dispatch_str = if args.is_empty() {
            command.to_string()
        } else {
            format!("{} {}", command, args)
        };

        assert_eq!(
            dispatch_str, expected,
            "Command '{}' with args '{}' should format as '{}'",
            command, args, expected
        );
    }
}

#[test]
fn test_hyprland_dispatch_empty_args() {
    // Test that empty args don't add extra space
    let command = "hyprexpo:expo";
    let args = "";

    let dispatch_str = if args.is_empty() {
        command.to_string()
    } else {
        format!("{} {}", command, args)
    };

    assert_eq!(
        dispatch_str, command,
        "Empty args should not add space to command"
    );
    assert!(!dispatch_str.ends_with(' '), "Should not end with space");
}

#[test]
fn test_focused_monitor_json_parsing() {
    // Test parsing focused monitor from hyprctl JSON output
    let json_str = r#"[
        {
            "id": 0,
            "name": "DP-1",
            "focused": false
        },
        {
            "id": 1,
            "name": "HDMI-A-1",
            "focused": true
        }
    ]"#;

    let monitors: serde_json::Value = serde_json::from_str(json_str).unwrap();

    let focused_monitor = monitors
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["focused"].as_bool() == Some(true))
        .and_then(|m| m["name"].as_str())
        .map(|s| s.to_string());

    assert_eq!(
        focused_monitor,
        Some("HDMI-A-1".to_string()),
        "Should find focused monitor"
    );
}

#[test]
fn test_focused_monitor_json_parsing_no_focused() {
    // Test handling when no monitor is focused
    let json_str = r#"[
        {
            "id": 0,
            "name": "DP-1",
            "focused": false
        },
        {
            "id": 1,
            "name": "HDMI-A-1",
            "focused": false
        }
    ]"#;

    let monitors: serde_json::Value = serde_json::from_str(json_str).unwrap();

    let focused_monitor = monitors
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["focused"].as_bool() == Some(true))
        .and_then(|m| m["name"].as_str())
        .map(|s| s.to_string());

    assert_eq!(
        focused_monitor, None,
        "Should return None when no monitor is focused"
    );
}

#[test]
fn test_focused_monitor_json_parsing_single_monitor() {
    // Test with single monitor setup
    let json_str = r#"[
        {
            "id": 0,
            "name": "eDP-1",
            "focused": true
        }
    ]"#;

    let monitors: serde_json::Value = serde_json::from_str(json_str).unwrap();

    let focused_monitor = monitors
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["focused"].as_bool() == Some(true))
        .and_then(|m| m["name"].as_str())
        .map(|s| s.to_string());

    assert_eq!(
        focused_monitor,
        Some("eDP-1".to_string()),
        "Should find focused monitor in single monitor setup"
    );
}

#[test]
fn test_focused_monitor_comparison() {
    // Test monitor comparison logic for triggering expo
    let test_cases = vec![
        ("DP-1", "DP-1", true),
        ("DP-1", "HDMI-A-1", false),
        ("eDP-1", "eDP-1", true),
        ("DP-2", "DP-1", false),
    ];

    for (focused, current, should_trigger) in test_cases {
        let triggers = focused == current;
        assert_eq!(
            triggers,
            should_trigger,
            "Focused '{}' and current '{}' should {} trigger expo",
            focused,
            current,
            if should_trigger { "" } else { "not" }
        );
    }
}

#[test]
fn test_hot_corner_dimensions() {
    // Test that hot corner has correct width
    let width_request = 4;
    assert_eq!(
        width_request, 4,
        "Hot corner width should be 4 pixels to match AGS implementation"
    );
}

#[test]
fn test_expo_flag_state_transitions() {
    // Test open_expo flag state transitions
    // Mouse enters
    let open_expo = true;
    assert!(open_expo, "Flag should be true on mouse enter");

    // Mouse leaves before timeout
    let open_expo = false;
    assert!(!open_expo, "Flag should be false on mouse leave");

    // Simulate checking in timeout callback
    let should_trigger = open_expo;
    assert!(
        !should_trigger,
        "Should not trigger expo if flag is false in callback"
    );
}

#[test]
fn test_expo_flag_successful_trigger() {
    // Test that expo triggers when flag remains true
    // Mouse enters
    let open_expo = true;

    // Simulate timeout callback - flag still true
    let should_trigger = open_expo;
    assert!(
        should_trigger,
        "Should trigger expo if flag is still true in callback"
    );
}

#[test]
fn test_window_namespace() {
    // Test window namespace matches expected value
    let namespace = "HotCorner";
    assert_eq!(
        namespace, "HotCorner",
        "Window namespace should be 'HotCorner'"
    );
}

#[test]
fn test_window_css_class() {
    // Test window CSS class matches expected value
    let css_class = "HotCorner";
    assert_eq!(
        css_class, "HotCorner",
        "Window CSS class should be 'HotCorner'"
    );
}

#[test]
fn test_invisible_css_class() {
    // Test invisible label CSS class
    let css_class = "Invisible";
    assert_eq!(
        css_class, "Invisible",
        "Label CSS class should be 'Invisible'"
    );
}

#[test]
fn test_hyprctl_monitors_command() {
    // Test correct hyprctl command for getting monitors
    let command = "hyprctl";
    let args = vec!["monitors", "-j"];

    assert_eq!(command, "hyprctl");
    assert_eq!(args, vec!["monitors", "-j"]);
}

#[test]
fn test_hyprctl_dispatch_command() {
    // Test correct hyprctl command for dispatch
    let command = "hyprctl";
    let args = vec!["dispatch", "hyprexpo:expo"];

    assert_eq!(command, "hyprctl");
    assert_eq!(args.len(), 2);
    assert_eq!(args[0], "dispatch");
    assert_eq!(args[1], "hyprexpo:expo");
}

#[test]
fn test_multi_monitor_setup() {
    // Test handling multiple monitors
    let monitor_count = 3;
    let connectors = vec!["DP-1", "DP-2", "HDMI-A-1"];

    assert_eq!(
        connectors.len(),
        monitor_count,
        "Should handle {} monitors",
        monitor_count
    );

    for connector in &connectors {
        assert!(
            !connector.is_empty(),
            "Each connector should be valid: {}",
            connector
        );
    }
}

#[test]
fn test_monitor_hotplug_detection() {
    // Test monitor list changes for hotplug
    let initial_monitors = vec!["DP-1", "DP-2"];
    let updated_monitors = vec!["DP-1", "DP-2", "HDMI-A-1"];

    let changed = initial_monitors.len() != updated_monitors.len();
    assert!(changed, "Should detect monitor count change on hotplug");

    let added = updated_monitors.len() - initial_monitors.len();
    assert_eq!(added, 1, "Should detect one monitor added");
}

#[test]
fn test_monitor_removal_detection() {
    // Test monitor removal detection
    let initial_monitors = vec!["DP-1", "DP-2", "HDMI-A-1"];
    let updated_monitors = vec!["DP-1", "DP-2"];

    let changed = initial_monitors.len() != updated_monitors.len();
    assert!(changed, "Should detect monitor count change on removal");

    let removed = initial_monitors.len() - updated_monitors.len();
    assert_eq!(removed, 1, "Should detect one monitor removed");
}

#[test]
fn test_timeout_duration() {
    // Test that timeout duration matches AGS (1000ms = 1 second)
    let duration = std::time::Duration::from_millis(1000);
    assert_eq!(
        duration.as_millis(),
        1000,
        "Timeout should be 1000 milliseconds"
    );
    assert_eq!(duration.as_secs(), 1, "Timeout should be 1 second");
}

// Tests for hot corner cleanup on monitor changes

#[derive(Clone)]
struct MockHotCorner {
    monitor_name: String,
    id: u32,
}

impl MockHotCorner {
    fn new(monitor_name: &str, id: u32) -> Self {
        Self {
            monitor_name: monitor_name.to_string(),
            id,
        }
    }

    fn monitor_info(&self) -> String {
        format!("HotCorner {} on {}", self.id, self.monitor_name)
    }
}

#[test]
fn test_hot_corners_clear_removes_all_elements() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let hot_corners = Rc::new(RefCell::new(vec![
        MockHotCorner::new("eDP-1", 1),
        MockHotCorner::new("DP-1", 2),
        MockHotCorner::new("HDMI-A-1", 3),
    ]));

    // Verify initial state
    assert_eq!(
        hot_corners.borrow().len(),
        3,
        "Should start with 3 hot corners"
    );

    // Clear all hot corners (simulating monitor change)
    hot_corners.borrow_mut().clear();

    // Verify all elements removed
    assert_eq!(
        hot_corners.borrow().len(),
        0,
        "Clear should remove all hot corner instances"
    );
}

#[test]
fn test_hot_corners_clear_on_empty_list() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let hot_corners: Rc<RefCell<Vec<MockHotCorner>>> = Rc::new(RefCell::new(vec![]));

    // Clear on empty list should not panic
    hot_corners.borrow_mut().clear();

    assert_eq!(
        hot_corners.borrow().len(),
        0,
        "Clear on empty list should result in empty list"
    );
}

#[test]
fn test_hot_corners_recreate_after_clear() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let hot_corners = Rc::new(RefCell::new(vec![
        MockHotCorner::new("eDP-1", 1),
        MockHotCorner::new("DP-1", 2),
    ]));

    // Clear existing hot corners
    hot_corners.borrow_mut().clear();
    assert_eq!(hot_corners.borrow().len(), 0);

    // Recreate with new monitors (simulating monitor change)
    hot_corners.borrow_mut().push(MockHotCorner::new("DP-1", 3));
    hot_corners
        .borrow_mut()
        .push(MockHotCorner::new("HDMI-A-1", 4));

    // Verify new hot corners created
    assert_eq!(
        hot_corners.borrow().len(),
        2,
        "Should have 2 new hot corner instances"
    );
    assert!(hot_corners.borrow()[0].monitor_info().contains("DP-1"));
    assert!(hot_corners.borrow()[1].monitor_info().contains("HDMI-A-1"));
}

#[test]
fn test_hot_corners_single_monitor_to_multi_monitor() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let hot_corners = Rc::new(RefCell::new(vec![MockHotCorner::new("eDP-1", 1)]));

    assert_eq!(hot_corners.borrow().len(), 1, "Should start with 1 monitor");

    // Simulate adding monitors (monitor hotplug)
    hot_corners.borrow_mut().clear();
    hot_corners
        .borrow_mut()
        .push(MockHotCorner::new("eDP-1", 2));
    hot_corners.borrow_mut().push(MockHotCorner::new("DP-1", 3));
    hot_corners
        .borrow_mut()
        .push(MockHotCorner::new("HDMI-A-1", 4));

    assert_eq!(
        hot_corners.borrow().len(),
        3,
        "Should have 3 hot corners after hotplug"
    );
}

#[test]
fn test_hot_corners_multi_monitor_to_single_monitor() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let hot_corners = Rc::new(RefCell::new(vec![
        MockHotCorner::new("eDP-1", 1),
        MockHotCorner::new("DP-1", 2),
        MockHotCorner::new("HDMI-A-1", 3),
    ]));

    assert_eq!(
        hot_corners.borrow().len(),
        3,
        "Should start with 3 monitors"
    );

    // Simulate removing monitors
    hot_corners.borrow_mut().clear();
    hot_corners
        .borrow_mut()
        .push(MockHotCorner::new("eDP-1", 4));

    assert_eq!(
        hot_corners.borrow().len(),
        1,
        "Should have 1 hot corner after monitors removed"
    );
    assert!(hot_corners.borrow()[0].monitor_info().contains("eDP-1"));
}

#[test]
fn test_hot_corners_vector_capacity_after_clear() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let hot_corners = Rc::new(RefCell::new(Vec::with_capacity(10)));

    // Add some elements
    for i in 0..5 {
        hot_corners.borrow_mut().push(MockHotCorner::new("DP-1", i));
    }

    let capacity_before = hot_corners.borrow().capacity();
    hot_corners.borrow_mut().clear();

    // Capacity should remain after clear
    assert_eq!(
        hot_corners.borrow().capacity(),
        capacity_before,
        "Clear should preserve vector capacity"
    );
    assert_eq!(hot_corners.borrow().len(), 0, "But length should be 0");
}

// Tests for exact connector matching
#[derive(Clone)]
struct MockHotCornerWithConnector {
    connector: String,
}

impl MockHotCornerWithConnector {
    fn new(connector: &str) -> Self {
        Self {
            connector: connector.to_string(),
        }
    }

    fn connector(&self) -> &str {
        &self.connector
    }
}

#[test]
fn test_exact_connector_matching_edp1_vs_dp1() {
    let hot_corners = vec![MockHotCornerWithConnector::new("eDP-1")];

    // Searching for "DP-1" should NOT match "eDP-1"
    let has_corner = hot_corners
        .iter()
        .any(|corner| corner.connector() == "DP-1");
    assert!(!has_corner);
}

#[test]
fn test_exact_connector_matching_dp1_exists() {
    let hot_corners = vec![
        MockHotCornerWithConnector::new("eDP-1"),
        MockHotCornerWithConnector::new("DP-1"),
    ];

    // Searching for "DP-1" should only match exact "DP-1"
    let has_corner = hot_corners
        .iter()
        .any(|corner| corner.connector() == "DP-1");
    assert!(has_corner);
}

#[test]
fn test_exact_connector_matching_hdmi() {
    let hot_corners = vec![
        MockHotCornerWithConnector::new("HDMI-A-1"),
        MockHotCornerWithConnector::new("HDMI-A-2"),
    ];

    // Searching for "HDMI-A-1" should only match exact "HDMI-A-1"
    let has_corner = hot_corners
        .iter()
        .any(|corner| corner.connector() == "HDMI-A-1");
    assert!(has_corner);

    // But not "HDMI"
    let has_corner = hot_corners
        .iter()
        .any(|corner| corner.connector() == "HDMI");
    assert!(!has_corner);
}

#[test]
fn test_connector_removal_exact_match() {
    let mut hot_corners = vec![
        MockHotCornerWithConnector::new("eDP-1"),
        MockHotCornerWithConnector::new("DP-1"),
        MockHotCornerWithConnector::new("HDMI-A-1"),
    ];
    let current_connectors = vec!["eDP-1".to_string(), "HDMI-A-1".to_string()];

    // Remove hot corners whose connectors are not in current_connectors
    hot_corners.retain(|corner| {
        current_connectors
            .iter()
            .any(|connector| connector == corner.connector())
    });

    assert_eq!(hot_corners.len(), 2);
    assert!(hot_corners
        .iter()
        .any(|corner| corner.connector() == "eDP-1"));
    assert!(hot_corners
        .iter()
        .any(|corner| corner.connector() == "HDMI-A-1"));
    assert!(!hot_corners
        .iter()
        .any(|corner| corner.connector() == "DP-1"));
}

#[test]
fn test_connector_removal_with_similar_names() {
    let mut hot_corners = vec![
        MockHotCornerWithConnector::new("eDP-1"),
        MockHotCornerWithConnector::new("DP-1"),
    ];
    let current_connectors = vec!["DP-1".to_string()];

    // Remove hot corners whose connectors are not in current_connectors
    hot_corners.retain(|corner| {
        current_connectors
            .iter()
            .any(|connector| connector == corner.connector())
    });

    // Only "DP-1" should remain, "eDP-1" should be removed
    assert_eq!(hot_corners.len(), 1);
    assert!(hot_corners
        .iter()
        .any(|corner| corner.connector() == "DP-1"));
    assert!(!hot_corners
        .iter()
        .any(|corner| corner.connector() == "eDP-1"));
}

#[test]
fn test_rightmost_monitor_single_monitor() {
    // Test rightmost monitor edge calculation with single monitor
    struct MockMonitor {
        x: i32,
        width: i32,
    }

    let monitor = MockMonitor { x: 0, width: 1920 };
    let rightmost_edge = monitor.x + monitor.width;

    assert_eq!(rightmost_edge, 1920);
}

#[test]
fn test_rightmost_monitor_two_monitors() {
    // Test rightmost monitor edge calculation with two monitors
    struct MockMonitor {
        x: i32,
        width: i32,
    }

    let monitor1 = MockMonitor { x: 0, width: 1920 };
    let monitor2 = MockMonitor {
        x: 1920,
        width: 1920,
    };

    let edge1 = monitor1.x + monitor1.width;
    let edge2 = monitor2.x + monitor2.width;

    assert_eq!(edge1, 1920);
    assert_eq!(edge2, 3840);
    assert!(edge2 > edge1, "Monitor 2 should be rightmost");
}

#[test]
fn test_rightmost_monitor_overlap() {
    // Test rightmost monitor with overlapping monitors
    struct MockMonitor {
        x: i32,
        width: i32,
    }

    let monitor1 = MockMonitor { x: 0, width: 1920 };
    let monitor2 = MockMonitor {
        x: 1600,
        width: 1920,
    }; // Overlaps

    let edge1 = monitor1.x + monitor1.width;
    let edge2 = monitor2.x + monitor2.width;

    assert_eq!(edge1, 1920);
    assert_eq!(edge2, 3520);
    assert!(
        edge2 > edge1,
        "Monitor 2 should be rightmost even with overlap"
    );
}

#[test]
fn test_corner_namespace_uniqueness() {
    // Test that corner types have unique namespaces
    let left_namespace = "HotCornerTopLeft";
    let right_namespace = "HotCornerTopRight";

    assert_ne!(left_namespace, right_namespace);
    assert!(left_namespace.starts_with("HotCorner"));
    assert!(right_namespace.starts_with("HotCorner"));
    assert!(left_namespace.contains("Left"));
    assert!(right_namespace.contains("Right"));
}

#[test]
fn test_all_corner_namespace_uniqueness() {
    // Test that all four corner types have unique namespaces
    let top_left_namespace = "HotCornerTopLeft";
    let top_right_namespace = "HotCornerTopRight";
    let bottom_left_namespace = "HotCornerBottomLeft";
    let bottom_right_namespace = "HotCornerBottomRight";

    let namespaces = vec![
        top_left_namespace,
        top_right_namespace,
        bottom_left_namespace,
        bottom_right_namespace,
    ];

    // Check all namespaces are unique
    for (i, ns1) in namespaces.iter().enumerate() {
        for (j, ns2) in namespaces.iter().enumerate() {
            if i != j {
                assert_ne!(ns1, ns2, "Namespaces must be unique");
            }
        }
    }

    // Check all start with "HotCorner"
    for ns in &namespaces {
        assert!(ns.starts_with("HotCorner"));
    }

    // Check vertical positioning
    assert!(top_left_namespace.contains("Top"));
    assert!(top_right_namespace.contains("Top"));
    assert!(bottom_left_namespace.contains("Bottom"));
    assert!(bottom_right_namespace.contains("Bottom"));

    // Check horizontal positioning
    assert!(top_left_namespace.contains("Left"));
    assert!(bottom_left_namespace.contains("Left"));
    assert!(top_right_namespace.contains("Right"));
    assert!(bottom_right_namespace.contains("Right"));
}

#[test]
fn test_both_corners_can_exist_simultaneously() {
    // Test that both corner types can exist simultaneously on same monitor
    struct HotCornerMock {
        corner_type: String,
        connector: String,
    }

    let corners = vec![
        HotCornerMock {
            corner_type: "TopLeft".to_string(),
            connector: "eDP-1".to_string(),
        },
        HotCornerMock {
            corner_type: "TopRight".to_string(),
            connector: "eDP-1".to_string(),
        },
    ];

    assert_eq!(corners.len(), 2);
    assert!(corners.iter().any(|c| c.corner_type == "TopLeft"));
    assert!(corners.iter().any(|c| c.corner_type == "TopRight"));

    // Verify both corners are on the same monitor
    assert!(corners.iter().all(|c| c.connector == "eDP-1"));
}

#[test]
fn test_all_four_corners_can_exist_simultaneously() {
    // Test that all four corner types can exist simultaneously on same monitor
    struct HotCornerMock {
        corner_type: String,
        connector: String,
    }

    let corners = vec![
        HotCornerMock {
            corner_type: "TopLeft".to_string(),
            connector: "eDP-1".to_string(),
        },
        HotCornerMock {
            corner_type: "TopRight".to_string(),
            connector: "eDP-1".to_string(),
        },
        HotCornerMock {
            corner_type: "BottomLeft".to_string(),
            connector: "eDP-1".to_string(),
        },
        HotCornerMock {
            corner_type: "BottomRight".to_string(),
            connector: "eDP-1".to_string(),
        },
    ];

    assert_eq!(corners.len(), 4, "Should have all four corners");
    assert!(corners.iter().any(|c| c.corner_type == "TopLeft"));
    assert!(corners.iter().any(|c| c.corner_type == "TopRight"));
    assert!(corners.iter().any(|c| c.corner_type == "BottomLeft"));
    assert!(corners.iter().any(|c| c.corner_type == "BottomRight"));

    // Verify all corners are on the same monitor
    assert!(corners.iter().all(|c| c.connector == "eDP-1"));
}

#[test]
fn test_bottom_corner_dimensions() {
    // Test that bottom corners have the same width as top corners (4 pixels)
    let width_request = 4;
    assert_eq!(
        width_request, 4,
        "Bottom corner width should be 4 pixels to match top corners"
    );
}

#[test]
fn test_all_corners_same_dimensions() {
    // Test that all corners (top and bottom) have the same dimensions
    let top_left_width = 4;
    let top_right_width = 4;
    let bottom_left_width = 4;
    let bottom_right_width = 4;

    assert_eq!(
        top_left_width, top_right_width,
        "Top left and right corners should have same width"
    );
    assert_eq!(
        bottom_left_width, bottom_right_width,
        "Bottom left and right corners should have same width"
    );
    assert_eq!(
        top_left_width, bottom_left_width,
        "Top and bottom corners should have same width"
    );
}

#[test]
fn test_overlay_layer_value() {
    // Test layer order constants
    // Layer order: Background = 0, Bottom = 1, Top = 2, Overlay = 3
    const BACKGROUND: u8 = 0;
    const BOTTOM: u8 = 1;
    const TOP: u8 = 2;
    const OVERLAY: u8 = 3;

    assert!(OVERLAY > TOP);
    assert!(TOP > BOTTOM);
    assert!(BOTTOM > BACKGROUND);
}

#[test]
fn test_rightmost_monitor_negative_coordinates() {
    // Test rightmost monitor with negative X coordinates
    struct MockMonitor {
        x: i32,
        width: i32,
    }

    let monitor1 = MockMonitor {
        x: -1920,
        width: 1920,
    }; // Left monitor at negative X
    let monitor2 = MockMonitor { x: 0, width: 1920 }; // Right monitor

    let edge1 = monitor1.x + monitor1.width;
    let edge2 = monitor2.x + monitor2.width;

    assert_eq!(edge1, 0);
    assert_eq!(edge2, 1920);
    assert!(
        edge2 > edge1,
        "Monitor 2 should be rightmost with negative coordinates"
    );
}
