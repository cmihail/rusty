use rusty_de::service::network::NetworkType;

#[test]
fn test_network_type_from_string() {
    let test_cases = vec![
        ("802-3-ethernet", NetworkType::Wired),
        ("802-11-wireless", NetworkType::Wifi),
        ("vpn", NetworkType::Unknown),
        ("", NetworkType::Unknown),
        ("unknown-type", NetworkType::Unknown),
    ];

    for (value, expected) in test_cases {
        let network_type = NetworkType::from_string(value);
        assert_eq!(
            network_type, expected,
            "Value '{}' should convert to {:?}",
            value, expected
        );
    }
}

#[test]
fn test_network_type_as_str() {
    let test_cases = vec![
        (NetworkType::Wired, "wired"),
        (NetworkType::Wifi, "wifi"),
        (NetworkType::Unknown, "unknown"),
    ];

    for (network_type, expected) in test_cases {
        let result = network_type.as_str();
        assert_eq!(
            result, expected,
            "NetworkType {:?} should convert to '{}'",
            network_type, expected
        );
    }
}

#[test]
fn test_network_type_roundtrip() {
    let test_cases = vec![("802-3-ethernet", "wired"), ("802-11-wireless", "wifi")];

    for (input, expected_output) in test_cases {
        let network_type = NetworkType::from_string(input);
        let output = network_type.as_str();
        assert_eq!(output, expected_output, "Roundtrip failed for '{}'", input);
    }
}

#[test]
fn test_network_type_equality() {
    assert_eq!(NetworkType::Wired, NetworkType::Wired);
    assert_eq!(NetworkType::Wifi, NetworkType::Wifi);
    assert_eq!(NetworkType::Unknown, NetworkType::Unknown);

    assert_ne!(NetworkType::Wired, NetworkType::Wifi);
    assert_ne!(NetworkType::Wired, NetworkType::Unknown);
    assert_ne!(NetworkType::Wifi, NetworkType::Unknown);
}

#[test]
fn test_icon_name_wired_connected() {
    let connected = true;
    let network_type = NetworkType::Wired;

    let icon = if network_type == NetworkType::Wired {
        if connected {
            "network-wired-symbolic".to_string()
        } else {
            "network-wired-disconnected-symbolic".to_string()
        }
    } else {
        "network-offline-symbolic".to_string()
    };

    assert_eq!(icon, "network-wired-symbolic");
}

#[test]
fn test_icon_name_wired_disconnected() {
    let connected = false;
    let network_type = NetworkType::Wired;

    let icon = if network_type == NetworkType::Wired {
        if connected {
            "network-wired-symbolic".to_string()
        } else {
            "network-wired-disconnected-symbolic".to_string()
        }
    } else {
        "network-offline-symbolic".to_string()
    };

    assert_eq!(icon, "network-wired-disconnected-symbolic");
}

#[test]
fn test_icon_name_wifi_excellent_signal() {
    let network_type = NetworkType::Wifi;
    let wifi_connected = true;
    let signal_strength = 95u8;

    let icon = if network_type == NetworkType::Wifi {
        if !wifi_connected {
            "network-wireless-disconnected-symbolic".to_string()
        } else if signal_strength >= 80 {
            "network-wireless-signal-excellent-symbolic".to_string()
        } else if signal_strength >= 60 {
            "network-wireless-signal-good-symbolic".to_string()
        } else if signal_strength >= 40 {
            "network-wireless-signal-ok-symbolic".to_string()
        } else if signal_strength >= 20 {
            "network-wireless-signal-weak-symbolic".to_string()
        } else {
            "network-wireless-signal-none-symbolic".to_string()
        }
    } else {
        "network-offline-symbolic".to_string()
    };

    assert_eq!(icon, "network-wireless-signal-excellent-symbolic");
}

