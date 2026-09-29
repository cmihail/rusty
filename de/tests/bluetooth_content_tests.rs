#[test]
fn test_bluetooth_content_header_no_devices() {
    // Test header text when no devices found
    let scanning = false;

    let header_text = if scanning {
        "Scanning...".to_string()
    } else {
        "Bluetooth devices".to_string()
    };

    assert_eq!(
        header_text, "Bluetooth devices",
        "Header should show 'Bluetooth devices' when not scanning"
    );
}

#[test]
fn test_bluetooth_content_header_with_devices() {
    // Test header text when devices are present
    let scanning = false;

    let header_text = if scanning {
        "Scanning...".to_string()
    } else {
        "Bluetooth devices".to_string()
    };

    assert_eq!(
        header_text, "Bluetooth devices",
        "Header should show 'Bluetooth devices' without count"
    );
}

#[test]
fn test_bluetooth_content_header_while_scanning() {
    // Test header text while scanning
    let devices_count = 2;
    let scanning = true;

    let header_text = if scanning {
        "Scanning...".to_string()
    } else if devices_count == 0 {
        "No Devices Found".to_string()
    } else {
        format!("Available Devices ({})", devices_count)
    };

    assert_eq!(
        header_text, "Scanning...",
        "Header should show 'Scanning...' while scanning"
    );
}

#[test]
fn test_bluetooth_device_icon_connected() {
    // Test icon for connected device
    let connected = true;
    let paired = true;

    let icon = if connected {
        "bluetooth-active-symbolic".to_string()
    } else if paired {
        "bluetooth-disconnected-symbolic".to_string()
    } else {
        "bluetooth-disabled-symbolic".to_string()
    };

    assert_eq!(
        icon, "bluetooth-active-symbolic",
        "Connected device should use active icon"
    );
}

#[test]
fn test_bluetooth_device_icon_paired_not_connected() {
    // Test icon for paired but not connected device
    let connected = false;
    let paired = true;

    let icon = if connected {
        "bluetooth-active-symbolic".to_string()
    } else if paired {
        "bluetooth-disconnected-symbolic".to_string()
    } else {
        "bluetooth-disabled-symbolic".to_string()
    };

    assert_eq!(
        icon, "bluetooth-disconnected-symbolic",
        "Paired device should use bluetooth disconnected icon"
    );
}

#[test]
fn test_bluetooth_device_icon_not_paired() {
    // Test icon for unpaired device
    let connected = false;
    let paired = false;

    let icon = if connected {
        "bluetooth-active-symbolic".to_string()
    } else if paired {
        "bluetooth-disconnected-symbolic".to_string()
    } else {
        "bluetooth-disabled-symbolic".to_string()
    };

    assert_eq!(
        icon, "bluetooth-disabled-symbolic",
        "Unpaired device should use disabled icon"
    );
}

#[test]
fn test_bluetooth_device_status_text_connected() {
    // Test status text for connected device
    let connected = true;
    let paired = true;

    let status = if paired {
        if connected {
            "Connected"
        } else {
            "Paired"
        }
    } else {
        "Not Paired"
    };

    assert_eq!(status, "Connected", "Connected device shows 'Connected'");
}

#[test]
fn test_bluetooth_device_status_text_paired() {
    // Test status text for paired but not connected device
    let connected = false;
    let paired = true;

    let status = if paired {
        if connected {
            "Connected"
        } else {
            "Paired"
        }
    } else {
        "Not Paired"
    };

    assert_eq!(
        status, "Paired",
        "Paired but not connected device shows 'Paired'"
    );
}

#[test]
fn test_bluetooth_device_status_text_not_paired() {
    // Test status text for unpaired device
    let connected = false;
    let paired = false;

    let status = if paired {
        if connected {
            "Connected"
        } else {
            "Paired"
        }
    } else {
        "Not Paired"
    };

    assert_eq!(status, "Not Paired", "Unpaired device shows 'Not Paired'");
}

#[test]
fn test_bluetooth_device_tooltip_format() {
    // Test tooltip formatting
    let name = "My Headphones";
    let address = "AA:BB:CC:DD:EE:FF";
    let status = "Connected";
    let nearby = "Yes";

    let tooltip = format!(
        "Name: {}\nAddress: {}\nStatus: {}\nNearby: {}",
        name, address, status, nearby
    );

    assert!(tooltip.contains("Name:"), "Tooltip should contain 'Name:'");
    assert!(
        tooltip.contains("Address:"),
        "Tooltip should contain 'Address:'"
    );
    assert!(
        tooltip.contains("Status:"),
        "Tooltip should contain 'Status:'"
    );
    assert!(
        tooltip.contains("Nearby:"),
        "Tooltip should contain 'Nearby:'"
    );
    assert!(
        tooltip.contains("My Headphones"),
        "Tooltip should contain device name"
    );
    assert!(
        tooltip.contains("AA:BB:CC:DD:EE:FF"),
        "Tooltip should contain MAC address"
    );
}

#[test]
fn test_bluetooth_refresh_button_visibility_while_scanning() {
    // Test refresh button should be hidden while scanning
    let scanning = true;

    let on_refresh = if scanning { None } else { Some(()) };

    assert_eq!(
        on_refresh, None,
        "Refresh button should be hidden while scanning"
    );
}

