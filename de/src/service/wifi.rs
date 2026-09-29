use futures_util::StreamExt;
use glib::subclass::prelude::*;
use glib::Object;
use gtk4::glib;
use std::cell::RefCell;
use std::sync::OnceLock;
use zbus::Connection;

glib::wrapper! {
    pub struct Wifi(ObjectSubclass<imp::Wifi>);
}

impl Wifi {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn instance() -> Self {
        use std::cell::OnceCell;
        thread_local! {
            static INSTANCE: OnceCell<Wifi> = const { OnceCell::new() };
        }

        INSTANCE.with(|cell| cell.get_or_init(Self::new).clone())
    }

    pub fn connected(&self) -> bool {
        self.imp().connected.get()
    }

    pub fn ssid(&self) -> String {
        self.imp().ssid.borrow().clone()
    }

    pub fn strength(&self) -> u8 {
        self.imp().strength.get()
    }

    pub fn state(&self) -> WifiState {
        WifiState::from_u32(self.imp().state.get())
    }

    pub fn state_raw(&self) -> u32 {
        self.imp().state.get()
    }

    pub fn icon_name(&self) -> String {
        // If wifi is disabled, show disabled icon
        if !self.enabled() {
            return "network-wireless-disabled-symbolic".to_string();
        }

        match self.state() {
            WifiState::Activated => {
                let strength = self.strength();
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
            WifiState::Deactivating => "network-wireless-no-route-symbolic".to_string(),
            WifiState::Disconnected
            | WifiState::Unavailable
            | WifiState::Unmanaged
            | WifiState::Failed
            | WifiState::Unknown => "network-wireless-offline-symbolic".to_string(),
        }
    }

    pub fn connect(&self, ssid: Option<String>, password: Option<String>) {
        self.imp().connect_to_network(ssid, password);
    }

    pub fn disconnect(&self) {
        self.imp().disconnect_from_network();
    }

    pub fn scan(&self) {
        self.imp().request_scan();
    }

    pub fn access_points(&self) -> Vec<AccessPoint> {
        self.imp().access_points.borrow().clone()
    }

    pub fn scanning(&self) -> bool {
        self.imp().scanning.get()
    }

    pub fn enabled(&self) -> bool {
        self.imp().enabled.get()
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.imp().set_enabled(enabled);
    }

    pub fn validate_connection_security(&self, ap_path: &str) -> Option<String> {
        self.imp().validate_connection_security(ap_path)
    }
}

impl Default for Wifi {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiState {
    Unknown = 0,
    Unmanaged = 10,
    Unavailable = 20,
    Disconnected = 30,
    Preparing = 40,
    Config = 50,
    NeedAuth = 60,
    IpConfig = 70,
    IpCheck = 80,
    Secondaries = 90,
    Activated = 100,
    Deactivating = 110,
    Failed = 120,
}

impl WifiState {
    pub fn from_u32(value: u32) -> Self {
        match value {
            10 => WifiState::Unmanaged,
            20 => WifiState::Unavailable,
            30 => WifiState::Disconnected,
            40 => WifiState::Preparing,
            50 => WifiState::Config,
            60 => WifiState::NeedAuth,
            70 => WifiState::IpConfig,
            80 => WifiState::IpCheck,
            90 => WifiState::Secondaries,
            100 => WifiState::Activated,
            110 => WifiState::Deactivating,
            120 => WifiState::Failed,
            _ => WifiState::Unknown,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AccessPoint {
    pub ssid: String,
    pub strength: u8,
    pub path: String,
    pub requires_password: bool,
    pub has_saved_connection: bool,
    pub frequencies: Vec<u32>,
}

mod imp {
    use super::*;
    use glib::prelude::*;
    use glib::subclass::prelude::*;
    use glib::{ParamSpec, ParamSpecBuilderExt, Value};
    use std::cell::Cell;
    use zbus::proxy;

    pub struct Wifi {
        pub connected: Cell<bool>,
        pub ssid: RefCell<String>,
        pub strength: Cell<u8>,
        pub state: Cell<u32>,
        pub scanning: Cell<bool>,
        pub access_points: RefCell<Vec<super::AccessPoint>>,
        pub enabled: Cell<bool>,
    }

    impl Default for Wifi {
        fn default() -> Self {
            Self {
                connected: Cell::new(false),
                ssid: RefCell::new(String::new()),
                strength: Cell::new(0),
                state: Cell::new(0),
                scanning: Cell::new(false),
                access_points: RefCell::new(Vec::new()),
                enabled: Cell::new(false),
            }
        }
    }

    #[proxy(
        interface = "org.freedesktop.NetworkManager",
        default_service = "org.freedesktop.NetworkManager",
        default_path = "/org/freedesktop/NetworkManager"
    )]
    trait NetworkManager {
        #[zbus(property)]
        fn devices(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;

        #[zbus(property)]
        fn active_connections(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;

        #[zbus(property)]
        fn wireless_enabled(&self) -> zbus::Result<bool>;

        #[zbus(property)]
        fn set_wireless_enabled(&self, value: bool) -> zbus::Result<()>;

        fn activate_connection(
            &self,
            connection: zbus::zvariant::ObjectPath<'_>,
            device: zbus::zvariant::ObjectPath<'_>,
            specific_object: zbus::zvariant::ObjectPath<'_>,
        ) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

        fn deactivate_connection(
            &self,
            active_connection: zbus::zvariant::ObjectPath<'_>,
        ) -> zbus::Result<()>;

        fn add_and_activate_connection(
            &self,
            connection: std::collections::HashMap<
                String,
                std::collections::HashMap<String, zbus::zvariant::Value<'_>>,
            >,
            device: zbus::zvariant::ObjectPath<'_>,
            specific_object: zbus::zvariant::ObjectPath<'_>,
        ) -> zbus::Result<(
            zbus::zvariant::OwnedObjectPath,
            zbus::zvariant::OwnedObjectPath,
        )>;
    }

    #[proxy(
        interface = "org.freedesktop.NetworkManager.Device",
        default_service = "org.freedesktop.NetworkManager"
    )]
    trait Device {
        #[zbus(property)]
        fn device_type(&self) -> zbus::Result<u32>;

        #[zbus(property)]
        fn state(&self) -> zbus::Result<u32>;

        #[zbus(property)]
        fn active_connection(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

        #[zbus(property)]
        fn available_connections(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;
    }

    #[proxy(
        interface = "org.freedesktop.NetworkManager.Device.Wireless",
        default_service = "org.freedesktop.NetworkManager"
    )]
    trait WirelessDevice {
        #[zbus(property)]
        fn active_access_point(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

        #[zbus(property)]
        fn last_scan(&self) -> zbus::Result<i64>;

        fn get_all_access_points(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;

        fn request_scan(
            &self,
            options: std::collections::HashMap<String, zbus::zvariant::Value<'_>>,
        ) -> zbus::Result<()>;

        #[zbus(signal)]
        fn access_point_added(
            &self,
            access_point: zbus::zvariant::ObjectPath<'_>,
        ) -> zbus::Result<()>;

        #[zbus(signal)]
        fn access_point_removed(
            &self,
            access_point: zbus::zvariant::ObjectPath<'_>,
        ) -> zbus::Result<()>;
    }

    #[proxy(
        interface = "org.freedesktop.NetworkManager.AccessPoint",
        default_service = "org.freedesktop.NetworkManager"
    )]
    trait AccessPoint {
        #[zbus(property)]
        fn ssid(&self) -> zbus::Result<Vec<u8>>;

        #[zbus(property)]
        fn strength(&self) -> zbus::Result<u8>;

        #[zbus(property)]
        fn wpa_flags(&self) -> zbus::Result<u32>;

        #[zbus(property)]
        fn rsn_flags(&self) -> zbus::Result<u32>;

        #[zbus(property)]
        fn flags(&self) -> zbus::Result<u32>;

        #[zbus(property)]
        fn frequency(&self) -> zbus::Result<u32>;
    }

    #[proxy(
        interface = "org.freedesktop.NetworkManager.Connection.Active",
        default_service = "org.freedesktop.NetworkManager"
    )]
    trait ActiveConnection {
        #[zbus(property)]
        fn id(&self) -> zbus::Result<String>;

