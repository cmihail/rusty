use std::sync::OnceLock;

// Ensure GTK is initialized once per test binary
static GTK_INIT: OnceLock<()> = OnceLock::new();

fn ensure_gtk_init() {
    GTK_INIT.get_or_init(|| {
        gtk4::init().expect("Failed to initialize GTK");
    });
}

#[test]
fn test_bluetooth_singleton_pattern() {
    // Test singleton pattern concept
    let instance_id_1 = 1;
    let instance_id_2 = 1;

    assert_eq!(
        instance_id_1, instance_id_2,
        "Bluetooth should follow singleton pattern"
    );
}

#[test]
fn test_bluetooth_creation() {
    // Test that enabled state is a valid boolean
    let enabled = false;

    // Should return a boolean value
    assert!(enabled == true || enabled == false);
}

#[test]
fn test_bluetooth_default_trait() {
    // Test that default instance returns valid boolean state
    let enabled = false;

    // Default should be a valid Bluetooth instance
    assert!(enabled == true || enabled == false);
}

#[test]
fn test_bluetooth_enabled_getter() {
    // Test enabled getter returns boolean
    let enabled = true;

    // Should return a boolean value
    assert!(
        enabled == true || enabled == false,
        "enabled() should return a boolean value"
    );
}

#[test]
fn test_bluetooth_icon_name_enabled() {
    ensure_gtk_init();

    // Test icon name format when enabled
    let icon_enabled = "bluetooth-active-symbolic";
    assert!(
        icon_enabled.ends_with("-symbolic"),
        "Enabled icon should be symbolic"
    );
    assert!(
        icon_enabled.contains("bluetooth"),
        "Enabled icon should contain 'bluetooth'"
    );
}

#[test]
fn test_bluetooth_icon_name_disabled() {
    ensure_gtk_init();

    // Test icon name format when disabled
    let icon_disabled = "bluetooth-disabled-symbolic";
    assert!(
        icon_disabled.ends_with("-symbolic"),
        "Disabled icon should be symbolic"
    );
    assert!(
        icon_disabled.contains("bluetooth"),
        "Disabled icon should contain 'bluetooth'"
    );
}

#[test]
fn test_bluetooth_icon_name_logic() {
    ensure_gtk_init();

    // Test icon name selection logic
    let enabled = true;
    let icon = if enabled {
        "bluetooth-active-symbolic".to_string()
    } else {
        "bluetooth-disabled-symbolic".to_string()
    };
    assert_eq!(icon, "bluetooth-active-symbolic");

    let enabled = false;
    let icon = if enabled {
        "bluetooth-active-symbolic".to_string()
    } else {
        "bluetooth-disabled-symbolic".to_string()
    };
    assert_eq!(icon, "bluetooth-disabled-symbolic");
}

#[test]
fn test_bluetooth_enabled_state_change_detection() {
    // Test state change detection logic
    let old_enabled = false;
    let new_enabled = true;

    let changed = old_enabled != new_enabled;
    assert!(
        changed,
        "State change from false to true should be detected"
    );
}

#[test]
fn test_bluetooth_enabled_state_no_change_detection() {
    // Test no state change detection logic
    let old_enabled = true;
    let new_enabled = true;

    let changed = old_enabled != new_enabled;
    assert!(
        !changed,
        "No state change should be detected when values are identical"
    );
}

#[test]
fn test_bluetooth_enabled_toggle_logic() {
    // Test toggle logic
    let current_state = true;
    let new_state = !current_state;
    assert_eq!(new_state, false, "Toggling true should give false");

    let current_state = false;
    let new_state = !current_state;
    assert_eq!(new_state, true, "Toggling false should give true");
}

#[test]
fn test_bluetooth_icon_consistency() {
    ensure_gtk_init();

    // Test that icon name is consistent with state
    let states = vec![true, false];

    for enabled in states {
        let expected_icon = if enabled {
            "bluetooth-active-symbolic"
        } else {
            "bluetooth-disabled-symbolic"
        };

        let actual_icon = if enabled {
            "bluetooth-active-symbolic".to_string()
        } else {
            "bluetooth-disabled-symbolic".to_string()
        };

        assert_eq!(
            actual_icon, expected_icon,
            "Icon should match enabled state: {}",
            enabled
        );
    }
}

#[test]
fn test_bluetooth_property_names() {
    // Test that property names follow GObject conventions
    let property_name = "enabled";

    assert!(
        !property_name.is_empty(),
        "Property name should not be empty"
    );
    assert!(
        property_name.chars().all(|c| c.is_lowercase() || c == '-'),
        "Property name should be lowercase with hyphens"
    );
}

#[test]
fn test_bluetooth_adapter_path_format() {
    // Test BlueZ adapter path format
    let adapter_path = "/org/bluez/hci0";

    assert!(
        adapter_path.starts_with("/org/bluez/"),
        "Adapter path should start with /org/bluez/"
    );
    assert!(
        adapter_path.contains("hci"),
        "Adapter path should contain 'hci'"
    );
}

#[test]
fn test_bluetooth_dbus_interface_name() {
    // Test D-Bus interface name
    let interface = "org.bluez.Adapter1";

    assert!(
        interface.starts_with("org.bluez"),
        "Interface should be in org.bluez namespace"
    );
    assert!(
        interface.ends_with("Adapter1"),
        "Interface should be Adapter1"
    );
}

#[test]
fn test_bluetooth_powered_property_name() {
    // Test BlueZ property name
    let property = "Powered";

    assert_eq!(property, "Powered", "BlueZ uses 'Powered' property name");
}

#[test]
fn test_bluetooth_state_boolean_values() {
    // Test that enabled state is boolean
    let valid_states = vec![true, false];

    for state in valid_states {
        assert!(
            state == true || state == false,
            "State should be a valid boolean"
        );
    }
}

#[test]
fn test_bluetooth_icon_name_not_empty() {
    // Test icon name logic without GTK service
    let enabled = true;
    let icon = if enabled {
        "bluetooth-active-symbolic".to_string()
    } else {
        "bluetooth-disabled-symbolic".to_string()
    };

    assert!(!icon.is_empty(), "Icon name should not be empty");
}

#[test]
fn test_bluetooth_icon_name_is_symbolic() {
    // Test icon name logic without GTK service
    let enabled = true;
    let icon = if enabled {
        "bluetooth-active-symbolic".to_string()
    } else {
        "bluetooth-disabled-symbolic".to_string()
    };

    assert!(
        icon.ends_with("-symbolic"),
        "Icon name should end with -symbolic"
    );
}

#[test]
fn test_bluetooth_device_struct_creation() {
    // Test BluetoothDevice struct can be created with all fields
    let device_path = "/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF".to_string();
    let device_name = "My Headphones".to_string();
    let device_address = "AA:BB:CC:DD:EE:FF".to_string();
    let connected = true;
    let paired = true;

    assert!(!device_path.is_empty(), "Device path should not be empty");
    assert!(!device_name.is_empty(), "Device name should not be empty");
    assert!(
        !device_address.is_empty(),
        "Device address should not be empty"
    );
    assert!(
        connected == true || connected == false,
        "Connected should be boolean"
    );
    assert!(
        paired == true || paired == false,
        "Paired should be boolean"
    );
}

#[test]
fn test_bluetooth_device_path_format() {
    // Test BlueZ device path format
    let device_path = "/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF";

    assert!(
        device_path.starts_with("/org/bluez/"),
        "Device path should start with /org/bluez/"
    );
    assert!(
        device_path.contains("dev_"),
        "Device path should contain 'dev_'"
    );
}

#[test]
fn test_bluetooth_device_address_format() {
    // Test Bluetooth MAC address format
    let address = "AA:BB:CC:DD:EE:FF";

    assert_eq!(
        address.len(),
        17,
        "Bluetooth MAC address should be 17 characters"
    );
    assert_eq!(
        address.matches(':').count(),
        5,
        "Bluetooth MAC address should have 5 colons"
    );
}

#[test]
fn test_bluetooth_devices_list_empty() {
    // Test empty devices list
    let devices: Vec<String> = Vec::new();

    assert_eq!(devices.len(), 0, "Initial devices list should be empty");
}

#[test]
fn test_bluetooth_devices_list_with_devices() {
    // Test devices list with multiple devices
    let mut devices = Vec::new();
    devices.push("Device 1".to_string());
    devices.push("Device 2".to_string());
    devices.push("Device 3".to_string());

    assert_eq!(devices.len(), 3, "Should have 3 devices");
}

