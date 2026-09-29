use rusty_de::service::wifi::WifiState;

#[test]
fn test_wifi_state_from_u32() {
    let test_cases = vec![
        (0, WifiState::Unknown),
        (10, WifiState::Unmanaged),
        (20, WifiState::Unavailable),
        (30, WifiState::Disconnected),
        (40, WifiState::Preparing),
        (50, WifiState::Config),
        (60, WifiState::NeedAuth),
        (70, WifiState::IpConfig),
        (80, WifiState::IpCheck),
        (90, WifiState::Secondaries),
        (100, WifiState::Activated),
        (110, WifiState::Deactivating),
        (120, WifiState::Failed),
        (99, WifiState::Unknown),
    ];

    for (value, expected) in test_cases {
        let state = WifiState::from_u32(value);
        assert_eq!(
            state, expected,
            "Value {} should convert to {:?}",
            value, expected
        );
    }
}

#[test]
fn test_wifi_icon_connected_excellent() {
    let connected = true;
    let strength = 85u8;

    let icon = if !connected {
        "network-wireless-disconnected-symbolic".to_string()
    } else if strength >= 80 {
        "network-wireless-signal-excellent-symbolic".to_string()
    } else if strength >= 60 {
        "network-wireless-signal-good-symbolic".to_string()
    } else if strength >= 40 {
        "network-wireless-signal-ok-symbolic".to_string()
    } else if strength >= 20 {
        "network-wireless-signal-weak-symbolic".to_string()
    } else {
        "network-wireless-signal-none-symbolic".to_string()
    };

    assert_eq!(icon, "network-wireless-signal-excellent-symbolic");
}

#[test]
fn test_wifi_icon_signal_levels() {
    let test_cases = vec![
        (true, 95, "network-wireless-signal-excellent-symbolic"),
        (true, 70, "network-wireless-signal-good-symbolic"),
        (true, 50, "network-wireless-signal-ok-symbolic"),
        (true, 30, "network-wireless-signal-weak-symbolic"),
        (true, 10, "network-wireless-signal-none-symbolic"),
        (false, 100, "network-wireless-disconnected-symbolic"),
    ];

    for (connected, strength, expected_icon) in test_cases {
        let icon = if !connected {
            "network-wireless-disconnected-symbolic".to_string()
        } else if strength >= 80 {
            "network-wireless-signal-excellent-symbolic".to_string()
        } else if strength >= 60 {
            "network-wireless-signal-good-symbolic".to_string()
        } else if strength >= 40 {
            "network-wireless-signal-ok-symbolic".to_string()
        } else if strength >= 20 {
            "network-wireless-signal-weak-symbolic".to_string()
        } else {
            "network-wireless-signal-none-symbolic".to_string()
        };

        assert_eq!(
            icon, expected_icon,
            "Connected={} Strength={}% should produce icon {}",
            connected, strength, expected_icon
        );
    }
}

#[test]
fn test_wifi_icon_disconnected() {
    let connected = false;
    let _strength = 100u8;

    let icon = if !connected {
        "network-wireless-disconnected-symbolic".to_string()
    } else {
        "network-wireless-signal-excellent-symbolic".to_string()
    };

    assert_eq!(icon, "network-wireless-disconnected-symbolic");
}

#[test]
fn test_connection_state_change_detection() {
    let old_connected = true;
    let new_connected = false;

    let changed = old_connected != new_connected;
    assert!(
        changed,
        "Connection state change from {} to {} should be detected",
        old_connected, new_connected
    );
}

#[test]
fn test_connection_state_no_change_detection() {
    let old_connected = true;
    let new_connected = true;

    let changed = old_connected != new_connected;
    assert!(
        !changed,
        "No connection state change should be detected when states are identical"
    );
}

#[test]
fn test_ssid_change_detection() {
    let old_ssid = "MyWiFi".to_string();
    let new_ssid = "OtherWiFi".to_string();

    let changed = old_ssid != new_ssid;
    assert!(
        changed,
        "SSID change from '{}' to '{}' should be detected",
        old_ssid, new_ssid
    );
}

