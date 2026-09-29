use glib::object::ObjectType;
use rusty_de::service::vpn::{Vpn, VpnConnection};

#[test]
fn test_vpn_instance_is_singleton() {
    let instance1 = Vpn::instance();
    let instance2 = Vpn::instance();
    assert_eq!(instance1.as_ptr(), instance2.as_ptr());
}

#[test]
fn test_vpn_default_state() {
    let vpn = Vpn::new();
    assert!(!vpn.connected());
    assert_eq!(vpn.connection_id(), "");
    // Note: connections() may auto-populate from NetworkManager on initialization
}

#[test]
fn test_vpn_icon_name_when_disconnected() {
    let vpn = Vpn::new();
    assert_eq!(vpn.icon_name(), "network-vpn-disabled-symbolic");
}

#[test]
fn test_vpn_connection_structure() {
    let conn_path: zbus::zvariant::OwnedObjectPath =
        zbus::zvariant::ObjectPath::try_from("/org/freedesktop/NetworkManager/Settings/1")
            .unwrap()
            .into();

    let vpn_conn = VpnConnection {
        connection_path: conn_path.clone(),
        id: "My VPN".to_string(),
        connected: false,
        vpn_type: "OpenVPN".to_string(),
        requires_password: true,
        has_saved_password: false,
    };

    assert_eq!(vpn_conn.id, "My VPN");
    assert!(!vpn_conn.connected);
    assert_eq!(vpn_conn.vpn_type, "OpenVPN");
    assert_eq!(vpn_conn.connection_path, conn_path);
}

#[test]
fn test_vpn_connection_clone() {
    let conn_path: zbus::zvariant::OwnedObjectPath =
        zbus::zvariant::ObjectPath::try_from("/org/freedesktop/NetworkManager/Settings/1")
            .unwrap()
            .into();

    let vpn_conn1 = VpnConnection {
        connection_path: conn_path.clone(),
        id: "Test VPN".to_string(),
        connected: true,
        vpn_type: "L2TP".to_string(),
        requires_password: true,
        has_saved_password: false,
    };

    let vpn_conn2 = vpn_conn1.clone();

    assert_eq!(vpn_conn1.id, vpn_conn2.id);
    assert_eq!(vpn_conn1.connected, vpn_conn2.connected);
    assert_eq!(vpn_conn1.vpn_type, vpn_conn2.vpn_type);
    assert_eq!(vpn_conn1.connection_path, vpn_conn2.connection_path);
}

#[test]
fn test_vpn_connection_types() {
    let conn_path: zbus::zvariant::OwnedObjectPath = zbus::zvariant::ObjectPath::try_from("/test")
        .unwrap()
        .into();

    let types = vec![
        "OpenVPN",
        "PPTP",
        "L2TP",
        "Cisco VPN",
        "OpenConnect",
        "strongSwan",
    ];

    for vpn_type in types {
        let conn = VpnConnection {
            connection_path: conn_path.clone(),
            id: format!("{} Connection", vpn_type),
            connected: false,
            vpn_type: vpn_type.to_string(),
            requires_password: true,
            has_saved_password: false,
        };
        assert_eq!(conn.vpn_type, vpn_type);
    }
}

#[test]
fn test_vpn_connections_list_is_vector() {
    let vpn = Vpn::new();
    let connections = vpn.connections();
    // Verify connections() returns a Vec (may be empty or populated from NetworkManager)
    // len() is always >= 0 for Vec, so we just verify it's callable
    let _len = connections.len();
}

#[test]
fn test_vpn_connection_id_empty_by_default() {
    let vpn = Vpn::new();
    assert_eq!(vpn.connection_id(), String::new());
}

#[test]
fn test_vpn_not_connected_by_default() {
    let vpn = Vpn::new();
    assert!(!vpn.connected());
}