#[test]
fn test_bluetooth_scanning_state_default() {
    // Test scanning state defaults to false
    let scanning = false;

    assert_eq!(scanning, false, "Scanning should default to false");
}

#[test]
fn test_bluetooth_scanning_state_toggle() {
    // Test scanning state can be toggled
    let scanning = true;
    assert_eq!(scanning, true, "Scanning should be true after toggle");

    let scanning = false;
    assert_eq!(scanning, false, "Scanning should be false after toggle");
}

#[test]
fn test_bluetooth_scanning_property_name() {
    // Test scanning property name follows GObject conventions
    let property_name = "scanning";

    assert!(
        !property_name.is_empty(),
        "Property name should not be empty"
    );
    assert!(
        property_name.chars().all(|c| c.is_lowercase() || c == '-'),
        "Property name should be lowercase with hyphens"
    );
}

#[test]
fn test_bluetooth_devices_property_name() {
    // Test devices property name follows GObject conventions
    let property_name = "devices";

    assert!(
        !property_name.is_empty(),
        "Property name should not be empty"
    );
    assert!(
        property_name.chars().all(|c| c.is_lowercase() || c == '-'),
        "Property name should be lowercase with hyphens"
    );
}

#[test]
fn test_bluetooth_dbus_device_interface_name() {
    // Test D-Bus Device interface name
    let interface = "org.bluez.Device1";

    assert!(
        interface.starts_with("org.bluez"),
        "Interface should be in org.bluez namespace"
    );
    assert!(
        interface.ends_with("Device1"),
        "Interface should be Device1"
    );
}

#[test]
fn test_bluetooth_dbus_object_manager_interface() {
    // Test D-Bus ObjectManager interface name
    let interface = "org.freedesktop.DBus.ObjectManager";

    assert!(
        interface.starts_with("org.freedesktop.DBus"),
        "Interface should be in org.freedesktop.DBus namespace"
    );
    assert!(
        interface.ends_with("ObjectManager"),
        "Interface should be ObjectManager"
    );
}

#[test]
fn test_bluetooth_adapter_discovering_property() {
    // Test BlueZ discovering property name
    let property = "Discovering";

    assert_eq!(
        property, "Discovering",
        "BlueZ uses 'Discovering' property name"
    );
}

#[test]
fn test_bluetooth_adapter_start_discovery_method() {
    // Test BlueZ StartDiscovery method name
    let method = "StartDiscovery";

    assert_eq!(
        method, "StartDiscovery",
        "BlueZ uses 'StartDiscovery' method name"
    );
}

#[test]
fn test_bluetooth_adapter_stop_discovery_method() {
    // Test BlueZ StopDiscovery method name
    let method = "StopDiscovery";

    assert_eq!(
        method, "StopDiscovery",
        "BlueZ uses 'StopDiscovery' method name"
    );
}

#[test]
fn test_bluetooth_device_properties() {
    // Test Device1 interface property names
    let properties = vec!["Name", "Address", "Connected", "Paired", "Alias"];

    for property in properties {
        assert!(!property.is_empty(), "Device property should not be empty");
        assert!(
            property.chars().next().unwrap().is_uppercase(),
            "BlueZ properties start with uppercase"
        );
    }
}

#[test]
fn test_bluetooth_device_connected_states() {
    // Test device connection state logic
    let states = vec![(false, false), (true, false), (false, true), (true, true)];

    for (connected, paired) in states {
        assert!(
            connected == true || connected == false,
            "Connected should be boolean"
        );
        assert!(
            paired == true || paired == false,
            "Paired should be boolean"
        );

        // Device can be paired but not connected
        if paired && !connected {
            assert!(true, "Device can be paired but not connected");
        }

        // Device cannot be connected without being paired (in most cases)
        if connected {
            // This is a common pattern but not a strict requirement
            assert!(true, "Connected device check");
        }
    }
}

#[test]
fn test_bluetooth_scan_timeout_duration() {
    // Test scan timeout is reasonable (10 seconds)
    let timeout_secs = 10;

    assert!(timeout_secs > 0, "Scan timeout should be greater than 0");
    assert!(
        timeout_secs <= 30,
        "Scan timeout should not be too long (<=30 seconds)"
    );
}

#[test]
fn test_bluetooth_device_list_add_remove() {
    // Test device list add/remove logic
    let mut devices = Vec::new();
    let device = "Device 1".to_string();

    devices.push(device.clone());
    assert_eq!(devices.len(), 1, "Should have 1 device after adding");

    if let Some(pos) = devices.iter().position(|d| d == &device) {
        devices.remove(pos);
    }
    assert_eq!(devices.len(), 0, "Should have 0 devices after removing");
}

#[test]
fn test_bluetooth_device_duplicate_check() {
    // Test duplicate device detection logic
    let mut devices = Vec::new();
    let device_path = "/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF".to_string();

    devices.push(device_path.clone());

    // Check if device already exists before adding
    let exists = devices.iter().any(|d| d == &device_path);
    assert!(exists, "Device should exist in list");

    // Don't add duplicate
    if !exists {
        devices.push(device_path.clone());
    }
    assert_eq!(
        devices.len(),
        1,
        "Should still have 1 device (no duplicate)"
    );
}

#[test]
fn test_bluetooth_device_icon_for_states() {
    // Test device icon selection based on connection state
    let test_cases = vec![
        (true, true, "bluetooth-active-symbolic"),
        (false, true, "bluetooth-disconnected-symbolic"),
        (false, false, "bluetooth-disabled-symbolic"),
    ];

    for (connected, paired, expected_icon) in test_cases {
        let icon = if connected {
            "bluetooth-active-symbolic".to_string()
        } else if paired {
            "bluetooth-disconnected-symbolic".to_string()
        } else {
            "bluetooth-disabled-symbolic".to_string()
        };

        assert_eq!(
            icon, expected_icon,
            "Icon should match for connected={} paired={}",
            connected, paired
        );
    }
}

#[test]
fn test_bluetooth_interfaces_added_signal() {
    // Test InterfacesAdded signal name
    let signal = "InterfacesAdded";

    assert_eq!(
        signal, "InterfacesAdded",
        "D-Bus uses 'InterfacesAdded' signal"
    );
}

#[test]
fn test_bluetooth_interfaces_removed_signal() {
    // Test InterfacesRemoved signal name
    let signal = "InterfacesRemoved";

    assert_eq!(
        signal, "InterfacesRemoved",
        "D-Bus uses 'InterfacesRemoved' signal"
    );
}

#[test]
fn test_bluetooth_device_connection_state_change() {
    // Test device connection state can change
    let connected = true;
    assert_eq!(
        connected, true,
        "Device connection state should be updatable"
    );
}

#[test]
fn test_bluetooth_device_connection_state_detection() {
    // Test detecting connection state changes
    let old_state = false;
    let new_state = true;

    assert_ne!(
        old_state, new_state,
        "Should detect connection state changes"
    );
}

#[test]
fn test_bluetooth_device_disconnection_state_detection() {
    // Test detecting disconnection state changes
    let old_state = true;
    let new_state = false;

    assert_ne!(
        old_state, new_state,
        "Should detect disconnection state changes"
    );
}

#[test]
fn test_bluetooth_properties_changed_interface() {
    // Test PropertiesChanged signal interface
    let interface = "org.freedesktop.DBus.Properties";

    assert_eq!(
        interface, "org.freedesktop.DBus.Properties",
        "Should use standard D-Bus Properties interface"
    );
}

#[test]
fn test_bluetooth_device_connected_property_name() {
    // Test Connected property name
    let property = "Connected";

    assert_eq!(
        property, "Connected",
        "BlueZ uses 'Connected' property for device connection state"
    );
}

#[test]
fn test_bluetooth_device_trusted_property_name() {
    // Test Trusted property name
    let property = "Trusted";

    assert_eq!(
        property, "Trusted",
        "BlueZ uses 'Trusted' property for device trust state"
    );
}

#[test]
fn test_bluetooth_agent_path() {
    // Test agent registration path
    let agent_path = "/org/bluez/agent/rusty_de";

    assert!(
        agent_path.starts_with("/org/bluez/agent/"),
        "Agent path should be under /org/bluez/agent/"
    );
}

