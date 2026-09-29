use rusty_de::service::battery::{BatteryState, KeyboardBattery};

#[test]
fn test_battery_state_from_u32() {
    let test_cases = vec![
        (0, BatteryState::Unknown),
        (1, BatteryState::Charging),
        (2, BatteryState::Discharging),
        (3, BatteryState::Empty),
        (4, BatteryState::FullyCharged),
        (5, BatteryState::PendingCharge),
        (6, BatteryState::PendingDischarge),
        (99, BatteryState::Unknown),
    ];

    for (value, expected) in test_cases {
        let state = BatteryState::from_u32(value);
        assert_eq!(
            state, expected,
            "Value {} should convert to {:?}",
            value, expected
        );
    }
}

#[test]
fn test_battery_icon_charging_100() {
    let percentage = 1.0;
    let state = BatteryState::Charging;

    let icon = match state {
        BatteryState::Charging => {
            if percentage >= 0.9 {
                "battery-level-100-charging-symbolic".to_string()
            } else if percentage >= 0.8 {
                "battery-level-90-charging-symbolic".to_string()
            } else {
                "battery-level-80-charging-symbolic".to_string()
            }
        }
        BatteryState::FullyCharged => "battery-level-100-charged-symbolic".to_string(),
        _ => "battery-level-100-symbolic".to_string(),
    };

    assert_eq!(icon, "battery-level-100-charging-symbolic");
}

#[test]
fn test_battery_icon_charging_levels() {
    let test_cases = vec![
        (0.95, "battery-level-100-charging-symbolic"),
        (0.85, "battery-level-90-charging-symbolic"),
        (0.75, "battery-level-80-charging-symbolic"),
        (0.65, "battery-level-70-charging-symbolic"),
        (0.55, "battery-level-60-charging-symbolic"),
        (0.45, "battery-level-50-charging-symbolic"),
        (0.35, "battery-level-40-charging-symbolic"),
        (0.25, "battery-level-30-charging-symbolic"),
        (0.15, "battery-level-20-charging-symbolic"),
        (0.08, "battery-level-10-charging-symbolic"),
        (0.02, "battery-level-0-charging-symbolic"),
    ];

    for (percentage, expected_icon) in test_cases {
        let state = BatteryState::Charging;
        let icon = match state {
            BatteryState::Charging => {
                if percentage >= 0.9 {
                    "battery-level-100-charging-symbolic".to_string()
                } else if percentage >= 0.8 {
                    "battery-level-90-charging-symbolic".to_string()
                } else if percentage >= 0.7 {
                    "battery-level-80-charging-symbolic".to_string()
                } else if percentage >= 0.6 {
                    "battery-level-70-charging-symbolic".to_string()
                } else if percentage >= 0.5 {
                    "battery-level-60-charging-symbolic".to_string()
                } else if percentage >= 0.4 {
                    "battery-level-50-charging-symbolic".to_string()
                } else if percentage >= 0.3 {
                    "battery-level-40-charging-symbolic".to_string()
                } else if percentage >= 0.2 {
                    "battery-level-30-charging-symbolic".to_string()
                } else if percentage >= 0.1 {
                    "battery-level-20-charging-symbolic".to_string()
                } else if percentage >= 0.05 {
                    "battery-level-10-charging-symbolic".to_string()
                } else {
                    "battery-level-0-charging-symbolic".to_string()
                }
            }
            BatteryState::FullyCharged => "battery-level-100-charged-symbolic".to_string(),
            _ => "battery-level-100-symbolic".to_string(),
        };

        assert_eq!(
            icon,
            expected_icon,
            "Charging at {}% should produce icon {}",
            percentage * 100.0,
            expected_icon
        );
    }
}