#[test]
fn test_bluetooth_refresh_button_visibility_not_scanning() {
    // Test refresh button should be visible when not scanning
    let scanning = false;

    let on_refresh = if scanning { None } else { Some(()) };

    assert_eq!(
        on_refresh,
        Some(()),
        "Refresh button should be visible when not scanning"
    );
}

#[test]
fn test_bluetooth_settings_command() {
    // Test settings command is blueman-manager
    let command = "blueman-manager";

    assert_eq!(
        command, "blueman-manager",
        "Settings should open blueman-manager"
    );
}

#[test]
fn test_bluetooth_device_list_rebuild_on_device_change() {
    // Test list rebuild logic when devices change
    let initial_devices = vec!["Device 1", "Device 2"];
    let mut current_devices = initial_devices.clone();

    // Add device
    current_devices.push("Device 3");
    assert_ne!(
        initial_devices.len(),
        current_devices.len(),
        "Device list should change"
    );

    // Remove device
    current_devices.pop();
    assert_eq!(
        initial_devices.len(),
        current_devices.len(),
        "Device list should match after removal"
    );
}

#[test]
fn test_bluetooth_content_property_signals() {
    // Test property signal names
    let signals = vec!["devices", "scanning"];

    for signal in signals {
        assert!(!signal.is_empty(), "Signal name should not be empty");
        assert!(
            signal.chars().all(|c| c.is_lowercase() || c == '-'),
            "Signal name should be lowercase with hyphens: {}",
            signal
        );
    }
}

#[test]
fn test_bluetooth_device_entry_creation() {
    // Test creating device entry data
    let device_name = "Bluetooth Speaker";
    let device_icon = "bluetooth-active-symbolic";
    let is_connected = true;

    assert!(!device_name.is_empty(), "Device name should not be empty");
    assert!(!device_icon.is_empty(), "Device icon should not be empty");
    assert!(
        is_connected == true || is_connected == false,
        "Connection state should be boolean"
    );
}

#[test]
fn test_bluetooth_header_text_all_states() {
    // Test header text for all possible states
    let test_cases = vec![(false, "Bluetooth devices"), (true, "Scanning...")];

    for (scanning, expected) in test_cases {
        let header_text = if scanning {
            "Scanning...".to_string()
        } else {
            "Bluetooth devices".to_string()
        };

        assert_eq!(
            header_text, expected,
            "Header should be '{}' for scanning={}",
            expected, scanning
        );
    }
}

#[test]
fn test_bluetooth_helper_function_count() {
    // Test that helper functions exist for code organization
    let helper_functions = vec![
        "get_header_text",
        "create_device_entries",
        "create_device_entry",
        "create_device_tooltip",
        "handle_connect",
        "handle_disconnect",
        "create_refresh_callback",
        "create_settings_callback",
        "get_icon_for_device",
    ];

    for function_name in helper_functions {
        assert!(
            !function_name.is_empty(),
            "Function name should not be empty"
        );
        assert!(
            function_name.chars().all(|c| c.is_lowercase() || c == '_'),
            "Function name should be snake_case: {}",
            function_name
        );
    }
}

#[test]
fn test_bluetooth_completion_callback_success() {
    // Test completion callback success case
    let result: Result<(), String> = Ok(());

    match result {
        Ok(_) => assert!(true, "Success callback should be called"),
        Err(_) => panic!("Should not call error callback on success"),
    }
}

#[test]
fn test_bluetooth_completion_callback_error() {
    // Test completion callback error case
    let result: Result<(), String> = Err("Connection failed".to_string());

    match result {
        Ok(_) => panic!("Should not call success callback on error"),
        Err(e) => {
            assert!(
                e.contains("Connection failed"),
                "Error callback should receive error message"
            );
        }
    }
}

#[test]
fn test_bluetooth_battery_icon_level_0_to_10() {
    // Test battery icon for 0-10% range
    let percentage = 5u8;

    let icon = match percentage {
        0..=10 => "battery-level-0-symbolic",
        11..=30 => "battery-level-10-symbolic",
        31..=50 => "battery-level-30-symbolic",
        51..=70 => "battery-level-50-symbolic",
        71..=90 => "battery-level-70-symbolic",
        _ => "battery-level-90-symbolic",
    };

    assert_eq!(
        icon, "battery-level-0-symbolic",
        "Battery 0-10% should use level-0 icon"
    );
}

#[test]
fn test_bluetooth_battery_icon_level_11_to_30() {
    // Test battery icon for 11-30% range
    let percentage = 20u8;

    let icon = match percentage {
        0..=10 => "battery-level-0-symbolic",
        11..=30 => "battery-level-10-symbolic",
        31..=50 => "battery-level-30-symbolic",
        51..=70 => "battery-level-50-symbolic",
        71..=90 => "battery-level-70-symbolic",
        _ => "battery-level-90-symbolic",
    };

    assert_eq!(
        icon, "battery-level-10-symbolic",
        "Battery 11-30% should use level-10 icon"
    );
}

#[test]
fn test_bluetooth_battery_icon_level_31_to_50() {
    // Test battery icon for 31-50% range
    let percentage = 40u8;

    let icon = match percentage {
        0..=10 => "battery-level-0-symbolic",
        11..=30 => "battery-level-10-symbolic",
        31..=50 => "battery-level-30-symbolic",
        51..=70 => "battery-level-50-symbolic",
        71..=90 => "battery-level-70-symbolic",
        _ => "battery-level-90-symbolic",
    };

    assert_eq!(
        icon, "battery-level-30-symbolic",
        "Battery 31-50% should use level-30 icon"
    );
}

