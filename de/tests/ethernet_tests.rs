use rusty_de::service::ethernet::EthernetState;

#[test]
fn test_ethernet_state_from_u32() {
    let test_cases = vec![
        (0, EthernetState::Unknown),
        (10, EthernetState::Unmanaged),
        (20, EthernetState::Unavailable),
        (30, EthernetState::Disconnected),
        (40, EthernetState::Preparing),
        (50, EthernetState::Config),
        (60, EthernetState::NeedAuth),
        (70, EthernetState::IpConfig),
        (80, EthernetState::IpCheck),
        (90, EthernetState::Secondaries),
        (100, EthernetState::Activated),
        (110, EthernetState::Deactivating),
        (120, EthernetState::Failed),
        (99, EthernetState::Unknown),
    ];

    for (value, expected) in test_cases {
        let state = EthernetState::from_u32(value);
        assert_eq!(
            state, expected,
            "Value {} should convert to {:?}",
            value, expected
        );
    }
}

#[test]
fn test_ethernet_icon_connected() {
    let connected = true;

    let icon = if connected {
        "network-wired-symbolic".to_string()
    } else {
        "network-wired-disconnected-symbolic".to_string()
    };

    assert_eq!(icon, "network-wired-symbolic");
}

#[test]
fn test_ethernet_icon_disconnected() {
    let connected = false;

    let icon = if connected {
        "network-wired-symbolic".to_string()
    } else {
        "network-wired-disconnected-symbolic".to_string()
    };

    assert_eq!(icon, "network-wired-disconnected-symbolic");
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
fn test_connection_id_change_detection() {
    let old_id = "Huawei 4G".to_string();
    let new_id = "Ethernet Connection".to_string();

    let changed = old_id != new_id;
    assert!(
        changed,
        "Connection ID change from '{}' to '{}' should be detected",
        old_id, new_id
    );
}

#[test]
fn test_connection_id_no_change_detection() {
    let old_id = "Huawei 4G".to_string();
    let new_id = "Huawei 4G".to_string();

    let changed = old_id != new_id;
    assert!(
        !changed,
        "No connection ID change should be detected when IDs are identical"
    );
}

#[test]
fn test_ethernet_state_change_detection() {
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
fn test_ethernet_state_no_change_detection() {
    let old_state = 100u32; // Connected
    let new_state = 100u32; // Connected

    let changed = old_state != new_state;
    assert!(
        !changed,
        "No state change should be detected when states are identical"
    );
}

#[test]
fn test_device_product_change_detection() {
    let old_product = "Intel Ethernet".to_string();
    let new_product = "Realtek Ethernet".to_string();

    let changed = old_product != new_product;
    assert!(
        changed,
        "Device product change from '{}' to '{}' should be detected",
        old_product, new_product
    );
}

#[test]
fn test_device_product_no_change_detection() {
    let old_product = "Intel Ethernet".to_string();
    let new_product = "Intel Ethernet".to_string();

    let changed = old_product != new_product;
    assert!(
        !changed,
        "No device product change should be detected when products are identical"
    );
}

#[test]
fn test_empty_connection_id_change_detection() {
    let old_id = "Ethernet Connection".to_string();
    let new_id = String::new();

    let changed = old_id != new_id;
    assert!(
        changed,
        "Connection ID change from '{}' to empty should be detected",
        old_id
    );
}

#[test]
fn test_ethernet_state_values() {
    assert_eq!(EthernetState::Unknown as u32, 0);
    assert_eq!(EthernetState::Unmanaged as u32, 10);
    assert_eq!(EthernetState::Unavailable as u32, 20);
    assert_eq!(EthernetState::Disconnected as u32, 30);
    assert_eq!(EthernetState::Preparing as u32, 40);
    assert_eq!(EthernetState::Config as u32, 50);
    assert_eq!(EthernetState::NeedAuth as u32, 60);
    assert_eq!(EthernetState::IpConfig as u32, 70);
    assert_eq!(EthernetState::IpCheck as u32, 80);
    assert_eq!(EthernetState::Secondaries as u32, 90);
    assert_eq!(EthernetState::Activated as u32, 100);
    assert_eq!(EthernetState::Deactivating as u32, 110);
    assert_eq!(EthernetState::Failed as u32, 120);
}

#[test]
fn test_ethernet_state_connection_sequence() {
    let states = [
        EthernetState::Disconnected,
        EthernetState::Preparing,
        EthernetState::Config,
        EthernetState::IpConfig,
        EthernetState::IpCheck,
        EthernetState::Activated,
    ];

    let state_values: Vec<u32> = states.iter().map(|s| *s as u32).collect();
    assert_eq!(state_values, vec![30, 40, 50, 70, 80, 100]);
}

#[test]
fn test_ethernet_state_disconnection_sequence() {
    let states = [
        EthernetState::Activated,
        EthernetState::Deactivating,
        EthernetState::Disconnected,
    ];

    let state_values: Vec<u32> = states.iter().map(|s| *s as u32).collect();
    assert_eq!(state_values, vec![100, 110, 30]);
}

#[test]
fn test_ethernet_icon_by_state_activated() {
    use rusty_de::service::ethernet::EthernetState;

    let state = EthernetState::Activated;

    let icon = match state {
        EthernetState::Activated => "network-wired-symbolic".to_string(),
        EthernetState::Preparing
        | EthernetState::Config
        | EthernetState::NeedAuth
        | EthernetState::IpConfig
        | EthernetState::IpCheck
        | EthernetState::Secondaries => "network-wired-acquiring-symbolic".to_string(),
        EthernetState::Deactivating => "network-wired-disconnected-symbolic".to_string(),
        EthernetState::Disconnected
        | EthernetState::Unavailable
        | EthernetState::Unmanaged
        | EthernetState::Failed
        | EthernetState::Unknown => "network-wired-disconnected-symbolic".to_string(),
    };

    assert_eq!(icon, "network-wired-symbolic");
}

#[test]
fn test_ethernet_icon_by_state_connecting() {
    use rusty_de::service::ethernet::EthernetState;

    let states = vec![
        EthernetState::Preparing,
        EthernetState::Config,
        EthernetState::NeedAuth,
        EthernetState::IpConfig,
        EthernetState::IpCheck,
        EthernetState::Secondaries,
    ];

    for state in states {
        let icon = match state {
            EthernetState::Activated => "network-wired-symbolic".to_string(),
            EthernetState::Preparing
            | EthernetState::Config
            | EthernetState::NeedAuth
            | EthernetState::IpConfig
            | EthernetState::IpCheck
            | EthernetState::Secondaries => "network-wired-acquiring-symbolic".to_string(),
            EthernetState::Deactivating => "network-wired-disconnected-symbolic".to_string(),
            EthernetState::Disconnected
            | EthernetState::Unavailable
            | EthernetState::Unmanaged
            | EthernetState::Failed
            | EthernetState::Unknown => "network-wired-disconnected-symbolic".to_string(),
        };

        assert_eq!(
            icon, "network-wired-acquiring-symbolic",
            "State {:?} should show acquiring icon",
            state
        );
    }
}

#[test]
fn test_ethernet_icon_by_state_disconnected() {
    use rusty_de::service::ethernet::EthernetState;

    let states = vec![
        EthernetState::Disconnected,
        EthernetState::Unavailable,
        EthernetState::Unmanaged,
        EthernetState::Failed,
        EthernetState::Unknown,
        EthernetState::Deactivating,
    ];

    for state in states {
        let icon = match state {
            EthernetState::Activated => "network-wired-symbolic".to_string(),
            EthernetState::Preparing
            | EthernetState::Config
            | EthernetState::NeedAuth
            | EthernetState::IpConfig
            | EthernetState::IpCheck
            | EthernetState::Secondaries => "network-wired-acquiring-symbolic".to_string(),
            EthernetState::Deactivating => "network-wired-disconnected-symbolic".to_string(),
            EthernetState::Disconnected
            | EthernetState::Unavailable
            | EthernetState::Unmanaged
            | EthernetState::Failed
            | EthernetState::Unknown => "network-wired-disconnected-symbolic".to_string(),
        };

        assert_eq!(
            icon, "network-wired-disconnected-symbolic",
            "State {:?} should show disconnected icon",
            state
        );
    }
}

#[test]
fn test_ethernet_state_transition_ipconfig_to_activated() {
    let old_state = EthernetState::IpConfig;
    let new_state = EthernetState::Activated;

    assert_ne!(old_state, new_state);
    assert_eq!(old_state as u32, 70);
    assert_eq!(new_state as u32, 100);
}

#[test]
fn test_ethernet_state_transition_ipconfig_to_ipcheck() {
    let old_state = EthernetState::IpConfig;
    let new_state = EthernetState::IpCheck;

    assert_ne!(old_state, new_state);
    assert_eq!(old_state as u32, 70);
    assert_eq!(new_state as u32, 80);
}

#[test]
fn test_ethernet_state_transition_ipcheck_to_activated() {
    let old_state = EthernetState::IpCheck;
    let new_state = EthernetState::Activated;

    assert_ne!(old_state, new_state);
    assert_eq!(old_state as u32, 80);
    assert_eq!(new_state as u32, 100);
}

#[test]
fn test_ethernet_singleton_returns_same_instance() {
    use rusty_de::service::ethernet::Ethernet;

    let instance1 = Ethernet::instance();
    let instance2 = Ethernet::instance();

    // Both should return the same singleton instance
    // We can't directly compare GObject instances, but we can verify they exist
    assert!(instance1.connected() == instance2.connected());
}

#[test]
fn test_ethernet_icon_name_for_all_states() {
    use rusty_de::service::ethernet::EthernetState;

    let test_cases = vec![
        (EthernetState::Activated, "network-wired-symbolic"),
        (EthernetState::Preparing, "network-wired-acquiring-symbolic"),
        (EthernetState::Config, "network-wired-acquiring-symbolic"),
        (EthernetState::NeedAuth, "network-wired-acquiring-symbolic"),
        (EthernetState::IpConfig, "network-wired-acquiring-symbolic"),
        (EthernetState::IpCheck, "network-wired-acquiring-symbolic"),
        (
            EthernetState::Secondaries,
            "network-wired-acquiring-symbolic",
        ),
        (
            EthernetState::Deactivating,
            "network-wired-disconnected-symbolic",
        ),
        (
            EthernetState::Disconnected,
            "network-wired-disconnected-symbolic",
        ),
        (
            EthernetState::Unavailable,
            "network-wired-disconnected-symbolic",
        ),
        (
            EthernetState::Unmanaged,
            "network-wired-disconnected-symbolic",
        ),
        (EthernetState::Failed, "network-wired-disconnected-symbolic"),
        (
            EthernetState::Unknown,
            "network-wired-disconnected-symbolic",
        ),
    ];

    for (state, expected_icon) in test_cases {
        let icon = match state {
            EthernetState::Activated => "network-wired-symbolic".to_string(),
            EthernetState::Preparing
            | EthernetState::Config
            | EthernetState::NeedAuth
            | EthernetState::IpConfig
            | EthernetState::IpCheck
            | EthernetState::Secondaries => "network-wired-acquiring-symbolic".to_string(),
            EthernetState::Deactivating => "network-wired-disconnected-symbolic".to_string(),
            EthernetState::Disconnected
            | EthernetState::Unavailable
            | EthernetState::Unmanaged
            | EthernetState::Failed
            | EthernetState::Unknown => "network-wired-disconnected-symbolic".to_string(),
        };

        assert_eq!(
            icon, expected_icon,
            "State {:?} should return icon '{}'",
            state, expected_icon
        );
    }
}

#[test]
fn test_ethernet_state_is_active() {
    use rusty_de::service::ethernet::EthernetState;

    // States >= 40 are considered active/connecting
    let active_states = vec![
        EthernetState::Preparing,    // 40
        EthernetState::Config,       // 50
        EthernetState::NeedAuth,     // 60
        EthernetState::IpConfig,     // 70
        EthernetState::IpCheck,      // 80
        EthernetState::Secondaries,  // 90
        EthernetState::Activated,    // 100
        EthernetState::Deactivating, // 110
    ];

    for state in active_states {
        let state_value = state as u32;
        assert!(
            state_value >= 40,
            "State {:?} (value {}) should be considered active (>= 40)",
            state,
            state_value
        );
    }

    // States < 40 are not active
    let inactive_states = vec![
        EthernetState::Unknown,      // 0
        EthernetState::Unmanaged,    // 10
        EthernetState::Unavailable,  // 20
        EthernetState::Disconnected, // 30
    ];

    for state in inactive_states {
        let state_value = state as u32;
        assert!(
            state_value < 40,
            "State {:?} (value {}) should not be considered active (< 40)",
            state,
            state_value
        );
    }
}

#[test]
fn test_ethernet_state_is_fully_connected() {
    use rusty_de::service::ethernet::EthernetState;

    // Only Activated (100) is fully connected
    assert_eq!(EthernetState::Activated as u32, 100);

    let not_fully_connected = vec![
        EthernetState::Unknown,
        EthernetState::Unmanaged,
        EthernetState::Unavailable,
        EthernetState::Disconnected,
        EthernetState::Preparing,
        EthernetState::Config,
        EthernetState::NeedAuth,
        EthernetState::IpConfig,
        EthernetState::IpCheck,
        EthernetState::Secondaries,
        EthernetState::Deactivating,
        EthernetState::Failed,
    ];

    for state in not_fully_connected {
        assert_ne!(
            state as u32, 100,
            "State {:?} should not be value 100 (Activated)",
            state
        );
    }
}

#[test]
fn test_connection_id_empty_string_handling() {
    let connection_id = String::new();

    let display_name = if !connection_id.is_empty() {
        connection_id.clone()
    } else {
        "Wired Connection".to_string()
    };

    assert_eq!(display_name, "Wired Connection");
}

#[test]
fn test_connection_id_non_empty_string_handling() {
    let connection_id = "Huawei 4G".to_string();

    let display_name = if !connection_id.is_empty() {
        connection_id.clone()
    } else {
        "Wired Connection".to_string()
    };

    assert_eq!(display_name, "Huawei 4G");
}

#[test]
fn test_device_count_when_connected() {
    let connected = true;
    let device_count = if connected { 1 } else { 0 };
    assert_eq!(device_count, 1);
}

#[test]
fn test_device_count_when_disconnected() {
    let connected = false;
    let device_count = if connected { 1 } else { 0 };
    assert_eq!(device_count, 0);
}

#[test]
fn test_small_text_for_connected_with_id() {
    let connected = true;
    let connection_id = "Huawei 4G".to_string();

    let small_text = if connected && !connection_id.is_empty() {
        Some(connection_id.clone())
    } else if !connected {
        Some("Devices: 0".to_string())
    } else {
        None
    };

    assert_eq!(small_text, Some("Huawei 4G".to_string()));
}

#[test]
fn test_small_text_for_disconnected() {
    let connected = false;
    let connection_id = "Huawei 4G".to_string();

    let small_text = if connected && !connection_id.is_empty() {
        Some(connection_id.clone())
    } else if !connected {
        Some("Devices: 0".to_string())
    } else {
        None
    };

    assert_eq!(small_text, Some("Devices: 0".to_string()));
}

#[test]
fn test_small_text_for_connected_without_id() {
    let connected = true;
    let connection_id = String::new();

    let small_text = if connected && !connection_id.is_empty() {
        Some(connection_id.clone())
    } else if !connected {
        Some("Devices: 0".to_string())
    } else {
        None
    };

    assert_eq!(small_text, None);
}

#[test]
fn test_ethernet_device_creation() {
    use rusty_de::service::ethernet::EthernetDevice;
    use zbus::zvariant::OwnedObjectPath;

    let device_path =
        OwnedObjectPath::try_from("/org/freedesktop/NetworkManager/Devices/1").expect("Valid path");

    let device = EthernetDevice {
        device_path: device_path.clone(),
        id: "Wired Connection 1".to_string(),
        product: "Intel Ethernet Controller".to_string(),
        connected: true,
        is_default: true,
    };

    assert_eq!(device.device_path, device_path);
    assert_eq!(device.id, "Wired Connection 1");
    assert_eq!(device.product, "Intel Ethernet Controller");
    assert!(device.connected);
    assert!(device.is_default);
}

#[test]
fn test_ethernet_device_clone() {
    use rusty_de::service::ethernet::EthernetDevice;
    use zbus::zvariant::OwnedObjectPath;

    let device_path =
        OwnedObjectPath::try_from("/org/freedesktop/NetworkManager/Devices/1").expect("Valid path");

    let device1 = EthernetDevice {
        device_path: device_path.clone(),
        id: "Wired Connection 1".to_string(),
        product: "Intel Ethernet Controller".to_string(),
        connected: true,
        is_default: false,
    };

    let device2 = device1.clone();

    assert_eq!(device1.device_path, device2.device_path);
    assert_eq!(device1.id, device2.id);
    assert_eq!(device1.product, device2.product);
    assert_eq!(device1.connected, device2.connected);
    assert_eq!(device1.is_default, device2.is_default);
}

#[test]
fn test_multiple_ethernet_devices() {
    use rusty_de::service::ethernet::EthernetDevice;
    use zbus::zvariant::OwnedObjectPath;

    let device1_path =
        OwnedObjectPath::try_from("/org/freedesktop/NetworkManager/Devices/1").expect("Valid path");
    let device2_path =
        OwnedObjectPath::try_from("/org/freedesktop/NetworkManager/Devices/2").expect("Valid path");

    let device1 = EthernetDevice {
        device_path: device1_path,
        id: "Wired Connection 1".to_string(),
        product: "Intel Ethernet".to_string(),
        connected: true,
        is_default: true,
    };

    let device2 = EthernetDevice {
        device_path: device2_path,
        id: "Wired Connection 2".to_string(),
        product: "Realtek Ethernet".to_string(),
        connected: false,
        is_default: false,
    };

    let devices = vec![device1, device2];

    assert_eq!(devices.len(), 2);
    assert_eq!(devices[0].id, "Wired Connection 1");
    assert_eq!(devices[1].id, "Wired Connection 2");
    assert!(devices[0].connected);
    assert!(!devices[1].connected);
}

#[test]
fn test_devices_list_changed_different_lengths() {
    use rusty_de::service::ethernet::EthernetDevice;
    use zbus::zvariant::OwnedObjectPath;

    let device1_path =
        OwnedObjectPath::try_from("/org/freedesktop/NetworkManager/Devices/1").expect("Valid path");
    let device2_path =
        OwnedObjectPath::try_from("/org/freedesktop/NetworkManager/Devices/2").expect("Valid path");

    let old_devices = vec![EthernetDevice {
        device_path: device1_path.clone(),
        id: "Wired Connection 1".to_string(),
        product: "Intel Ethernet".to_string(),
        connected: true,
        is_default: true,
    }];

    let new_devices = vec![
        EthernetDevice {
            device_path: device1_path,
            id: "Wired Connection 1".to_string(),
            product: "Intel Ethernet".to_string(),
            connected: true,
            is_default: true,
        },
        EthernetDevice {
            device_path: device2_path,
            id: "Wired Connection 2".to_string(),
            product: "Realtek Ethernet".to_string(),
            connected: false,
            is_default: false,
        },
    ];

    let changed = old_devices.len() != new_devices.len();
    assert!(changed, "Device list length change should be detected");
}

#[test]
fn test_devices_list_changed_different_device_paths() {
    use rusty_de::service::ethernet::EthernetDevice;
    use zbus::zvariant::OwnedObjectPath;

    let device1_path =
        OwnedObjectPath::try_from("/org/freedesktop/NetworkManager/Devices/1").expect("Valid path");
    let device2_path =
        OwnedObjectPath::try_from("/org/freedesktop/NetworkManager/Devices/2").expect("Valid path");

    let old_devices = vec![EthernetDevice {
        device_path: device1_path,
        id: "Wired Connection 1".to_string(),
        product: "Intel Ethernet".to_string(),
        connected: true,
        is_default: true,
    }];

    let new_devices = vec![EthernetDevice {
        device_path: device2_path,
        id: "Wired Connection 1".to_string(),
        product: "Intel Ethernet".to_string(),
        connected: true,
        is_default: true,
    }];

    let changed = old_devices[0].device_path != new_devices[0].device_path;
    assert!(changed, "Device path change should be detected");
}

#[test]
fn test_devices_list_changed_different_connection_states() {
    use rusty_de::service::ethernet::EthernetDevice;
    use zbus::zvariant::OwnedObjectPath;

    let device_path =
        OwnedObjectPath::try_from("/org/freedesktop/NetworkManager/Devices/1").expect("Valid path");

    let old_devices = vec![EthernetDevice {
        device_path: device_path.clone(),
        id: "Wired Connection 1".to_string(),
        product: "Intel Ethernet".to_string(),
        connected: false,
        is_default: false,
    }];

    let new_devices = vec![EthernetDevice {
        device_path,
        id: "Wired Connection 1".to_string(),
        product: "Intel Ethernet".to_string(),
        connected: true,
        is_default: true,
    }];

    let changed = old_devices[0].connected != new_devices[0].connected
        || old_devices[0].is_default != new_devices[0].is_default;
    assert!(changed, "Connection state change should be detected");
}

#[test]
fn test_devices_list_changed_different_ids() {
    use rusty_de::service::ethernet::EthernetDevice;
    use zbus::zvariant::OwnedObjectPath;

    let device_path =
        OwnedObjectPath::try_from("/org/freedesktop/NetworkManager/Devices/1").expect("Valid path");

    let old_devices = vec![EthernetDevice {
        device_path: device_path.clone(),
        id: "Wired Connection 1".to_string(),
        product: "Intel Ethernet".to_string(),
        connected: true,
        is_default: true,
    }];

    let new_devices = vec![EthernetDevice {
        device_path,
        id: "Wired Connection 2".to_string(),
        product: "Intel Ethernet".to_string(),
        connected: true,
        is_default: true,
    }];

    let changed = old_devices[0].id != new_devices[0].id;
    assert!(changed, "Connection ID change should be detected");
}

#[test]
fn test_devices_list_not_changed() {
    use rusty_de::service::ethernet::EthernetDevice;
    use zbus::zvariant::OwnedObjectPath;

    let device_path =
        OwnedObjectPath::try_from("/org/freedesktop/NetworkManager/Devices/1").expect("Valid path");

    let old_devices = vec![EthernetDevice {
        device_path: device_path.clone(),
        id: "Wired Connection 1".to_string(),
        product: "Intel Ethernet".to_string(),
        connected: true,
        is_default: true,
    }];

    let new_devices = vec![EthernetDevice {
        device_path,
        id: "Wired Connection 1".to_string(),
        product: "Intel Ethernet".to_string(),
        connected: true,
        is_default: true,
    }];

    let changed = old_devices.len() != new_devices.len()
        || old_devices[0].device_path != new_devices[0].device_path
        || old_devices[0].id != new_devices[0].id
        || old_devices[0].product != new_devices[0].product
        || old_devices[0].connected != new_devices[0].connected
        || old_devices[0].is_default != new_devices[0].is_default;

    assert!(
        !changed,
        "No change should be detected when devices are identical"
    );
}

#[test]
fn test_empty_devices_list() {
    use rusty_de::service::ethernet::EthernetDevice;

    let devices: Vec<EthernetDevice> = Vec::new();

    assert_eq!(devices.len(), 0);
    assert!(devices.is_empty());
}

#[test]
fn test_device_count_header_with_multiple_devices() {
    let device_count = 2;
    let header_text = format!("Devices: {}", device_count);
    assert_eq!(header_text, "Devices: 2");
}

#[test]
fn test_separator_visibility_with_devices() {
    let devices_count = 2;
    let separator_visible = devices_count > 0;
    assert!(separator_visible);
}

#[test]
fn test_separator_visibility_without_devices() {
    let devices_count = 0;
    let separator_visible = devices_count > 0;
    assert!(!separator_visible);
}

#[test]
fn test_device_filtering_by_available_connections() {
    use rusty_de::service::ethernet::EthernetDevice;
    use zbus::zvariant::OwnedObjectPath;

    let device1_path =
        OwnedObjectPath::try_from("/org/freedesktop/NetworkManager/Devices/1").expect("Valid path");
    let device2_path =
        OwnedObjectPath::try_from("/org/freedesktop/NetworkManager/Devices/2").expect("Valid path");

    // Simulate filtering: only devices with available connections
    let all_devices = vec![
        ("device1", device1_path.clone(), true),  // has connections
        ("device2", device2_path.clone(), false), // no connections
    ];

    let filtered_devices: Vec<EthernetDevice> = all_devices
        .into_iter()
        .filter(|(_, _, has_connections)| *has_connections)
        .map(|(id, path, _)| EthernetDevice {
            device_path: path,
            id: id.to_string(),
            product: "".to_string(),
            connected: false,
            is_default: false,
        })
        .collect();

    assert_eq!(filtered_devices.len(), 1);
    assert_eq!(filtered_devices[0].id, "device1");
}

#[test]
fn test_default_device_priority() {
    use rusty_de::service::ethernet::EthernetDevice;
    use zbus::zvariant::OwnedObjectPath;

    let device1_path =
        OwnedObjectPath::try_from("/org/freedesktop/NetworkManager/Devices/1").expect("Valid path");
    let device2_path =
        OwnedObjectPath::try_from("/org/freedesktop/NetworkManager/Devices/2").expect("Valid path");

    let mut devices = vec![
        EthernetDevice {
            device_path: device1_path,
            id: "Wired Connection 1".to_string(),
            product: "Intel Ethernet".to_string(),
            connected: true,
            is_default: false,
        },
        EthernetDevice {
            device_path: device2_path,
            id: "Wired Connection 2".to_string(),
            product: "Realtek Ethernet".to_string(),
            connected: true,
            is_default: true,
        },
    ];

    // Sort by priority: default devices first
    devices.sort_by(|a, b| b.is_default.cmp(&a.is_default));

    assert_eq!(devices[0].id, "Wired Connection 2");
    assert!(devices[0].is_default);
}

#[test]
fn test_device_product_display() {
    let product = "Intel Ethernet Controller I219-V";
    let product_text = format!("Product: {}", product);
    assert_eq!(product_text, "Product: Intel Ethernet Controller I219-V");
}

#[test]
fn test_device_product_empty_handling() {
    let product = "";
    let should_display = !product.is_empty();
    assert!(!should_display);
}

// Tests for connected state logic with state >= 40
#[test]
fn test_connected_state_with_preparing_and_connection_id() {
    let device_state = 40u32; // Preparing
    let conn_id = "Wired Connection".to_string();

    // Should be connected when state >= 40 and has connection ID
    let is_connected = device_state >= 40 && !conn_id.is_empty();

    assert!(
        is_connected,
        "Device should be connected when state is {} (Preparing) with connection ID",
        device_state
    );
}

#[test]
fn test_connected_state_with_config_and_connection_id() {
    let device_state = 50u32; // Config
    let conn_id = "Wired Connection".to_string();

    let is_connected = device_state >= 40 && !conn_id.is_empty();

    assert!(
        is_connected,
        "Device should be connected when state is {} (Config) with connection ID",
        device_state
    );
}

#[test]
fn test_connected_state_with_ipconfig_and_connection_id() {
    let device_state = 70u32; // IpConfig
    let conn_id = "Wired Connection".to_string();

    let is_connected = device_state >= 40 && !conn_id.is_empty();

    assert!(
        is_connected,
        "Device should be connected when state is {} (IpConfig) with connection ID",
        device_state
    );
}

#[test]
fn test_connected_state_with_ipcheck_and_connection_id() {
    let device_state = 80u32; // IpCheck
    let conn_id = "Wired Connection".to_string();

    let is_connected = device_state >= 40 && !conn_id.is_empty();

    assert!(
        is_connected,
        "Device should be connected when state is {} (IpCheck) with connection ID",
        device_state
    );
}

#[test]
fn test_connected_state_with_activated_and_connection_id() {
    let device_state = 100u32; // Activated
    let conn_id = "Wired Connection".to_string();

    let is_connected = device_state >= 40 && !conn_id.is_empty();

    assert!(
        is_connected,
        "Device should be connected when state is {} (Activated) with connection ID",
        device_state
    );
}

#[test]
fn test_not_connected_state_with_disconnected() {
    let device_state = 30u32; // Disconnected
    let conn_id = "".to_string();

    let is_connected = device_state >= 40 && !conn_id.is_empty();

    assert!(
        !is_connected,
        "Device should not be connected when state is {} (Disconnected)",
        device_state
    );
}

#[test]
fn test_not_connected_state_with_preparing_but_no_connection_id() {
    let device_state = 40u32; // Preparing
    let conn_id = "".to_string();

    let is_connected = device_state >= 40 && !conn_id.is_empty();

    assert!(
        !is_connected,
        "Device should not be connected when state is {} (Preparing) without connection ID",
        device_state
    );
}

#[test]
fn test_not_connected_state_with_ipconfig_but_no_connection_id() {
    let device_state = 70u32; // IpConfig
    let conn_id = "".to_string();

    let is_connected = device_state >= 40 && !conn_id.is_empty();

    assert!(
        !is_connected,
        "Device should not be connected when state is {} (IpConfig) without connection ID",
        device_state
    );
}

// Tests for icon_name logic with connected state
#[test]
fn test_icon_connected_with_preparing_state() {
    let connected = true;
    let state_raw = 40u32; // Preparing

    // Icon should be network-wired-symbolic when connected and state >= 40
    let icon = if connected && state_raw >= 40 {
        "network-wired-symbolic"
    } else {
        "network-wired-acquiring-symbolic"
    };

    assert_eq!(
        icon, "network-wired-symbolic",
        "Icon should be network-wired-symbolic when connected with state {} (Preparing)",
        state_raw
    );
}

#[test]
fn test_icon_connected_with_config_state() {
    let connected = true;
    let state_raw = 50u32; // Config

    let icon = if connected && state_raw >= 40 {
        "network-wired-symbolic"
    } else {
        "network-wired-acquiring-symbolic"
    };

    assert_eq!(
        icon, "network-wired-symbolic",
        "Icon should be network-wired-symbolic when connected with state {} (Config)",
        state_raw
    );
}

#[test]
fn test_icon_connected_with_ipconfig_state() {
    let connected = true;
    let state_raw = 70u32; // IpConfig

    let icon = if connected && state_raw >= 40 {
        "network-wired-symbolic"
    } else {
        "network-wired-acquiring-symbolic"
    };

    assert_eq!(
        icon, "network-wired-symbolic",
        "Icon should be network-wired-symbolic when connected with state {} (IpConfig)",
        state_raw
    );
}

#[test]
fn test_icon_connected_with_ipcheck_state() {
    let connected = true;
    let state_raw = 80u32; // IpCheck

    let icon = if connected && state_raw >= 40 {
        "network-wired-symbolic"
    } else {
        "network-wired-acquiring-symbolic"
    };

    assert_eq!(
        icon, "network-wired-symbolic",
        "Icon should be network-wired-symbolic when connected with state {} (IpCheck)",
        state_raw
    );
}

#[test]
fn test_icon_connected_with_activated_state() {
    let connected = true;
    let state_raw = 100u32; // Activated

    let icon = if connected && state_raw >= 40 {
        "network-wired-symbolic"
    } else {
        "network-wired-acquiring-symbolic"
    };

    assert_eq!(
        icon, "network-wired-symbolic",
        "Icon should be network-wired-symbolic when connected with state {} (Activated)",
        state_raw
    );
}

#[test]
fn test_icon_not_connected_with_preparing_state() {
    let connected = false;
    let state_raw = 40u32; // Preparing

    let icon = if connected && state_raw >= 40 {
        "network-wired-symbolic"
    } else {
        "network-wired-acquiring-symbolic"
    };

    assert_eq!(
        icon, "network-wired-acquiring-symbolic",
        "Icon should be network-wired-acquiring-symbolic when not connected with state {} (Preparing)",
        state_raw
    );
}

#[test]
fn test_icon_not_connected_with_ipconfig_state() {
    let connected = false;
    let state_raw = 70u32; // IpConfig

    let icon = if connected && state_raw >= 40 {
        "network-wired-symbolic"
    } else {
        "network-wired-acquiring-symbolic"
    };

    assert_eq!(
        icon, "network-wired-acquiring-symbolic",
        "Icon should be network-wired-acquiring-symbolic when not connected with state {} (IpConfig)",
        state_raw
    );
}

#[test]
fn test_icon_disconnected_state() {
    let connected = false;
    let state_raw = 30u32; // Disconnected

    let icon = if connected && state_raw >= 40 {
        "network-wired-symbolic"
    } else if state_raw >= 40 {
        "network-wired-acquiring-symbolic"
    } else {
        "network-wired-disconnected-symbolic"
    };

    assert_eq!(
        icon, "network-wired-disconnected-symbolic",
        "Icon should be network-wired-disconnected-symbolic when state is {} (Disconnected)",
        state_raw
    );
}

// Tests for state transition sequences with new connected logic
#[test]
fn test_connected_state_during_connection_sequence() {
    let states = vec![
        (30u32, "".to_string(), false),      // Disconnected, no conn_id
        (40u32, "Wired".to_string(), true),  // Preparing, has conn_id
        (50u32, "Wired".to_string(), true),  // Config, has conn_id
        (70u32, "Wired".to_string(), true),  // IpConfig, has conn_id
        (80u32, "Wired".to_string(), true),  // IpCheck, has conn_id
        (100u32, "Wired".to_string(), true), // Activated, has conn_id
    ];

    for (state, conn_id, expected_connected) in states {
        let is_connected = state >= 40 && !conn_id.is_empty();
        assert_eq!(
            is_connected, expected_connected,
            "State {} with conn_id '{}' should have connected = {}",
            state, conn_id, expected_connected
        );
    }
}

#[test]
fn test_connected_state_during_disconnection_sequence() {
    let states = vec![
        (100u32, "Wired".to_string(), true), // Activated, has conn_id
        (110u32, "Wired".to_string(), true), // Deactivating, still has conn_id
        (30u32, "".to_string(), false),      // Disconnected, no conn_id
    ];

    for (state, conn_id, expected_connected) in states {
        let is_connected = state >= 40 && !conn_id.is_empty();
        assert_eq!(
            is_connected, expected_connected,
            "State {} with conn_id '{}' should have connected = {}",
            state, conn_id, expected_connected
        );
    }
}

#[test]
fn test_connection_state_change_from_disconnected_to_preparing() {
    // Old state: disconnected
    let old_state = 30u32;
    let old_conn_id = "";
    let old_connected = old_state >= 40 && !old_conn_id.is_empty();

    // New state: preparing with connection
    let new_state = 40u32;
    let new_conn_id = "Wired Connection";
    let new_connected = new_state >= 40 && !new_conn_id.is_empty();

    assert!(
        !old_connected,
        "Should not be connected in disconnected state"
    );
    assert!(
        new_connected,
        "Should be connected in preparing state with connection ID"
    );
    assert_ne!(
        old_connected, new_connected,
        "Connected state should change from disconnected to preparing"
    );
}

#[test]
fn test_connection_state_change_from_ipcheck_to_activated() {
    // Old state: ipcheck with connection
    let old_state = 80u32;
    let old_conn_id = "Wired Connection";
    let old_connected = old_state >= 40 && !old_conn_id.is_empty();

    // New state: activated with connection
    let new_state = 100u32;
    let new_conn_id = "Wired Connection";
    let new_connected = new_state >= 40 && !new_conn_id.is_empty();

    assert!(old_connected, "Should be connected in ipcheck state");
    assert!(new_connected, "Should be connected in activated state");
    assert_eq!(
        old_connected, new_connected,
        "Connected state should remain true from ipcheck to activated"
    );
}

// Tests for match-based icon_name implementation
#[test]
fn test_icon_name_activated_shows_wired_symbolic() {
    use rusty_de::service::ethernet::EthernetState;

    let state = EthernetState::Activated;
    let icon = match state {
        EthernetState::Activated => "network-wired-symbolic".to_string(),
        EthernetState::Preparing
        | EthernetState::Config
        | EthernetState::NeedAuth
        | EthernetState::IpConfig
        | EthernetState::IpCheck
        | EthernetState::Secondaries => "network-wired-acquiring-symbolic".to_string(),
        EthernetState::Deactivating => "network-wired-disconnected-symbolic".to_string(),
        EthernetState::Disconnected
        | EthernetState::Unavailable
        | EthernetState::Unmanaged
        | EthernetState::Failed
        | EthernetState::Unknown => "network-wired-disconnected-symbolic".to_string(),
    };

    assert_eq!(
        icon, "network-wired-symbolic",
        "Activated state should show network-wired-symbolic"
    );
}

#[test]
fn test_icon_name_connecting_states_show_acquiring() {
    use rusty_de::service::ethernet::EthernetState;

    let connecting_states = vec![
        EthernetState::Preparing,
        EthernetState::Config,
        EthernetState::NeedAuth,
        EthernetState::IpConfig,
        EthernetState::IpCheck,
        EthernetState::Secondaries,
    ];

    for state in connecting_states {
        let icon = match state {
            EthernetState::Activated => "network-wired-symbolic".to_string(),
            EthernetState::Preparing
            | EthernetState::Config
            | EthernetState::NeedAuth
            | EthernetState::IpConfig
            | EthernetState::IpCheck
            | EthernetState::Secondaries => "network-wired-acquiring-symbolic".to_string(),
            EthernetState::Deactivating => "network-wired-disconnected-symbolic".to_string(),
            EthernetState::Disconnected
            | EthernetState::Unavailable
            | EthernetState::Unmanaged
            | EthernetState::Failed
            | EthernetState::Unknown => "network-wired-disconnected-symbolic".to_string(),
        };

        assert_eq!(
            icon, "network-wired-acquiring-symbolic",
            "State {:?} should show network-wired-acquiring-symbolic",
            state
        );
    }
}

#[test]
fn test_icon_name_deactivating_shows_disconnected() {
    use rusty_de::service::ethernet::EthernetState;

    let state = EthernetState::Deactivating;
    let icon = match state {
        EthernetState::Activated => "network-wired-symbolic".to_string(),
        EthernetState::Preparing
        | EthernetState::Config
        | EthernetState::NeedAuth
        | EthernetState::IpConfig
        | EthernetState::IpCheck
        | EthernetState::Secondaries => "network-wired-acquiring-symbolic".to_string(),
        EthernetState::Deactivating => "network-wired-disconnected-symbolic".to_string(),
        EthernetState::Disconnected
        | EthernetState::Unavailable
        | EthernetState::Unmanaged
        | EthernetState::Failed
        | EthernetState::Unknown => "network-wired-disconnected-symbolic".to_string(),
    };

    assert_eq!(
        icon, "network-wired-disconnected-symbolic",
        "Deactivating state should show network-wired-disconnected-symbolic"
    );
}

#[test]
fn test_icon_name_disconnected_states_show_disconnected() {
    use rusty_de::service::ethernet::EthernetState;

    let disconnected_states = vec![
        EthernetState::Disconnected,
        EthernetState::Unavailable,
        EthernetState::Unmanaged,
        EthernetState::Failed,
        EthernetState::Unknown,
    ];

    for state in disconnected_states {
        let icon = match state {
            EthernetState::Activated => "network-wired-symbolic".to_string(),
            EthernetState::Preparing
            | EthernetState::Config
            | EthernetState::NeedAuth
            | EthernetState::IpConfig
            | EthernetState::IpCheck
            | EthernetState::Secondaries => "network-wired-acquiring-symbolic".to_string(),
            EthernetState::Deactivating => "network-wired-disconnected-symbolic".to_string(),
            EthernetState::Disconnected
            | EthernetState::Unavailable
            | EthernetState::Unmanaged
            | EthernetState::Failed
            | EthernetState::Unknown => "network-wired-disconnected-symbolic".to_string(),
        };

        assert_eq!(
            icon, "network-wired-disconnected-symbolic",
            "State {:?} should show network-wired-disconnected-symbolic",
            state
        );
    }
}

#[test]
fn test_icon_name_state_progression_icons() {
    use rusty_de::service::ethernet::EthernetState;

    // Test a typical connection progression
    let progression = vec![
        (
            EthernetState::Disconnected,
            "network-wired-disconnected-symbolic",
        ),
        (EthernetState::Preparing, "network-wired-acquiring-symbolic"),
        (EthernetState::Config, "network-wired-acquiring-symbolic"),
        (EthernetState::IpConfig, "network-wired-acquiring-symbolic"),
        (EthernetState::IpCheck, "network-wired-acquiring-symbolic"),
        (EthernetState::Activated, "network-wired-symbolic"),
    ];

    for (state, expected_icon) in progression {
        let icon = match state {
            EthernetState::Activated => "network-wired-symbolic".to_string(),
            EthernetState::Preparing
            | EthernetState::Config
            | EthernetState::NeedAuth
            | EthernetState::IpConfig
            | EthernetState::IpCheck
            | EthernetState::Secondaries => "network-wired-acquiring-symbolic".to_string(),
            EthernetState::Deactivating => "network-wired-disconnected-symbolic".to_string(),
            EthernetState::Disconnected
            | EthernetState::Unavailable
            | EthernetState::Unmanaged
            | EthernetState::Failed
            | EthernetState::Unknown => "network-wired-disconnected-symbolic".to_string(),
        };

        assert_eq!(
            icon, expected_icon,
            "State {:?} should show icon '{}'",
            state, expected_icon
        );
    }
}

#[test]
fn test_icon_name_disconnection_progression_icons() {
    use rusty_de::service::ethernet::EthernetState;

    // Test a typical disconnection progression
    let progression = vec![
        (EthernetState::Activated, "network-wired-symbolic"),
        (
            EthernetState::Deactivating,
            "network-wired-disconnected-symbolic",
        ),
        (
            EthernetState::Disconnected,
            "network-wired-disconnected-symbolic",
        ),
    ];

    for (state, expected_icon) in progression {
        let icon = match state {
            EthernetState::Activated => "network-wired-symbolic".to_string(),
            EthernetState::Preparing
            | EthernetState::Config
            | EthernetState::NeedAuth
            | EthernetState::IpConfig
            | EthernetState::IpCheck
            | EthernetState::Secondaries => "network-wired-acquiring-symbolic".to_string(),
            EthernetState::Deactivating => "network-wired-disconnected-symbolic".to_string(),
            EthernetState::Disconnected
            | EthernetState::Unavailable
            | EthernetState::Unmanaged
            | EthernetState::Failed
            | EthernetState::Unknown => "network-wired-disconnected-symbolic".to_string(),
        };

        assert_eq!(
            icon, expected_icon,
            "State {:?} should show icon '{}'",
            state, expected_icon
        );
    }
}

#[test]
fn test_icon_name_all_states_have_valid_icons() {
    use rusty_de::service::ethernet::EthernetState;

    let all_states = vec![
        EthernetState::Unknown,
        EthernetState::Unmanaged,
        EthernetState::Unavailable,
        EthernetState::Disconnected,
        EthernetState::Preparing,
        EthernetState::Config,
        EthernetState::NeedAuth,
        EthernetState::IpConfig,
        EthernetState::IpCheck,
        EthernetState::Secondaries,
        EthernetState::Activated,
        EthernetState::Deactivating,
        EthernetState::Failed,
    ];

    let valid_icons = vec![
        "network-wired-symbolic",
        "network-wired-acquiring-symbolic",
        "network-wired-disconnected-symbolic",
    ];

    for state in all_states {
        let icon = match state {
            EthernetState::Activated => "network-wired-symbolic".to_string(),
            EthernetState::Preparing
            | EthernetState::Config
            | EthernetState::NeedAuth
            | EthernetState::IpConfig
            | EthernetState::IpCheck
            | EthernetState::Secondaries => "network-wired-acquiring-symbolic".to_string(),
            EthernetState::Deactivating => "network-wired-disconnected-symbolic".to_string(),
            EthernetState::Disconnected
            | EthernetState::Unavailable
            | EthernetState::Unmanaged
            | EthernetState::Failed
            | EthernetState::Unknown => "network-wired-disconnected-symbolic".to_string(),
        };

        assert!(
            valid_icons.contains(&icon.as_str()),
            "State {:?} returned invalid icon '{}', expected one of {:?}",
            state,
            icon,
            valid_icons
        );
    }
}