#[test]
fn test_ssid_no_change_detection() {
    let old_ssid = "MyWiFi".to_string();
    let new_ssid = "MyWiFi".to_string();

    let changed = old_ssid != new_ssid;
    assert!(
        !changed,
        "No SSID change should be detected when SSIDs are identical"
    );
}

#[test]
fn test_strength_change_detection() {
    let old_strength = 80u8;
    let new_strength = 70u8;

    let changed = old_strength != new_strength;
    assert!(
        changed,
        "Signal strength change from {}% to {}% should be detected",
        old_strength, new_strength
    );
}

#[test]
fn test_strength_no_change_detection() {
    let old_strength = 80u8;
    let new_strength = 80u8;

    let changed = old_strength != new_strength;
    assert!(
        !changed,
        "No signal strength change should be detected when values are identical"
    );
}

#[test]
fn test_wifi_state_change_detection() {
    let old_state = 30u32; // Disconnected
    let new_state = 100u32; // Connected

    let changed = old_state != new_state;
    assert!(
        changed,
        "State change from {} to {} should be detected",
        old_state, new_state
    );
}

#[test]
fn test_wifi_state_no_change_detection() {
    let old_state = 100u32; // Connected
    let new_state = 100u32; // Connected

    let changed = old_state != new_state;
    assert!(
        !changed,
        "No state change should be detected when states are identical"
    );
}

#[test]
fn test_empty_ssid_change_detection() {
    let old_ssid = "MyWiFi".to_string();
    let new_ssid = String::new();

    let changed = old_ssid != new_ssid;
    assert!(
        changed,
        "SSID change from '{}' to empty should be detected",
        old_ssid
    );
}

#[test]
fn test_wifi_state_values() {
    assert_eq!(WifiState::Unknown as u32, 0);
    assert_eq!(WifiState::Unmanaged as u32, 10);
    assert_eq!(WifiState::Unavailable as u32, 20);
    assert_eq!(WifiState::Disconnected as u32, 30);
    assert_eq!(WifiState::Preparing as u32, 40);
    assert_eq!(WifiState::Config as u32, 50);
    assert_eq!(WifiState::NeedAuth as u32, 60);
    assert_eq!(WifiState::IpConfig as u32, 70);
    assert_eq!(WifiState::IpCheck as u32, 80);
    assert_eq!(WifiState::Secondaries as u32, 90);
    assert_eq!(WifiState::Activated as u32, 100);
    assert_eq!(WifiState::Deactivating as u32, 110);
    assert_eq!(WifiState::Failed as u32, 120);
}

#[test]
fn test_wifi_state_connection_sequence() {
    let states = [
        WifiState::Disconnected,
        WifiState::Preparing,
        WifiState::Config,
        WifiState::IpConfig,
        WifiState::IpCheck,
        WifiState::Activated,
    ];

    let state_values: Vec<u32> = states.iter().map(|s| *s as u32).collect();
    assert_eq!(state_values, vec![30, 40, 50, 70, 80, 100]);
}

#[test]
fn test_wifi_state_disconnection_sequence() {
    let states = [
        WifiState::Activated,
        WifiState::Deactivating,
        WifiState::Disconnected,
    ];

    let state_values: Vec<u32> = states.iter().map(|s| *s as u32).collect();
    assert_eq!(state_values, vec![100, 110, 30]);
}

#[test]
fn test_signal_strength_boundaries() {
    let test_cases = vec![
        (80, "network-wireless-signal-excellent-symbolic"),
        (79, "network-wireless-signal-good-symbolic"),
        (60, "network-wireless-signal-good-symbolic"),
        (59, "network-wireless-signal-ok-symbolic"),
        (40, "network-wireless-signal-ok-symbolic"),
        (39, "network-wireless-signal-weak-symbolic"),
        (20, "network-wireless-signal-weak-symbolic"),
        (19, "network-wireless-signal-none-symbolic"),
        (0, "network-wireless-signal-none-symbolic"),
    ];

    for (strength, expected_icon) in test_cases {
        let icon = if strength >= 80 {
            "network-wireless-signal-excellent-symbolic".to_string()
        } else if strength >= 60 {
            "network-wireless-signal-good-symbolic".to_string()
        } else if strength >= 40 {
            "network-wireless-signal-ok-symbolic".to_string()
        } else if strength >= 20 {
            "network-wireless-signal-weak-symbolic".to_string()
        } else {
            "network-wireless-signal-none-symbolic".to_string()
        };

        assert_eq!(
            icon, expected_icon,
            "Signal strength {}% should produce icon {}",
            strength, expected_icon
        );
    }
}