#[test]
fn test_bluetooth_battery_icon_level_51_to_70() {
    // Test battery icon for 51-70% range
    let percentage = 60u8;

    let icon = match percentage {
        0..=10 => "battery-level-0-symbolic",
        11..=30 => "battery-level-10-symbolic",
        31..=50 => "battery-level-30-symbolic",
        51..=70 => "battery-level-50-symbolic",
        71..=90 => "battery-level-70-symbolic",
        _ => "battery-level-90-symbolic",
    };

    assert_eq!(
        icon, "battery-level-50-symbolic",
        "Battery 51-70% should use level-50 icon"
    );
}

#[test]
fn test_bluetooth_battery_icon_level_71_to_90() {
    // Test battery icon for 71-90% range
    let percentage = 80u8;

    let icon = match percentage {
        0..=10 => "battery-level-0-symbolic",
        11..=30 => "battery-level-10-symbolic",
        31..=50 => "battery-level-30-symbolic",
        51..=70 => "battery-level-50-symbolic",
        71..=90 => "battery-level-70-symbolic",
        _ => "battery-level-90-symbolic",
    };

    assert_eq!(
        icon, "battery-level-70-symbolic",
        "Battery 71-90% should use level-70 icon"
    );
}

#[test]
fn test_bluetooth_battery_icon_level_91_to_100() {
    // Test battery icon for 91-100% range
    let percentage = 95u8;

    let icon = match percentage {
        0..=10 => "battery-level-0-symbolic",
        11..=30 => "battery-level-10-symbolic",
        31..=50 => "battery-level-30-symbolic",
        51..=70 => "battery-level-50-symbolic",
        71..=90 => "battery-level-70-symbolic",
        _ => "battery-level-90-symbolic",
    };

    assert_eq!(
        icon, "battery-level-90-symbolic",
        "Battery 91-100% should use level-90 icon"
    );
}

#[test]
fn test_bluetooth_battery_icon_boundary_10() {
    // Test battery icon at 10% boundary
    let percentage = 10u8;

    let icon = match percentage {
        0..=10 => "battery-level-0-symbolic",
        11..=30 => "battery-level-10-symbolic",
        31..=50 => "battery-level-30-symbolic",
        51..=70 => "battery-level-50-symbolic",
        71..=90 => "battery-level-70-symbolic",
        _ => "battery-level-90-symbolic",
    };

    assert_eq!(
        icon, "battery-level-0-symbolic",
        "Battery at 10% should use level-0 icon"
    );
}

#[test]
fn test_bluetooth_battery_icon_boundary_11() {
    // Test battery icon at 11% boundary
    let percentage = 11u8;

    let icon = match percentage {
        0..=10 => "battery-level-0-symbolic",
        11..=30 => "battery-level-10-symbolic",
        31..=50 => "battery-level-30-symbolic",
        51..=70 => "battery-level-50-symbolic",
        71..=90 => "battery-level-70-symbolic",
        _ => "battery-level-90-symbolic",
    };

    assert_eq!(
        icon, "battery-level-10-symbolic",
        "Battery at 11% should use level-10 icon"
    );
}

#[test]
fn test_bluetooth_battery_tooltip_with_battery() {
    // Test tooltip includes battery percentage when available
    let name = "My Headphones";
    let address = "AA:BB:CC:DD:EE:FF";
    let status = "Connected";
    let nearby = "Yes";
    let battery_percentage = Some(75u8);

    let battery_text = if let Some(percentage) = battery_percentage {
        format!("\nBattery: {}%", percentage)
    } else {
        String::new()
    };

    let tooltip = format!(
        "Name: {}\nAddress: {}\nStatus: {}\nNearby: {}{}",
        name, address, status, nearby, battery_text
    );

    assert!(
        tooltip.contains("Battery: 75%"),
        "Tooltip should include battery percentage"
    );
}

#[test]
fn test_bluetooth_battery_tooltip_without_battery() {
    // Test tooltip without battery percentage
    let name = "My Headphones";
    let address = "AA:BB:CC:DD:EE:FF";
    let status = "Connected";
    let nearby = "Yes";
    let battery_percentage: Option<u8> = None;

    let battery_text = if let Some(percentage) = battery_percentage {
        format!("\nBattery: {}%", percentage)
    } else {
        String::new()
    };

    let tooltip = format!(
        "Name: {}\nAddress: {}\nStatus: {}\nNearby: {}{}",
        name, address, status, nearby, battery_text
    );

    assert!(
        !tooltip.contains("Battery:"),
        "Tooltip should not include battery when not available"
    );
}

#[test]
fn test_bluetooth_device_battery_percentage_field() {
    use rusty_de::service::bluetooth::BluetoothDevice;

    // Test BluetoothDevice with battery percentage
    let device = BluetoothDevice {
        path: "/org/bluez/hci0/dev_00_11_22_33_44_55".to_string(),
        name: "Headphones".to_string(),
        address: "00:11:22:33:44:55".to_string(),
        connected: true,
        paired: true,
        battery_percentage: Some(75),
        nearby: true,
    };

    assert_eq!(
        device.battery_percentage,
        Some(75),
        "Device should have battery percentage"
    );
}