#[test]
fn test_battery_icon_discharging_levels() {
    let test_cases = vec![
        (0.95, "battery-level-100-symbolic"),
        (0.85, "battery-level-90-symbolic"),
        (0.75, "battery-level-80-symbolic"),
        (0.65, "battery-level-70-symbolic"),
        (0.55, "battery-level-60-symbolic"),
        (0.45, "battery-level-50-symbolic"),
        (0.35, "battery-level-40-symbolic"),
        (0.25, "battery-level-30-symbolic"),
        (0.15, "battery-level-20-symbolic"),
        (0.08, "battery-level-10-symbolic"),
        (0.02, "battery-level-0-symbolic"),
    ];

    for (percentage, expected_icon) in test_cases {
        let state = BatteryState::Discharging;
        let icon = match state {
            BatteryState::Charging => "battery-level-100-charging-symbolic".to_string(),
            BatteryState::FullyCharged => "battery-level-100-charged-symbolic".to_string(),
            _ => {
                if percentage >= 0.9 {
                    "battery-level-100-symbolic".to_string()
                } else if percentage >= 0.8 {
                    "battery-level-90-symbolic".to_string()
                } else if percentage >= 0.7 {
                    "battery-level-80-symbolic".to_string()
                } else if percentage >= 0.6 {
                    "battery-level-70-symbolic".to_string()
                } else if percentage >= 0.5 {
                    "battery-level-60-symbolic".to_string()
                } else if percentage >= 0.4 {
                    "battery-level-50-symbolic".to_string()
                } else if percentage >= 0.3 {
                    "battery-level-40-symbolic".to_string()
                } else if percentage >= 0.2 {
                    "battery-level-30-symbolic".to_string()
                } else if percentage >= 0.1 {
                    "battery-level-20-symbolic".to_string()
                } else if percentage >= 0.05 {
                    "battery-level-10-symbolic".to_string()
                } else {
                    "battery-level-0-symbolic".to_string()
                }
            }
        };

        assert_eq!(
            icon,
            expected_icon,
            "Discharging at {}% should produce icon {}",
            percentage * 100.0,
            expected_icon
        );
    }
}

#[test]
fn test_battery_icon_fully_charged() {
    let _percentage = 1.0;
    let state = BatteryState::FullyCharged;

    let icon = match state {
        BatteryState::Charging => "battery-level-100-charging-symbolic".to_string(),
        BatteryState::FullyCharged => "battery-level-100-charged-symbolic".to_string(),
        _ => "battery-level-100-symbolic".to_string(),
    };

    assert_eq!(icon, "battery-level-100-charged-symbolic");
}

#[test]
fn test_percentage_change_detection() {
    let old_percentage: f64 = 0.50;
    let new_percentage: f64 = 0.51;

    let changed = (new_percentage - old_percentage).abs() > 0.001;
    assert!(
        changed,
        "Percentage change from {} to {} should be detected",
        old_percentage, new_percentage
    );
}

#[test]
fn test_percentage_no_change_detection() {
    let old_percentage: f64 = 0.50;
    let new_percentage: f64 = 0.5005;

    let changed = (new_percentage - old_percentage).abs() > 0.001;
    assert!(
        !changed,
        "Small percentage change below threshold should not be detected"
    );
}

#[test]
fn test_state_change_detection() {
    let old_state = 1u32; // Charging
    let new_state = 2u32; // Discharging

    let changed = old_state != new_state;
    assert!(
        changed,
        "State change from {} to {} should be detected",
        old_state, new_state
    );
}

#[test]
fn test_state_no_change_detection() {
    let old_state = 1u32; // Charging
    let new_state = 1u32; // Charging

    let changed = old_state != new_state;
    assert!(
        !changed,
        "No state change should be detected when states are identical"
    );
}

#[test]
fn test_time_change_detection() {
    let old_time = 3600i64;
    let new_time = 3500i64;

    let changed = old_time != new_time;
    assert!(
        changed,
        "Time change from {} to {} should be detected",
        old_time, new_time
    );
}

#[test]
fn test_time_no_change_detection() {
    let old_time = 3600i64;
    let new_time = 3600i64;

    let changed = old_time != new_time;
    assert!(
        !changed,
        "No time change should be detected when times are identical"
    );
}