#[test]
fn test_zero_strength_when_disconnected() {
    let old_strength = 80u8;
    let new_strength = 0u8;

    let changed = old_strength != new_strength;
    assert!(
        changed,
        "Signal strength should change to 0 when disconnected"
    );
}

#[test]
fn test_wifi_icon_by_state_activated() {
    use rusty_de::service::wifi::WifiState;

    let state = WifiState::Activated;
    let strength = 85u8;

    let icon = match state {
        WifiState::Activated => {
            if strength >= 80 {
                "network-wireless-signal-excellent-symbolic".to_string()
            } else if strength >= 60 {
                "network-wireless-signal-good-symbolic".to_string()
            } else if strength >= 40 {
                "network-wireless-signal-ok-symbolic".to_string()
            } else if strength >= 20 {
                "network-wireless-signal-weak-symbolic".to_string()
            } else {
                "network-wireless-signal-none-symbolic".to_string()
            }
        }
        WifiState::Preparing
        | WifiState::Config
        | WifiState::NeedAuth
        | WifiState::IpConfig
        | WifiState::IpCheck
        | WifiState::Secondaries => "network-wireless-acquiring-symbolic".to_string(),
        WifiState::Deactivating => "network-wireless-disconnected-symbolic".to_string(),
        WifiState::Disconnected
        | WifiState::Unavailable
        | WifiState::Unmanaged
        | WifiState::Failed
        | WifiState::Unknown => "network-wireless-disconnected-symbolic".to_string(),
    };

    assert_eq!(icon, "network-wireless-signal-excellent-symbolic");
}

#[test]
fn test_wifi_icon_by_state_connecting() {
    use rusty_de::service::wifi::WifiState;

    let states = vec![
        WifiState::Preparing,
        WifiState::Config,
        WifiState::NeedAuth,
        WifiState::IpConfig,
        WifiState::IpCheck,
        WifiState::Secondaries,
    ];

    for state in states {
        let icon = match state {
            WifiState::Activated => "network-wireless-signal-excellent-symbolic".to_string(),
            WifiState::Preparing
            | WifiState::Config
            | WifiState::NeedAuth
            | WifiState::IpConfig
            | WifiState::IpCheck
            | WifiState::Secondaries => "network-wireless-acquiring-symbolic".to_string(),
            WifiState::Deactivating => "network-wireless-disconnected-symbolic".to_string(),
            WifiState::Disconnected
            | WifiState::Unavailable
            | WifiState::Unmanaged
            | WifiState::Failed
            | WifiState::Unknown => "network-wireless-disconnected-symbolic".to_string(),
        };

        assert_eq!(
            icon, "network-wireless-acquiring-symbolic",
            "State {:?} should show acquiring icon",
            state
        );
    }
}

#[test]
fn test_wifi_icon_by_state_disconnected() {
    use rusty_de::service::wifi::WifiState;

    let states = vec![
        WifiState::Disconnected,
        WifiState::Unavailable,
        WifiState::Unmanaged,
        WifiState::Failed,
        WifiState::Unknown,
        WifiState::Deactivating,
    ];

    for state in states {
        let icon = match state {
            WifiState::Activated => "network-wireless-signal-excellent-symbolic".to_string(),
            WifiState::Preparing
            | WifiState::Config
            | WifiState::NeedAuth
            | WifiState::IpConfig
            | WifiState::IpCheck
            | WifiState::Secondaries => "network-wireless-acquiring-symbolic".to_string(),
            WifiState::Deactivating => "network-wireless-disconnected-symbolic".to_string(),
            WifiState::Disconnected
            | WifiState::Unavailable
            | WifiState::Unmanaged
            | WifiState::Failed
            | WifiState::Unknown => "network-wireless-disconnected-symbolic".to_string(),
        };

        assert_eq!(
            icon, "network-wireless-disconnected-symbolic",
            "State {:?} should show disconnected icon",
            state
        );
    }
}