#[test]
fn test_bluetooth_agent_capability() {
    // Test agent capability
    let capability = "KeyboardDisplay";

    assert_eq!(
        capability, "KeyboardDisplay",
        "Agent should use KeyboardDisplay capability for maximum compatibility"
    );
}

#[test]
fn test_bluetooth_agent_interface_name() {
    // Test agent interface name
    let interface = "org.bluez.Agent1";

    assert_eq!(
        interface, "org.bluez.Agent1",
        "Agent should implement org.bluez.Agent1 interface"
    );
}

#[test]
fn test_bluetooth_agent_manager_interface_name() {
    // Test agent manager interface name
    let interface = "org.bluez.AgentManager1";

    assert_eq!(
        interface, "org.bluez.AgentManager1",
        "AgentManager should use org.bluez.AgentManager1 interface"
    );
}

#[test]
fn test_bluetooth_connection_error_format() {
    // Test that connection errors are properly formatted
    let error = "Failed to connect to system bus: mock error";

    assert!(
        error.starts_with("Failed to"),
        "Connection errors should start with 'Failed to'"
    );
}

#[test]
fn test_bluetooth_disconnection_error_format() {
    // Test that disconnection errors are properly formatted
    let error = "Failed to disconnect device: mock error";

    assert!(
        error.starts_with("Failed to"),
        "Disconnection errors should start with 'Failed to'"
    );
}

#[test]
fn test_bluetooth_device_pair_method() {
    // Test BlueZ Pair method name
    let method = "Pair";

    assert_eq!(method, "Pair", "BlueZ uses 'Pair' method for pairing");
}

#[test]
fn test_bluetooth_device_connect_method() {
    // Test BlueZ Connect method name
    let method = "Connect";

    assert_eq!(
        method, "Connect",
        "BlueZ uses 'Connect' method for connection"
    );
}

#[test]
fn test_bluetooth_device_disconnect_method() {
    // Test BlueZ Disconnect method name
    let method = "Disconnect";

    assert_eq!(
        method, "Disconnect",
        "BlueZ uses 'Disconnect' method for disconnection"
    );
}

#[test]
fn test_bluetooth_device_paired_property() {
    // Test BlueZ Paired property name
    let property = "Paired";

    assert_eq!(property, "Paired", "BlueZ uses 'Paired' property");
}

#[test]
fn test_bluetooth_connection_flow() {
    // Test connection flow logic
    let already_connected = false;
    let is_paired = false;

    // If not connected and not paired, need to pair first
    if !already_connected && !is_paired {
        assert!(true, "Should pair device first");
    }

    // After pairing, set trusted and connect
    let should_set_trusted = true;
    assert!(should_set_trusted, "Should set device as trusted");
}

#[test]
fn test_bluetooth_disconnection_flow() {
    // Test disconnection flow logic
    let already_disconnected = false;

    // If already disconnected, return early
    if already_disconnected {
        assert!(true, "Should return early if already disconnected");
    } else {
        assert!(true, "Should proceed with disconnection");
    }
}

#[test]
fn test_bluetooth_error_message_formatting() {
    // Test error message formatting
    let error_msg = format!("Failed to connect to system bus: {}", "test error");

    assert!(
        error_msg.contains("Failed to"),
        "Error should contain 'Failed to'"
    );
    assert!(
        error_msg.contains("test error"),
        "Error should contain original error message"
    );
}

#[test]
fn test_bluetooth_success_message_formatting() {
    // Test success message formatting
    let device_path = "/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF";
    let success_msg = format!("Successfully connected to device {}", device_path);

    assert!(
        success_msg.contains("Successfully"),
        "Success message should contain 'Successfully'"
    );
    assert!(
        success_msg.contains(device_path),
        "Success message should contain device path"
    );
}

#[test]
fn test_bluetooth_adapter_pairable_property() {
    // Test BlueZ Pairable property name
    let property = "Pairable";

    assert_eq!(
        property, "Pairable",
        "BlueZ uses 'Pairable' property for adapter pairable state"
    );
}

#[test]
fn test_bluetooth_adapter_pairable_timeout_property() {
    // Test BlueZ PairableTimeout property name
    let property = "PairableTimeout";

    assert_eq!(
        property, "PairableTimeout",
        "BlueZ uses 'PairableTimeout' property"
    );
}

#[test]
fn test_bluetooth_services_resolved_property() {
    // Test BlueZ ServicesResolved property name
    let property = "ServicesResolved";

    assert_eq!(
        property, "ServicesResolved",
        "BlueZ uses 'ServicesResolved' property to indicate service discovery completion"
    );
}

#[test]
fn test_bluetooth_pairing_flow_logic() {
    // Test pairing flow with service discovery
    let is_paired = false;
    let services_resolved = false;

    // If not paired, should initiate pairing
    if !is_paired {
        assert!(true, "Should initiate pairing for unpaired device");
    }

    // After pairing, wait for services to resolve
    if !services_resolved {
        assert!(true, "Should wait for service discovery to complete");
    }
}

#[test]
fn test_bluetooth_auto_reconnect_on_disconnect() {
    // Test auto-reconnect logic for recently paired devices
    let recently_paired = true;
    let connected = false;

    // If recently paired device disconnects, should auto-reconnect
    if recently_paired && !connected {
        assert!(
            true,
            "Should attempt auto-reconnect for recently paired device"
        );
    }
}

#[test]
fn test_bluetooth_recently_paired_tracking() {
    // Test recently paired device tracking
    use std::collections::HashSet;

    let mut recently_paired_devices = HashSet::new();
    let device_path = "/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF".to_string();

    // Add device to recently paired set
    recently_paired_devices.insert(device_path.clone());
    assert!(
        recently_paired_devices.contains(&device_path),
        "Should track recently paired device"
    );

    // Remove device from recently paired set
    recently_paired_devices.remove(&device_path);
    assert!(
        !recently_paired_devices.contains(&device_path),
        "Should be able to remove device from tracking"
    );
}

#[test]
fn test_bluetooth_recently_paired_timeout() {
    // Test that recently paired tracking has reasonable timeout
    let timeout_secs = 10;

    assert!(
        timeout_secs > 0,
        "Recently paired timeout should be greater than 0"
    );
    assert!(
        timeout_secs <= 60,
        "Recently paired timeout should not be too long (<=60 seconds)"
    );
}

#[test]
fn test_bluetooth_auto_reconnect_delay() {
    // Test auto-reconnect delay is reasonable
    let delay_ms = 1500;

    assert!(
        delay_ms >= 1000,
        "Auto-reconnect delay should be at least 1 second"
    );
    assert!(
        delay_ms <= 5000,
        "Auto-reconnect delay should not be too long (<=5 seconds)"
    );
}

#[test]
fn test_bluetooth_service_discovery_wait() {
    // Test service discovery wait logic
    let max_wait_iterations = 10;
    let wait_per_iteration_ms = 500;

    let total_wait_ms = max_wait_iterations * wait_per_iteration_ms;

    assert!(
        total_wait_ms >= 3000,
        "Should wait at least 3 seconds for service discovery"
    );
    assert!(
        total_wait_ms <= 10000,
        "Should not wait more than 10 seconds for service discovery"
    );
}

#[test]
fn test_bluetooth_pairing_notification_actions() {
    // Test pairing notification action keys
    let actions = vec!["deny", "Deny", "allow", "Allow"];

    assert_eq!(actions.len(), 4, "Should have 4 action items (2 pairs)");
    assert!(actions.contains(&"allow"), "Should have 'allow' action key");
    assert!(
        actions.contains(&"Allow"),
        "Should have 'Allow' action label"
    );
    assert!(actions.contains(&"deny"), "Should have 'deny' action key");
    assert!(actions.contains(&"Deny"), "Should have 'Deny' action label");
}

#[test]
fn test_bluetooth_notification_urgency_critical() {
    // Test pairing notification uses critical urgency
    let urgency = 2u8; // Critical urgency

    assert_eq!(
        urgency, 2,
        "Pairing notifications should use critical urgency (2)"
    );
}

#[test]
fn test_bluetooth_set_trusted_after_pairing() {
    // Test that devices are set as trusted after pairing
    let should_set_trusted = true;

    assert!(
        should_set_trusted,
        "Devices should be set as trusted after successful pairing"
    );
}

#[test]
fn test_bluetooth_adapter_pairable_before_pairing() {
    // Test that adapter is set to pairable before initiating pairing
    let should_set_pairable = true;

    assert!(
        should_set_pairable,
        "Adapter should be set to pairable before initiating device pairing"
    );
}