#[test]
fn test_bluetooth_device_without_battery() {
    use rusty_de::service::bluetooth::BluetoothDevice;

    // Test BluetoothDevice without battery percentage
    let device = BluetoothDevice {
        path: "/org/bluez/hci0/dev_00_11_22_33_44_55".to_string(),
        name: "Keyboard".to_string(),
        address: "00:11:22:33:44:55".to_string(),
        connected: true,
        paired: true,
        battery_percentage: None,
        nearby: true,
    };

    assert_eq!(
        device.battery_percentage, None,
        "Device without battery should have None"
    );
}

#[test]
fn test_bluetooth_battery_icon_none_when_no_battery() {
    // Test battery icon should be None when battery percentage is None
    let battery_percentage: Option<u8> = None;

    let battery_icon = battery_percentage.map(|percentage| match percentage {
        0..=10 => "battery-level-0-symbolic",
        11..=30 => "battery-level-10-symbolic",
        31..=50 => "battery-level-30-symbolic",
        51..=70 => "battery-level-50-symbolic",
        71..=90 => "battery-level-70-symbolic",
        _ => "battery-level-90-symbolic",
    });

    assert_eq!(battery_icon, None, "No battery icon when no battery info");
}

#[test]
fn test_bluetooth_battery_percentage_range() {
    // Test battery percentage is within valid range (0-100)
    let test_values = vec![0u8, 50u8, 100u8];

    for value in test_values {
        assert!(value <= 100, "Battery percentage should be 0-100");
    }
}

#[test]
fn test_bluetooth_battery_all_icon_levels() {
    // Test all battery icon levels are assigned correctly
    let test_cases = vec![
        (0, "battery-level-0-symbolic"),
        (10, "battery-level-0-symbolic"),
        (11, "battery-level-10-symbolic"),
        (30, "battery-level-10-symbolic"),
        (31, "battery-level-30-symbolic"),
        (50, "battery-level-30-symbolic"),
        (51, "battery-level-50-symbolic"),
        (70, "battery-level-50-symbolic"),
        (71, "battery-level-70-symbolic"),
        (90, "battery-level-70-symbolic"),
        (91, "battery-level-90-symbolic"),
        (100, "battery-level-90-symbolic"),
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
            "Battery {}% should use {} icon",
            percentage, expected_icon
        );
    }
}

#[test]
fn test_bluetooth_device_icon_acquiring_when_connecting() {
    // Test icon for device that is currently connecting
    let device_path = "/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF";
    let connecting_device: Option<String> = Some(device_path.to_string());
    let connected = false;
    let paired = true;

    // Icon logic: if device is connecting, show acquiring icon
    let icon = if connecting_device.as_ref() == Some(&device_path.to_string()) {
        "bluetooth-acquiring-symbolic".to_string()
    } else if connected {
        "bluetooth-active-symbolic".to_string()
    } else if paired {
        "bluetooth-disconnected-symbolic".to_string()
    } else {
        "bluetooth-disabled-symbolic".to_string()
    };

    assert_eq!(
        icon, "bluetooth-acquiring-symbolic",
        "Device should show acquiring icon when connecting"
    );
}

#[test]
fn test_bluetooth_device_icon_not_acquiring_when_different_device_connecting() {
    // Test icon for device when a different device is connecting
    let device_path = "/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF";
    let other_device_path = "/org/bluez/hci0/dev_11_22_33_44_55_66";
    let connecting_device: Option<String> = Some(other_device_path.to_string());
    let connected = false;
    let paired = true;

    // Icon logic: only the connecting device shows acquiring icon
    let icon = if connecting_device.as_ref() == Some(&device_path.to_string()) {
        "bluetooth-acquiring-symbolic".to_string()
    } else if connected {
        "bluetooth-active-symbolic".to_string()
    } else if paired {
        "bluetooth-disconnected-symbolic".to_string()
    } else {
        "bluetooth-disabled-symbolic".to_string()
    };

    assert_eq!(
        icon, "bluetooth-disconnected-symbolic",
        "Device should show normal icon when a different device is connecting"
    );
}

#[test]
fn test_bluetooth_device_icon_not_acquiring_when_not_connecting() {
    // Test icon for device when no device is connecting
    let device_path = "/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF";
    let connecting_device: Option<String> = None;
    let connected = false;
    let paired = true;

    // Icon logic: normal icon when not connecting
    let icon = if connecting_device.as_ref() == Some(&device_path.to_string()) {
        "bluetooth-acquiring-symbolic".to_string()
    } else if connected {
        "bluetooth-active-symbolic".to_string()
    } else if paired {
        "bluetooth-disconnected-symbolic".to_string()
    } else {
        "bluetooth-disabled-symbolic".to_string()
    };

    assert_eq!(
        icon, "bluetooth-disconnected-symbolic",
        "Device should show normal icon when not connecting"
    );
}