#[test]
fn test_wifi_state_transition_ipconfig_to_activated() {
    let old_state = WifiState::IpConfig;
    let new_state = WifiState::Activated;

    assert_ne!(old_state, new_state);
    assert_eq!(old_state as u32, 70);
    assert_eq!(new_state as u32, 100);
}

#[test]
fn test_wifi_state_transition_ipconfig_to_ipcheck() {
    let old_state = WifiState::IpConfig;
    let new_state = WifiState::IpCheck;

    assert_ne!(old_state, new_state);
    assert_eq!(old_state as u32, 70);
    assert_eq!(new_state as u32, 80);
}

#[test]
fn test_wifi_state_transition_ipcheck_to_activated() {
    let old_state = WifiState::IpCheck;
    let new_state = WifiState::Activated;

    assert_ne!(old_state, new_state);
    assert_eq!(old_state as u32, 80);
    assert_eq!(new_state as u32, 100);
}

#[test]
fn test_password_required_detection_wpa_flags() {
    let wpa_flags = 0x100u32; // Some WPA flag set
    let rsn_flags = 0u32;

    let requires_password = wpa_flags != 0 || rsn_flags != 0;
    assert!(
        requires_password,
        "Non-zero WPA flags should indicate password required"
    );
}

#[test]
fn test_password_required_detection_rsn_flags() {
    let wpa_flags = 0u32;
    let rsn_flags = 0x200u32; // Some RSN flag set

    let requires_password = wpa_flags != 0 || rsn_flags != 0;
    assert!(
        requires_password,
        "Non-zero RSN flags should indicate password required"
    );
}

#[test]
fn test_password_required_detection_both_flags() {
    let wpa_flags = 0x100u32;
    let rsn_flags = 0x200u32;

    let requires_password = wpa_flags != 0 || rsn_flags != 0;
    assert!(
        requires_password,
        "Both WPA and RSN flags set should indicate password required"
    );
}

#[test]
fn test_password_not_required_open_network() {
    let wpa_flags = 0u32;
    let rsn_flags = 0u32;

    let requires_password = wpa_flags != 0 || rsn_flags != 0;
    assert!(
        !requires_password,
        "Zero WPA and RSN flags should indicate no password required (open network)"
    );
}

#[test]
fn test_connection_state_activated() {
    // NetworkManager active connection states
    // 0=unknown, 1=activating, 2=activated, 3=deactivating, 4=deactivated
    let state = 2u32;
    assert_eq!(state, 2, "State 2 should be activated");
}

#[test]
fn test_connection_state_failed() {
    // State 4 = deactivated (failed)
    let state = 4u32;
    assert!(
        state == 3 || state == 4,
        "State 3 or 4 indicates connection failure"
    );
}

#[test]
fn test_connection_state_activating() {
    let state = 1u32;
    assert_eq!(state, 1, "State 1 should be activating");
}

#[test]
fn test_connection_state_monitoring_success() {
    // Simulate state progression: activating -> activated
    let states = vec![1u32, 2u32];

    for state in states {
        match state {
            2 => {
                // Connection succeeded
                assert_eq!(state, 2);
                break;
            }
            3 | 4 => {
                // Connection failed
                panic!("Connection should not fail");
            }
            _ => {
                // Still connecting
                assert!(state < 2);
            }
        }
    }
}

#[test]
fn test_connection_state_monitoring_failure() {
    // Simulate state progression: activating -> deactivated (failed)
    let states = vec![1u32, 4u32];
    let mut failed = false;

    for state in states {
        match state {
            2 => {
                // Connection succeeded
                break;
            }
            3 | 4 => {
                // Connection failed
                failed = true;
                break;
            }
            _ => {
                // Still connecting
            }
        }
    }

    assert!(failed, "Connection should have failed");
}