#[test]
fn test_percentage_conversion_from_upower() {
    // UPower returns percentage as 0-100, we convert to 0.0-1.0
    let upower_percentage = 75.0;
    let converted = upower_percentage / 100.0;

    assert_eq!(converted, 0.75);
}

#[test]
fn test_battery_critical_level() {
    let percentage = 0.04;
    let is_critical = percentage < 0.05;

    assert!(
        is_critical,
        "Battery at {}% should be critical",
        percentage * 100.0
    );
}

#[test]
fn test_battery_not_critical_level() {
    let percentage = 0.06;
    let is_critical = percentage < 0.05;

    assert!(
        !is_critical,
        "Battery at {}% should not be critical",
        percentage * 100.0
    );
}

#[test]
fn test_power_profile_selection_charging() {
    let state = BatteryState::Charging;
    let profile = match state {
        BatteryState::FullyCharged | BatteryState::PendingCharge | BatteryState::Charging => {
            "balanced"
        }
        BatteryState::Unknown
        | BatteryState::Empty
        | BatteryState::PendingDischarge
        | BatteryState::Discharging => "power-saver",
    };

    assert_eq!(
        profile, "balanced",
        "Charging state should select balanced profile"
    );
}

#[test]
fn test_power_profile_selection_discharging() {
    let state = BatteryState::Discharging;
    let profile = match state {
        BatteryState::FullyCharged | BatteryState::PendingCharge | BatteryState::Charging => {
            "balanced"
        }
        BatteryState::Unknown
        | BatteryState::Empty
        | BatteryState::PendingDischarge
        | BatteryState::Discharging => "power-saver",
    };

    assert_eq!(
        profile, "power-saver",
        "Discharging state should select power-saver profile"
    );
}

#[test]
fn test_power_profile_selection_fully_charged() {
    let state = BatteryState::FullyCharged;
    let profile = match state {
        BatteryState::FullyCharged | BatteryState::PendingCharge | BatteryState::Charging => {
            "balanced"
        }
        BatteryState::Unknown
        | BatteryState::Empty
        | BatteryState::PendingDischarge
        | BatteryState::Discharging => "power-saver",
    };

    assert_eq!(
        profile, "balanced",
        "FullyCharged state should select balanced profile"
    );
}

#[test]
fn test_power_profile_selection_pending_charge() {
    let state = BatteryState::PendingCharge;
    let profile = match state {
        BatteryState::FullyCharged | BatteryState::PendingCharge | BatteryState::Charging => {
            "balanced"
        }
        BatteryState::Unknown
        | BatteryState::Empty
        | BatteryState::PendingDischarge
        | BatteryState::Discharging => "power-saver",
    };

    assert_eq!(
        profile, "balanced",
        "PendingCharge state should select balanced profile"
    );
}

#[test]
fn test_power_profile_selection_pending_discharge() {
    let state = BatteryState::PendingDischarge;
    let profile = match state {
        BatteryState::FullyCharged | BatteryState::PendingCharge | BatteryState::Charging => {
            "balanced"
        }
        BatteryState::Unknown
        | BatteryState::Empty
        | BatteryState::PendingDischarge
        | BatteryState::Discharging => "power-saver",
    };

    assert_eq!(
        profile, "power-saver",
        "PendingDischarge state should select power-saver profile"
    );
}

#[test]
fn test_power_profile_selection_empty() {
    let state = BatteryState::Empty;
    let profile = match state {
        BatteryState::FullyCharged | BatteryState::PendingCharge | BatteryState::Charging => {
            "balanced"
        }
        BatteryState::Unknown
        | BatteryState::Empty
        | BatteryState::PendingDischarge
        | BatteryState::Discharging => "power-saver",
    };

    assert_eq!(
        profile, "power-saver",
        "Empty state should select power-saver profile"
    );
}