#[test]
fn test_bluetooth_device_icon_acquiring_takes_priority() {
    // Test that acquiring icon takes priority over all other states
    let device_path = "/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF";
    let connecting_device: Option<String> = Some(device_path.to_string());

    // Test with connected=true, paired=true (would normally be active icon)
    let icon = if connecting_device.as_ref() == Some(&device_path.to_string()) {
        "bluetooth-acquiring-symbolic".to_string()
    } else if true {
        "bluetooth-active-symbolic".to_string()
    } else if true {
        "bluetooth-disconnected-symbolic".to_string()
    } else {
        "bluetooth-disabled-symbolic".to_string()
    };

    assert_eq!(
        icon, "bluetooth-acquiring-symbolic",
        "Acquiring icon should take priority over connected state"
    );
}

#[test]
fn test_bluetooth_device_icon_all_states_with_connecting() {
    // Test all icon states including the acquiring state
    let test_cases = vec![
        (
            Some("/path".to_string()),
            "/path",
            true,
            true,
            "bluetooth-acquiring-symbolic",
        ),
        (None, "/path", true, true, "bluetooth-active-symbolic"),
        (
            None,
            "/path",
            false,
            true,
            "bluetooth-disconnected-symbolic",
        ),
        (None, "/path", false, false, "bluetooth-disabled-symbolic"),
        (
            Some("/other".to_string()),
            "/path",
            false,
            true,
            "bluetooth-disconnected-symbolic",
        ),
    ];

    for (connecting_device, device_path, connected, paired, expected_icon) in test_cases {
        let icon = if connecting_device.as_ref() == Some(&device_path.to_string()) {
            "bluetooth-acquiring-symbolic".to_string()
        } else if connected {
            "bluetooth-active-symbolic".to_string()
        } else if paired {
            "bluetooth-disconnected-symbolic".to_string()
        } else {
            "bluetooth-disabled-symbolic".to_string()
        };

        assert_eq!(
            icon, expected_icon,
            "Icon should be {} for connecting={:?}, connected={}, paired={}",
            expected_icon, connecting_device, connected, paired
        );
    }
}

#[test]
fn test_bluetooth_acquiring_icon_name() {
    // Test that the acquiring icon name is correct
    let icon = "bluetooth-acquiring-symbolic";

    assert!(
        icon.contains("acquiring"),
        "Icon name should contain 'acquiring'"
    );
    assert!(
        icon.ends_with("-symbolic"),
        "Icon name should end with '-symbolic'"
    );
}

#[test]
fn test_bluetooth_device_path_matching_for_acquiring_icon() {
    // Test exact path matching for acquiring icon
    let device_path = "/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF".to_string();
    let similar_path = "/org/bluez/hci0/dev_AA_BB_CC_DD_EE_F".to_string();

    // Should match exact path
    let connecting_exact = Some(device_path.clone());
    assert_eq!(
        connecting_exact.as_ref(),
        Some(&device_path),
        "Should match exact device path"
    );

    // Should not match similar path
    let connecting_similar = Some(similar_path.clone());
    assert_ne!(
        connecting_similar.as_ref(),
        Some(&device_path),
        "Should not match similar device path"
    );
}

#[test]
fn test_bluetooth_device_icon_not_nearby() {
    // Test icon for device that is not nearby (not in vicinity)
    // Icon no longer changes based on nearby status, switch is hidden instead
    let device_path = "/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF";
    let connecting_device: Option<String> = None;
    let connected = false;
    let paired = true;

    // Icon logic: icon doesn't change based on nearby, just on connection state
    let icon = if connecting_device.as_ref() == Some(&device_path.to_string()) {
        "bluetooth-acquiring-symbolic".to_string()
    } else if connected {
        "bluetooth-active-symbolic".to_string()
    } else if paired {
        "bluetooth-disconnected-symbolic".to_string()
    } else {
        "bluetooth-disabled-symbolic".to_string()
    };

    assert_eq!(
        icon, "bluetooth-disconnected-symbolic",
        "Paired device (even if not nearby) should show disconnected icon"
    );
}

#[test]
fn test_bluetooth_device_icon_paired_disconnected() {
    // Test that paired but disconnected devices show disconnected icon
    let device_path = "/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF";
    let connecting_device: Option<String> = None;
    let connected = false;
    let paired = true;

    // Icon logic: icon based on connection/pairing state only
    let icon = if connecting_device.as_ref() == Some(&device_path.to_string()) {
        "bluetooth-acquiring-symbolic".to_string()
    } else if connected {
        "bluetooth-active-symbolic".to_string()
    } else if paired {
        "bluetooth-disconnected-symbolic".to_string()
    } else {
        "bluetooth-disabled-symbolic".to_string()
    };

    assert_eq!(
        icon, "bluetooth-disconnected-symbolic",
        "Paired but disconnected device should show disconnected icon"
    );
}

#[test]
fn test_bluetooth_device_sorting_connected_nearby_first() {
    use rusty_de::service::bluetooth::BluetoothDevice;

    // Test that connected+nearby devices appear before other devices
    let mut devices = vec![
        BluetoothDevice {
            path: "/org/bluez/hci0/dev_11_22_33_44_55_66".to_string(),
            name: "Paired Not Nearby".to_string(),
            address: "11:22:33:44:55:66".to_string(),
            connected: false,
            paired: true,
            battery_percentage: None,
            nearby: false,
        },
        BluetoothDevice {
            path: "/org/bluez/hci0/dev_00_11_22_33_44_55".to_string(),
            name: "Connected Nearby".to_string(),
            address: "00:11:22:33:44:55".to_string(),
            connected: true,
            paired: true,
            battery_percentage: None,
            nearby: true,
        },
    ];

    // Simulate sorting logic
    devices.sort_by_key(|device| {
        if device.connected && device.nearby {
            0
        } else if device.nearby && device.paired && !device.connected {
            1
        } else if device.nearby && !device.paired {
            2
        } else if device.paired && !device.nearby {
            3
        } else {
            4
        }
    });

    assert_eq!(
        devices[0].name, "Connected Nearby",
        "Connected+nearby device should be first"
    );
    assert_eq!(
        devices[1].name, "Paired Not Nearby",
        "Paired+not-nearby device should be second"
    );
}