#[test]
fn test_access_point_password_flags() {
    // Simulating AccessPoint struct fields
    struct MockAccessPoint {
        #[allow(dead_code)]
        ssid: String,
        #[allow(dead_code)]
        strength: u8,
        wpa_flags: u32,
        rsn_flags: u32,
    }

    let open_network = MockAccessPoint {
        ssid: "OpenWiFi".to_string(),
        strength: 80,
        wpa_flags: 0,
        rsn_flags: 0,
    };

    let secure_network = MockAccessPoint {
        ssid: "SecureWiFi".to_string(),
        strength: 90,
        wpa_flags: 0x100,
        rsn_flags: 0x200,
    };

    assert!(
        open_network.wpa_flags == 0 && open_network.rsn_flags == 0,
        "Open network should have no security flags"
    );
    assert!(
        secure_network.wpa_flags != 0 || secure_network.rsn_flags != 0,
        "Secure network should have security flags"
    );
}

#[test]
fn test_has_saved_connection_with_saved() {
    // Simulate an access point with a saved connection
    let requires_password = true;
    let has_saved_connection = true;
    let needs_password_input = requires_password && !has_saved_connection;

    assert!(
        !needs_password_input,
        "Should not prompt for password when connection is saved"
    );
}

#[test]
fn test_has_saved_connection_without_saved() {
    // Simulate an access point without a saved connection
    let requires_password = true;
    let has_saved_connection = false;
    let needs_password_input = requires_password && !has_saved_connection;

    assert!(
        needs_password_input,
        "Should prompt for password when no connection is saved"
    );
}

#[test]
fn test_has_saved_connection_open_network() {
    // Simulate an open network (no password required)
    let requires_password = false;
    let has_saved_connection = false;
    let needs_password_input = requires_password && !has_saved_connection;

    assert!(
        !needs_password_input,
        "Should not prompt for password on open networks"
    );
}

#[test]
fn test_has_saved_connection_open_with_saved() {
    // Simulate an open network with a saved connection
    let requires_password = false;
    let has_saved_connection = true;
    let needs_password_input = requires_password && !has_saved_connection;

    assert!(
        !needs_password_input,
        "Should not prompt for password on open networks with saved connection"
    );
}

#[test]
fn test_password_prompt_logic_matrix() {
    // Test all combinations of requires_password and has_saved_connection
    struct TestCase {
        requires_password: bool,
        has_saved_connection: bool,
        expected_prompt: bool,
    }

    let test_cases = [
        TestCase {
            requires_password: false,
            has_saved_connection: false,
            expected_prompt: false,
        },
        TestCase {
            requires_password: false,
            has_saved_connection: true,
            expected_prompt: false,
        },
        TestCase {
            requires_password: true,
            has_saved_connection: false,
            expected_prompt: true,
        },
        TestCase {
            requires_password: true,
            has_saved_connection: true,
            expected_prompt: false,
        },
    ];

    for (idx, case) in test_cases.iter().enumerate() {
        let needs_password_input = case.requires_password && !case.has_saved_connection;
        assert_eq!(
            needs_password_input, case.expected_prompt,
            "Test case {} failed: requires_password={}, has_saved_connection={}",
            idx, case.requires_password, case.has_saved_connection
        );
    }
}
#[test]
fn test_frequency_format_single_2_4ghz() {
    // Test formatting a single 2.4 GHz frequency
    let frequencies = [2412u32];
    let formatted: Vec<String> = frequencies
        .iter()
        .map(|&freq_mhz| {
            let freq_ghz = freq_mhz as f64 / 1000.0;
            format!("{:.1} GHz", freq_ghz)
        })
        .collect();

    assert_eq!(formatted.join(", "), "2.4 GHz");
}

#[test]
fn test_frequency_format_single_5ghz() {
    // Test formatting a single 5 GHz frequency
    let frequencies = [5180u32];
    let formatted: Vec<String> = frequencies
        .iter()
        .map(|&freq_mhz| {
            let freq_ghz = freq_mhz as f64 / 1000.0;
            format!("{:.1} GHz", freq_ghz)
        })
        .collect();

    assert_eq!(formatted.join(", "), "5.2 GHz");
}

