use rusty_de::service::hyprland::*;

#[test]
fn test_monitor_creation() {
    let workspace = Workspace {
        id: 1,
        name: "1".to_string(),
    };

    let monitor = Monitor {
        id: 0,
        name: "DP-1".to_string(),
        model: "LG Monitor".to_string(),
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
        active_workspace: workspace.clone(),
    };

    assert_eq!(monitor.id, 0);
    assert_eq!(monitor.name, "DP-1");
    assert_eq!(monitor.model, "LG Monitor");
    assert_eq!(monitor.x, 0);
    assert_eq!(monitor.y, 0);
    assert_eq!(monitor.width, 1920);
    assert_eq!(monitor.height, 1080);
    assert_eq!(monitor.active_workspace.id, 1);
}

#[test]
fn test_monitor_with_offset() {
    let workspace = Workspace {
        id: 2,
        name: "2".to_string(),
    };

    let monitor = Monitor {
        id: 1,
        name: "HDMI-A-1".to_string(),
        model: "Samsung Monitor".to_string(),
        x: 1920,
        y: 0,
        width: 2560,
        height: 1440,
        active_workspace: workspace,
    };

    assert_eq!(monitor.x, 1920);
    assert_eq!(monitor.width, 2560);
}

#[test]
fn test_workspace_creation() {
    let workspace = Workspace {
        id: 5,
        name: "5".to_string(),
    };

    assert_eq!(workspace.id, 5);
    assert_eq!(workspace.name, "5");
}

#[test]
fn test_workspace_named() {
    let workspace = Workspace {
        id: 10,
        name: "special:scratchpad".to_string(),
    };

    assert_eq!(workspace.id, 10);
    assert_eq!(workspace.name, "special:scratchpad");
}

#[test]
fn test_client_creation() {
    let workspace = Workspace {
        id: 1,
        name: "1".to_string(),
    };

    let client = Client {
        address: "abcd1234".to_string(),
        x: 100,
        y: 200,
        width: 800,
        height: 600,
        title: "Firefox".to_string(),
        class: "firefox".to_string(),
        workspace,
        monitor: 0,
    };

    assert_eq!(client.address, "abcd1234");
    assert_eq!(client.x, 100);
    assert_eq!(client.y, 200);
    assert_eq!(client.width, 800);
    assert_eq!(client.height, 600);
    assert_eq!(client.title, "Firefox");
    assert_eq!(client.class, "firefox");
}

#[test]
fn test_client_fullscreen() {
    let workspace = Workspace {
        id: 1,
        name: "1".to_string(),
    };

    let client = Client {
        address: "xyz789".to_string(),
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
        title: "VLC".to_string(),
        class: "vlc".to_string(),
        workspace,
        monitor: 0,
    };

    assert_eq!(client.x, 0);
    assert_eq!(client.y, 0);
    assert_eq!(client.width, 1920);
    assert_eq!(client.height, 1080);
}

#[test]
fn test_monitor_clone() {
    let workspace = Workspace {
        id: 1,
        name: "1".to_string(),
    };

    let original = Monitor {
        id: 0,
        name: "DP-1".to_string(),
        model: "Test Monitor".to_string(),
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
        active_workspace: workspace,
    };

    let cloned = original.clone();
    assert_eq!(cloned.id, original.id);
    assert_eq!(cloned.name, original.name);
    assert_eq!(cloned.model, original.model);
    assert_eq!(cloned.active_workspace.id, original.active_workspace.id);
}

#[test]
fn test_workspace_clone() {
    let original = Workspace {
        id: 3,
        name: "3".to_string(),
    };

    let cloned = original.clone();
    assert_eq!(cloned.id, original.id);
    assert_eq!(cloned.name, original.name);
}

#[test]
fn test_client_clone() {
    let workspace = Workspace {
        id: 1,
        name: "1".to_string(),
    };

    let original = Client {
        address: "test123".to_string(),
        x: 10,
        y: 20,
        width: 400,
        height: 300,
        title: "Test Window".to_string(),
        class: "test".to_string(),
        workspace,
        monitor: 0,
    };

    let cloned = original.clone();
    assert_eq!(cloned.address, original.address);
    assert_eq!(cloned.x, original.x);
    assert_eq!(cloned.y, original.y);
    assert_eq!(cloned.width, original.width);
    assert_eq!(cloned.height, original.height);
    assert_eq!(cloned.title, original.title);
    assert_eq!(cloned.class, original.class);
}

#[test]
fn test_monitor_equality() {
    let workspace = Workspace {
        id: 1,
        name: "1".to_string(),
    };

    let mon1 = Monitor {
        id: 0,
        name: "DP-1".to_string(),
        model: "Monitor".to_string(),
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
        active_workspace: workspace.clone(),
    };

    let mon2 = Monitor {
        id: 0,
        name: "DP-1".to_string(),
        model: "Monitor".to_string(),
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
        active_workspace: workspace,
    };

    assert_eq!(mon1, mon2);
}

#[test]
fn test_workspace_equality() {
    let ws1 = Workspace {
        id: 1,
        name: "1".to_string(),
    };

    let ws2 = Workspace {
        id: 1,
        name: "1".to_string(),
    };

    assert_eq!(ws1, ws2);
}

#[test]
fn test_client_equality() {
    let workspace = Workspace {
        id: 1,
        name: "1".to_string(),
    };

    let client1 = Client {
        address: "abc".to_string(),
        x: 0,
        y: 0,
        width: 100,
        height: 100,
        title: "Test".to_string(),
        class: "test".to_string(),
        workspace: workspace.clone(),
        monitor: 0,
    };

    let client2 = Client {
        address: "abc".to_string(),
        x: 0,
        y: 0,
        width: 100,
        height: 100,
        title: "Test".to_string(),
        class: "test".to_string(),
        workspace,
        monitor: 0,
    };

    assert_eq!(client1, client2);
}

#[test]
fn test_monitor_different_workspaces() {
    let ws1 = Workspace {
        id: 1,
        name: "1".to_string(),
    };

    let ws2 = Workspace {
        id: 2,
        name: "2".to_string(),
    };

    let mon1 = Monitor {
        id: 0,
        name: "DP-1".to_string(),
        model: "Monitor".to_string(),
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
        active_workspace: ws1,
    };

    let mon2 = Monitor {
        id: 0,
        name: "DP-1".to_string(),
        model: "Monitor".to_string(),
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
        active_workspace: ws2,
    };

    assert_ne!(mon1, mon2);
}

#[test]
fn test_client_address_without_0x_prefix() {
    let workspace = Workspace {
        id: 1,
        name: "1".to_string(),
    };

    let client = Client {
        address: "1234abcd".to_string(),
        x: 0,
        y: 0,
        width: 800,
        height: 600,
        title: "Test".to_string(),
        class: "test".to_string(),
        workspace,
        monitor: 0,
    };

    // Address should not have 0x prefix
    assert!(!client.address.starts_with("0x"));
}

#[test]
fn test_address_arg_restores_0x_prefix() {
    assert_eq!(address_arg("1234abcd"), "address:0x1234abcd");
}

// Note: Service tests are skipped because they require Hyprland to be running
// and would conflict when running tests in parallel. The Hyprland service connects
// to Hyprland sockets on construction which requires an active Hyprland session.