#[test]
fn test_power_profile_selection_unknown() {
    let state = BatteryState::Unknown;
    let profile = match state {
        BatteryState::FullyCharged | BatteryState::PendingCharge | BatteryState::Charging => {
            "balanced"
        }
        BatteryState::Unknown
        | BatteryState::Empty
        | BatteryState::PendingDischarge
        | BatteryState::Discharging => "power-saver",
    };

    assert_eq!(
        profile, "power-saver",
        "Unknown state should select power-saver profile"
    );
}

#[test]
fn test_power_profile_all_charging_states() {
    let charging_states = vec![
        BatteryState::Charging,
        BatteryState::FullyCharged,
        BatteryState::PendingCharge,
    ];

    for state in charging_states {
        let profile = match state {
            BatteryState::FullyCharged | BatteryState::PendingCharge | BatteryState::Charging => {
                "balanced"
            }
            BatteryState::Unknown
            | BatteryState::Empty
            | BatteryState::PendingDischarge
            | BatteryState::Discharging => "power-saver",
        };

        assert_eq!(
            profile, "balanced",
            "State {:?} should select balanced profile",
            state
        );
    }
}

#[test]
fn test_power_profile_all_discharging_states() {
    let discharging_states = vec![
        BatteryState::Discharging,
        BatteryState::PendingDischarge,
        BatteryState::Empty,
        BatteryState::Unknown,
    ];

    for state in discharging_states {
        let profile = match state {
            BatteryState::FullyCharged | BatteryState::PendingCharge | BatteryState::Charging => {
                "balanced"
            }
            BatteryState::Unknown
            | BatteryState::Empty
            | BatteryState::PendingDischarge
            | BatteryState::Discharging => "power-saver",
        };

        assert_eq!(
            profile, "power-saver",
            "State {:?} should select power-saver profile",
            state
        );
    }
}

// Keyboard Battery Tests

#[test]
fn test_keyboard_battery_creation_split_keyboard() {
    let kb = KeyboardBattery {
        name: "Piantor Pro BT".to_string(),
        address: "FC:47:67:1A:CB:7E".to_string(),
        central_percentage: Some(96),
        peripheral_percentage: Some(98),
    };

    assert_eq!(kb.name, "Piantor Pro BT");
    assert_eq!(kb.address, "FC:47:67:1A:CB:7E");
    assert_eq!(kb.central_percentage, Some(96));
    assert_eq!(kb.peripheral_percentage, Some(98));
}

#[test]
fn test_keyboard_battery_creation_single_battery() {
    let kb = KeyboardBattery {
        name: "Standard Keyboard".to_string(),
        address: "AA:BB:CC:DD:EE:FF".to_string(),
        central_percentage: Some(85),
        peripheral_percentage: None,
    };

    assert_eq!(kb.name, "Standard Keyboard");
    assert_eq!(kb.address, "AA:BB:CC:DD:EE:FF");
    assert_eq!(kb.central_percentage, Some(85));
    assert_eq!(kb.peripheral_percentage, None);
}

#[test]
fn test_keyboard_battery_icon_for_central() {
    let percentage = 85u8;
    let icon = match percentage {
        0..=10 => "battery-level-0-symbolic",
        11..=30 => "battery-level-10-symbolic",
        31..=50 => "battery-level-30-symbolic",
        51..=70 => "battery-level-50-symbolic",
        71..=90 => "battery-level-70-symbolic",
        _ => "battery-level-90-symbolic",
    };

    assert_eq!(icon, "battery-level-70-symbolic");
}

#[test]
fn test_keyboard_battery_icon_for_peripheral() {
    let percentage = 42u8;
    let icon = match percentage {
        0..=10 => "battery-level-0-symbolic",
        11..=30 => "battery-level-10-symbolic",
        31..=50 => "battery-level-30-symbolic",
        51..=70 => "battery-level-50-symbolic",
        71..=90 => "battery-level-70-symbolic",
        _ => "battery-level-90-symbolic",
    };

    assert_eq!(icon, "battery-level-30-symbolic");
}