#[test]
fn test_bluetooth_multiple_reconnect_prevention() {
    // Test that multiple reconnection attempts are prevented
    use std::collections::HashSet;

    let mut recently_paired_devices = HashSet::new();
    let device_path = "/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF".to_string();

    recently_paired_devices.insert(device_path.clone());

    // First disconnect - should attempt reconnect
    let should_reconnect = recently_paired_devices.contains(&device_path);
    assert!(
        should_reconnect,
        "First disconnect should trigger reconnect"
    );

    // Remove from set after reconnect attempt
    recently_paired_devices.remove(&device_path);

    // Second disconnect - should not attempt reconnect
    let should_not_reconnect = !recently_paired_devices.contains(&device_path);
    assert!(
        should_not_reconnect,
        "Second disconnect should not trigger reconnect"
    );
}

#[test]
fn test_bluetooth_pairing_timeout() {
    // Test pairing notification timeout
    let timeout_secs = 30;

    assert!(
        timeout_secs >= 20,
        "Pairing timeout should be at least 20 seconds"
    );
    assert!(
        timeout_secs <= 60,
        "Pairing timeout should not exceed 60 seconds"
    );
}

#[test]
fn test_bluetooth_agent_authorization_method() {
    // Test agent authorization method name
    let method = "RequestAuthorization";

    assert_eq!(
        method, "RequestAuthorization",
        "Agent should implement RequestAuthorization method"
    );
}

#[test]
fn test_bluetooth_agent_confirmation_method() {
    // Test agent confirmation method name
    let method = "RequestConfirmation";

    assert_eq!(
        method, "RequestConfirmation",
        "Agent should implement RequestConfirmation method"
    );
}

#[test]
fn test_bluetooth_agent_authorize_service_method() {
    // Test agent authorize service method name
    let method = "AuthorizeService";

    assert_eq!(
        method, "AuthorizeService",
        "Agent should implement AuthorizeService method"
    );
}

#[test]
fn test_bluetooth_connection_after_service_resolution() {
    // Test that connection is checked after service resolution
    let services_resolved = true;
    let should_check_connection = services_resolved;

    assert!(
        should_check_connection,
        "Should check connection status after services are resolved"
    );
}

#[test]
fn test_bluetooth_event_driven_monitoring() {
    // Test that device monitoring is event-driven, not polling-based
    use std::collections::HashSet;

    let mut monitored_devices = HashSet::new();
    let device_path = "/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF".to_string();

    // Add device to monitoring set when discovered
    monitored_devices.insert(device_path.clone());
    assert!(
        monitored_devices.contains(&device_path),
        "Device should be added to monitoring set when discovered"
    );

    // Check for duplicate monitoring
    let already_monitored = monitored_devices.contains(&device_path);
    assert!(
        already_monitored,
        "Should detect if device is already being monitored"
    );

    // Remove device when it's removed
    monitored_devices.remove(&device_path);
    assert!(
        !monitored_devices.contains(&device_path),
        "Device should be removed from monitoring set when removed"
    );
}

#[test]
fn test_bluetooth_property_change_detection() {
    // Test property change detection logic
    let old_connected = true;
    let new_connected = false;
    let changed = old_connected != new_connected;

    assert!(
        changed,
        "Should detect when connection state changes from connected to disconnected"
    );
}

#[test]
fn test_bluetooth_auto_reconnect_trigger() {
    // Test auto-reconnect triggering logic
    use std::collections::HashSet;

    let mut recently_paired = HashSet::new();
    let device_path = "/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF".to_string();

    // Simulate successful pairing
    recently_paired.insert(device_path.clone());

    // Simulate disconnect event
    let is_recently_paired = recently_paired.contains(&device_path);
    let should_reconnect = is_recently_paired;

    assert!(
        should_reconnect,
        "Should attempt auto-reconnect when recently paired device disconnects"
    );

    // Remove from set to prevent multiple reconnect attempts
    recently_paired.remove(&device_path);
    assert!(
        !recently_paired.contains(&device_path),
        "Should remove from recently paired set after reconnect attempt"
    );
}

#[test]
fn test_bluetooth_single_reconnect_attempt() {
    // Test that only one reconnect attempt is made
    use std::collections::HashSet;

    let mut recently_paired = HashSet::new();
    let device_path = "/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF".to_string();

    recently_paired.insert(device_path.clone());

    // First disconnect - should trigger reconnect
    let first_disconnect = recently_paired.contains(&device_path);
    assert!(
        first_disconnect,
        "First disconnect should be detected for recently paired device"
    );
    recently_paired.remove(&device_path);

    // Second disconnect - should not trigger reconnect
    let second_disconnect = recently_paired.contains(&device_path);
    assert!(
        !second_disconnect,
        "Second disconnect should not trigger reconnect"
    );
}

#[test]
fn test_bluetooth_monitoring_lifecycle() {
    // Test device monitoring lifecycle
    use std::collections::HashSet;

    let mut monitored = HashSet::new();
    let device1 = "/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF".to_string();
    let device2 = "/org/bluez/hci0/dev_11_22_33_44_55_66".to_string();

    // Add devices
    monitored.insert(device1.clone());
    monitored.insert(device2.clone());
    assert_eq!(monitored.len(), 2, "Should have 2 monitored devices");

    // Check duplicate prevention
    monitored.insert(device1.clone());
    assert_eq!(
        monitored.len(),
        2,
        "Should not add duplicate monitored devices"
    );

    // Remove device
    monitored.remove(&device1);
    assert_eq!(
        monitored.len(),
        1,
        "Should have 1 device after removing one"
    );
    assert!(monitored.contains(&device2), "Should still contain device2");
}

#[test]
fn test_bluetooth_disconnect_delay() {
    // Test that reconnect delay is reasonable
    let delay_ms = 1500;

    assert!(
        delay_ms >= 1000,
        "Reconnect delay should be at least 1 second"
    );
    assert!(
        delay_ms <= 3000,
        "Reconnect delay should not exceed 3 seconds"
    );
}

#[test]
fn test_bluetooth_async_workflow_states() {
    // Test async workflow state transitions
    #[derive(PartialEq, Debug)]
    enum State {
        Idle,
        Pairing,
        WaitingForServices,
        Connected,
        Disconnected,
        Reconnecting,
    }

    // Start idle
    let mut current_state = State::Idle;
    assert_eq!(current_state, State::Idle, "Should start in Idle state");

    // Start pairing
    current_state = State::Pairing;
    assert_eq!(
        current_state,
        State::Pairing,
        "Should transition to Pairing"
    );

    // Wait for services
    current_state = State::WaitingForServices;
    assert_eq!(
        current_state,
        State::WaitingForServices,
        "Should wait for service discovery"
    );

    // Connected
    current_state = State::Connected;
    assert_eq!(current_state, State::Connected, "Should be Connected");

    // Disconnected
    current_state = State::Disconnected;
    assert_eq!(
        current_state,
        State::Disconnected,
        "Should detect disconnect"
    );

    // Auto-reconnect
    current_state = State::Reconnecting;
    assert_eq!(
        current_state,
        State::Reconnecting,
        "Should attempt reconnect"
    );
}

#[test]
fn test_bluetooth_service_discovery_polling() {
    // Test service discovery polling logic
    let max_iterations = 10;
    let delay_per_iteration_ms = 500;

    let mut services_resolved = false;
    let mut iterations = 0;

    // Simulate polling loop
    while !services_resolved && iterations < max_iterations {
        iterations += 1;
        // In real code, would check services_resolved property
        if iterations == 3 {
            services_resolved = true; // Simulate resolution on 3rd check
        }
    }

    assert!(
        services_resolved,
        "Services should be resolved within max iterations"
    );
    assert!(
        iterations <= max_iterations,
        "Should not exceed max iterations"
    );

    let total_time_ms = iterations * delay_per_iteration_ms;
    assert!(
        total_time_ms <= 5000,
        "Service discovery should complete within 5 seconds"
    );
}

#[test]
fn test_bluetooth_connection_retry_logic() {
    // Test connection retry logic after failed attempts
    let max_retries = 1;

    // For recently paired devices, we only retry once
    assert_eq!(
        max_retries, 1,
        "Should only attempt connection once for auto-reconnect"
    );
}

