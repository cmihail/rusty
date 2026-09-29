use futures_util::StreamExt;
use glib::subclass::prelude::*;
use glib::Object;
use gtk4::glib;
use std::cell::{Cell, RefCell};
use std::sync::{Arc, Mutex, OnceLock};
use tokio::sync::oneshot;
use zbus::Connection;

glib::wrapper! {
    pub struct Bluetooth(ObjectSubclass<imp::Bluetooth>);
}

#[derive(Debug)]
struct PairingRequest {
    device_path: String,
    #[allow(dead_code)]
    device_name: String,
    passkey: Option<u32>,
    response_tx: oneshot::Sender<bool>,
}

type PairingRequestSender = Arc<Mutex<Option<async_channel::Sender<PairingRequest>>>>;

static PAIRING_REQUEST_TX: OnceLock<PairingRequestSender> = OnceLock::new();

impl Bluetooth {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn instance() -> Self {
        use std::cell::OnceCell;
        thread_local! {
            static INSTANCE: OnceCell<Bluetooth> = const { OnceCell::new() };
        }

        INSTANCE.with(|cell| cell.get_or_init(Self::new).clone())
    }

    pub fn enabled(&self) -> bool {
        self.imp().enabled.get()
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.imp().set_enabled(enabled);
    }

    pub fn icon_name(&self) -> String {
        if !self.enabled() {
            "bluetooth-disabled-symbolic".to_string()
        } else {
            let devices = self.devices();
            let has_connected_device = devices.iter().any(|d| d.connected);
            if has_connected_device {
                "bluetooth-active-symbolic".to_string()
            } else {
                "bluetooth-disconnected-symbolic".to_string()
            }
        }
    }

    pub fn has_missing_expected_devices(&self) -> bool {
        if !self.enabled() {
            return false;
        }

        let config = crate::config::Config::instance();
        let expected_devices = config.bluetooth_expected_devices();

        if expected_devices.is_empty() {
            return false;
        }

        let connected_devices = self.devices();
        let connected_addresses: Vec<String> = connected_devices
            .iter()
            .filter(|d| d.connected)
            .map(|d| d.address.to_uppercase())
            .collect();

        expected_devices
            .iter()
            .any(|expected| !connected_addresses.contains(&expected.address.to_uppercase()))
    }

    pub fn expected_devices_percentage(&self) -> f64 {
        let config = crate::config::Config::instance();
        let expected_devices = config.bluetooth_expected_devices();

        if expected_devices.is_empty() {
            return 1.0;
        }

        let connected_devices = self.devices();
        let connected_addresses: Vec<String> = connected_devices
            .iter()
            .filter(|d| d.connected)
            .map(|d| d.address.to_uppercase())
            .collect();

        let connected_count = expected_devices
            .iter()
            .filter(|expected| connected_addresses.contains(&expected.address.to_uppercase()))
            .count();

        connected_count as f64 / expected_devices.len() as f64
    }

    pub fn lowest_connected_expected_battery(&self) -> Option<u8> {
        let config = crate::config::Config::instance();
        let expected_devices = config.bluetooth_expected_devices();

        if expected_devices.is_empty() {
            return None;
        }

        let expected_addresses: Vec<String> = expected_devices
            .iter()
            .map(|d| d.address.to_uppercase())
            .collect();

        self.devices()
            .iter()
            .filter(|d| d.connected && expected_addresses.contains(&d.address.to_uppercase()))
            .filter_map(|d| d.battery_percentage)
            .min()
    }

    pub fn devices(&self) -> Vec<BluetoothDevice> {
        self.imp().devices.borrow().clone()
    }

    pub fn scanning(&self) -> bool {
        self.imp().scanning.get()
    }

    pub fn scan(&self) {
        self.imp().scan();
    }

    pub fn connect_device<F>(&self, device_path: &str, on_complete: F)
    where
        F: Fn(Result<(), String>) + 'static,
    {
        self.imp().connect_device(device_path, on_complete);
    }

    pub fn disconnect_device<F>(&self, device_path: &str, on_complete: F)
    where
        F: Fn(Result<(), String>) + 'static,
    {
        self.imp().disconnect_device(device_path, on_complete);
    }

    pub fn respond_to_pairing(&self, device_path: &str, approved: bool) {
        self.imp().respond_to_pairing(device_path, approved);
    }

    pub fn get_pairing_device(&self) -> Option<String> {
        self.imp().current_pairing_device.borrow().clone()
    }

    pub fn get_pairing_code(&self) -> u32 {
        self.imp().current_pairing_code.get()
    }

    pub fn get_connecting_device(&self) -> Option<String> {
        self.imp().connecting_device.borrow().clone()
    }

    pub fn get_device_error(&self, device_path: &str) -> Option<String> {
        self.imp().device_errors.borrow().get(device_path).cloned()
    }

    pub fn clear_device_error(&self, device_path: &str) {
        self.imp().device_errors.borrow_mut().remove(device_path);
    }

    pub fn clear_all_errors(&self) {
        self.imp().device_errors.borrow_mut().clear();
    }
}

#[derive(Debug, Clone)]
pub struct BluetoothDevice {
    pub path: String,
    pub name: String,
    pub address: String,
    pub connected: bool,
    pub paired: bool,
    pub battery_percentage: Option<u8>,
    pub nearby: bool,
}

impl Default for Bluetooth {
    fn default() -> Self {
        Self::new()
    }
}

mod imp {
    use super::*;
    use glib::prelude::*;
    use glib::subclass::prelude::*;
    use glib::{ParamSpec, ParamSpecBuilderExt, Value};
    use zbus::proxy;

    pub struct Bluetooth {
        pub enabled: Cell<bool>,
        pub devices: RefCell<Vec<super::BluetoothDevice>>,
        pub scanning: Cell<bool>,
        pub agent_connection: RefCell<Option<Connection>>,
        pub recently_paired_devices: RefCell<std::collections::HashSet<String>>,
        pub recently_scanned_devices: RefCell<std::collections::HashSet<String>>,
        pub monitored_devices: RefCell<std::collections::HashSet<String>>,
        pub pending_pairing_response: RefCell<Option<oneshot::Sender<bool>>>,
        pub current_pairing_device: RefCell<Option<String>>,
        pub current_pairing_code: Cell<u32>,
        pub connecting_device: RefCell<Option<String>>,
        pub device_errors: RefCell<std::collections::HashMap<String, String>>,
        pub last_notified_battery: RefCell<std::collections::HashMap<String, u8>>,
    }