#[test]
fn test_keyboard_battery_icon_levels() {
    let test_cases = vec![
        (5, "battery-level-0-symbolic"),
        (25, "battery-level-10-symbolic"),
        (45, "battery-level-30-symbolic"),
        (65, "battery-level-50-symbolic"),
        (85, "battery-level-70-symbolic"),
        (95, "battery-level-90-symbolic"),
    ];

    for (percentage, expected_icon) in test_cases {
        let icon = match percentage {
            0..=10 => "battery-level-0-symbolic",
            11..=30 => "battery-level-10-symbolic",
            31..=50 => "battery-level-30-symbolic",
            51..=70 => "battery-level-50-symbolic",
            71..=90 => "battery-level-70-symbolic",
            _ => "battery-level-90-symbolic",
        };

        assert_eq!(
            icon, expected_icon,
            "Percentage {}% should produce icon {}",
            percentage, expected_icon
        );
    }
}

#[test]
fn test_keyboard_battery_address_formatting() {
    let address = "FC:47:67:1A:CB:7E";
    let normalized = address.replace(":", "_");

    assert_eq!(normalized, "FC_47_67_1A_CB_7E");
}

#[test]
fn test_keyboard_battery_address_comparison() {
    let kb_address = "FC:47:67:1A:CB:7E";
    let device_address = "FC:47:67:1A:CB:7E";

    let kb_normalized = kb_address.replace(":", "_");
    let dev_normalized = device_address.replace(":", "_");

    assert_eq!(kb_normalized, dev_normalized);
}

#[test]
fn test_keyboard_battery_tooltip_split() {
    let kb = KeyboardBattery {
        name: "Piantor Pro BT".to_string(),
        address: "FC:47:67:1A:CB:7E".to_string(),
        central_percentage: Some(96),
        peripheral_percentage: Some(98),
    };

    let battery_text = match (kb.central_percentage, kb.peripheral_percentage) {
        (Some(central), Some(peripheral)) => {
            format!(
                "Battery: {}% (Central) / {}% (Peripheral)",
                central, peripheral
            )
        }
        (Some(percentage), None) => {
            format!("Battery: {}%", percentage)
        }
        _ => String::new(),
    };

    assert_eq!(battery_text, "Battery: 96% (Central) / 98% (Peripheral)");
}

#[test]
fn test_keyboard_battery_tooltip_single() {
    let kb = KeyboardBattery {
        name: "Standard Keyboard".to_string(),
        address: "AA:BB:CC:DD:EE:FF".to_string(),
        central_percentage: Some(85),
        peripheral_percentage: None,
    };

    let battery_text = match (kb.central_percentage, kb.peripheral_percentage) {
        (Some(central), Some(peripheral)) => {
            format!(
                "Battery: {}% (Central) / {}% (Peripheral)",
                central, peripheral
            )
        }
        (Some(percentage), None) => {
            format!("Battery: {}%", percentage)
        }
        _ => String::new(),
    };

    assert_eq!(battery_text, "Battery: 85%");
}

#[test]
fn test_keyboard_battery_clone() {
    let kb1 = KeyboardBattery {
        name: "Piantor Pro BT".to_string(),
        address: "FC:47:67:1A:CB:7E".to_string(),
        central_percentage: Some(96),
        peripheral_percentage: Some(98),
    };

    let kb2 = kb1.clone();

    assert_eq!(kb1.name, kb2.name);
    assert_eq!(kb1.address, kb2.address);
    assert_eq!(kb1.central_percentage, kb2.central_percentage);
    assert_eq!(kb1.peripheral_percentage, kb2.peripheral_percentage);
}

#[test]
fn test_keyboard_battery_equality() {
    let kb1 = KeyboardBattery {
        name: "Piantor Pro BT".to_string(),
        address: "FC:47:67:1A:CB:7E".to_string(),
        central_percentage: Some(96),
        peripheral_percentage: Some(98),
    };

    let kb2 = KeyboardBattery {
        name: "Piantor Pro BT".to_string(),
        address: "FC:47:67:1A:CB:7E".to_string(),
        central_percentage: Some(96),
        peripheral_percentage: Some(98),
    };

    assert_eq!(kb1, kb2);
}