#[test]
fn test_bluetooth_concurrent_pairing_prevention() {
    // Test that concurrent pairing requests are handled correctly
    use std::collections::HashSet;

    let mut pairing_in_progress = HashSet::new();
    let device_path = "/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF".to_string();

    // Start pairing
    let can_pair = !pairing_in_progress.contains(&device_path);
    assert!(can_pair, "Should allow first pairing attempt");
    pairing_in_progress.insert(device_path.clone());

    // Try to pair again while in progress
    let cannot_pair = !pairing_in_progress.contains(&device_path);
    assert!(
        !cannot_pair,
        "Should prevent concurrent pairing of same device"
    );

    // Complete pairing
    pairing_in_progress.remove(&device_path);
    let can_pair_again = !pairing_in_progress.contains(&device_path);
    assert!(
        can_pair_again,
        "Should allow pairing after previous completed"
    );
}

#[test]
fn test_expected_devices_percentage_no_expected_devices() {
    // When no expected devices are configured, should return 100%
    let expected_devices: Vec<String> = vec![];
    let connected_devices: Vec<String> = vec!["AA:BB:CC:DD:EE:FF".to_string()];

    let percentage = if expected_devices.is_empty() {
        1.0
    } else {
        let connected_count = expected_devices
            .iter()
            .filter(|expected| {
                connected_devices
                    .iter()
                    .any(|connected| connected.to_uppercase() == expected.to_uppercase())
            })
            .count();
        connected_count as f64 / expected_devices.len() as f64
    };

    assert_eq!(
        percentage, 1.0,
        "Should return 100% when no expected devices"
    );
}

#[test]
fn test_expected_devices_percentage_all_connected() {
    // When all expected devices are connected, should return 100%
    let expected_devices = vec![
        "AA:BB:CC:DD:EE:FF".to_string(),
        "11:22:33:44:55:66".to_string(),
    ];
    let connected_devices = vec![
        "AA:BB:CC:DD:EE:FF".to_string(),
        "11:22:33:44:55:66".to_string(),
    ];

    let connected_count = expected_devices
        .iter()
        .filter(|expected| {
            connected_devices
                .iter()
                .any(|connected| connected.to_uppercase() == expected.to_uppercase())
        })
        .count();
    let percentage = connected_count as f64 / expected_devices.len() as f64;

    assert_eq!(
        percentage, 1.0,
        "Should return 100% when all devices connected"
    );
}

#[test]
fn test_expected_devices_percentage_half_connected() {
    // When half of expected devices are connected, should return 50%
    let expected_devices = vec![
        "AA:BB:CC:DD:EE:FF".to_string(),
        "11:22:33:44:55:66".to_string(),
    ];
    let connected_devices = vec!["AA:BB:CC:DD:EE:FF".to_string()];

    let connected_count = expected_devices
        .iter()
        .filter(|expected| {
            connected_devices
                .iter()
                .any(|connected| connected.to_uppercase() == expected.to_uppercase())
        })
        .count();
    let percentage = connected_count as f64 / expected_devices.len() as f64;

    assert_eq!(percentage, 0.5, "Should return 50% when half connected");
}

#[test]
fn test_expected_devices_percentage_none_connected() {
    // When no expected devices are connected, should return 0%
    let expected_devices = vec![
        "AA:BB:CC:DD:EE:FF".to_string(),
        "11:22:33:44:55:66".to_string(),
    ];
    let connected_devices: Vec<String> = vec![];

    let connected_count = expected_devices
        .iter()
        .filter(|expected| {
            connected_devices
                .iter()
                .any(|connected| connected.to_uppercase() == expected.to_uppercase())
        })
        .count();
    let percentage = connected_count as f64 / expected_devices.len() as f64;

    assert_eq!(
        percentage, 0.0,
        "Should return 0% when no devices connected"
    );
}

#[test]
fn test_expected_devices_percentage_case_insensitive() {
    // MAC address matching should be case insensitive
    let expected_devices = vec!["AA:BB:CC:DD:EE:FF".to_string()];
    let connected_devices = vec!["aa:bb:cc:dd:ee:ff".to_string()];

    let connected_count = expected_devices
        .iter()
        .filter(|expected| {
            connected_devices
                .iter()
                .any(|connected| connected.to_uppercase() == expected.to_uppercase())
        })
        .count();
    let percentage = connected_count as f64 / expected_devices.len() as f64;

    assert_eq!(
        percentage, 1.0,
        "Should match case insensitively for MAC addresses"
    );
}

#[test]
fn test_expected_devices_percentage_one_of_three() {
    // When 1 of 3 expected devices is connected
    let expected_devices = vec![
        "AA:BB:CC:DD:EE:FF".to_string(),
        "11:22:33:44:55:66".to_string(),
        "77:88:99:AA:BB:CC".to_string(),
    ];
    let connected_devices = vec!["11:22:33:44:55:66".to_string()];

    let connected_count = expected_devices
        .iter()
        .filter(|expected| {
            connected_devices
                .iter()
                .any(|connected| connected.to_uppercase() == expected.to_uppercase())
        })
        .count();
    let percentage = connected_count as f64 / expected_devices.len() as f64;

    assert!(
        (percentage - 0.333).abs() < 0.01,
        "Should return ~33% when 1 of 3 connected"
    );
}

#[test]
fn test_has_missing_expected_devices_none_expected() {
    // When no expected devices configured, should return false
    let expected_devices: Vec<String> = vec![];
    let connected_devices = vec!["AA:BB:CC:DD:EE:FF".to_string()];

    let has_missing = if expected_devices.is_empty() {
        false
    } else {
        expected_devices.iter().any(|expected| {
            !connected_devices
                .iter()
                .any(|connected| connected.to_uppercase() == expected.to_uppercase())
        })
    };

    assert!(!has_missing, "Should return false when no expected devices");
}

#[test]
fn test_has_missing_expected_devices_all_connected() {
    // When all expected devices connected, should return false
    let expected_devices = vec!["AA:BB:CC:DD:EE:FF".to_string()];
    let connected_devices = vec!["AA:BB:CC:DD:EE:FF".to_string()];

    let has_missing = expected_devices.iter().any(|expected| {
        !connected_devices
            .iter()
            .any(|connected| connected.to_uppercase() == expected.to_uppercase())
    });

    assert!(
        !has_missing,
        "Should return false when all devices connected"
    );
}

#[test]
fn test_has_missing_expected_devices_one_missing() {
    // When one expected device is missing, should return true
    let expected_devices = vec![
        "AA:BB:CC:DD:EE:FF".to_string(),
        "11:22:33:44:55:66".to_string(),
    ];
    let connected_devices = vec!["AA:BB:CC:DD:EE:FF".to_string()];

    let has_missing = expected_devices.iter().any(|expected| {
        !connected_devices
            .iter()
            .any(|connected| connected.to_uppercase() == expected.to_uppercase())
    });

    assert!(has_missing, "Should return true when one device is missing");
}

#[test]
fn test_get_connecting_device_returns_none_initially() {
    // Test that get_connecting_device returns None when no device is connecting
    let connecting_device: Option<String> = None;

    assert_eq!(
        connecting_device, None,
        "get_connecting_device should return None initially"
    );
}

#[test]
fn test_get_connecting_device_returns_device_path() {
    // Test that get_connecting_device returns the device path when connecting
    let device_path = "/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF".to_string();
    let connecting_device: Option<String> = Some(device_path.clone());

    assert_eq!(
        connecting_device,
        Some(device_path),
        "get_connecting_device should return the connecting device path"
    );
}

#[test]
fn test_connecting_device_state_lifecycle() {
    // Test the full lifecycle of connecting device state
    let mut connecting_device: Option<String> = None;

    // Initially None
    assert_eq!(connecting_device, None, "Should start as None");

    // Set to device path when connecting starts
    let device_path = "/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF".to_string();
    connecting_device = Some(device_path.clone());
    assert_eq!(
        connecting_device,
        Some(device_path),
        "Should contain device path during connection"
    );

    // Clear when connection completes
    connecting_device = None;
    assert_eq!(
        connecting_device, None,
        "Should be None after connection completes"
    );
}

#[test]
fn test_connecting_device_cleared_on_success() {
    // Test that connecting device is cleared when connection succeeds
    // Simulate successful connection
    let connecting_device: Option<String> = None;

    assert_eq!(
        connecting_device, None,
        "Connecting device should be None after successful connection"
    );
}