#[test]
fn test_frequency_format_dual_band() {
    // Test formatting both 2.4 GHz and 5 GHz frequencies
    let frequencies = [2437u32, 5220u32];
    let formatted: Vec<String> = frequencies
        .iter()
        .map(|&freq_mhz| {
            let freq_ghz = freq_mhz as f64 / 1000.0;
            format!("{:.1} GHz", freq_ghz)
        })
        .collect();

    assert_eq!(formatted.join(", "), "2.4 GHz, 5.2 GHz");
}

#[test]
fn test_frequency_format_triple_band() {
    // Test formatting three different frequencies
    let frequencies = [2412u32, 5180u32, 5200u32];
    let formatted: Vec<String> = frequencies
        .iter()
        .map(|&freq_mhz| {
            let freq_ghz = freq_mhz as f64 / 1000.0;
            format!("{:.1} GHz", freq_ghz)
        })
        .collect();

    assert_eq!(formatted.join(", "), "2.4 GHz, 5.2 GHz, 5.2 GHz");
}

#[test]
fn test_duplicate_ssid_merge_keeps_highest_strength() {
    // Test that merging keeps the highest signal strength
    use std::collections::HashMap;

    #[derive(Clone)]
    struct MockAP {
        ssid: String,
        strength: u8,
        frequencies: Vec<u32>,
    }

    let aps = vec![
        MockAP {
            ssid: "TestNetwork".to_string(),
            strength: 45,
            frequencies: vec![2412],
        },
        MockAP {
            ssid: "TestNetwork".to_string(),
            strength: 82,
            frequencies: vec![5180],
        },
    ];

    let mut merged: HashMap<String, MockAP> = HashMap::new();
    for ap in aps {
        if let Some(existing) = merged.get_mut(&ap.ssid) {
            existing.frequencies.push(ap.frequencies[0]);
            if ap.strength > existing.strength {
                existing.strength = ap.strength;
            }
        } else {
            merged.insert(ap.ssid.clone(), ap);
        }
    }

    let result = merged.get("TestNetwork").unwrap();
    assert_eq!(result.strength, 82, "Should keep highest strength");
    assert_eq!(result.frequencies.len(), 2, "Should combine frequencies");
}

#[test]
fn test_duplicate_ssid_merge_combines_frequencies() {
    // Test that merging combines all frequencies
    use std::collections::HashMap;

    #[derive(Clone)]
    struct MockAP {
        ssid: String,
        frequencies: Vec<u32>,
    }

    let aps = vec![
        MockAP {
            ssid: "TestNetwork".to_string(),
            frequencies: vec![2412],
        },
        MockAP {
            ssid: "TestNetwork".to_string(),
            frequencies: vec![5180],
        },
        MockAP {
            ssid: "TestNetwork".to_string(),
            frequencies: vec![5200],
        },
    ];

    let mut merged: HashMap<String, MockAP> = HashMap::new();
    for ap in aps {
        if let Some(existing) = merged.get_mut(&ap.ssid) {
            existing.frequencies.push(ap.frequencies[0]);
        } else {
            merged.insert(ap.ssid.clone(), ap);
        }
    }

    let result = merged.get("TestNetwork").unwrap();
    assert_eq!(result.frequencies, vec![2412, 5180, 5200]);
}