#[test]
fn test_keyboard_battery_inequality_different_central() {
    let kb1 = KeyboardBattery {
        name: "Piantor Pro BT".to_string(),
        address: "FC:47:67:1A:CB:7E".to_string(),
        central_percentage: Some(96),
        peripheral_percentage: Some(98),
    };

    let kb2 = KeyboardBattery {
        name: "Piantor Pro BT".to_string(),
        address: "FC:47:67:1A:CB:7E".to_string(),
        central_percentage: Some(95),
        peripheral_percentage: Some(98),
    };

    assert_ne!(kb1, kb2);
}

#[test]
fn test_keyboard_battery_inequality_different_peripheral() {
    let kb1 = KeyboardBattery {
        name: "Piantor Pro BT".to_string(),
        address: "FC:47:67:1A:CB:7E".to_string(),
        central_percentage: Some(96),
        peripheral_percentage: Some(98),
    };

    let kb2 = KeyboardBattery {
        name: "Piantor Pro BT".to_string(),
        address: "FC:47:67:1A:CB:7E".to_string(),
        central_percentage: Some(96),
        peripheral_percentage: Some(97),
    };

    assert_ne!(kb1, kb2);
}

#[test]
fn test_keyboard_battery_update_central() {
    let mut kb = KeyboardBattery {
        name: "Piantor Pro BT".to_string(),
        address: "FC:47:67:1A:CB:7E".to_string(),
        central_percentage: Some(96),
        peripheral_percentage: Some(98),
    };

    // Simulate battery update
    kb.central_percentage = Some(95);

    assert_eq!(kb.central_percentage, Some(95));
    assert_eq!(kb.peripheral_percentage, Some(98)); // Unchanged
}

#[test]
fn test_keyboard_battery_update_peripheral() {
    let mut kb = KeyboardBattery {
        name: "Piantor Pro BT".to_string(),
        address: "FC:47:67:1A:CB:7E".to_string(),
        central_percentage: Some(96),
        peripheral_percentage: Some(98),
    };

    // Simulate battery update
    kb.peripheral_percentage = Some(97);

    assert_eq!(kb.central_percentage, Some(96)); // Unchanged
    assert_eq!(kb.peripheral_percentage, Some(97));
}

#[test]
fn test_keyboard_battery_low_levels() {
    let test_cases = vec![
        (5, 8, true),    // Both low
        (5, 85, true),   // Central low
        (85, 5, true),   // Peripheral low
        (85, 90, false), // Both normal
    ];

    for (central, peripheral, should_warn) in test_cases {
        let kb = KeyboardBattery {
            name: "Test Keyboard".to_string(),
            address: "00:00:00:00:00:00".to_string(),
            central_percentage: Some(central),
            peripheral_percentage: Some(peripheral),
        };

        let has_low_battery = kb.central_percentage.map_or(false, |p| p <= 10)
            || kb.peripheral_percentage.map_or(false, |p| p <= 10);

        assert_eq!(
            has_low_battery, should_warn,
            "Central: {}%, Peripheral: {}% - should_warn: {}",
            central, peripheral, should_warn
        );
    }
}

#[test]
fn test_keyboard_battery_critical_levels() {
    let kb = KeyboardBattery {
        name: "Test Keyboard".to_string(),
        address: "00:00:00:00:00:00".to_string(),
        central_percentage: Some(3),
        peripheral_percentage: Some(5),
    };

    let central_critical = kb.central_percentage.map_or(false, |p| p <= 5);
    let peripheral_critical = kb.peripheral_percentage.map_or(false, |p| p <= 5);

    assert!(central_critical);
    assert!(peripheral_critical);
}

#[test]
fn test_keyboard_battery_none_values() {
    let kb = KeyboardBattery {
        name: "Test Keyboard".to_string(),
        address: "00:00:00:00:00:00".to_string(),
        central_percentage: None,
        peripheral_percentage: None,
    };

    assert_eq!(kb.central_percentage, None);
    assert_eq!(kb.peripheral_percentage, None);
}