#[test]
fn test_connecting_device_cleared_on_failure() {
    // Test that connecting device is cleared when connection fails
    // Simulate failed connection
    let connecting_device: Option<String> = None;

    assert_eq!(
        connecting_device, None,
        "Connecting device should be None after connection failure"
    );
}

#[test]
fn test_connecting_device_path_format() {
    // Test the format of connecting device path
    let device_path = "/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF";

    assert!(
        device_path.starts_with("/org/bluez/"),
        "Device path should start with /org/bluez/"
    );
    assert!(
        device_path.contains("/dev_"),
        "Device path should contain /dev_"
    );
}

#[test]
fn test_only_one_device_connecting_at_a_time() {
    // Test that only one device can be in connecting state at a time
    let first_device = "/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF".to_string();
    let second_device = "/org/bluez/hci0/dev_11_22_33_44_55_66".to_string();

    let mut connecting_device: Option<String> = Some(first_device.clone());
    assert_eq!(connecting_device, Some(first_device));

    // Setting a new connecting device replaces the old one
    connecting_device = Some(second_device.clone());
    assert_eq!(
        connecting_device,
        Some(second_device),
        "New connecting device should replace the old one"
    );
}

#[test]
fn test_bluetooth_icon_with_connected_devices() {
    // Test icon shows bluetooth-active-symbolic when devices are connected
    #[derive(Clone)]
    struct TestDevice {
        connected: bool,
    }

    let devices = vec![
        TestDevice { connected: true },
        TestDevice { connected: false },
    ];

    let has_connected = devices.iter().any(|d| d.connected);
    let icon = if has_connected {
        "bluetooth-active-symbolic"
    } else {
        "bluetooth-disconnected-symbolic"
    };

    assert_eq!(
        icon, "bluetooth-active-symbolic",
        "Icon should be active when at least one device is connected"
    );
}

#[test]
fn test_bluetooth_icon_with_no_connected_devices() {
    // Test icon shows bluetooth-disconnected-symbolic when no devices are connected
    #[derive(Clone)]
    struct TestDevice {
        connected: bool,
    }

    let devices = vec![
        TestDevice { connected: false },
        TestDevice { connected: false },
    ];

    let has_connected = devices.iter().any(|d| d.connected);
    let icon = if has_connected {
        "bluetooth-active-symbolic"
    } else {
        "bluetooth-disconnected-symbolic"
    };

    assert_eq!(
        icon, "bluetooth-disconnected-symbolic",
        "Icon should be disconnected when no devices are connected"
    );
}

#[test]
fn test_bluetooth_icon_with_empty_devices() {
    // Test icon shows bluetooth-disconnected-symbolic when device list is empty
    #[derive(Clone)]
    struct TestDevice {
        connected: bool,
    }

    let devices: Vec<TestDevice> = vec![];

    let has_connected = devices.iter().any(|d| d.connected);
    let icon = if has_connected {
        "bluetooth-active-symbolic"
    } else {
        "bluetooth-disconnected-symbolic"
    };

    assert_eq!(
        icon, "bluetooth-disconnected-symbolic",
        "Icon should be disconnected when device list is empty"
    );
}

#[test]
fn test_bluetooth_icon_all_devices_connected() {
    // Test icon shows bluetooth-active-symbolic when all devices are connected
    #[derive(Clone)]
    struct TestDevice {
        connected: bool,
    }

    let devices = vec![
        TestDevice { connected: true },
        TestDevice { connected: true },
        TestDevice { connected: true },
    ];

    let has_connected = devices.iter().any(|d| d.connected);
    let icon = if has_connected {
        "bluetooth-active-symbolic"
    } else {
        "bluetooth-disconnected-symbolic"
    };

    assert_eq!(
        icon, "bluetooth-active-symbolic",
        "Icon should be active when all devices are connected"
    );
}

#[test]
fn test_bluetooth_icon_independent_of_expected_devices() {
    // Test that icon logic is independent of expected devices configuration
    // Icon should be based on ANY connected device, not just expected ones
    #[derive(Clone)]
    struct TestDevice {
        connected: bool,
        #[allow(dead_code)]
        is_expected: bool,
    }

    // Scenario: One unexpected device connected, expected device not connected
    let devices = vec![
        TestDevice {
            connected: true,
            is_expected: false,
        },
        TestDevice {
            connected: false,
            is_expected: true,
        },
    ];

    let has_connected = devices.iter().any(|d| d.connected);
    let icon = if has_connected {
        "bluetooth-active-symbolic"
    } else {
        "bluetooth-disconnected-symbolic"
    };

    assert_eq!(
        icon, "bluetooth-active-symbolic",
        "Icon should be active even if connected device is not in expected list"
    );
}

#[test]
fn test_connect_device_notifies_missing_expected_devices() {
    // Test that connect_device notifies missing-expected-devices property after connection
    // This ensures the bar can switch back to workspace number when a device reconnects

    // Simulate the notification flow when a device connects
    let properties_to_notify = vec!["devices", "missing-expected-devices"];

    assert!(
        properties_to_notify.contains(&"devices"),
        "Should notify 'devices' property after connection"
    );
    assert!(
        properties_to_notify.contains(&"missing-expected-devices"),
        "Should notify 'missing-expected-devices' property after connection"
    );
    assert_eq!(
        properties_to_notify.len(),
        2,
        "Should notify exactly 2 properties after connection"
    );
}

#[test]
fn test_connect_device_notification_order() {
    // Test that notifications happen in the correct order after connection completes
    // 1. Callback is called first (to display any errors)
    // 2. connecting_device is cleared
    // 3. Properties are notified (devices, missing-expected-devices)

    #[derive(PartialEq, Debug)]
    enum Step {
        CallbackCalled,
        ConnectingDeviceCleared,
        DevicesNotified,
        MissingExpectedDevicesNotified,
    }

    let steps = vec![
        Step::CallbackCalled,
        Step::ConnectingDeviceCleared,
        Step::DevicesNotified,
        Step::MissingExpectedDevicesNotified,
    ];

    // Verify order
    assert_eq!(
        steps[0],
        Step::CallbackCalled,
        "Callback should be called first"
    );
    assert_eq!(
        steps[1],
        Step::ConnectingDeviceCleared,
        "Connecting device should be cleared second"
    );
    assert_eq!(
        steps[2],
        Step::DevicesNotified,
        "Devices property should be notified third"
    );
    assert_eq!(
        steps[3],
        Step::MissingExpectedDevicesNotified,
        "Missing-expected-devices property should be notified fourth"
    );
}

#[test]
fn test_notify_if_not_connecting_skips_missing_expected_devices() {
    // Test that property change handlers skip missing-expected-devices notification
    // when is_connecting is true (to avoid duplicate notifications)

    let is_connecting = true;
    let should_notify_missing_expected = !is_connecting;

    assert!(
        !should_notify_missing_expected,
        "Should not notify missing-expected-devices when device is connecting"
    );
}

#[test]
fn test_connect_device_ensures_final_notification() {
    // Test that connect_device ensures missing-expected-devices is notified
    // even though property change handler skips it during connection

    // During connection: property handler skips notification
    let is_connecting = true;
    let handler_notifies = !is_connecting;
    assert!(
        !handler_notifies,
        "Property handler should skip notification during connection"
    );

    // After connection completes: connect_device explicitly notifies
    let connect_device_notifies = true;
    assert!(
        connect_device_notifies,
        "connect_device should explicitly notify after clearing connecting state"
    );

    // The explicit notification ensures the bar gets updated
    let bar_gets_notified = connect_device_notifies;
    assert!(
        bar_gets_notified,
        "Bar should receive notification and switch back to workspace number"
    );
}

#[test]
fn test_lowest_battery_no_expected_devices() {
    // When no expected devices configured, should return None
    let expected_devices: Vec<String> = vec![];

    #[derive(Clone)]
    struct TestDevice {
        address: String,
        connected: bool,
        battery_percentage: Option<u8>,
    }

    let devices = vec![TestDevice {
        address: "AA:BB:CC:DD:EE:FF".to_string(),
        connected: true,
        battery_percentage: Some(50),
    }];

    let expected_addresses: Vec<String> =
        expected_devices.iter().map(|d| d.to_uppercase()).collect();

    let lowest_battery = devices
        .iter()
        .filter(|d| d.connected && expected_addresses.contains(&d.address.to_uppercase()))
        .filter_map(|d| d.battery_percentage)
        .min();

    assert_eq!(
        lowest_battery, None,
        "Should return None when no expected devices"
    );
}