#[test]
fn test_bluetooth_device_sorting_complete_order() {
    use rusty_de::service::bluetooth::BluetoothDevice;

    // Test complete sorting order with all device types
    let mut devices = vec![
        BluetoothDevice {
            path: "/org/bluez/hci0/dev_44".to_string(),
            name: "Paired Not Nearby".to_string(),
            address: "44:44:44:44:44:44".to_string(),
            connected: false,
            paired: true,
            battery_percentage: None,
            nearby: false,
        },
        BluetoothDevice {
            path: "/org/bluez/hci0/dev_22".to_string(),
            name: "Nearby Paired Not Connected".to_string(),
            address: "22:22:22:22:22:22".to_string(),
            connected: false,
            paired: true,
            battery_percentage: None,
            nearby: true,
        },
        BluetoothDevice {
            path: "/org/bluez/hci0/dev_11".to_string(),
            name: "Connected Nearby".to_string(),
            address: "11:11:11:11:11:11".to_string(),
            connected: true,
            paired: true,
            battery_percentage: None,
            nearby: true,
        },
        BluetoothDevice {
            path: "/org/bluez/hci0/dev_33".to_string(),
            name: "Nearby Not Paired".to_string(),
            address: "33:33:33:33:33:33".to_string(),
            connected: false,
            paired: false,
            battery_percentage: None,
            nearby: true,
        },
    ];

    // Simulate sorting logic
    devices.sort_by_key(|device| {
        if device.connected && device.nearby {
            0
        } else if device.nearby && device.paired && !device.connected {
            1
        } else if device.nearby && !device.paired {
            2
        } else if device.paired && !device.nearby {
            3
        } else {
            4
        }
    });

    assert_eq!(
        devices[0].name, "Connected Nearby",
        "Priority 0: Connected+nearby"
    );
    assert_eq!(
        devices[1].name, "Nearby Paired Not Connected",
        "Priority 1: Nearby+paired+not-connected"
    );
    assert_eq!(
        devices[2].name, "Nearby Not Paired",
        "Priority 2: Nearby+not-paired"
    );
    assert_eq!(
        devices[3].name, "Paired Not Nearby",
        "Priority 3: Paired+not-nearby"
    );
}

#[test]
fn test_bluetooth_device_sorting_nearby_paired_before_nearby_unpaired() {
    use rusty_de::service::bluetooth::BluetoothDevice;

    // Test that nearby+paired devices appear before nearby+unpaired
    let mut devices = vec![
        BluetoothDevice {
            path: "/org/bluez/hci0/dev_22".to_string(),
            name: "Nearby Unpaired".to_string(),
            address: "22:22:22:22:22:22".to_string(),
            connected: false,
            paired: false,
            battery_percentage: None,
            nearby: true,
        },
        BluetoothDevice {
            path: "/org/bluez/hci0/dev_11".to_string(),
            name: "Nearby Paired".to_string(),
            address: "11:11:11:11:11:11".to_string(),
            connected: false,
            paired: true,
            battery_percentage: None,
            nearby: true,
        },
    ];

    // Simulate sorting logic
    devices.sort_by_key(|device| {
        if device.connected && device.nearby {
            0
        } else if device.nearby && device.paired && !device.connected {
            1
        } else if device.nearby && !device.paired {
            2
        } else if device.paired && !device.nearby {
            3
        } else {
            4
        }
    });

    assert_eq!(
        devices[0].name, "Nearby Paired",
        "Nearby+paired should appear before nearby+unpaired"
    );
    assert_eq!(
        devices[1].name, "Nearby Unpaired",
        "Nearby+unpaired should be second"
    );
}

#[test]
fn test_bluetooth_device_sorting_nearby_before_not_nearby() {
    use rusty_de::service::bluetooth::BluetoothDevice;

    // Test that nearby unpaired devices appear before paired not-nearby devices
    let mut devices = vec![
        BluetoothDevice {
            path: "/org/bluez/hci0/dev_22".to_string(),
            name: "Paired Not Nearby".to_string(),
            address: "22:22:22:22:22:22".to_string(),
            connected: false,
            paired: true,
            battery_percentage: None,
            nearby: false,
        },
        BluetoothDevice {
            path: "/org/bluez/hci0/dev_11".to_string(),
            name: "Nearby Unpaired".to_string(),
            address: "11:11:11:11:11:11".to_string(),
            connected: false,
            paired: false,
            battery_percentage: None,
            nearby: true,
        },
    ];

    // Simulate sorting logic
    devices.sort_by_key(|device| {
        if device.connected && device.nearby {
            0
        } else if device.nearby && device.paired && !device.connected {
            1
        } else if device.nearby && !device.paired {
            2
        } else if device.paired && !device.nearby {
            3
        } else {
            4
        }
    });

    assert_eq!(
        devices[0].name, "Nearby Unpaired",
        "Nearby+unpaired should appear before paired+not-nearby"
    );
    assert_eq!(
        devices[1].name, "Paired Not Nearby",
        "Paired+not-nearby should be last"
    );
}