    impl Default for Bluetooth {
        fn default() -> Self {
            Self {
                enabled: Cell::new(false),
                devices: RefCell::new(Vec::new()),
                scanning: Cell::new(false),
                agent_connection: RefCell::new(None),
                recently_paired_devices: RefCell::new(std::collections::HashSet::new()),
                recently_scanned_devices: RefCell::new(std::collections::HashSet::new()),
                monitored_devices: RefCell::new(std::collections::HashSet::new()),
                pending_pairing_response: RefCell::new(None),
                current_pairing_device: RefCell::new(None),
                current_pairing_code: Cell::new(0),
                connecting_device: RefCell::new(None),
                device_errors: RefCell::new(std::collections::HashMap::new()),
                last_notified_battery: RefCell::new(std::collections::HashMap::new()),
            }
        }
    }

    #[proxy(interface = "org.bluez.Adapter1", default_service = "org.bluez")]
    trait Adapter {
        #[zbus(property)]
        fn powered(&self) -> zbus::Result<bool>;

        #[zbus(property)]
        fn set_powered(&self, value: bool) -> zbus::Result<()>;

        #[zbus(property)]
        fn discovering(&self) -> zbus::Result<bool>;

        #[zbus(property)]
        fn pairable(&self) -> zbus::Result<bool>;

        #[zbus(property)]
        fn set_pairable(&self, value: bool) -> zbus::Result<()>;

        #[zbus(property)]
        fn pairable_timeout(&self) -> zbus::Result<u32>;

        #[zbus(property)]
        fn set_pairable_timeout(&self, value: u32) -> zbus::Result<()>;

        fn start_discovery(&self) -> zbus::Result<()>;

        fn stop_discovery(&self) -> zbus::Result<()>;

        fn remove_device(&self, device: zbus::zvariant::ObjectPath<'_>) -> zbus::Result<()>;
    }