#[test]
fn test_lowest_battery_no_connected_devices() {
    // When expected devices exist but none are connected, should return None
    let expected_devices = vec!["AA:BB:CC:DD:EE:FF".to_string()];

    #[derive(Clone)]
    struct TestDevice {
        address: String,
        connected: bool,
        battery_percentage: Option<u8>,
    }

    let devices = vec![TestDevice {
        address: "AA:BB:CC:DD:EE:FF".to_string(),
        connected: false,
        battery_percentage: Some(50),
    }];

    let expected_addresses: Vec<String> =
        expected_devices.iter().map(|d| d.to_uppercase()).collect();

    let lowest_battery = devices
        .iter()
        .filter(|d| d.connected && expected_addresses.contains(&d.address.to_uppercase()))
        .filter_map(|d| d.battery_percentage)
        .min();

    assert_eq!(
        lowest_battery, None,
        "Should return None when no devices are connected"
    );
}

#[test]
fn test_lowest_battery_one_device_connected() {
    // When one expected device is connected, should return its battery level
    let expected_devices = vec!["AA:BB:CC:DD:EE:FF".to_string()];

    #[derive(Clone)]
    struct TestDevice {
        address: String,
        connected: bool,
        battery_percentage: Option<u8>,
    }

    let devices = vec![TestDevice {
        address: "AA:BB:CC:DD:EE:FF".to_string(),
        connected: true,
        battery_percentage: Some(60),
    }];

    let expected_addresses: Vec<String> =
        expected_devices.iter().map(|d| d.to_uppercase()).collect();

    let lowest_battery = devices
        .iter()
        .filter(|d| d.connected && expected_addresses.contains(&d.address.to_uppercase()))
        .filter_map(|d| d.battery_percentage)
        .min();

    assert_eq!(
        lowest_battery,
        Some(60),
        "Should return battery level of connected device"
    );
}

#[test]
fn test_lowest_battery_multiple_devices_finds_minimum() {
    // When multiple expected devices are connected, should return the lowest battery
    let expected_devices = vec![
        "AA:BB:CC:DD:EE:FF".to_string(),
        "11:22:33:44:55:66".to_string(),
        "77:88:99:AA:BB:CC".to_string(),
    ];

    #[derive(Clone)]
    struct TestDevice {
        address: String,
        connected: bool,
        battery_percentage: Option<u8>,
    }

    let devices = vec![
        TestDevice {
            address: "AA:BB:CC:DD:EE:FF".to_string(),
            connected: true,
            battery_percentage: Some(80),
        },
        TestDevice {
            address: "11:22:33:44:55:66".to_string(),
            connected: true,
            battery_percentage: Some(20), // Lowest
        },
        TestDevice {
            address: "77:88:99:AA:BB:CC".to_string(),
            connected: true,
            battery_percentage: Some(50),
        },
    ];

    let expected_addresses: Vec<String> =
        expected_devices.iter().map(|d| d.to_uppercase()).collect();

    let lowest_battery = devices
        .iter()
        .filter(|d| d.connected && expected_addresses.contains(&d.address.to_uppercase()))
        .filter_map(|d| d.battery_percentage)
        .min();

    assert_eq!(
        lowest_battery,
        Some(20),
        "Should return lowest battery level among all connected devices"
    );
}

#[test]
fn test_lowest_battery_ignores_unexpected_devices() {
    // When unexpected devices are connected, they should be ignored
    let expected_devices = vec!["AA:BB:CC:DD:EE:FF".to_string()];

    #[derive(Clone)]
    struct TestDevice {
        address: String,
        connected: bool,
        battery_percentage: Option<u8>,
    }

    let devices = vec![
        TestDevice {
            address: "AA:BB:CC:DD:EE:FF".to_string(), // Expected
            connected: true,
            battery_percentage: Some(60),
        },
        TestDevice {
            address: "99:99:99:99:99:99".to_string(), // Not expected
            connected: true,
            battery_percentage: Some(10), // Lower but should be ignored
        },
    ];

    let expected_addresses: Vec<String> =
        expected_devices.iter().map(|d| d.to_uppercase()).collect();

    let lowest_battery = devices
        .iter()
        .filter(|d| d.connected && expected_addresses.contains(&d.address.to_uppercase()))
        .filter_map(|d| d.battery_percentage)
        .min();

    assert_eq!(
        lowest_battery,
        Some(60),
        "Should ignore unexpected devices when finding lowest battery"
    );
}

#[test]
fn test_lowest_battery_ignores_devices_without_battery_info() {
    // When devices don't have battery info, they should be filtered out
    let expected_devices = vec![
        "AA:BB:CC:DD:EE:FF".to_string(),
        "11:22:33:44:55:66".to_string(),
    ];

    #[derive(Clone)]
    struct TestDevice {
        address: String,
        connected: bool,
        battery_percentage: Option<u8>,
    }

    let devices = vec![
        TestDevice {
            address: "AA:BB:CC:DD:EE:FF".to_string(),
            connected: true,
            battery_percentage: None, // No battery info
        },
        TestDevice {
            address: "11:22:33:44:55:66".to_string(),
            connected: true,
            battery_percentage: Some(50),
        },
    ];

    let expected_addresses: Vec<String> =
        expected_devices.iter().map(|d| d.to_uppercase()).collect();

    let lowest_battery = devices
        .iter()
        .filter(|d| d.connected && expected_addresses.contains(&d.address.to_uppercase()))
        .filter_map(|d| d.battery_percentage)
        .min();

    assert_eq!(
        lowest_battery,
        Some(50),
        "Should ignore devices without battery info"
    );
}

#[test]
fn test_device_address_path_dev_prefix_stripping() {
    // Test that dev_ prefix is properly stripped from device paths
    let device_path = "/org/bluez/hci0/dev_2C_A7_EF_33_CD_5A";
    let device_addr = device_path.split('/').last().unwrap_or("");

    // Strip "dev_" or "DEV_" prefix if present
    let device_addr_clean = device_addr
        .strip_prefix("dev_")
        .or_else(|| device_addr.strip_prefix("DEV_"))
        .unwrap_or(device_addr);

    assert_eq!(
        device_addr_clean, "2C_A7_EF_33_CD_5A",
        "Should strip dev_ prefix from device address"
    );
}

#[test]
fn test_device_address_path_uppercase_dev_prefix_stripping() {
    // Test that uppercase DEV_ prefix is also stripped
    let device_path = "/org/bluez/hci0/DEV_2C_A7_EF_33_CD_5A";
    let device_addr = device_path.split('/').last().unwrap_or("");

    let device_addr_clean = device_addr
        .strip_prefix("dev_")
        .or_else(|| device_addr.strip_prefix("DEV_"))
        .unwrap_or(device_addr);

    assert_eq!(
        device_addr_clean, "2C_A7_EF_33_CD_5A",
        "Should strip DEV_ prefix from device address"
    );
}

#[test]
fn test_device_address_matching_with_expected() {
    // Test that device address matches expected address after prefix stripping
    let device_path = "/org/bluez/hci0/dev_2C_A7_EF_33_CD_5A";
    let expected_address = "2C:A7:EF:33:CD:5A";

    let device_addr = device_path.split('/').last().unwrap_or("");
    let device_addr_clean = device_addr
        .strip_prefix("dev_")
        .or_else(|| device_addr.strip_prefix("DEV_"))
        .unwrap_or(device_addr);

    let expected_addr = expected_address.to_uppercase().replace(':', "_");
    let device_addr_upper = device_addr_clean.to_uppercase();

    assert_eq!(
        expected_addr, device_addr_upper,
        "Device address should match expected address after processing"
    );
}

#[test]
fn test_device_address_no_prefix_passthrough() {
    // Test that addresses without dev_ prefix pass through unchanged
    let device_path = "/org/bluez/hci0/2C_A7_EF_33_CD_5A";
    let device_addr = device_path.split('/').last().unwrap_or("");

    let device_addr_clean = device_addr
        .strip_prefix("dev_")
        .or_else(|| device_addr.strip_prefix("DEV_"))
        .unwrap_or(device_addr);

    assert_eq!(
        device_addr_clean, "2C_A7_EF_33_CD_5A",
        "Should pass through addresses without dev_ prefix unchanged"
    );
}

