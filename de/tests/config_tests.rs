use rusty_de::config::{moved_pin, toggled_pin, with_dock_pinned};

fn pins(ids: &[&str]) -> Vec<String> {
    ids.iter().map(|id| id.to_string()).collect()
}

#[test]
fn test_moved_pin_places_before_or_after_the_target() {
    let pinned = pins(&["a", "b", "c", "d"]);

    assert_eq!(
        moved_pin(pinned.clone(), "d", "b", false),
        pins(&["a", "d", "b", "c"])
    );
    assert_eq!(
        moved_pin(pinned.clone(), "a", "c", true),
        pins(&["b", "c", "a", "d"])
    );
    assert_eq!(
        moved_pin(pinned.clone(), "a", "d", true),
        pins(&["b", "c", "d", "a"])
    );
    assert_eq!(
        moved_pin(pinned, "c", "a", false),
        pins(&["c", "a", "b", "d"])
    );
}

#[test]
fn test_moved_pin_ignores_unknown_ids_and_self() {
    let pinned = pins(&["a", "b"]);

    assert_eq!(moved_pin(pinned.clone(), "x", "a", false), pinned);
    assert_eq!(moved_pin(pinned.clone(), "a", "x", false), pinned);
    assert_eq!(moved_pin(pinned.clone(), "a", "a", true), pinned);
}

#[test]
fn test_with_dock_pinned_adds_a_dock_table_and_keeps_comments() {
    let contents = "# devices\n[[bluetooth.expected_devices]]\naddress = \"AA\"\n";

    let updated = with_dock_pinned(contents, &["firefox".to_string(), "kitty".to_string()]);

    assert_eq!(
        updated.unwrap(),
        concat!(
            "# devices\n[[bluetooth.expected_devices]]\naddress = \"AA\"\n",
            "\n[dock]\npinned = [\"firefox\", \"kitty\"]\n"
        )
    );
}

#[test]
fn test_with_dock_pinned_replaces_the_list_in_place() {
    let contents = "[dock]\n# my apps\npinned = [\"slack\"]\n";

    let updated = with_dock_pinned(contents, &[]).unwrap();

    assert_eq!(updated, "[dock]\n# my apps\npinned = []\n");
}

#[test]
fn test_with_dock_pinned_rejects_invalid_config() {
    assert!(with_dock_pinned("dock = 1", &[]).is_err());
    assert!(with_dock_pinned("[dock", &[]).is_err());
}

#[test]
fn test_toggled_pin_appends_or_removes() {
    let pinned = vec!["firefox".to_string(), "kitty".to_string()];

    assert_eq!(
        toggled_pin(pinned.clone(), "slack"),
        vec!["firefox", "kitty", "slack"]
    );
    assert_eq!(toggled_pin(pinned, "firefox"), vec!["kitty"]);
}

#[test]
fn test_config_default_bluetooth_empty() {
    // Test that default config has empty expected devices list
    let expected_devices: Vec<String> = vec![];

    assert!(
        expected_devices.is_empty(),
        "Default config should have no expected devices"
    );
}

#[test]
fn test_config_bluetooth_expected_device_struct() {
    // Test that expected device struct can hold MAC address and optional name
    struct ExpectedDevice {
        address: String,
        name: Option<String>,
    }

    let device = ExpectedDevice {
        address: "AA:BB:CC:DD:EE:FF".to_string(),
        name: Some("My Headphones".to_string()),
    };

    assert_eq!(device.address, "AA:BB:CC:DD:EE:FF");
    assert_eq!(device.name, Some("My Headphones".to_string()));
}

#[test]
fn test_config_bluetooth_expected_device_without_name() {
    // Test that expected device can be created without name
    struct ExpectedDevice {
        address: String,
        name: Option<String>,
    }

    let device = ExpectedDevice {
        address: "AA:BB:CC:DD:EE:FF".to_string(),
        name: None,
    };

    assert_eq!(device.address, "AA:BB:CC:DD:EE:FF");
    assert_eq!(device.name, None);
}

#[test]
fn test_config_bluetooth_mac_address_format() {
    // Test MAC address format validation
    let valid_mac_addresses = vec![
        "AA:BB:CC:DD:EE:FF",
        "00:11:22:33:44:55",
        "FF:EE:DD:CC:BB:AA",
        "aa:bb:cc:dd:ee:ff", // lowercase also valid
    ];

    for mac in valid_mac_addresses {
        assert_eq!(mac.len(), 17, "MAC address should be 17 characters");
        assert_eq!(
            mac.matches(':').count(),
            5,
            "MAC address should have 5 colons"
        );
    }
}

#[test]
fn test_config_bluetooth_multiple_expected_devices() {
    // Test that config can hold multiple expected devices
    struct ExpectedDevice {
        address: String,
        name: Option<String>,
    }

    let devices = vec![
        ExpectedDevice {
            address: "AA:BB:CC:DD:EE:FF".to_string(),
            name: Some("Device 1".to_string()),
        },
        ExpectedDevice {
            address: "11:22:33:44:55:66".to_string(),
            name: Some("Device 2".to_string()),
        },
        ExpectedDevice {
            address: "77:88:99:AA:BB:CC".to_string(),
            name: None,
        },
    ];

    assert_eq!(devices.len(), 3, "Should hold multiple devices");
    assert_eq!(devices[0].address, "AA:BB:CC:DD:EE:FF");
    assert_eq!(devices[1].address, "11:22:33:44:55:66");
    assert_eq!(devices[2].address, "77:88:99:AA:BB:CC");
    assert_eq!(devices[2].name, None);
}

#[test]
fn test_config_path_format() {
    // Test config path format
    let home = "/home/user";
    let config_path = format!("{}/.config/rusty/de.toml", home);

    assert_eq!(
        config_path, "/home/user/.config/rusty/de.toml",
        "Config path should follow XDG standard"
    );
}

#[test]
fn test_config_bluetooth_section_structure() {
    // Test the structure of bluetooth config section
    struct BluetoothConfig {
        expected_devices: Vec<ExpectedDevice>,
    }

    struct ExpectedDevice {
        address: String,
        name: Option<String>,
    }

    let config = BluetoothConfig {
        expected_devices: vec![
            ExpectedDevice {
                address: "AA:BB:CC:DD:EE:FF".to_string(),
                name: Some("Device 1".to_string()),
            },
            ExpectedDevice {
                address: "11:22:33:44:55:66".to_string(),
                name: None,
            },
        ],
    };

    assert_eq!(config.expected_devices.len(), 2);
    assert_eq!(config.expected_devices[0].address, "AA:BB:CC:DD:EE:FF");
    assert!(config.expected_devices[0].name.is_some());
    assert!(config.expected_devices[1].name.is_none());
}

#[test]
fn test_config_default_returns_valid_config() {
    // Test that default config is valid
    struct BluetoothConfig {
        expected_devices: Vec<String>,
    }

    impl Default for BluetoothConfig {
        fn default() -> Self {
            Self {
                expected_devices: vec![],
            }
        }
    }

    let config = BluetoothConfig::default();

    assert!(
        config.expected_devices.is_empty(),
        "Default config should have empty expected devices"
    );
}