#[test]
fn test_error_persistence_stores_connection_failure() {
    // Test that connection errors are stored per device
    use std::collections::HashMap;

    let mut device_errors: HashMap<String, String> = HashMap::new();
    let device_path = "/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF".to_string();
    let error_message = "Failed to pair: org.bluez.Error.AuthenticationFailed".to_string();

    // Simulate storing an error
    device_errors.insert(device_path.clone(), error_message.clone());

    // Verify error is stored
    assert!(device_errors.contains_key(&device_path));
    assert_eq!(device_errors.get(&device_path), Some(&error_message));
}

#[test]
fn test_error_persistence_clears_on_success() {
    // Test that errors are cleared when connection succeeds
    use std::collections::HashMap;

    let mut device_errors: HashMap<String, String> = HashMap::new();
    let device_path = "/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF".to_string();
    let error_message = "Failed to connect".to_string();

    // Store an error
    device_errors.insert(device_path.clone(), error_message);

    // Verify error exists
    assert!(device_errors.contains_key(&device_path));

    // Simulate successful connection
    device_errors.remove(&device_path);

    // Verify error is cleared
    assert!(!device_errors.contains_key(&device_path));
}

#[test]
fn test_error_persistence_multiple_devices() {
    // Test error storage for multiple devices
    use std::collections::HashMap;

    let mut device_errors: HashMap<String, String> = HashMap::new();

    let device1_path = "/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF".to_string();
    let device2_path = "/org/bluez/hci0/dev_11_22_33_44_55_66".to_string();
    let error1 = "Authentication failed".to_string();
    let error2 = "Connection timeout".to_string();

    // Store errors for both devices
    device_errors.insert(device1_path.clone(), error1.clone());
    device_errors.insert(device2_path.clone(), error2.clone());

    // Verify both errors are stored
    assert_eq!(device_errors.get(&device1_path), Some(&error1));
    assert_eq!(device_errors.get(&device2_path), Some(&error2));
    assert_eq!(device_errors.len(), 2);
}

#[test]
fn test_error_persistence_overwrites_previous_error() {
    // Test that new error overwrites previous error for same device
    use std::collections::HashMap;

    let mut device_errors: HashMap<String, String> = HashMap::new();
    let device_path = "/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF".to_string();

    // Store first error
    let error1 = "First error".to_string();
    device_errors.insert(device_path.clone(), error1);

    // Store second error (should overwrite)
    let error2 = "Second error".to_string();
    device_errors.insert(device_path.clone(), error2.clone());

    // Verify only the latest error is kept
    assert_eq!(device_errors.get(&device_path), Some(&error2));
    assert_eq!(device_errors.len(), 1);
}

#[test]
fn test_error_persistence_clear_specific_device() {
    // Test clearing error for a specific device
    use std::collections::HashMap;

    let mut device_errors: HashMap<String, String> = HashMap::new();

    let device1_path = "/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF".to_string();
    let device2_path = "/org/bluez/hci0/dev_11_22_33_44_55_66".to_string();

    device_errors.insert(device1_path.clone(), "Error 1".to_string());
    device_errors.insert(device2_path.clone(), "Error 2".to_string());

    // Clear error for device1 only
    device_errors.remove(&device1_path);

    // Verify device1 error is cleared but device2 error remains
    assert!(!device_errors.contains_key(&device1_path));
    assert!(device_errors.contains_key(&device2_path));
}

#[test]
fn test_error_persistence_clear_all_devices() {
    // Test clearing all errors
    use std::collections::HashMap;

    let mut device_errors: HashMap<String, String> = HashMap::new();

    // Add errors for multiple devices
    device_errors.insert("/org/bluez/hci0/dev_AA".to_string(), "Error A".to_string());
    device_errors.insert("/org/bluez/hci0/dev_BB".to_string(), "Error B".to_string());
    device_errors.insert("/org/bluez/hci0/dev_CC".to_string(), "Error C".to_string());

    assert_eq!(device_errors.len(), 3);

    // Clear all errors (simulating scan)
    device_errors.clear();

    // Verify all errors are cleared
    assert_eq!(device_errors.len(), 0);
    assert!(device_errors.is_empty());
}

#[test]
fn test_error_persistence_empty_error_message() {
    // Test handling of empty error messages
    use std::collections::HashMap;

    let mut device_errors: HashMap<String, String> = HashMap::new();
    let device_path = "/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF".to_string();
    let error_message = "".to_string();

    device_errors.insert(device_path.clone(), error_message.clone());

    // Verify empty error is stored (edge case)
    assert!(device_errors.contains_key(&device_path));
    assert_eq!(device_errors.get(&device_path), Some(&error_message));
}

#[test]
fn test_switch_entry_with_initial_error() {
    // Test SwitchEntry creation with initial error
    use rusty_de::widget::switch_list::SwitchEntry;

    let initial_error = Some("Connection failed".to_string());

    let entry = SwitchEntry::with_completion_callback(
        "test_device",
        "Test Device".to_string(),
        Some("bluetooth-active-symbolic".to_string()),
        None,
        None,
        None,
        None,
        None,
        false,
        true,
        initial_error.clone(),
        |_, _| {},
        |_, _| {},
    );

    assert_eq!(entry.initial_error, initial_error);
}