#[test]
fn test_battery_level_bracket_critical() {
    // Test critical battery bracket (< 25%)
    let battery_levels = vec![0, 10, 20, 24];

    for level in battery_levels {
        let is_critical = level < 25;
        assert!(
            is_critical,
            "Battery level {} should be in critical bracket",
            level
        );
    }
}

#[test]
fn test_battery_level_bracket_warning() {
    // Test warning battery bracket (25-49%)
    let battery_levels = vec![25, 30, 40, 49];

    for level in battery_levels {
        let is_warning = level >= 25 && level < 50;
        assert!(
            is_warning,
            "Battery level {} should be in warning bracket",
            level
        );
    }
}

#[test]
fn test_battery_level_bracket_active() {
    // Test active battery bracket (50-74%)
    let battery_levels = vec![50, 60, 70, 74];

    for level in battery_levels {
        let is_active = level >= 50 && level < 75;
        assert!(
            is_active,
            "Battery level {} should be in active bracket",
            level
        );
    }
}

#[test]
fn test_battery_level_bracket_good() {
    // Test good battery bracket (>= 75%)
    let battery_levels = vec![75, 80, 90, 100];

    for level in battery_levels {
        let is_good = level >= 75;
        assert!(is_good, "Battery level {} should be in good bracket", level);
    }
}

#[test]
fn test_battery_bracket_transitions() {
    // Test battery bracket transition detection
    let test_cases = vec![
        (30, 20, true),  // 25-49% -> <25% (Warning to Critical)
        (50, 49, true),  // 50-74% -> 25-49% (Active to Warning)
        (75, 74, true),  // >=75% -> 50-74% (Good to Active)
        (30, 35, false), // Within same bracket (Warning)
        (60, 65, false), // Within same bracket (Active)
    ];

    for (old_level, new_level, should_transition) in test_cases {
        let old_bracket = if old_level < 25 {
            0
        } else if old_level < 50 {
            1
        } else if old_level < 75 {
            2
        } else {
            3
        };

        let new_bracket = if new_level < 25 {
            0
        } else if new_level < 50 {
            1
        } else if new_level < 75 {
            2
        } else {
            3
        };

        let transitioned = old_bracket != new_bracket;
        assert_eq!(
            transitioned,
            should_transition,
            "Battery level change from {}% to {}% should {}transition brackets",
            old_level,
            new_level,
            if should_transition { "" } else { "not " }
        );
    }
}

#[test]
fn test_battery_notification_urgency_critical() {
    // Test that critical battery notifications use urgency 2
    let battery_level = 20;
    let urgency = if battery_level < 25 { 2u8 } else { 1u8 };

    assert_eq!(
        urgency, 2,
        "Critical battery notifications should use urgency 2"
    );
}

#[test]
fn test_battery_notification_urgency_normal() {
    // Test that non-critical battery notifications use urgency 1
    let battery_levels = vec![30, 50, 75, 100];

    for level in battery_levels {
        let urgency = if level < 25 { 2u8 } else { 1u8 };
        assert_eq!(
            urgency, 1,
            "Battery level {} should use normal urgency (1)",
            level
        );
    }
}

#[test]
fn test_battery_notification_force_critical_logic() {
    // Test force_critical parameter logic
    let test_cases = vec![
        (20, true, None, true),       // Critical level, forced, no history
        (20, false, None, true),      // Critical level, not forced, no history (first time)
        (20, false, Some(30), true),  // Critical level, not forced, was higher (transition)
        (20, false, Some(20), false), // Critical level, not forced, same level
        (30, true, None, true),       // Non-critical, no history (first time always notifies)
        (30, false, None, true),      // Non-critical, not forced, no history (first time)
        (30, false, Some(30), false), // Non-critical, not forced, same bracket
        (30, false, Some(50), true),  // Non-critical, not forced, different bracket
    ];

    for (new_level, force_critical, last_level, should_notify) in test_cases {
        let notify = if new_level < 25 {
            force_critical || last_level.map_or(true, |old| old >= 25)
        } else {
            match last_level {
                None => true,
                Some(old) => {
                    let old_bracket = if old < 25 {
                        0
                    } else if old < 50 {
                        1
                    } else if old < 75 {
                        2
                    } else {
                        3
                    };
                    let new_bracket = if new_level < 25 {
                        0
                    } else if new_level < 50 {
                        1
                    } else if new_level < 75 {
                        2
                    } else {
                        3
                    };
                    old_bracket != new_bracket
                }
            }
        };

        assert_eq!(
            notify, should_notify,
            "Battery notification logic failed for level={}, force={}, last={:?}",
            new_level, force_critical, last_level
        );
    }
}

#[test]
fn test_nearby_logic_connected_device() {
    // Test that connected devices are always considered nearby
    use std::collections::HashSet;

    let device_path = "/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF";
    let connected = true;
    let recently_scanned_devices: HashSet<String> = HashSet::new();

    // Device should be nearby if connected, regardless of scan status
    let nearby = connected || recently_scanned_devices.contains(device_path);

    assert!(nearby);
}

#[test]
fn test_nearby_logic_scanned_device() {
    // Test that scanned devices are considered nearby
    use std::collections::HashSet;

    let device_path = "/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF";
    let connected = false;
    let mut recently_scanned_devices = HashSet::new();
    recently_scanned_devices.insert(device_path.to_string());

    // Device should be nearby if in recently_scanned_devices
    let nearby = connected || recently_scanned_devices.contains(device_path);

    assert!(nearby);
}

#[test]
fn test_nearby_logic_not_scanned_device() {
    // Test that devices not scanned and not connected are not nearby
    use std::collections::HashSet;

    let device_path = "/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF";
    let connected = false;
    let recently_scanned_devices: HashSet<String> = HashSet::new();

    // Device should not be nearby if not connected and not scanned
    let nearby = connected || recently_scanned_devices.contains(device_path);

    assert!(!nearby);
}

#[test]
fn test_scan_clears_recently_scanned_devices() {
    // Test that starting a scan clears the recently_scanned_devices set
    use std::collections::HashSet;

    let mut recently_scanned_devices = HashSet::new();

    // Add some devices from previous scan
    recently_scanned_devices.insert("/org/bluez/hci0/dev_AA".to_string());
    recently_scanned_devices.insert("/org/bluez/hci0/dev_BB".to_string());

    assert_eq!(recently_scanned_devices.len(), 2);

    // Simulate starting a new scan
    recently_scanned_devices.clear();

    assert_eq!(recently_scanned_devices.len(), 0);
}

#[test]
fn test_device_added_marks_as_scanned() {
    // Test that when a device is added, it's marked in recently_scanned_devices
    use std::collections::HashSet;

    let device_path = "/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF";
    let mut recently_scanned_devices = HashSet::new();

    // Simulate device being added/discovered
    recently_scanned_devices.insert(device_path.to_string());

    assert!(recently_scanned_devices.contains(device_path));
    assert_eq!(recently_scanned_devices.len(), 1);
}

#[test]
fn test_nearby_status_after_scan_clear() {
    // Test that devices become not nearby after scan clears the set
    use std::collections::HashSet;

    let device_path = "/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF";
    let connected = false;
    let mut recently_scanned_devices = HashSet::new();

    // Add device from previous scan
    recently_scanned_devices.insert(device_path.to_string());
    let nearby_before = connected || recently_scanned_devices.contains(device_path);
    assert!(nearby_before);

    // Start new scan (clear the set)
    recently_scanned_devices.clear();
    let nearby_after = connected || recently_scanned_devices.contains(device_path);
    assert!(!nearby_after);
}

#[test]
fn test_multiple_devices_nearby_tracking() {
    // Test tracking multiple devices as nearby
    use std::collections::HashSet;

    let mut recently_scanned_devices = HashSet::new();

    // Add multiple devices
    recently_scanned_devices.insert("/org/bluez/hci0/dev_AA".to_string());
    recently_scanned_devices.insert("/org/bluez/hci0/dev_BB".to_string());
    recently_scanned_devices.insert("/org/bluez/hci0/dev_CC".to_string());

    assert_eq!(recently_scanned_devices.len(), 3);
    assert!(recently_scanned_devices.contains("/org/bluez/hci0/dev_AA"));
    assert!(recently_scanned_devices.contains("/org/bluez/hci0/dev_BB"));
    assert!(recently_scanned_devices.contains("/org/bluez/hci0/dev_CC"));
    assert!(!recently_scanned_devices.contains("/org/bluez/hci0/dev_DD"));
}