    #[proxy(
        interface = "org.bluez.AgentManager1",
        default_service = "org.bluez",
        default_path = "/org/bluez"
    )]
    trait AgentManager {
        fn register_agent(
            &self,
            agent: zbus::zvariant::ObjectPath<'_>,
            capability: &str,
        ) -> zbus::Result<()>;

        fn unregister_agent(&self, agent: zbus::zvariant::ObjectPath<'_>) -> zbus::Result<()>;

        fn request_default_agent(&self, agent: zbus::zvariant::ObjectPath<'_>) -> zbus::Result<()>;
    }

    #[proxy(interface = "org.bluez.Device1", default_service = "org.bluez")]
    trait Device {
        #[zbus(property)]
        fn name(&self) -> zbus::Result<String>;

        #[zbus(property)]
        fn address(&self) -> zbus::Result<String>;

        #[zbus(property)]
        fn connected(&self) -> zbus::Result<bool>;

        #[zbus(property)]
        fn paired(&self) -> zbus::Result<bool>;

        #[zbus(property)]
        fn trusted(&self) -> zbus::Result<bool>;

        #[zbus(property)]
        fn set_trusted(&self, value: bool) -> zbus::Result<()>;

        #[zbus(property)]
        fn alias(&self) -> zbus::Result<String>;

        fn pair(&self) -> zbus::Result<()>;

        fn connect(&self) -> zbus::Result<()>;

        fn disconnect(&self) -> zbus::Result<()>;
    }

    #[proxy(interface = "org.bluez.Battery1", default_service = "org.bluez")]
    trait Battery {
        #[zbus(property)]
        fn percentage(&self) -> zbus::Result<u8>;
    }

    type InterfaceProperties = std::collections::HashMap<String, zbus::zvariant::OwnedValue>;
    type InterfaceMap = std::collections::HashMap<String, InterfaceProperties>;
    type ManagedObjects = std::collections::HashMap<zbus::zvariant::OwnedObjectPath, InterfaceMap>;

    #[proxy(
        interface = "org.freedesktop.DBus.ObjectManager",
        default_service = "org.bluez",
        default_path = "/"
    )]
    trait ObjectManager {
        fn get_managed_objects(&self) -> zbus::Result<ManagedObjects>;

        #[zbus(signal)]
        fn interfaces_added(
            &self,
            object_path: zbus::zvariant::ObjectPath<'_>,
            interfaces: std::collections::HashMap<
                String,
                std::collections::HashMap<String, zbus::zvariant::OwnedValue>,
            >,
        ) -> zbus::Result<()>;

        #[zbus(signal)]
        fn interfaces_removed(
            &self,
            object_path: zbus::zvariant::ObjectPath<'_>,
            interfaces: Vec<String>,
        ) -> zbus::Result<()>;
    }

    #[proxy(
        interface = "org.freedesktop.Notifications",
        default_service = "org.freedesktop.Notifications",
        default_path = "/org/freedesktop/Notifications"
    )]
    trait NotificationsDBus {
        #[allow(clippy::too_many_arguments)]
        fn notify(
            &self,
            app_name: &str,
            replaces_id: u32,
            app_icon: &str,
            summary: &str,
            body: &str,
            actions: &[&str],
            hints: std::collections::HashMap<&str, zbus::zvariant::Value<'_>>,
            expire_timeout: i32,
        ) -> zbus::Result<u32>;

        fn close_notification(&self, id: u32) -> zbus::Result<()>;

        #[zbus(signal)]
        fn action_invoked(&self, id: u32, action_key: String) -> zbus::Result<()>;

        #[zbus(signal)]
        fn notification_closed(&self, id: u32, reason: u32) -> zbus::Result<()>;
    }

    // BlueZ Agent implementation
    struct BluetoothAgent;

    impl BluetoothAgent {
        async fn request_user_confirmation(
            device_path: String,
            passkey: Option<u32>,
        ) -> zbus::fdo::Result<()> {
            // Get device name
            let device_name = Self::get_device_name(&device_path).await;

            let sender_lock = PAIRING_REQUEST_TX
                .get_or_init(|| Arc::new(Mutex::new(None)))
                .clone();

            let sender = {
                let lock = sender_lock.lock().unwrap();
                lock.clone()
            };

            if let Some(tx) = sender {
                let (response_tx, response_rx) = oneshot::channel();

                let request = super::PairingRequest {
                    device_path: device_path.clone(),
                    device_name,
                    passkey,
                    response_tx,
                };

                if tx.send(request).await.is_ok() {
                    match response_rx.await {
                        Ok(true) => {
                            return Ok(());
                        }
                        Ok(false) => {
                            return Err(zbus::fdo::Error::Failed(
                                "User denied pairing".to_string(),
                            ));
                        }
                        Err(_) => {
                            return Err(zbus::fdo::Error::Failed(
                                "Response channel closed".to_string(),
                            ));
                        }
                    }
                }
            }

            // Fallback: auto-deny if channel not set up
            Err(zbus::fdo::Error::Failed(
                "Pairing handler not initialized".to_string(),
            ))
        }

        async fn get_device_name(device_path: &str) -> String {
            let connection = match Connection::system().await {
                Ok(conn) => conn,
                Err(_) => return "Unknown Device".to_string(),
            };

            let device = match DeviceProxy::builder(&connection).path(device_path) {
                Ok(builder) => match builder.build().await {
                    Ok(d) => d,
                    Err(_) => return "Unknown Device".to_string(),
                },
                Err(_) => return "Unknown Device".to_string(),
            };

            if let Ok(alias) = device.alias().await {
                alias
            } else if let Ok(name) = device.name().await {
                name
            } else {
                "Unknown Device".to_string()
            }
        }

        async fn show_passkey_notification(device_name: &str, passkey: u32) {
            use std::collections::HashMap;
            use zbus::zvariant;

            let summary = "Bluetooth Pairing";
            let body = format!(
                "Pairing with {}\nEnter this passkey on the device:\n{:06}",
                device_name, passkey
            );

            let connection = match Connection::session().await {
                Ok(conn) => conn,
                Err(_) => {
                    return;
                }
            };

            let proxy = match NotificationsDBusProxy::new(&connection).await {
                Ok(p) => p,
                Err(_) => {
                    return;
                }
            };

            let mut hints = HashMap::new();
            hints.insert("urgency", zvariant::Value::U8(2)); // Critical

            let _ = proxy
                .notify(
                    "Bluetooth",
                    0,
                    "bluetooth-active-symbolic",
                    summary,
                    &body,
                    &[],
                    hints,
                    10000, // 10 second timeout
                )
                .await;
        }
    }

    #[zbus::interface(name = "org.bluez.Agent1")]
    impl BluetoothAgent {
        // Called when the service daemon needs to confirm a passkey for authentication
        async fn request_confirmation(
            &self,
            device: zbus::zvariant::ObjectPath<'_>,
            passkey: u32,
        ) -> zbus::fdo::Result<()> {
            Self::request_user_confirmation(device.to_string(), Some(passkey)).await
        }

        // Request authorization for incoming pairing attempt (just-works model)
        async fn request_authorization(
            &self,
            device: zbus::zvariant::ObjectPath<'_>,
        ) -> zbus::fdo::Result<()> {
            Self::request_user_confirmation(device.to_string(), None).await
        }

        // Display passkey with progress indicator
        async fn display_passkey(
            &self,
            device: zbus::zvariant::ObjectPath<'_>,
            passkey: u32,
            entered: u16,
        ) {
            // Show notification with the passkey
            if entered == 0 {
                let device_name = Self::get_device_name(&device.to_string()).await;
                Self::show_passkey_notification(&device_name, passkey).await;
            }
        }

        // Request passkey from user
        async fn request_passkey(
            &self,
            _device: zbus::zvariant::ObjectPath<'_>,
        ) -> zbus::fdo::Result<u32> {
            // For now, return error - we don't have UI for passkey input yet
            Err(zbus::fdo::Error::Failed(
                "Passkey input not supported yet".to_string(),
            ))
        }

        // Authorize service connection
        async fn authorize_service(
            &self,
            _device: zbus::zvariant::ObjectPath<'_>,
            _uuid: &str,
        ) -> zbus::fdo::Result<()> {
            // Auto-authorize common services
            Ok(())
        }

        // Called when agent is being removed
        fn release(&self) {}

        // Called to cancel any ongoing request
        fn cancel(&self) {}
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Bluetooth {
        const NAME: &'static str = "Bluetooth";
        type Type = super::Bluetooth;
    }

    impl ObjectImpl for Bluetooth {
        fn constructed(&self) {
            self.parent_constructed();

            // Setup pairing request handler
            self.setup_pairing_handler();

            // Register Bluetooth agent
            self.register_agent();

            // Initialize current state
            self.update_bluetooth_state();

            // Setup event listener
            self.setup_event_listener();

            // Discover devices
            self.discover_devices();

            // Check for critical battery levels on initialization
            self.check_initial_battery_levels();

            // Setup device event listener
            self.setup_device_listener();

            // Setup device property change listener
            self.setup_device_property_listener();

            // Setup config change listener
            self.setup_config_listener();
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: OnceLock<Vec<ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![
                    glib::ParamSpecBoolean::builder("enabled")
                        .read_only()
                        .build(),
                    glib::ParamSpecInt::builder("devices").read_only().build(),
                    glib::ParamSpecBoolean::builder("scanning")
                        .read_only()
                        .build(),
                    glib::ParamSpecBoolean::builder("missing-expected-devices")
                        .read_only()
                        .build(),
                ]
            })
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "enabled" => self.enabled.get().to_value(),
                "devices" => (self.devices.borrow().len() as i32).to_value(),
                "scanning" => self.scanning.get().to_value(),
                "missing-expected-devices" => self.obj().has_missing_expected_devices().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl Bluetooth {
        fn setup_pairing_handler(&self) {
            let (pairing_tx, pairing_rx) = async_channel::unbounded::<super::PairingRequest>();

            // Initialize the global pairing request sender
            let sender_lock = PAIRING_REQUEST_TX.get_or_init(|| Arc::new(Mutex::new(None)));
            *sender_lock.lock().unwrap() = Some(pairing_tx);

            // Spawn handler for pairing requests
            glib::spawn_future_local(async move {
                while let Ok(request) = pairing_rx.recv().await {
                    Self::handle_pairing_request(request).await;
                }
            });
        }

        async fn handle_pairing_request(request: super::PairingRequest) {
            let passkey = request.passkey.unwrap_or(0);
            let device_path = request.device_path.clone();

            // Store the response channel and pairing info
            let bt = super::Bluetooth::instance();
            *bt.imp().pending_pairing_response.borrow_mut() = Some(request.response_tx);
            *bt.imp().current_pairing_device.borrow_mut() = Some(device_path.clone());
            bt.imp().current_pairing_code.set(passkey);

            // Notify devices changed to trigger UI rebuild with pairing UI
            bt.notify("devices");
        }

        fn register_agent(&self) {
            let obj = self.obj().clone();
            glib::MainContext::default().spawn_local(async move {
                let connection = match Connection::system().await {
                    Ok(conn) => conn,
                    Err(_) => {
                        return;
                    }
                };

                // Create agent object path
                let agent_path = "/org/bluez/agent/rusty_de";

                // Create and register the agent interface
                let agent = BluetoothAgent;
                match connection.object_server().at(agent_path, agent).await {
                    Ok(_) => {}
                    Err(_) => {
                        return;
                    }
                }

                // Register agent with BlueZ AgentManager
                let agent_manager = match AgentManagerProxy::new(&connection).await {
                    Ok(am) => am,
                    Err(_) => {
                        return;
                    }
                };

                // Register with "KeyboardDisplay" capability for maximum compatibility
                match agent_manager
                    .register_agent(agent_path.try_into().unwrap(), "KeyboardDisplay")
                    .await
                {
                    Ok(_) => {}
                    Err(_) => {
                        return;
                    }
                }

                // Request to be the default agent
                let _ = agent_manager
                    .request_default_agent(agent_path.try_into().unwrap())
                    .await;

                // Store the connection to keep it alive
                obj.imp().agent_connection.replace(Some(connection));
            });
        }

        fn update_bluetooth_state(&self) {
            let obj = self.obj().clone();

            glib::MainContext::default().spawn_local(async move {
                let connection = match Connection::system().await {
                    Ok(conn) => conn,
                    Err(_) => return,
                };

                if let Some(adapter_path) = Self::find_adapter(&connection).await {
                    let adapter = match AdapterProxy::builder(&connection).path(adapter_path) {
                        Ok(builder) => match builder.build().await {
                            Ok(p) => p,
                            Err(_) => return,
                        },
                        Err(_) => return,
                    };

                    if let Ok(powered) = adapter.powered().await {
                        obj.imp().enabled.set(powered);
                        obj.notify("enabled");
                    }

                    if let Ok(discovering) = adapter.discovering().await {
                        obj.imp().scanning.set(discovering);
                        obj.notify("scanning");
                    }
                }
            });
        }

        fn setup_event_listener(&self) {
            let obj = self.obj().clone();

            glib::MainContext::default().spawn_local(async move {
                let connection = match Connection::system().await {
                    Ok(conn) => conn,
                    Err(_) => return,
                };

                let adapter_path = match Self::find_adapter(&connection).await {
                    Some(path) => path,
                    None => return,
                };

                Self::setup_property_stream(adapter_path, &connection, obj).await;
            });
        }

        async fn find_adapter(connection: &Connection) -> Option<zbus::zvariant::OwnedObjectPath> {
            // BlueZ adapters are typically at /org/bluez/hci0, /org/bluez/hci1, etc.
            // Try to find the first available adapter
            let adapter_path = "/org/bluez/hci0";

            match AdapterProxy::builder(connection).path(adapter_path) {
                Ok(builder) => match builder.build().await {
                    Ok(_) => Some(adapter_path.try_into().ok()?),
                    Err(_) => None,
                },
                Err(_) => None,
            }
        }

        async fn setup_property_stream(
            adapter_path: zbus::zvariant::OwnedObjectPath,
            connection: &Connection,
            obj: super::Bluetooth,
        ) {
            let adapter = match AdapterProxy::builder(connection).path(adapter_path) {
                Ok(builder) => match builder.build().await {
                    Ok(p) => p,
                    Err(_) => return,
                },
                Err(_) => return,
            };

            // Listen to Powered property changes
            let mut powered_stream = adapter.receive_powered_changed().await;

            // Listen to Discovering property changes
            let mut discovering_stream = adapter.receive_discovering_changed().await;

            loop {
                tokio::select! {
                    Some(change) = powered_stream.next() => {
                        if let Ok(powered) = change.get().await {
                            let old_enabled = obj.imp().enabled.get();
                            if old_enabled != powered {
                                obj.imp().enabled.set(powered);
                                obj.notify("enabled");
                                obj.notify("missing-expected-devices");
                            }
                        }
                    }
                    Some(change) = discovering_stream.next() => {
                        if let Ok(discovering) = change.get().await {
                            let old_scanning = obj.imp().scanning.get();
                            if old_scanning != discovering {
                                obj.imp().scanning.set(discovering);
                                obj.notify("scanning");
                            }
                        }
                    }
                }
            }
        }

        pub fn set_enabled(&self, enabled: bool) {
            let _obj = self.obj().clone();

            glib::MainContext::default().spawn_local(async move {
                let connection = match Connection::system().await {
                    Ok(conn) => conn,
                    Err(_) => return,
                };

                let adapter_path = match Self::find_adapter(&connection).await {
                    Some(path) => path,
                    None => return,
                };

                let adapter = match AdapterProxy::builder(&connection).path(adapter_path) {
                    Ok(builder) => match builder.build().await {
                        Ok(p) => p,
                        Err(_) => return,
                    },
                    Err(_) => return,
                };

                let _ = adapter.set_powered(enabled).await;
                // Property change will be picked up by the stream listener
            });
        }

        fn check_initial_battery_levels(&self) {
            let obj = self.obj().clone();

            // Delay to allow discover_devices to complete
            glib::timeout_add_local_once(std::time::Duration::from_millis(2000), move || {
                let devices = obj.devices();
                let config = crate::config::Config::instance();
                let expected_devices = config.bluetooth_expected_devices();

                if expected_devices.is_empty() {
                    return;
                }

                let expected_addresses: Vec<String> = expected_devices
                    .iter()
                    .map(|d| d.address.to_uppercase())
                    .collect();

                for device in devices {
                    // Check if this is an expected device
                    if !expected_addresses.contains(&device.address.to_uppercase()) {
                        continue;
                    }

                    // Only notify for connected devices with critical battery
                    if device.connected {
                        if let Some(battery_percentage) = device.battery_percentage {
                            if battery_percentage < 20 {
                                obj.imp().check_and_notify_battery(
                                    &device.path,
                                    &device.name,
                                    battery_percentage,
                                    true,
                                );
                            }
                        }
                    }
                }
            });
        }

        fn discover_devices(&self) {
            let obj = self.obj().clone();

            glib::MainContext::default().spawn_local(async move {
                let connection = match Connection::system().await {
                    Ok(conn) => conn,
                    Err(_) => {
                        return;
                    }
                };

                let object_manager = match ObjectManagerProxy::new(&connection).await {
                    Ok(om) => om,
                    Err(_) => {
                        return;
                    }
                };

                let objects = match object_manager.get_managed_objects().await {
                    Ok(objs) => objs,
                    Err(_) => {
                        return;
                    }
                };

                let mut devices = Vec::new();
                for (path, interfaces) in objects {
                    if interfaces.contains_key("org.bluez.Device1") {
                        if let Some(device) = Self::create_device(&connection, &path, &obj).await {
                            devices.push(device);

                            // Set up property monitoring for this device
                            Self::setup_device_monitoring(&connection, &path, &obj);
                        }
                    }
                }

                obj.imp().devices.replace(devices);
                obj.notify("devices");
                obj.notify("missing-expected-devices");
            });
        }

        fn setup_device_listener(&self) {
            let obj = self.obj().clone();
            glib::MainContext::default().spawn_local(async move {
                let (connection, object_manager) =
                    match Self::init_device_listener_connection().await {
                        Some(result) => result,
                        None => return,
                    };

                let (mut added_stream, mut removed_stream) =
                    match Self::create_device_event_streams(&object_manager).await {
                        Some(streams) => streams,
                        None => return,
                    };

                Self::process_device_events(
                    &mut added_stream,
                    &mut removed_stream,
                    &connection,
                    obj,
                )
                .await;
            });
        }

        async fn init_device_listener_connection(
        ) -> Option<(Connection, ObjectManagerProxy<'static>)> {
            let connection = match Connection::system().await {
                Ok(conn) => conn,
                Err(_) => {
                    return None;
                }
            };

            let object_manager = match ObjectManagerProxy::new(&connection).await {
                Ok(om) => om,
                Err(_) => {
                    return None;
                }
            };

            Some((connection, object_manager))
        }

        async fn create_device_event_streams(
            object_manager: &ObjectManagerProxy<'static>,
        ) -> Option<(
            InterfacesAddedStream<'static>,
            InterfacesRemovedStream<'static>,
        )> {
            let added_stream = match object_manager.receive_interfaces_added().await {
                Ok(stream) => stream,
                Err(_) => {
                    return None;
                }
            };

            let removed_stream = match object_manager.receive_interfaces_removed().await {
                Ok(stream) => stream,
                Err(_) => {
                    return None;
                }
            };

            Some((added_stream, removed_stream))
        }

        async fn process_device_events(
            added_stream: &mut InterfacesAddedStream<'static>,
            removed_stream: &mut InterfacesRemovedStream<'static>,
            connection: &Connection,
            obj: super::Bluetooth,
        ) {
            loop {
                tokio::select! {
                    Some(signal) = added_stream.next() => {
                        Self::handle_device_added(signal, connection, &obj).await;
                    }
                    Some(signal) = removed_stream.next() => {
                        Self::handle_device_removed(signal, &obj).await;
                    }
                }
            }
        }

        async fn handle_device_added(
            signal: InterfacesAdded,
            connection: &Connection,
            obj: &super::Bluetooth,
        ) {
            if let Ok(args) = signal.args() {
                let path = args.object_path.to_string();
                if args.interfaces.contains_key("org.bluez.Device1") {
                    // Mark device as recently scanned (it was just discovered)
                    obj.imp()
                        .recently_scanned_devices
                        .borrow_mut()
                        .insert(path.clone());

                    if let Some(device) = Self::create_device(connection, &path, obj).await {
                        let mut devices = obj.imp().devices.borrow_mut();
                        if !devices.iter().any(|d| d.path == device.path) {
                            devices.push(device.clone());
                            drop(devices);
                            obj.notify("devices");
                            obj.notify("missing-expected-devices");

                            // Set up property monitoring for this device
                            Self::setup_device_monitoring(connection, &path, obj);
                        }
                    }
                }
            }
        }

        async fn handle_device_removed(signal: InterfacesRemoved, obj: &super::Bluetooth) {
            if let Ok(args) = signal.args() {
                let path = args.object_path.to_string();
                if args.interfaces.iter().any(|i| *i == "org.bluez.Device1") {
                    let mut devices = obj.imp().devices.borrow_mut();
                    if let Some(pos) = devices.iter().position(|d| d.path == path) {
                        let _device = devices.remove(pos);
                        drop(devices);

                        // Remove from monitored devices set
                        obj.imp().monitored_devices.borrow_mut().remove(&path);

                        obj.notify("devices");
                        obj.notify("missing-expected-devices");
                    }
                }
            }
        }

        fn setup_device_property_listener(&self) {}

        fn setup_config_listener(&self) {
            let config = crate::config::Config::instance();
            let obj = self.obj().clone();

            config.connect_notify_local(
                Some("bluetooth-expected-devices"),
                glib::clone!(
                    #[weak]
                    obj,
                    move |_, _| {
                        obj.notify("missing-expected-devices");
                    }
                ),
            );
        }

        fn setup_device_monitoring(
            connection: &Connection,
            device_path: &str,
            obj: &super::Bluetooth,
        ) {
            // Check if already monitoring this device
            if obj.imp().monitored_devices.borrow().contains(device_path) {
                return;
            }

            // Add to monitored devices set
            obj.imp()
                .monitored_devices
                .borrow_mut()
                .insert(device_path.to_string());

            let obj_clone = obj.clone();
            let connection_clone = connection.clone();
            let device_path_clone = device_path.to_string();

            glib::MainContext::default().spawn_local(async move {
                Self::monitor_single_device_properties(
                    &connection_clone,
                    &device_path_clone,
                    &device_path_clone,
                    obj_clone,
                )
                .await;
            });
        }

        async fn monitor_single_device_properties(
            connection: &Connection,
            device_path_clone: &str,
            device_path: &str,
            obj: super::Bluetooth,
        ) {
            use zbus::fdo::PropertiesProxy;

            let builder = match PropertiesProxy::builder(connection).destination("org.bluez") {
                Ok(b) => b,
                Err(_) => return,
            };

            let builder = match builder.path(device_path_clone) {
                Ok(b) => b,
                Err(_) => return,
            };

            let properties_proxy = match builder.build().await {
                Ok(proxy) => proxy,
                Err(_) => return,
            };

            let mut changed_stream = match properties_proxy.receive_properties_changed().await {
                Ok(stream) => stream,
                Err(_) => return,
            };

            Self::process_property_changes(&mut changed_stream, connection, device_path, obj).await;
        }

        async fn process_property_changes(
            changed_stream: &mut zbus::fdo::PropertiesChangedStream<'_>,
            connection: &Connection,
            device_path: &str,
            obj: super::Bluetooth,
        ) {
            while let Some(signal) = changed_stream.next().await {
                if let Ok(args) = signal.args() {
                    if args.interface_name() == "org.bluez.Device1"
                        && (args.changed_properties().contains_key("Connected")
                            || args.changed_properties().contains_key("Paired"))
                    {
                        Self::handle_device_property_change(connection, device_path, &obj).await;
                    } else if args.interface_name() == "org.bluez.Battery1"
                        && args.changed_properties().contains_key("Percentage")
                    {
                        Self::handle_battery_property_change(connection, device_path, &obj).await;
                    }
                }
            }
        }

        async fn handle_battery_property_change(
            connection: &Connection,
            device_path: &str,
            obj: &super::Bluetooth,
        ) {
            let battery_percentage = Self::get_battery_percentage(connection, device_path).await;
            if let Some(battery_percentage) = battery_percentage {
                let mut devices = obj.imp().devices.borrow_mut();
                if let Some(pos) = devices.iter().position(|d| d.path == device_path) {
                    let device_name = devices[pos].name.clone();
                    devices[pos].battery_percentage = Some(battery_percentage);
                    drop(devices);
                    obj.notify("devices");

                    // Send notification on battery change (not forced)
                    obj.imp().check_and_notify_battery(
                        device_path,
                        &device_name,
                        battery_percentage,
                        false,
                    );
                }
            }
        }

        async fn handle_device_property_change(
            connection: &Connection,
            device_path: &str,
            obj: &super::Bluetooth,
        ) {
            // Mark device as recently scanned (property change indicates activity)
            obj.imp()
                .recently_scanned_devices
                .borrow_mut()
                .insert(device_path.to_string());

            if let Some(updated_device) = Self::create_device(connection, device_path, obj).await {
                let mut devices = obj.imp().devices.borrow_mut();
                if let Some(pos) = devices.iter().position(|d| d.path == device_path) {
                    let old_connected = devices[pos].connected;
                    devices[pos] = updated_device.clone();

                    if old_connected != updated_device.connected {
                        Self::handle_connection_state_change(
                            connection,
                            device_path,
                            &updated_device,
                            obj,
                        );

                        drop(devices);
                        Self::notify_if_not_connecting(device_path, obj);
                    }
                }
            }
        }

        fn handle_connection_state_change(
            connection: &Connection,
            device_path: &str,
            updated_device: &super::BluetoothDevice,
            obj: &super::Bluetooth,
        ) {
            if !updated_device.connected {
                Self::handle_device_disconnection(connection, device_path, obj);
            } else {
                Self::schedule_battery_check(connection, device_path, obj);

                // Send critical battery notification on device connection if battery < 20%
                if let Some(battery_percentage) = updated_device.battery_percentage {
                    if battery_percentage < 20 {
                        obj.imp().check_and_notify_battery(
                            device_path,
                            &updated_device.name,
                            battery_percentage,
                            true,
                        );
                    }
                }
            }
        }

        fn handle_device_disconnection(
            connection: &Connection,
            device_path: &str,
            obj: &super::Bluetooth,
        ) {
            let is_recently_paired = obj
                .imp()
                .recently_paired_devices
                .borrow()
                .contains(device_path);
            if is_recently_paired {
                Self::handle_recently_paired_disconnect(connection, device_path, obj);
            }
        }

        fn schedule_battery_check(
            connection: &Connection,
            device_path: &str,
            obj: &super::Bluetooth,
        ) {
            let device_path_battery = device_path.to_string();
            let connection_battery = connection.clone();
            let obj_battery = obj.clone();
            glib::timeout_add_local_once(std::time::Duration::from_millis(1000), move || {
                glib::MainContext::default().spawn_local(async move {
                    Self::refresh_battery_level(
                        &connection_battery,
                        &device_path_battery,
                        &obj_battery,
                    )
                    .await;
                });
            });
        }

        fn notify_if_not_connecting(device_path: &str, obj: &super::Bluetooth) {
            let is_connecting = obj
                .imp()
                .connecting_device
                .borrow()
                .as_ref()
                .map(|path| path == device_path)
                .unwrap_or(false);

            if !is_connecting {
                obj.notify("devices");
                obj.notify("missing-expected-devices");
            }
        }

        async fn refresh_battery_level(
            connection: &Connection,
            device_path: &str,
            obj: &super::Bluetooth,
        ) {
            let battery_percentage = Self::get_battery_percentage(connection, device_path).await;
            if let Some(battery_percentage) = battery_percentage {
                let mut devices = obj.imp().devices.borrow_mut();
                if let Some(pos) = devices.iter().position(|d| d.path == device_path) {
                    devices[pos].battery_percentage = Some(battery_percentage);
                    drop(devices);
                    obj.notify("devices");
                }
            }
        }

        fn handle_recently_paired_disconnect(
            connection: &Connection,
            device_path: &str,
            obj: &super::Bluetooth,
        ) {
            // Remove from recently paired set to avoid multiple reconnection attempts
            obj.imp()
                .recently_paired_devices
                .borrow_mut()
                .remove(device_path);

            // Spawn reconnection task
            let device_path_reconnect = device_path.to_string();
            let connection_clone = connection.clone();
            glib::MainContext::default().spawn_local(async move {
                // Wait a bit before reconnecting
                glib::timeout_future(std::time::Duration::from_millis(1500)).await;

                let device = match DeviceProxy::builder(&connection_clone)
                    .path(device_path_reconnect.as_str())
                {
                    Ok(builder) => match builder.build().await {
                        Ok(d) => d,
                        Err(_) => {
                            return;
                        }
                    },
                    Err(_) => {
                        return;
                    }
                };

                let _ = device.connect().await;
            });
        }

        fn determine_device_nearby(path: &str, connected: bool, obj: &super::Bluetooth) -> bool {
            // Device is nearby if:
            // 1. Currently connected, OR
            // 2. Seen in recent scan session
            connected || obj.imp().recently_scanned_devices.borrow().contains(path)
        }

        async fn create_device(
            connection: &Connection,
            path: &str,
            obj: &super::Bluetooth,
        ) -> Option<super::BluetoothDevice> {
            let device_proxy = match DeviceProxy::builder(connection).path(path) {
                Ok(builder) => match builder.build().await {
                    Ok(p) => p,
                    Err(_) => return None,
                },
                Err(_) => return None,
            };

            let name = if let Ok(alias) = device_proxy.alias().await {
                alias
            } else if let Ok(name) = device_proxy.name().await {
                name
            } else {
                "Unknown".to_string()
            };
            let address = device_proxy.address().await.unwrap_or_default();
            let connected = device_proxy.connected().await.unwrap_or(false);
            let paired = device_proxy.paired().await.unwrap_or(false);

            let battery_percentage = Self::get_battery_percentage(connection, path).await;

            let nearby = Self::determine_device_nearby(path, connected, obj);

            Some(super::BluetoothDevice {
                path: path.to_string(),
                name,
                address,
                connected,
                paired,
                battery_percentage,
                nearby,
            })
        }

        async fn get_battery_percentage(connection: &Connection, path: &str) -> Option<u8> {
            match BatteryProxy::builder(connection).path(path) {
                Ok(builder) => match builder.build().await {
                    Ok(battery_proxy) => battery_proxy.percentage().await.ok(),
                    Err(_) => None,
                },
                Err(_) => None,
            }
        }

        pub fn scan(&self) {
            let obj = self.obj().clone();

            // Clear all device errors when starting a new scan
            obj.imp().device_errors.borrow_mut().clear();

            // Clear recently scanned devices so only devices found in this scan are nearby
            obj.imp().recently_scanned_devices.borrow_mut().clear();
            obj.notify("devices");

            glib::MainContext::default().spawn_local(async move {
                let connection = match Connection::system().await {
                    Ok(conn) => conn,
                    Err(_) => {
                        return;
                    }
                };

                let adapter_path = match Self::find_adapter(&connection).await {
                    Some(path) => path,
                    None => {
                        return;
                    }
                };

                let adapter = match AdapterProxy::builder(&connection).path(adapter_path) {
                    Ok(builder) => match builder.build().await {
                        Ok(p) => p,
                        Err(_) => {
                            return;
                        }
                    },
                    Err(_) => {
                        return;
                    }
                };

                // Check if already discovering
                if let Ok(discovering) = adapter.discovering().await {
                    if discovering {
                        return;
                    }
                }

                // Start discovery
                if (adapter.start_discovery().await).is_ok() {
                    obj.imp().scanning.set(true);
                    obj.notify("scanning");

                    // Stop discovery after 10 seconds
                    glib::timeout_add_local_once(std::time::Duration::from_secs(10), move || {
                        glib::MainContext::default().spawn_local(async move {
                            if let Ok(still_discovering) = adapter.discovering().await {
                                if still_discovering {
                                    let _ = adapter.stop_discovery().await;
                                }
                            }
                            obj.imp().scanning.set(false);
                            obj.notify("scanning");
                        });
                    });
                }
            });
        }

        pub fn connect_device<F>(&self, device_path: &str, on_complete: F)
        where
            F: Fn(Result<(), String>) + 'static,
        {
            let device_path = device_path.to_string();
            let obj = self.obj().clone();

            // Set connecting state
            *obj.imp().connecting_device.borrow_mut() = Some(device_path.clone());
            obj.notify("devices");

            glib::MainContext::default().spawn_local(async move {
                let result = Self::connect_device_async(&device_path).await;

                match &result {
                    Ok(_) => {
                        // Clear any previous error for this device on success
                        obj.imp().device_errors.borrow_mut().remove(&device_path);
                    }
                    Err(e) => {
                        // Store error for persistent display
                        obj.imp()
                            .device_errors
                            .borrow_mut()
                            .insert(device_path.clone(), e.clone());
                    }
                }

                // Call completion callback BEFORE clearing state and notifying
                // This allows error to be displayed in the UI before widget is rebuilt
                on_complete(result);

                // Clear connecting state
                *obj.imp().connecting_device.borrow_mut() = None;
                obj.notify("devices");
                obj.notify("missing-expected-devices");
            });
        }

        async fn connect_device_async(device_path: &str) -> Result<(), String> {
            let connection = Self::get_system_connection().await?;
            let device = Self::get_device_proxy(&connection, device_path).await?;
            Self::ensure_device_connected(&device, device_path).await?;
            Ok(())
        }

        async fn get_system_connection() -> Result<Connection, String> {
            Connection::system()
                .await
                .map_err(|e| format!("Failed to connect to system bus: {}", e))
        }

        async fn get_device_proxy<'a>(
            connection: &'a Connection,
            device_path: &'a str,
        ) -> Result<DeviceProxy<'a>, String> {
            let builder = DeviceProxy::builder(connection)
                .path(device_path)
                .map_err(|e| format!("Failed to create device proxy: {}", e))?;

            builder
                .build()
                .await
                .map_err(|e| format!("Failed to build device proxy: {}", e))
        }

        async fn ensure_device_connected(
            device: &DeviceProxy<'_>,
            device_path: &str,
        ) -> Result<(), String> {
            if device.connected().await.unwrap_or(false) {
                return Ok(());
            }

            // Check if device is paired
            let is_paired = device.paired().await.unwrap_or(false);

            if !is_paired {
                // Set adapter as pairable before pairing (like blueman does)
                let _ = Self::set_adapter_pairable(true).await;

                match device.pair().await {
                    Ok(_) => {
                        // Reset adapter pairable state
                        let _ = Self::set_adapter_pairable(false).await;

                        // After pairing, wait a moment and check if already connected
                        // (some devices auto-connect after pairing)
                        glib::timeout_future(std::time::Duration::from_millis(500)).await;

                        if device.connected().await.unwrap_or(false) {
                            Self::set_device_trusted(device).await;

                            // Add to recently paired devices set for auto-reconnect on disconnect
                            let obj = super::Bluetooth::instance();
                            obj.imp()
                                .recently_paired_devices
                                .borrow_mut()
                                .insert(device_path.to_string());

                            return Ok(());
                        }
                    }
                    Err(e) => {
                        // Reset adapter pairable state on error
                        let _ = Self::set_adapter_pairable(false).await;
                        return Err(format!("Failed to pair: {}", e));
                    }
                }
            }

            // Now try to connect (if not already connected after pairing)
            match device.connect().await {
                Ok(_) => {
                    Self::set_device_trusted(device).await;
                    Ok(())
                }
                Err(e) => Err(format!("Failed to connect: {}", e)),
            }
        }

        async fn set_adapter_pairable(pairable: bool) -> Result<(), String> {
            let connection = Connection::system()
                .await
                .map_err(|e| format!("Failed to connect to system bus: {}", e))?;

            let adapter_path = Self::find_adapter(&connection)
                .await
                .ok_or_else(|| "No adapter found".to_string())?;

            let adapter = AdapterProxy::builder(&connection)
                .path(adapter_path)
                .map_err(|e| format!("Failed to create adapter proxy: {}", e))?
                .build()
                .await
                .map_err(|e| format!("Failed to build adapter proxy: {}", e))?;

            // Set pairable
            adapter
                .set_pairable(pairable)
                .await
                .map_err(|e| format!("Failed to set pairable: {}", e))?;

            // Set pairable timeout (0 = no timeout)
            let timeout = 0;
            adapter
                .set_pairable_timeout(timeout)
                .await
                .map_err(|e| format!("Failed to set pairable timeout: {}", e))?;

            Ok(())
        }

        async fn set_device_trusted(device: &DeviceProxy<'_>) {
            let _ = device.set_trusted(true).await;
        }

        pub fn disconnect_device<F>(&self, device_path: &str, on_complete: F)
        where
            F: Fn(Result<(), String>) + 'static,
        {
            let device_path = device_path.to_string();

            glib::MainContext::default().spawn_local(async move {
                Self::disconnect_device_async(&device_path, on_complete).await;
            });
        }

        pub fn respond_to_pairing(&self, device_path: &str, approved: bool) {
            let _ = device_path; // device_path not needed, but keeping signature consistent
            if let Some(tx) = self.pending_pairing_response.borrow_mut().take() {
                let _ = tx.send(approved);
            }
            // Clear pairing state
            *self.current_pairing_device.borrow_mut() = None;
            self.current_pairing_code.set(0);
            // Notify to rebuild UI without pairing UI
            self.obj().notify("devices");
        }

        async fn disconnect_device_async<F>(device_path: &str, on_complete: F)
        where
            F: Fn(Result<(), String>),
        {
            let connection = match Self::get_system_connection().await {
                Ok(conn) => conn,
                Err(e) => {
                    on_complete(Err(e));
                    return;
                }
            };

            let device = match Self::get_device_proxy(&connection, device_path).await {
                Ok(dev) => dev,
                Err(e) => {
                    on_complete(Err(e));
                    return;
                }
            };

            if let Err(e) = Self::ensure_device_disconnected(&device, device_path).await {
                on_complete(Err(e));
                return;
            }

            on_complete(Ok(()));
        }

        async fn ensure_device_disconnected(
            device: &DeviceProxy<'_>,
            _device_path: &str,
        ) -> Result<(), String> {
            if !device.connected().await.unwrap_or(true) {
                return Ok(());
            }

            device
                .disconnect()
                .await
                .map_err(|e| format!("Failed to disconnect device: {}", e))?;
            Ok(())
        }

        async fn send_battery_notification(device_name: &str, battery_level: u8) {
            use std::collections::HashMap;
            use zbus::zvariant;

            let (urgency, level_text) = if battery_level < 20 {
                (2, "Critical battery level")
            } else if battery_level < 40 {
                (1, "Low battery level")
            } else if battery_level < 60 {
                (1, "Medium battery level")
            } else {
                (1, "Good battery level")
            };

            let summary = "Bluetooth Device Battery";
            let body = format!("{}: {} ({}%)", device_name, level_text, battery_level);

            let connection = match Connection::session().await {
                Ok(conn) => conn,
                Err(_) => return,
            };

            let proxy = match NotificationsDBusProxy::new(&connection).await {
                Ok(p) => p,
                Err(_) => return,
            };

            let mut hints = HashMap::new();
            hints.insert("urgency", zvariant::Value::U8(urgency));

            let _ = proxy
                .notify(
                    "Bluetooth",
                    0,
                    "bluetooth-active-symbolic",
                    summary,
                    &body,
                    &[],
                    hints,
                    5000,
                )
                .await;
        }

        fn check_and_notify_battery(
            &self,
            device_path: &str,
            device_name: &str,
            new_level: u8,
            force_critical: bool,
        ) {
            let config = crate::config::Config::instance();
            let expected_devices = config.bluetooth_expected_devices();

            // Only notify for expected devices
            let device_addr = device_path.split('/').next_back().unwrap_or("");

            // Strip "dev_" or "DEV_" prefix if present
            let device_addr_clean = device_addr
                .strip_prefix("dev_")
                .or_else(|| device_addr.strip_prefix("DEV_"))
                .unwrap_or(device_addr);

            let is_expected = expected_devices.iter().any(|d| {
                let expected_addr = d.address.to_uppercase().replace(':', "_");
                let device_addr_upper = device_addr_clean.to_uppercase();
                expected_addr == device_addr_upper
            });

            if !is_expected {
                return;
            }

            let last_level = self
                .last_notified_battery
                .borrow()
                .get(device_path)
                .copied();

            let should_notify = if new_level < 20 {
                // For critical level: notify if forced OR crossing into critical bracket
                force_critical || last_level.is_none_or(|old| old >= 20)
            } else {
                // For non-critical: only notify when crossing bracket boundaries
                match last_level {
                    None => true, // First time seeing this device
                    Some(old) => {
                        let old_bracket = if old < 20 {
                            0
                        } else if old < 40 {
                            1
                        } else if old < 60 {
                            2
                        } else {
                            3
                        };
                        let new_bracket = if new_level < 20 {
                            0
                        } else if new_level < 40 {
                            1
                        } else if new_level < 60 {
                            2
                        } else {
                            3
                        };
                        old_bracket != new_bracket
                    }
                }
            };

            if should_notify {
                self.last_notified_battery
                    .borrow_mut()
                    .insert(device_path.to_string(), new_level);

                let device_name = device_name.to_string();
                glib::MainContext::default().spawn_local(async move {
                    Self::send_battery_notification(&device_name, new_level).await;
                });
            }
        }
    }
}