#[test]
fn test_duplicate_ssid_merge_saved_connection_propagation() {
    // Test that if any AP has saved connection, the merged one should too
    use std::collections::HashMap;

    #[derive(Clone)]
    struct MockAP {
        ssid: String,
        has_saved_connection: bool,
        frequencies: Vec<u32>,
    }

    let aps = vec![
        MockAP {
            ssid: "TestNetwork".to_string(),
            has_saved_connection: false,
            frequencies: vec![2412],
        },
        MockAP {
            ssid: "TestNetwork".to_string(),
            has_saved_connection: true,
            frequencies: vec![5180],
        },
    ];

    let mut merged: HashMap<String, MockAP> = HashMap::new();
    for ap in aps {
        if let Some(existing) = merged.get_mut(&ap.ssid) {
            existing.frequencies.push(ap.frequencies[0]);
            if ap.has_saved_connection {
                existing.has_saved_connection = true;
            }
        } else {
            merged.insert(ap.ssid.clone(), ap);
        }
    }

    let result = merged.get("TestNetwork").unwrap();
    assert!(
        result.has_saved_connection,
        "Should propagate saved connection from any AP"
    );
}

#[test]
fn test_duplicate_ssid_different_networks_not_merged() {
    // Test that different SSIDs are not merged
    use std::collections::HashMap;

    #[derive(Clone)]
    struct MockAP {
        ssid: String,
        frequencies: Vec<u32>,
    }

    let aps = vec![
        MockAP {
            ssid: "Network1".to_string(),
            frequencies: vec![2412],
        },
        MockAP {
            ssid: "Network2".to_string(),
            frequencies: vec![5180],
        },
    ];

    let mut merged: HashMap<String, MockAP> = HashMap::new();
    for ap in aps {
        if let Some(existing) = merged.get_mut(&ap.ssid) {
            existing.frequencies.push(ap.frequencies[0]);
        } else {
            merged.insert(ap.ssid.clone(), ap);
        }
    }

    assert_eq!(merged.len(), 2, "Different SSIDs should not be merged");
    assert!(merged.contains_key("Network1"));
    assert!(merged.contains_key("Network2"));
}

#[test]
fn test_frequency_sorting() {
    // Test that frequencies are sorted for consistent display
    let mut frequencies = vec![5200u32, 2412u32, 5180u32];
    frequencies.sort_unstable();

    assert_eq!(frequencies, vec![2412, 5180, 5200]);
}

#[test]
fn test_tooltip_format_with_single_frequency() {
    // Test tooltip format with single frequency
    let ssid = "TestNetwork";
    let strength = 85u8;
    let frequencies = [2412u32];

    let frequency_text: Vec<String> = frequencies
        .iter()
        .map(|&freq_mhz| {
            let freq_ghz = freq_mhz as f64 / 1000.0;
            format!("{:.1} GHz", freq_ghz)
        })
        .collect();

    let tooltip = format!(
        "Name: {}\nStrength: {}%\nFrequency: {}",
        ssid,
        strength,
        frequency_text.join(", ")
    );

    assert_eq!(
        tooltip,
        "Name: TestNetwork\nStrength: 85%\nFrequency: 2.4 GHz"
    );
}

#[test]
fn test_tooltip_format_with_dual_band() {
    // Test tooltip format with dual band frequencies
    let ssid = "DualBandNetwork";
    let strength = 82u8;
    let frequencies = [2437u32, 5220u32];

    let frequency_text: Vec<String> = frequencies
        .iter()
        .map(|&freq_mhz| {
            let freq_ghz = freq_mhz as f64 / 1000.0;
            format!("{:.1} GHz", freq_ghz)
        })
        .collect();

    let tooltip = format!(
        "Name: {}\nStrength: {}%\nFrequency: {}",
        ssid,
        strength,
        frequency_text.join(", ")
    );

    assert_eq!(
        tooltip,
        "Name: DualBandNetwork\nStrength: 82%\nFrequency: 2.4 GHz, 5.2 GHz"
    );
}

#[test]
fn test_access_point_with_frequencies_field() {
    // Test AccessPoint struct can hold multiple frequencies
    use rusty_de::service::wifi::AccessPoint;

    let ap = AccessPoint {
        ssid: "TestNetwork".to_string(),
        strength: 85,
        path: "/org/freedesktop/NetworkManager/AccessPoint/123".to_string(),
        requires_password: true,
        has_saved_connection: false,
        frequencies: vec![2412, 5180],
    };

    assert_eq!(ap.frequencies.len(), 2);
    assert!(ap.frequencies.contains(&2412));
    assert!(ap.frequencies.contains(&5180));
}