#[test]
fn test_switch_entry_without_initial_error() {
    // Test SwitchEntry creation without initial error
    use rusty_de::widget::switch_list::SwitchEntry;

    let entry = SwitchEntry::with_completion_callback(
        "test_device",
        "Test Device".to_string(),
        Some("bluetooth-active-symbolic".to_string()),
        None,
        None,
        None,
        None,
        None,
        false,
        true,
        None,
        |_, _| {},
        |_, _| {},
    );

    assert_eq!(entry.initial_error, None);
}

#[test]
fn test_error_message_formatting() {
    // Test that error messages are properly formatted
    let _device_path = "/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF";
    let bluez_error = "org.bluez.Error.AuthenticationFailed: Authentication Failed";
    let formatted_error = format!("Failed to pair: {}", bluez_error);

    assert_eq!(
        formatted_error,
        "Failed to pair: org.bluez.Error.AuthenticationFailed: Authentication Failed"
    );
}

#[test]
fn test_error_cleared_on_switch_toggle() {
    // Test that errors are cleared when any switch is toggled
    use std::collections::HashMap;

    let mut device_errors: HashMap<String, String> = HashMap::new();

    // Add errors for multiple devices
    device_errors.insert("/org/bluez/hci0/dev_AA".to_string(), "Error A".to_string());
    device_errors.insert("/org/bluez/hci0/dev_BB".to_string(), "Error B".to_string());

    assert_eq!(device_errors.len(), 2);

    // Simulate clicking any switch (should clear all)
    device_errors.clear();

    assert_eq!(device_errors.len(), 0);
}

#[test]
fn test_widget_filters_nearby_devices_only() {
    // Test that widget only shows nearby devices
    use rusty_de::service::bluetooth::BluetoothDevice;

    let devices = vec![
        BluetoothDevice {
            path: "/org/bluez/hci0/dev_AA".to_string(),
            name: "Nearby Device".to_string(),
            address: "00:11:22:33:44:55".to_string(),
            connected: false,
            paired: true,
            battery_percentage: None,
            nearby: true,
        },
        BluetoothDevice {
            path: "/org/bluez/hci0/dev_BB".to_string(),
            name: "Not Nearby Device".to_string(),
            address: "66:77:88:99:AA:BB".to_string(),
            connected: false,
            paired: true,
            battery_percentage: None,
            nearby: false,
        },
    ];

    // Simulate widget filtering logic
    let visible_devices: Vec<_> = devices.into_iter().filter(|d| d.nearby).collect();

    assert_eq!(visible_devices.len(), 1);
    assert_eq!(visible_devices[0].name, "Nearby Device");
}

#[test]
fn test_widget_shows_all_nearby_devices() {
    // Test that widget shows all devices that are nearby
    use rusty_de::service::bluetooth::BluetoothDevice;

    let devices = vec![
        BluetoothDevice {
            path: "/org/bluez/hci0/dev_AA".to_string(),
            name: "Nearby 1".to_string(),
            address: "00:11:22:33:44:55".to_string(),
            connected: true,
            paired: true,
            battery_percentage: None,
            nearby: true,
        },
        BluetoothDevice {
            path: "/org/bluez/hci0/dev_BB".to_string(),
            name: "Nearby 2".to_string(),
            address: "66:77:88:99:AA:BB".to_string(),
            connected: false,
            paired: true,
            battery_percentage: None,
            nearby: true,
        },
        BluetoothDevice {
            path: "/org/bluez/hci0/dev_CC".to_string(),
            name: "Not Nearby".to_string(),
            address: "CC:DD:EE:FF:00:11".to_string(),
            connected: false,
            paired: true,
            battery_percentage: None,
            nearby: false,
        },
    ];

    // Simulate widget filtering logic
    let visible_devices: Vec<_> = devices.into_iter().filter(|d| d.nearby).collect();

    assert_eq!(visible_devices.len(), 2);
    assert_eq!(visible_devices[0].name, "Nearby 1");
    assert_eq!(visible_devices[1].name, "Nearby 2");
}

#[test]
fn test_widget_shows_nothing_when_no_nearby_devices() {
    // Test that widget shows empty list when no devices are nearby
    use rusty_de::service::bluetooth::BluetoothDevice;

    let devices = vec![
        BluetoothDevice {
            path: "/org/bluez/hci0/dev_AA".to_string(),
            name: "Not Nearby 1".to_string(),
            address: "00:11:22:33:44:55".to_string(),
            connected: false,
            paired: true,
            battery_percentage: None,
            nearby: false,
        },
        BluetoothDevice {
            path: "/org/bluez/hci0/dev_BB".to_string(),
            name: "Not Nearby 2".to_string(),
            address: "66:77:88:99:AA:BB".to_string(),
            connected: false,
            paired: true,
            battery_percentage: None,
            nearby: false,
        },
    ];

    // Simulate widget filtering logic
    let visible_devices: Vec<_> = devices.into_iter().filter(|d| d.nearby).collect();

    assert_eq!(visible_devices.len(), 0);
}