        #[zbus(property)]
        fn default(&self) -> zbus::Result<bool>;

        #[zbus(property)]
        fn state(&self) -> zbus::Result<u32>;

        #[zbus(property)]
        fn connection(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

        #[zbus(signal, name = "StateChanged")]
        fn state_changed_signal(&self, state: u32, reason: u32) -> zbus::Result<()>;
    }

    #[proxy(
        interface = "org.freedesktop.NetworkManager.Settings.Connection",
        default_service = "org.freedesktop.NetworkManager"
    )]
    trait SettingsConnection {
        fn get_settings(
            &self,
        ) -> zbus::Result<
            std::collections::HashMap<
                String,
                std::collections::HashMap<String, zbus::zvariant::OwnedValue>,
            >,
        >;

        fn delete(&self) -> zbus::Result<()>;
    }

    #[proxy(
        interface = "org.freedesktop.NetworkManager.Settings",
        default_service = "org.freedesktop.NetworkManager",
        default_path = "/org/freedesktop/NetworkManager/Settings"
    )]
    trait Settings {
        fn list_connections(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;
    }

    // NetworkManager device types
    const NM_DEVICE_TYPE_WIFI: u32 = 2;

    #[glib::object_subclass]
    impl ObjectSubclass for Wifi {
        const NAME: &'static str = "Wifi";
        type Type = super::Wifi;
    }

    impl ObjectImpl for Wifi {
        fn constructed(&self) {
            self.parent_constructed();

            // Initialize current state
            self.update_wifi_state();
            self.update_enabled_state();

            // Setup event listener
            self.setup_event_listener();
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: OnceLock<Vec<ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![
                    glib::ParamSpecBoolean::builder("connected")
                        .read_only()
                        .build(),
                    glib::ParamSpecString::builder("ssid").read_only().build(),
                    glib::ParamSpecUChar::builder("strength")
                        .read_only()
                        .build(),
                    glib::ParamSpecUInt::builder("state").read_only().build(),
                    glib::ParamSpecBoolean::builder("scanning")
                        .read_only()
                        .build(),
                    glib::ParamSpecBoxed::builder::<glib::Bytes>("access-points")
                        .read_only()
                        .build(),
                    glib::ParamSpecString::builder("icon-name")
                        .read_only()
                        .build(),
                    glib::ParamSpecBoolean::builder("enabled")
                        .read_only()
                        .build(),
                ]
            })
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "connected" => self.connected.get().to_value(),
                "ssid" => self.ssid.borrow().to_value(),
                "strength" => self.strength.get().to_value(),
                "state" => self.state.get().to_value(),
                "scanning" => self.scanning.get().to_value(),
                "access-points" => {
                    // Return empty bytes as placeholder - actual data accessed via method
                    glib::Bytes::from_static(&[]).to_value()
                }
                "icon-name" => self.obj().icon_name().to_value(),
                "enabled" => self.enabled.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl Wifi {
        async fn discover_wifi_devices(
            nm_proxy: &NetworkManagerProxy<'_>,
            connection: &Connection,
        ) -> Option<zbus::zvariant::OwnedObjectPath> {
            let devices = match nm_proxy.devices().await {
                Ok(d) => d,
                Err(_) => return None,
            };

            for device_path in devices {
                let device_proxy = match DeviceProxy::builder(connection).path(device_path.clone())
                {
                    Ok(builder) => match builder.build().await {
                        Ok(p) => p,
                        Err(_) => continue,
                    },
                    Err(_) => continue,
                };

                if let Ok(device_type) = device_proxy.device_type().await {
                    if device_type == NM_DEVICE_TYPE_WIFI {
                        return Some(device_path);
                    }
                }
            }
            None
        }

        async fn setup_property_streams(
            device_path: zbus::zvariant::OwnedObjectPath,
            connection: &Connection,
            obj: super::Wifi,
        ) {
            let device_proxy = match DeviceProxy::builder(connection).path(device_path.clone()) {
                Ok(builder) => match builder.build().await {
                    Ok(p) => p,
                    Err(_) => return,
                },
                Err(_) => return,
            };

            let wireless_proxy = match WirelessDeviceProxy::builder(connection).path(device_path) {
                Ok(builder) => match builder.build().await {
                    Ok(p) => p,
                    Err(_) => return,
                },
                Err(_) => return,
            };

            let nm_proxy = match NetworkManagerProxy::new(connection).await {
                Ok(p) => p,
                Err(_) => return,
            };

            let mut state_stream = device_proxy.receive_state_changed().await;
            let mut active_conn_stream = device_proxy.receive_active_connection_changed().await;
            let mut active_ap_stream = wireless_proxy.receive_active_access_point_changed().await;
            let mut wireless_enabled_stream = nm_proxy.receive_wireless_enabled_changed().await;
            let mut last_scan_stream = wireless_proxy.receive_last_scan_changed().await;

            let mut ap_added_stream = match wireless_proxy.receive_access_point_added().await {
                Ok(stream) => stream,
                Err(_) => return,
            };

            let mut ap_removed_stream = match wireless_proxy.receive_access_point_removed().await {
                Ok(stream) => stream,
                Err(_) => return,
            };

            loop {
                use futures_util::StreamExt;

                tokio::select! {
                    result = state_stream.next() => {
                        if result.is_some() {
                            let changes = obj.imp().update_wifi_state();
                            Self::notify_changes(&obj, changes);
                        } else {
                            break;
                        }
                    }
                    result = active_conn_stream.next() => {
                        if result.is_some() {
                            let changes = obj.imp().update_wifi_state();
                            Self::notify_changes(&obj, changes);
                        } else {
                            break;
                        }
                    }
                    result = active_ap_stream.next() => {
                        if result.is_some() {
                            let changes = obj.imp().update_wifi_state();
                            Self::notify_changes(&obj, changes);
                        } else {
                            break;
                        }
                    }
                    result = wireless_enabled_stream.next() => {
                        if result.is_some() {
                            obj.imp().update_enabled_state();
                        } else {
                            break;
                        }
                    }
                    result = last_scan_stream.next() => {
                        if result.is_some() {
                            // Scan completed, update scanning state to false
                            if obj.imp().scanning.get() {
                                obj.imp().scanning.set(false);
                                obj.notify("scanning");
                            }
                        } else {
                            break;
                        }
                    }
                    result = ap_added_stream.next() => {
                        if result.is_some() {
                            obj.imp().update_access_points_list();
                            obj.notify("access-points");
                        } else {
                            break;
                        }
                    }
                    result = ap_removed_stream.next() => {
                        if result.is_some() {
                            obj.imp().update_access_points_list();
                            obj.notify("access-points");
                        } else {
                            break;
                        }
                    }
                }
            }
        }

        fn setup_event_listener(&self) {
            let obj = self.obj().clone();

            // Initial population of access points
            self.update_access_points_list();

            glib::MainContext::default().spawn_local(async move {
                let connection = match Connection::system().await {
                    Ok(conn) => conn,
                    Err(_) => return,
                };

                let nm_proxy = match NetworkManagerProxy::new(&connection).await {
                    Ok(p) => p,
                    Err(_) => return,
                };

                let device_path = match Wifi::discover_wifi_devices(&nm_proxy, &connection).await {
                    Some(path) => path,
                    None => return,
                };

                Wifi::setup_property_streams(device_path, &connection, obj).await;
            });
        }

        fn notify_changes(obj: &super::Wifi, changes: WifiChanges) {
            if changes.connected_changed {
                obj.notify("connected");
            }
            if changes.ssid_changed {
                obj.notify("ssid");
            }
            if changes.strength_changed {
                obj.notify("strength");
                // Strength changes can also affect icon
                obj.notify("icon-name");
            }
            if changes.state_changed {
                obj.notify("state");
                obj.notify("icon-name");
            }
        }

        async fn find_active_wifi_device(
            devices: Vec<zbus::zvariant::OwnedObjectPath>,
            connection: &Connection,
        ) -> Option<(
            zbus::zvariant::OwnedObjectPath,
            u32,
            zbus::zvariant::OwnedObjectPath,
            bool,
        )> {
            for device_path in devices {
                let device_proxy = match DeviceProxy::builder(connection).path(device_path.clone())
                {
                    Ok(builder) => match builder.build().await {
                        Ok(p) => p,
                        Err(_) => continue,
                    },
                    Err(_) => continue,
                };

                let device_type = match device_proxy.device_type().await {
                    Ok(t) => t,
                    Err(_) => continue,
                };

                if device_type != NM_DEVICE_TYPE_WIFI {
                    continue;
                }

                let device_state = match device_proxy.state().await {
                    Ok(s) => s,
                    Err(_) => continue,
                };

                let active_conn_path = match device_proxy.active_connection().await {
                    Ok(p) => p,
                    Err(_) => continue,
                };

                if active_conn_path.as_str() == "/" || device_state < 40 {
                    continue;
                }

                let active_conn_proxy = match ActiveConnectionProxy::builder(connection)
                    .path(active_conn_path.clone())
                {
                    Ok(builder) => match builder.build().await {
                        Ok(p) => p,
                        Err(_) => continue,
                    },
                    Err(_) => continue,
                };

                let is_default = active_conn_proxy.default().await.unwrap_or_default();

                return Some((device_path, device_state, active_conn_path, is_default));
            }
            None
        }

        async fn get_wifi_connection_info(
            device_path: zbus::zvariant::OwnedObjectPath,
            connection: &Connection,
        ) -> Option<(String, u8)> {
            let wireless_proxy = match WirelessDeviceProxy::builder(connection).path(device_path) {
                Ok(builder) => match builder.build().await {
                    Ok(p) => p,
                    Err(_) => return None,
                },
                Err(_) => return None,
            };

            let ap_path = match wireless_proxy.active_access_point().await {
                Ok(p) => p,
                Err(_) => return None,
            };

            if ap_path.as_str() == "/" {
                return None;
            }

            let ap_proxy = match AccessPointProxy::builder(connection).path(ap_path) {
                Ok(builder) => match builder.build().await {
                    Ok(p) => p,
                    Err(_) => return None,
                },
                Err(_) => return None,
            };

            let ssid_bytes = match ap_proxy.ssid().await {
                Ok(s) => s,
                Err(_) => return None,
            };

            let ssid = String::from_utf8_lossy(&ssid_bytes).to_string();
            let signal_strength = ap_proxy.strength().await.unwrap_or_default();

            Some((ssid, signal_strength))
        }

        fn apply_wifi_state_changes(
            &self,
            device_state: u32,
            ssid: String,
            signal_strength: u8,
            changes: &mut WifiChanges,
        ) {
            let old_state = self.state.get();
            if device_state != old_state {
                self.state.set(device_state);
                changes.state_changed = true;
            }

            let old_ssid = self.ssid.borrow().clone();
            if ssid != old_ssid {
                self.ssid.replace(ssid);
                changes.ssid_changed = true;
            }

            let old_strength = self.strength.get();
            if signal_strength != old_strength {
                self.strength.set(signal_strength);
                changes.strength_changed = true;
            }

            let old_connected = self.connected.get();
            let new_connected = device_state == 100;
            if new_connected != old_connected {
                self.connected.set(new_connected);
                changes.connected_changed = true;
            }
        }

        fn handle_no_active_connection(&self, changes: &mut WifiChanges) {
            let old_connected = self.connected.get();
            if old_connected {
                self.connected.set(false);
                changes.connected_changed = true;
            }

            let old_state = self.state.get();
            if old_state != 30 {
                self.state.set(30);
                changes.state_changed = true;
            }

            let old_ssid = self.ssid.borrow().clone();
            if !old_ssid.is_empty() {
                self.ssid.replace(String::new());
                changes.ssid_changed = true;
            }

            let old_strength = self.strength.get();
            if old_strength != 0 {
                self.strength.set(0);
                changes.strength_changed = true;
            }
        }

        fn update_wifi_state(&self) -> WifiChanges {
            let mut changes = WifiChanges::default();

            let runtime = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(_) => return changes,
            };

            runtime.block_on(async {
                let connection = match Connection::system().await {
                    Ok(conn) => conn,
                    Err(_) => return,
                };

                let nm_proxy = match NetworkManagerProxy::new(&connection).await {
                    Ok(p) => p,
                    Err(_) => return,
                };

                let devices = match nm_proxy.devices().await {
                    Ok(d) => d,
                    Err(_) => return,
                };

                if let Some((device_path, device_state, _, _)) =
                    Self::find_active_wifi_device(devices, &connection).await
                {
                    if let Some((ssid, signal_strength)) =
                        Self::get_wifi_connection_info(device_path, &connection).await
                    {
                        self.apply_wifi_state_changes(
                            device_state,
                            ssid,
                            signal_strength,
                            &mut changes,
                        );
                    }
                } else {
                    self.handle_no_active_connection(&mut changes);
                }
            });

            changes
        }

        async fn find_wifi_device_by_ssid(
            devices: Vec<zbus::zvariant::OwnedObjectPath>,
            connection: &Connection,
        ) -> Option<zbus::zvariant::OwnedObjectPath> {
            for device_path in devices {
                let device_proxy = match DeviceProxy::builder(connection).path(device_path.clone())
                {
                    Ok(builder) => match builder.build().await {
                        Ok(p) => p,
                        Err(_) => continue,
                    },
                    Err(_) => continue,
                };

                let device_type = match device_proxy.device_type().await {
                    Ok(t) => t,
                    Err(_) => continue,
                };

                if device_type == NM_DEVICE_TYPE_WIFI {
                    return Some(device_path);
                }
            }
            None
        }

        async fn find_valid_connection_for_ap(
            connection: &Connection,
            ap_path: &str,
        ) -> Option<zbus::zvariant::OwnedObjectPath> {
            // Get the AP's SSID
            let ap_proxy = match AccessPointProxy::builder(connection).path(ap_path) {
                Ok(builder) => match builder.build().await {
                    Ok(p) => p,
                    Err(_) => return None,
                },
                Err(_) => return None,
            };

            let ap_ssid_bytes = match ap_proxy.ssid().await {
                Ok(s) => s,
                Err(_) => return None,
            };

            // Get all connections from Settings
            let settings_proxy = match SettingsProxy::new(connection).await {
                Ok(p) => p,
                Err(_) => return None,
            };

            let all_connections = match settings_proxy.list_connections().await {
                Ok(conns) => conns,
                Err(_) => return None,
            };

            // Check each connection to see if its SSID matches the AP's SSID
            for conn_path in all_connections {
                let conn_proxy =
                    match SettingsConnectionProxy::builder(connection).path(conn_path.clone()) {
                        Ok(builder) => match builder.build().await {
                            Ok(p) => p,
                            Err(_) => continue,
                        },
                        Err(_) => continue,
                    };

                let settings = match conn_proxy.get_settings().await {
                    Ok(s) => s,
                    Err(_) => continue,
                };

                // Check if this is a wifi connection
                if let Some(conn_settings) = settings.get("connection") {
                    if let Some(conn_type) = conn_settings.get("type") {
                        // Try to convert to string
                        let type_str: String = match conn_type.try_clone() {
                            Ok(v) => match <&str>::try_from(&v) {
                                Ok(s) => s.to_string(),
                                Err(_) => continue,
                            },
                            Err(_) => continue,
                        };
                        if type_str != "802-11-wireless" {
                            continue;
                        }
                    } else {
                        continue;
                    }
                } else {
                    continue;
                }

                // Check if the SSID matches
                if let Some(wireless_settings) = settings.get("802-11-wireless") {
                    if let Some(ssid_value) = wireless_settings.get("ssid") {
                        // SSID is stored as array of bytes
                        let conn_ssid_bytes: Vec<u8> = match ssid_value.try_clone() {
                            Ok(v) => match <Vec<u8>>::try_from(v) {
                                Ok(bytes) => bytes,
                                Err(_) => continue,
                            },
                            Err(_) => continue,
                        };

                        if conn_ssid_bytes == ap_ssid_bytes {
                            return Some(conn_path);
                        }
                    }
                }
            }

            None
        }

        async fn create_new_wifi_connection(
            nm_proxy: &NetworkManagerProxy<'_>,
            device_path: &zbus::zvariant::OwnedObjectPath,
            ap_path: &str,
            ssid_str: String,
            password: Option<String>,
        ) -> Result<(), Box<dyn std::error::Error>> {
            use std::collections::HashMap;
            use zbus::zvariant::Value;

            log::debug!(
                "WiFi: Creating new connection (SSID: '{}', has_password: {}, device: {}, ap: {})",
                ssid_str,
                password.is_some(),
                device_path,
                ap_path
            );

            let mut connection_settings: HashMap<String, HashMap<String, Value>> = HashMap::new();

            let mut wireless: HashMap<String, Value> = HashMap::new();
            wireless.insert("ssid".to_string(), Value::new(ssid_str.as_bytes()));
            connection_settings.insert("802-11-wireless".to_string(), wireless);

            if let Some(pwd) = password {
                let mut security: HashMap<String, Value> = HashMap::new();
                security.insert("key-mgmt".to_string(), Value::new("wpa-psk"));
                security.insert("psk".to_string(), Value::new(pwd));
                connection_settings.insert("802-11-wireless-security".to_string(), security);
                log::debug!("WiFi: Using WPA-PSK security for SSID '{}'", ssid_str);
            } else {
                log::debug!(
                    "WiFi: Creating open network connection for SSID '{}'",
                    ssid_str
                );
            }

            let mut connection: HashMap<String, Value> = HashMap::new();
            connection.insert("type".to_string(), Value::new("802-11-wireless"));
            connection.insert("id".to_string(), Value::new(ssid_str.clone()));
            connection_settings.insert("connection".to_string(), connection);

            log::info!(
                "WiFi: Calling add_and_activate_connection for SSID '{}'",
                ssid_str
            );
            let (connection_path, active_connection_path) = match nm_proxy
                .add_and_activate_connection(
                    connection_settings,
                    zbus::zvariant::ObjectPath::try_from(device_path.as_str())?,
                    zbus::zvariant::ObjectPath::try_from(ap_path)?,
                )
                .await
            {
                Ok(paths) => {
                    log::info!(
                        "WiFi: add_and_activate_connection succeeded \
                         (connection: {}, active_connection: {})",
                        paths.0,
                        paths.1
                    );
                    paths
                }
                Err(e) => {
                    log::error!(
                        "WiFi: add_and_activate_connection failed for SSID '{}': {}",
                        ssid_str,
                        e
                    );
                    return Err(e.into());
                }
            };

            // Monitor the active connection state to delete if it fails
            glib::MainContext::default().spawn_local(async move {
                let _ = Self::monitor_connection_state(
                    active_connection_path,
                    connection_path,
                    ssid_str,
                )
                .await;
            });

            Ok(())
        }

        async fn monitor_connection_state(
            active_conn_path: zbus::zvariant::OwnedObjectPath,
            conn_path: zbus::zvariant::OwnedObjectPath,
            ssid: String,
        ) -> Result<(), Box<dyn std::error::Error>> {
            log::debug!(
                "WiFi: Monitoring connection state for SSID '{}' (active_conn: {}, conn: {})",
                ssid,
                active_conn_path,
                conn_path
            );

            let connection = Connection::system().await?;

            let active_conn_proxy = ActiveConnectionProxy::builder(&connection)
                .path(active_conn_path.clone())?
                .build()
                .await?;

            // Check initial state
            let state = active_conn_proxy.state().await?;
            let state_name = Self::connection_state_name(state);
            log::info!(
                "WiFi: Initial connection state for SSID '{}': {} ({})",
                ssid,
                state,
                state_name
            );

            // State values: 0=unknown, 1=activating, 2=activated, 3=deactivating, 4=deactivated
            if state == 2 {
                log::info!("WiFi: Connection already activated for SSID '{}'", ssid);
                return Ok(());
            } else if state == 4 || state == 3 {
                // Failed or deactivating before activation
                // Note: We don't have access to the reason here since it's only in the signal
                log::warn!(
                    "WiFi: Connection already in failed/deactivating state for SSID '{}' \
                     (state: {} - {})",
                    ssid,
                    state,
                    state_name
                );

                // Send notification to user
                let notification_body = format!("Failed to connect to '{}'", ssid);
                glib::MainContext::default().spawn_local(async move {
                    let notifications = crate::service::notifications::Notifications::instance();
                    notifications.send_notification(
                        "rusty-de",
                        "WiFi Connection Failed",
                        &notification_body,
                    );
                });

                Self::delete_failed_connection(&connection, &conn_path, &ssid).await?;
                return Ok(());
            }

            // Monitor StateChanged signal which includes state AND reason
            let mut state_signal = active_conn_proxy.receive_state_changed_signal().await?;
            log::debug!("WiFi: Monitoring StateChanged signal for SSID '{}'", ssid);

            while let Some(signal) = state_signal.next().await {
                let args = signal.args()?;
                let state = args.state;
                let reason = args.reason;
                let state_name = Self::connection_state_name(state);
                let error_message = Self::state_reason_to_message(reason);

                log::info!(
                    "WiFi: Connection state changed for SSID '{}': {} ({}) with reason {} ({})",
                    ssid,
                    state,
                    state_name,
                    reason,
                    error_message
                );

                if state == 2 {
                    log::info!(
                        "WiFi: Connection successfully activated for SSID '{}'",
                        ssid
                    );
                    break;
                } else if state == 4 || state == 3 {
                    // Failed or deactivating before activation
                    log::warn!("WiFi: Connection failed for SSID '{}'", ssid);

                    // Send notification to user
                    let notification_body =
                        format!("Failed to connect to '{}': {}", ssid, error_message);
                    glib::MainContext::default().spawn_local(async move {
                        let notifications =
                            crate::service::notifications::Notifications::instance();
                        notifications.send_notification(
                            "rusty-de",
                            "WiFi Connection Failed",
                            &notification_body,
                        );
                    });

                    Self::delete_failed_connection(&connection, &conn_path, &ssid).await?;
                    break;
                }
            }

            Ok(())
        }

        fn connection_state_name(state: u32) -> &'static str {
            match state {
                0 => "unknown",
                1 => "activating",
                2 => "activated",
                3 => "deactivating",
                4 => "deactivated",
                _ => "invalid",
            }
        }

        fn state_reason_to_message(reason: u32) -> &'static str {
            match reason {
                0 => "Unknown error",
                1 => "Connection succeeded",
                2 => "User disconnected",
                3 => "Device disconnected",
                4 => "Service stopped",
                5 => "IP configuration invalid",
                6 => "IP configuration timeout",
                7 => "Service start timeout",
                8 => "Service start failed",
                9 => "No secrets (password required)",
                10 => "Login failed - check password",
                11 => "Connection removed",
                12 => "Dependency failed",
                13 => "Connection superseded",
                14 => "User service stopped",
                15 => "Carrier changed",
                16 => "Connection assumed",
                17 => "Supplicant available",
                18 => "Modem not found",
                19 => "Bluetooth failed",
                20 => "GSM/UMTS SIM not inserted",
                21 => "GSM/UMTS SIM PIN required",
                22 => "GSM/UMTS SIM PUK required",
                23 => "GSM/UMTS SIM wrong",
                24 => "InfiniBand mode",
                25 => "Connection dependency failed",
                26 => "Bridge controller failed",
                27 => "Modem manager unavailable",
                28 => "SSID not found",
                29 => "Secondary connection failed",
                30 => "DCB or FCoE setup failed",
                31 => "TeamD control failed",
                32 => "Modem failed or unavailable",
                33 => "Modem initialization failed",
                34 => "GSM APN selection failed",
                35 => "GSM registration denied",
                36 => "GSM registration timeout",
                37 => "GSM registration failed",
                38 => "GSM PIN check failed",
                39 => "Firmware missing",
                40 => "Device removed",
                41 => "Sleeping",
                42 => "Connection removed by user",
                43 => "Carrier/link changed",
                44 => "Connection sharing failed",
                45 => "Supplicant timeout",
                46 => "Supplicant disconnected",
                47 => "Supplicant configuration failed",
                48 => "Supplicant failed",
                49 => "Supplicant timeout",
                50 => "PPP start failed",
                51 => "PPP disconnected",
                52 => "PPP failed",
                53 => "DHCP start failed",
                54 => "DHCP error",
                55 => "DHCP failed",
                56 => "Shared start failed",
                57 => "Shared failed",
                58 => "AutoIP start failed",
                59 => "AutoIP error",
                60 => "AutoIP failed",
                61 => "Line busy",
                62 => "No dial tone",
                63 => "No carrier",
                64 => "Dial timeout",
                65 => "Dial failed",
                _ => "Connection failed",
            }
        }

        async fn monitor_existing_connection_state(
            active_conn_path: zbus::zvariant::OwnedObjectPath,
            ssid: String,
        ) -> Result<(), Box<dyn std::error::Error>> {
            log::debug!(
                "WiFi: Monitoring existing connection state for SSID '{}' (active_conn: {})",
                ssid,
                active_conn_path
            );

            let connection = Connection::system().await?;

            let active_conn_proxy = ActiveConnectionProxy::builder(&connection)
                .path(active_conn_path.clone())?
                .build()
                .await?;

            // Check initial state
            let state = active_conn_proxy.state().await?;
            let state_name = Self::connection_state_name(state);
            log::info!(
                "WiFi: Initial connection state for SSID '{}': {} ({})",
                ssid,
                state,
                state_name
            );

            // State values: 0=unknown, 1=activating, 2=activated, 3=deactivating, 4=deactivated
            if state == 2 {
                log::info!("WiFi: Connection already activated for SSID '{}'", ssid);
                return Ok(());
            } else if state == 4 || state == 3 {
                // Failed or deactivating before we started monitoring
                // Note: We don't have access to the reason here since it's only in the signal
                log::warn!(
                    "WiFi: Connection already in failed/deactivating state for SSID '{}' \
                     (state: {} - {})",
                    ssid,
                    state,
                    state_name
                );

                // Send notification to user
                let notification_body = format!("Failed to connect to '{}'", ssid);
                glib::MainContext::default().spawn_local(async move {
                    let notifications = crate::service::notifications::Notifications::instance();
                    notifications.send_notification(
                        "rusty-de",
                        "WiFi Connection Failed",
                        &notification_body,
                    );
                });

                return Ok(());
            }

            // Monitor StateChanged signal which includes state AND reason
            let mut state_signal = active_conn_proxy.receive_state_changed_signal().await?;
            log::debug!(
                "WiFi: Monitoring StateChanged signal for existing connection '{}'",
                ssid
            );

            while let Some(signal) = state_signal.next().await {
                let args = signal.args()?;
                let state = args.state;
                let reason = args.reason;
                let state_name = Self::connection_state_name(state);
                let error_message = Self::state_reason_to_message(reason);

                log::info!(
                    "WiFi: Connection state changed for SSID '{}': {} ({}) with reason {} ({})",
                    ssid,
                    state,
                    state_name,
                    reason,
                    error_message
                );

                if state == 2 {
                    log::info!(
                        "WiFi: Connection successfully activated for SSID '{}'",
                        ssid
                    );
                    break;
                } else if state == 4 || state == 3 {
                    // Connection failed or deactivating
                    log::warn!("WiFi: Connection failed for SSID '{}'", ssid);

                    // Send notification to user
                    let notification_body =
                        format!("Failed to connect to '{}': {}", ssid, error_message);
                    glib::MainContext::default().spawn_local(async move {
                        let notifications =
                            crate::service::notifications::Notifications::instance();
                        notifications.send_notification(
                            "rusty-de",
                            "WiFi Connection Failed",
                            &notification_body,
                        );
                    });

                    break;
                }
            }

            Ok(())
        }

        async fn delete_failed_connection(
            connection: &Connection,
            conn_path: &zbus::zvariant::OwnedObjectPath,
            ssid: &str,
        ) -> Result<(), Box<dyn std::error::Error>> {
            log::info!(
                "WiFi: Deleting failed connection for SSID '{}' at path: {}",
                ssid,
                conn_path
            );

            let conn_proxy = SettingsConnectionProxy::builder(connection)
                .path(conn_path.clone())?
                .build()
                .await?;

            match conn_proxy.delete().await {
                Ok(_) => {
                    log::info!(
                        "WiFi: Successfully deleted failed connection for SSID '{}'",
                        ssid
                    );
                }
                Err(e) => {
                    log::warn!(
                        "WiFi: Failed to delete connection for SSID '{}': {}",
                        ssid,
                        e
                    );
                }
            }

            Ok(())
        }

        async fn activate_existing_connection(
            nm_proxy: &NetworkManagerProxy<'_>,
            device_proxy: &DeviceProxy<'_>,
            device_path: &zbus::zvariant::OwnedObjectPath,
        ) -> Result<(), Box<dyn std::error::Error>> {
            let connections = device_proxy.available_connections().await?;

            if connections.is_empty() {
                log::warn!("WiFi: No available connections found on device");
                return Err("No available connections".into());
            }

            let connection_path = &connections[0];
            log::info!(
                "WiFi: Activating first available connection (path: {})",
                connection_path
            );

            match nm_proxy
                .activate_connection(
                    zbus::zvariant::ObjectPath::try_from(connection_path.as_str())?,
                    zbus::zvariant::ObjectPath::try_from(device_path.as_str())?,
                    zbus::zvariant::ObjectPath::try_from("/")?,
                )
                .await
            {
                Ok(active_conn) => {
                    log::info!(
                        "WiFi: Successfully activated connection, active connection: {}",
                        active_conn
                    );
                    Ok(())
                }
                Err(e) => {
                    log::error!("WiFi: Failed to activate connection: {}", e);
                    Err(e.into())
                }
            }
        }

        async fn find_ap_by_ssid(
            connection: &Connection,
            device_path: &zbus::zvariant::OwnedObjectPath,
            ssid: &str,
        ) -> Option<String> {
            let wireless_proxy =
                match WirelessDeviceProxy::builder(connection).path(device_path.clone()) {
                    Ok(builder) => match builder.build().await {
                        Ok(p) => p,
                        Err(_) => return None,
                    },
                    Err(_) => return None,
                };

            let ap_paths = match wireless_proxy.get_all_access_points().await {
                Ok(paths) => paths,
                Err(_) => return None,
            };

            for ap_path in ap_paths {
                let ap_proxy = match AccessPointProxy::builder(connection).path(ap_path.clone()) {
                    Ok(builder) => match builder.build().await {
                        Ok(p) => p,
                        Err(_) => continue,
                    },
                    Err(_) => continue,
                };

                let ssid_bytes = match ap_proxy.ssid().await {
                    Ok(s) => s,
                    Err(_) => continue,
                };

                let ap_ssid = String::from_utf8_lossy(&ssid_bytes).to_string();
                if ap_ssid == ssid {
                    return Some(ap_path.to_string());
                }
            }
            None
        }

        pub fn connect_to_network(&self, ssid: Option<String>, password: Option<String>) {
            log::info!(
                "WiFi: Attempting to connect to network (SSID: {}, has_password: {})",
                ssid.as_deref().unwrap_or("<using saved connection>"),
                password.is_some()
            );

            glib::MainContext::default().spawn_local(async move {
                let connection = match Connection::system().await {
                    Ok(conn) => conn,
                    Err(e) => {
                        log::error!("WiFi: Failed to connect to system D-Bus: {}", e);
                        return;
                    }
                };

                let nm_proxy = match NetworkManagerProxy::new(&connection).await {
                    Ok(p) => p,
                    Err(e) => {
                        log::error!("WiFi: Failed to create NetworkManager proxy: {}", e);
                        return;
                    }
                };

                let devices = match nm_proxy.devices().await {
                    Ok(d) => d,
                    Err(e) => {
                        log::error!("WiFi: Failed to get device list: {}", e);
                        return;
                    }
                };

                let device_path = match Self::find_wifi_device_by_ssid(devices, &connection).await {
                    Some(path) => {
                        log::debug!("WiFi: Found WiFi device at path: {}", path);
                        path
                    }
                    None => {
                        log::error!("WiFi: No WiFi device found");
                        return;
                    }
                };

                if let Some(ssid_str) = ssid {
                    // Find the access point by SSID
                    let ap_path =
                        match Self::find_ap_by_ssid(&connection, &device_path, &ssid_str).await {
                            Some(path) => {
                                log::info!(
                                    "WiFi: Found access point for SSID '{}' at path: {}",
                                    ssid_str,
                                    path
                                );
                                path
                            }
                            None => {
                                log::error!("WiFi: Access point not found for SSID '{}'", ssid_str);
                                return;
                            }
                        };

                    // Check if there's an existing valid connection for this AP
                    let existing_conn =
                        Self::find_valid_connection_for_ap(&connection, &ap_path).await;
                    if let Some(existing_conn) = existing_conn {
                        log::info!(
                            "WiFi: Found existing saved connection at path: {}",
                            existing_conn
                        );
                        let conn_path =
                            zbus::zvariant::ObjectPath::try_from(existing_conn.as_str()).unwrap();
                        let device_obj_path =
                            zbus::zvariant::ObjectPath::try_from(device_path.as_str()).unwrap();
                        let ap_obj_path =
                            zbus::zvariant::ObjectPath::try_from(ap_path.as_str()).unwrap();
                        log::info!(
                            "WiFi: Activating existing connection for SSID '{}'",
                            ssid_str
                        );
                        match nm_proxy
                            .activate_connection(conn_path, device_obj_path, ap_obj_path)
                            .await
                        {
                            Ok(active_conn_path) => {
                                log::info!(
                                    "WiFi: Successfully activated connection, \
                                     active connection path: {}",
                                    active_conn_path
                                );

                                // Monitor the connection state to see if it actually completes
                                let ssid_clone = ssid_str.clone();
                                glib::MainContext::default().spawn_local(async move {
                                    let _ = Self::monitor_existing_connection_state(
                                        active_conn_path,
                                        ssid_clone,
                                    )
                                    .await;
                                });
                            }
                            Err(e) => {
                                log::error!("WiFi: Failed to activate existing connection: {}", e);
                            }
                        }
                    } else {
                        log::info!(
                            "WiFi: No existing connection found, creating new connection \
                             for SSID '{}'",
                            ssid_str
                        );
                        match Self::create_new_wifi_connection(
                            &nm_proxy,
                            &device_path,
                            &ap_path,
                            ssid_str.clone(),
                            password,
                        )
                        .await
                        {
                            Ok(_) => {
                                log::info!(
                                    "WiFi: Successfully initiated new connection for SSID '{}'",
                                    ssid_str
                                );
                            }
                            Err(e) => {
                                log::error!("WiFi: Failed to create new connection: {}", e);
                            }
                        }
                    }
                } else {
                    log::info!(
                        "WiFi: No SSID provided, attempting to activate first available \
                         connection"
                    );
                    let device_proxy =
                        match DeviceProxy::builder(&connection).path(device_path.clone()) {
                            Ok(builder) => match builder.build().await {
                                Ok(p) => p,
                                Err(e) => {
                                    log::error!("WiFi: Failed to build device proxy: {}", e);
                                    return;
                                }
                            },
                            Err(e) => {
                                log::error!("WiFi: Failed to create device proxy builder: {}", e);
                                return;
                            }
                        };

                    match Self::activate_existing_connection(&nm_proxy, &device_proxy, &device_path)
                        .await
                    {
                        Ok(_) => {
                            log::info!("WiFi: Successfully activated first available connection");
                        }
                        Err(e) => {
                            log::error!("WiFi: Failed to activate existing connection: {}", e);
                        }
                    }
                }
            });
        }

        async fn find_device_with_active_connection(
            devices: Vec<zbus::zvariant::OwnedObjectPath>,
            connection: &Connection,
        ) -> Option<zbus::zvariant::OwnedObjectPath> {
            for device_path in devices {
                let device_proxy = match DeviceProxy::builder(connection).path(device_path.clone())
                {
                    Ok(builder) => match builder.build().await {
                        Ok(p) => p,
                        Err(_) => continue,
                    },
                    Err(_) => continue,
                };

                let device_type = match device_proxy.device_type().await {
                    Ok(t) => t,
                    Err(_) => continue,
                };

                if device_type != NM_DEVICE_TYPE_WIFI {
                    continue;
                }

                let active_conn_path = match device_proxy.active_connection().await {
                    Ok(p) => p,
                    Err(_) => continue,
                };

                if active_conn_path.as_str() != "/" {
                    return Some(active_conn_path);
                }
            }
            None
        }

        pub fn set_enabled(&self, enabled: bool) {
            log::info!("WiFi: Setting wireless enabled state to: {}", enabled);

            glib::MainContext::default().spawn_local(async move {
                let connection = match Connection::system().await {
                    Ok(conn) => conn,
                    Err(e) => {
                        log::error!("WiFi: Failed to connect to system D-Bus: {}", e);
                        return;
                    }
                };

                let nm_proxy = match NetworkManagerProxy::new(&connection).await {
                    Ok(p) => p,
                    Err(e) => {
                        log::error!("WiFi: Failed to create NetworkManager proxy: {}", e);
                        return;
                    }
                };

                match nm_proxy.set_wireless_enabled(enabled).await {
                    Ok(_) => {
                        log::info!("WiFi: Successfully set wireless enabled to: {}", enabled);
                    }
                    Err(e) => {
                        log::error!("WiFi: Failed to set wireless enabled: {}", e);
                    }
                }
            });
        }

        pub fn disconnect_from_network(&self) {
            log::info!("WiFi: Attempting to disconnect from network");

            glib::MainContext::default().spawn_local(async move {
                let connection = match Connection::system().await {
                    Ok(conn) => conn,
                    Err(e) => {
                        log::error!("WiFi: Failed to connect to system D-Bus: {}", e);
                        return;
                    }
                };

                let nm_proxy = match NetworkManagerProxy::new(&connection).await {
                    Ok(p) => p,
                    Err(e) => {
                        log::error!("WiFi: Failed to create NetworkManager proxy: {}", e);
                        return;
                    }
                };

                let devices = match nm_proxy.devices().await {
                    Ok(d) => d,
                    Err(e) => {
                        log::error!("WiFi: Failed to get device list: {}", e);
                        return;
                    }
                };

                let active_conn_path =
                    match Self::find_device_with_active_connection(devices, &connection).await {
                        Some(path) => {
                            log::info!("WiFi: Found active connection at path: {}", path);
                            path
                        }
                        None => {
                            log::warn!("WiFi: No active WiFi connection found to disconnect");
                            return;
                        }
                    };

                match nm_proxy
                    .deactivate_connection(
                        zbus::zvariant::ObjectPath::try_from(active_conn_path.as_str()).unwrap(),
                    )
                    .await
                {
                    Ok(_) => {
                        log::info!("WiFi: Successfully deactivated connection");
                    }
                    Err(e) => {
                        log::error!("WiFi: Failed to deactivate connection: {}", e);
                    }
                }
            });
        }

        pub fn request_scan(&self) {
            let obj = self.obj().clone();

            // Set scanning to true
            self.scanning.set(true);
            obj.notify("scanning");

            glib::MainContext::default().spawn_local(async move {
                let connection = match Connection::system().await {
                    Ok(conn) => conn,
                    Err(_) => {
                        obj.imp().scanning.set(false);
                        obj.notify("scanning");
                        return;
                    }
                };

                let nm_proxy = match NetworkManagerProxy::new(&connection).await {
                    Ok(p) => p,
                    Err(_) => {
                        obj.imp().scanning.set(false);
                        obj.notify("scanning");
                        return;
                    }
                };

                let devices = match nm_proxy.devices().await {
                    Ok(d) => d,
                    Err(_) => {
                        obj.imp().scanning.set(false);
                        obj.notify("scanning");
                        return;
                    }
                };

                let device_path = match Wifi::find_wifi_device_by_ssid(devices, &connection).await {
                    Some(path) => path,
                    None => {
                        obj.imp().scanning.set(false);
                        obj.notify("scanning");
                        return;
                    }
                };

                let wireless_proxy =
                    match WirelessDeviceProxy::builder(&connection).path(device_path) {
                        Ok(builder) => match builder.build().await {
                            Ok(p) => p,
                            Err(_) => {
                                obj.imp().scanning.set(false);
                                obj.notify("scanning");
                                return;
                            }
                        },
                        Err(_) => {
                            obj.imp().scanning.set(false);
                            obj.notify("scanning");
                            return;
                        }
                    };

                let last_scan = wireless_proxy.last_scan().await.unwrap_or(-1);

                match wireless_proxy
                    .request_scan(std::collections::HashMap::new())
                    .await
                {
                    Ok(_) => {
                        // Poll last_scan to detect when scan completes
                        glib::timeout_add_local(std::time::Duration::from_secs(1), {
                            let obj = obj.clone();
                            let wireless_proxy = wireless_proxy.clone();
                            move || {
                                let current_last_scan = match tokio::runtime::Runtime::new() {
                                    Ok(rt) => rt.block_on(async {
                                        wireless_proxy.last_scan().await.unwrap_or(-1)
                                    }),
                                    Err(_) => -1,
                                };

                                if current_last_scan == last_scan {
                                    return glib::ControlFlow::Continue;
                                }

                                obj.imp().scanning.set(false);
                                obj.notify("scanning");
                                glib::ControlFlow::Break
                            }
                        });
                    }
                    Err(_) => {
                        obj.imp().scanning.set(false);
                        obj.notify("scanning");
                    }
                }
            });
        }

        fn update_enabled_state(&self) {
            let runtime = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(_) => return,
            };

            runtime.block_on(async {
                let connection = match Connection::system().await {
                    Ok(conn) => conn,
                    Err(_) => return,
                };

                let nm_proxy = match NetworkManagerProxy::new(&connection).await {
                    Ok(p) => p,
                    Err(_) => return,
                };

                let wireless_enabled = match nm_proxy.wireless_enabled().await {
                    Ok(enabled) => enabled,
                    Err(_) => return,
                };

                let old_enabled = self.enabled.get();
                if wireless_enabled != old_enabled {
                    self.enabled.set(wireless_enabled);
                    self.obj().notify("enabled");
                }
            });
        }

        pub fn validate_connection_security(&self, ap_path: &str) -> Option<String> {
            log::info!(
                "WiFi: validate_connection_security called for AP path: {}",
                ap_path
            );

            let runtime = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(_) => {
                    log::error!("WiFi: Failed to create runtime for validation");
                    return Some("Failed to create runtime".to_string());
                }
            };

            runtime.block_on(async {
                let connection = match Connection::system().await {
                    Ok(conn) => conn,
                    Err(_) => return Some("Failed to connect to D-Bus".to_string()),
                };

                // Get AP security info
                let ap_proxy = match AccessPointProxy::builder(&connection).path(ap_path) {
                    Ok(builder) => match builder.build().await {
                        Ok(p) => p,
                        Err(_) => return Some("Failed to get access point info".to_string()),
                    },
                    Err(_) => return Some("Failed to create AP proxy".to_string()),
                };

                let wpa_flags = ap_proxy.wpa_flags().await.unwrap_or(0);
                let rsn_flags = ap_proxy.rsn_flags().await.unwrap_or(0);
                let ap_requires_password = wpa_flags != 0 || rsn_flags != 0;

                log::debug!(
                    "WiFi: AP security - wpa_flags: {}, rsn_flags: {}, requires_password: {}",
                    wpa_flags,
                    rsn_flags,
                    ap_requires_password
                );

                // Find saved connection for this AP
                let conn_path = match Self::find_valid_connection_for_ap(&connection, ap_path).await
                {
                    Some(path) => path,
                    None => {
                        // No saved connection, validation passes
                        // (user will be prompted for password)
                        return None;
                    }
                };

                // Get connection settings
                let conn_proxy =
                    match SettingsConnectionProxy::builder(&connection).path(conn_path.clone()) {
                        Ok(builder) => match builder.build().await {
                            Ok(p) => p,
                            Err(_) => {
                                return Some("Failed to get saved connection settings".to_string())
                            }
                        },
                        Err(_) => return Some("Failed to create connection proxy".to_string()),
                    };

                let settings = match conn_proxy.get_settings().await {
                    Ok(s) => s,
                    Err(_) => return Some("Failed to read connection settings".to_string()),
                };

                // Check if connection has security settings
                let conn_has_password = settings.contains_key("802-11-wireless-security");

                log::debug!(
                    "WiFi: Connection security - has_password: {}, AP requires: {}",
                    conn_has_password,
                    ap_requires_password
                );

                // Validate security match
                if ap_requires_password && !conn_has_password {
                    return Some(
                        "Network requires password but saved connection has none. \
                         Delete and reconnect."
                            .to_string(),
                    );
                }

                if !ap_requires_password && conn_has_password {
                    return Some(
                        "Network is open but saved connection has password. \
                         Delete and reconnect."
                            .to_string(),
                    );
                }

                None // Validation passed
            })
        }

        fn update_access_points_list(&self) {
            let runtime = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(_) => return,
            };

            runtime.block_on(async {
                let connection = match Connection::system().await {
                    Ok(conn) => conn,
                    Err(_) => return,
                };

                let nm_proxy = match NetworkManagerProxy::new(&connection).await {
                    Ok(p) => p,
                    Err(_) => return,
                };

                let devices = match nm_proxy.devices().await {
                    Ok(d) => d,
                    Err(_) => return,
                };

                let device_path = match Self::find_wifi_device_by_ssid(devices, &connection).await {
                    Some(path) => path,
                    None => return,
                };

                let wireless_proxy =
                    match WirelessDeviceProxy::builder(&connection).path(device_path) {
                        Ok(builder) => match builder.build().await {
                            Ok(p) => p,
                            Err(_) => return,
                        },
                        Err(_) => return,
                    };

                let ap_paths = match wireless_proxy.get_all_access_points().await {
                    Ok(paths) => paths,
                    Err(_) => return,
                };

                let mut access_points = Vec::new();
                for ap_path in ap_paths {
                    let ap_proxy =
                        match AccessPointProxy::builder(&connection).path(ap_path.clone()) {
                            Ok(builder) => match builder.build().await {
                                Ok(p) => p,
                                Err(_) => continue,
                            },
                            Err(_) => continue,
                        };

                    let ssid_bytes = match ap_proxy.ssid().await {
                        Ok(s) => s,
                        Err(_) => continue,
                    };

                    let ssid = String::from_utf8_lossy(&ssid_bytes).to_string();
                    if ssid.is_empty() {
                        continue;
                    }

                    let strength = ap_proxy.strength().await.unwrap_or_default();
                    let frequency = ap_proxy.frequency().await.unwrap_or_default();

                    // Check if password is required
                    let wpa_flags = ap_proxy.wpa_flags().await.unwrap_or_default();
                    let rsn_flags = ap_proxy.rsn_flags().await.unwrap_or_default();
                    let requires_password = wpa_flags != 0 || rsn_flags != 0;

                    // Check if there's a saved connection for this AP
                    let has_saved_connection =
                        Wifi::find_valid_connection_for_ap(&connection, &ap_path.to_string())
                            .await
                            .is_some();

                    access_points.push(super::AccessPoint {
                        ssid,
                        strength,
                        path: ap_path.to_string(),
                        requires_password,
                        has_saved_connection,
                        frequencies: vec![frequency],
                    });
                }

                // Merge duplicate SSIDs
                use std::collections::HashMap;
                let mut merged: HashMap<String, super::AccessPoint> = HashMap::new();

                for ap in access_points {
                    if let Some(existing) = merged.get_mut(&ap.ssid) {
                        // Merge with existing entry
                        existing.frequencies.push(ap.frequencies[0]);
                        // Keep the highest strength
                        if ap.strength > existing.strength {
                            existing.strength = ap.strength;
                            existing.path = ap.path;
                        }
                        // If any AP has saved connection, mark as having saved connection
                        if ap.has_saved_connection {
                            existing.has_saved_connection = true;
                        }
                    } else {
                        // First occurrence of this SSID
                        merged.insert(ap.ssid.clone(), ap);
                    }
                }

                // Convert back to Vec and sort by frequency for display
                let mut access_points: Vec<_> = merged.into_values().collect();

                // Sort frequencies within each AP for consistent display
                for ap in &mut access_points {
                    ap.frequencies.sort_unstable();
                }

                access_points.sort_by(|a, b| b.strength.cmp(&a.strength));
                self.access_points.replace(access_points);
            })
        }
    }

    #[derive(Default)]
    struct WifiChanges {
        connected_changed: bool,
        ssid_changed: bool,
        strength_changed: bool,
        state_changed: bool,
    }
}