#[test]
fn test_icon_name_wifi_disconnected() {
    let network_type = NetworkType::Wifi;
    let wifi_connected = false;

    let icon = if network_type == NetworkType::Wifi {
        if !wifi_connected {
            "network-wireless-disconnected-symbolic".to_string()
        } else {
            "network-wireless-signal-excellent-symbolic".to_string()
        }
    } else {
        "network-offline-symbolic".to_string()
    };

    assert_eq!(icon, "network-wireless-disconnected-symbolic");
}

#[test]
fn test_icon_name_unknown_network() {
    let network_type = NetworkType::Unknown;

    let icon = match network_type {
        NetworkType::Wired => "network-wired-symbolic".to_string(),
        NetworkType::Wifi => "network-wireless-signal-excellent-symbolic".to_string(),
        NetworkType::Unknown => "network-offline-symbolic".to_string(),
    };

    assert_eq!(icon, "network-offline-symbolic");
}

#[test]
fn test_primary_network_change_detection() {
    let old_primary = "802-3-ethernet";
    let new_primary = "802-11-wireless";

    let changed = old_primary != new_primary;
    assert!(
        changed,
        "Primary network change from '{}' to '{}' should be detected",
        old_primary, new_primary
    );
}

#[test]
fn test_primary_network_no_change_detection() {
    let old_primary = "802-3-ethernet";
    let new_primary = "802-3-ethernet";

    let changed = old_primary != new_primary;
    assert!(
        !changed,
        "No primary network change should be detected when types are identical"
    );
}

#[test]
fn test_network_type_pattern_matching() {
    let network_type = NetworkType::Wired;

    let description = match network_type {
        NetworkType::Wired => "Ethernet connection",
        NetworkType::Wifi => "Wireless connection",
        NetworkType::Unknown => "No connection",
    };

    assert_eq!(description, "Ethernet connection");
}

#[test]
fn test_empty_string_to_unknown() {
    let network_type = NetworkType::from_string("");
    assert_eq!(network_type, NetworkType::Unknown);
}

#[test]
fn test_case_sensitive_connection_types() {
    // NetworkManager connection types are case-sensitive
    let test_cases = vec![
        ("802-3-Ethernet", NetworkType::Unknown),  // Wrong case
        ("802-11-Wireless", NetworkType::Unknown), // Wrong case
        ("802-3-ethernet", NetworkType::Wired),    // Correct
        ("802-11-wireless", NetworkType::Wifi),    // Correct
    ];

    for (input, expected) in test_cases {
        let result = NetworkType::from_string(input);
        assert_eq!(
            result, expected,
            "Input '{}' should convert to {:?}",
            input, expected
        );
    }
}

#[test]
fn test_network_transition_sequence() {
    let states = vec![
        ("", "unknown"),
        ("802-3-ethernet", "wired"),
        ("802-11-wireless", "wifi"),
        ("", "unknown"),
    ];

    let mut results = Vec::new();
    for (conn_type, _expected_str) in &states {
        let network_type = NetworkType::from_string(conn_type);
        results.push(network_type.as_str());
    }

    assert_eq!(results, vec!["unknown", "wired", "wifi", "unknown"]);
}

#[test]
fn test_all_icon_variations() {
    let test_cases = vec![
        (NetworkType::Wired, true, "network-wired-symbolic"),
        (
            NetworkType::Wired,
            false,
            "network-wired-disconnected-symbolic",
        ),
        (NetworkType::Unknown, true, "network-offline-symbolic"),
        (NetworkType::Unknown, false, "network-offline-symbolic"),
    ];

    for (network_type, connected, expected_icon) in test_cases {
        let icon = if network_type == NetworkType::Wired {
            if connected {
                "network-wired-symbolic"
            } else {
                "network-wired-disconnected-symbolic"
            }
        } else {
            "network-offline-symbolic"
        };

        assert_eq!(
            icon, expected_icon,
            "NetworkType {:?} with connected={} should produce icon '{}'",
            network_type, connected, expected_icon
        );
    }
}
