use glib::subclass::prelude::*;
use glib::Object;
use gtk4::glib;
use std::cell::RefCell;
use std::sync::OnceLock;
use zbus::Connection;

#[derive(Debug, Clone)]
pub struct EthernetDevice {
    pub device_path: zbus::zvariant::OwnedObjectPath,
    pub id: String,
    pub product: String,
    pub connected: bool,
    pub is_default: bool,
}

glib::wrapper! {
    pub struct Ethernet(ObjectSubclass<imp::Ethernet>);
}

impl Ethernet {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn instance() -> Self {
        use std::cell::OnceCell;
        thread_local! {
            static INSTANCE: OnceCell<Ethernet> = const { OnceCell::new() };
        }

        INSTANCE.with(|cell| cell.get_or_init(Self::new).clone())
    }

    pub fn connected(&self) -> bool {
        self.imp().connected.get()
    }

    pub fn connection_id(&self) -> String {
        self.imp().connection_id.borrow().clone()
    }

    pub fn device_product(&self) -> String {
        self.imp().device_product.borrow().clone()
    }

    pub fn state(&self) -> EthernetState {
        EthernetState::from_u32(self.imp().state.get())
    }

    pub fn state_raw(&self) -> u32 {
        self.imp().state.get()
    }

    pub fn devices(&self) -> Vec<EthernetDevice> {
        self.imp().devices.borrow().clone()
    }

    pub fn icon_name(&self) -> String {
        match self.state() {
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
        }
    }

    pub fn connect<F>(&self, on_error: F)
    where
        F: Fn(String) + 'static,
    {
        let device_path = self.imp().device_path.borrow().clone();

        glib::MainContext::default().spawn_local(async move {
            if let Some(path) = device_path {
                if let Err(e) = Self::connect_device_async(&path).await {
                    on_error(format!("Failed to connect: {}", e));
                }
            } else {
                on_error("No ethernet device available".to_string());
            }
        });
    }

    pub fn disconnect<F>(&self, on_error: F)
    where
        F: Fn(String) + 'static,
    {
        let device_path = self.imp().device_path.borrow().clone();

        glib::MainContext::default().spawn_local(async move {
            if let Some(path) = device_path {
                if let Err(e) = Self::disconnect_device_async(&path).await {
                    on_error(format!("Failed to disconnect: {}", e));
                }
            } else {
                on_error("No ethernet device available".to_string());
            }
        });
    }

    pub fn connect_device<F>(&self, device_path: &zbus::zvariant::OwnedObjectPath, on_error: F)
    where
        F: Fn(String) + 'static,
    {
        let path = device_path.clone();

        glib::MainContext::default().spawn_local(async move {
            if let Err(e) = Self::connect_device_async(&path).await {
                on_error(format!("Failed to connect: {}", e));
            }
        });
    }

    pub fn disconnect_device<F>(&self, device_path: &zbus::zvariant::OwnedObjectPath, on_error: F)
    where
        F: Fn(String) + 'static,
    {
        let path = device_path.clone();

        glib::MainContext::default().spawn_local(async move {
            if let Err(e) = Self::disconnect_device_async(&path).await {
                on_error(format!("Failed to disconnect: {}", e));
            }
        });
    }

    pub fn connect_all<F>(&self, on_error: F)
    where
        F: Fn(String) + 'static + Clone,
    {
        let devices = self.devices();

        for device in devices {
            let error_handler = on_error.clone();
            self.connect_device(&device.device_path, move |err| {
                error_handler(err);
            });
        }
    }

    pub fn disconnect_all<F>(&self, on_error: F)
    where
        F: Fn(String) + 'static + Clone,
    {
        let devices = self.devices();

        for device in devices {
            let error_handler = on_error.clone();
            self.disconnect_device(&device.device_path, move |err| {
                error_handler(err);
            });
        }
    }

    async fn connect_device_async(
        device_path: &zbus::zvariant::OwnedObjectPath,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let connection = Connection::system().await?;
        let nm_proxy = imp::NetworkManagerProxy::new(&connection).await?;

        let device_proxy = imp::DeviceProxy::builder(&connection)
            .path(device_path.clone())?
            .build()
            .await?;

        // Check if already connected
        let active_conn_path = device_proxy.active_connection().await?;
        if active_conn_path.as_str() != "/" {
            return Ok(()); // Already connected
        }

        // Get available connections
        let connections = Self::get_available_connections(&connection, device_path).await?;
        if connections.is_empty() {
            return Err("No available connections".into());
        }

        // Activate first available connection
        Self::activate_connection(&connection, &nm_proxy, &connections[0], device_path).await?;
        Ok(())
    }

    async fn disconnect_device_async(
        device_path: &zbus::zvariant::OwnedObjectPath,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let connection = Connection::system().await?;
        let nm_proxy = imp::NetworkManagerProxy::new(&connection).await?;

        let device_proxy = imp::DeviceProxy::builder(&connection)
            .path(device_path.clone())?
            .build()
            .await?;

        let active_conn_path = device_proxy.active_connection().await?;
        if active_conn_path.as_str() == "/" {
            return Ok(()); // Not connected
        }

        // Deactivate connection
        Self::deactivate_connection(&connection, &nm_proxy, &active_conn_path).await?;
        Ok(())
    }

    async fn get_available_connections(
        connection: &Connection,
        device_path: &zbus::zvariant::OwnedObjectPath,
    ) -> Result<Vec<zbus::zvariant::OwnedObjectPath>, Box<dyn std::error::Error>> {
        #[zbus::proxy(
            interface = "org.freedesktop.NetworkManager.Device",
            default_service = "org.freedesktop.NetworkManager"
        )]
        trait DeviceConnections {
            #[zbus(property, name = "AvailableConnections")]
            fn available_connections(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;
        }

        let proxy = DeviceConnectionsProxy::builder(connection)
            .path(device_path.clone())?
            .build()
            .await?;

        Ok(proxy.available_connections().await?)
    }

    async fn activate_connection(
        connection: &Connection,
        _nm_proxy: &imp::NetworkManagerProxy<'_>,
        conn_path: &zbus::zvariant::OwnedObjectPath,
        device_path: &zbus::zvariant::OwnedObjectPath,
    ) -> Result<(), Box<dyn std::error::Error>> {
        #[zbus::proxy(
            interface = "org.freedesktop.NetworkManager",
            default_service = "org.freedesktop.NetworkManager",
            default_path = "/org/freedesktop/NetworkManager"
        )]
        trait NetworkManagerActivate {
            fn activate_connection(
                &self,
                connection: zbus::zvariant::ObjectPath<'_>,
                device: zbus::zvariant::ObjectPath<'_>,
                specific_object: zbus::zvariant::ObjectPath<'_>,
            ) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
        }

        let proxy = NetworkManagerActivateProxy::new(connection).await?;
        proxy
            .activate_connection(
                conn_path.as_ref(),
                device_path.as_ref(),
                zbus::zvariant::ObjectPath::try_from("/")?,
            )
            .await?;

        Ok(())
    }

    async fn deactivate_connection(
        connection: &Connection,
        _nm_proxy: &imp::NetworkManagerProxy<'_>,
        active_conn_path: &zbus::zvariant::OwnedObjectPath,
    ) -> Result<(), Box<dyn std::error::Error>> {
        #[zbus::proxy(
            interface = "org.freedesktop.NetworkManager",
            default_service = "org.freedesktop.NetworkManager",
            default_path = "/org/freedesktop/NetworkManager"
        )]
        trait NetworkManagerDeactivate {
            fn deactivate_connection(
                &self,
                active_connection: zbus::zvariant::ObjectPath<'_>,
            ) -> zbus::Result<()>;
        }

        let proxy = NetworkManagerDeactivateProxy::new(connection).await?;
        proxy
            .deactivate_connection(active_conn_path.as_ref())
            .await?;

        Ok(())
    }
}

impl Default for Ethernet {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EthernetState {
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

impl EthernetState {
    pub fn from_u32(value: u32) -> Self {
        match value {
            10 => EthernetState::Unmanaged,
            20 => EthernetState::Unavailable,
            30 => EthernetState::Disconnected,
            40 => EthernetState::Preparing,
            50 => EthernetState::Config,
            60 => EthernetState::NeedAuth,
            70 => EthernetState::IpConfig,
            80 => EthernetState::IpCheck,
            90 => EthernetState::Secondaries,
            100 => EthernetState::Activated,
            110 => EthernetState::Deactivating,
            120 => EthernetState::Failed,
            _ => EthernetState::Unknown,
        }
    }
}

mod imp {
    use super::*;
    use glib::prelude::*;
    use glib::subclass::prelude::*;
    use glib::{ParamSpec, ParamSpecBuilderExt, Value};
    use std::cell::Cell;
    use zbus::proxy;

    pub struct Ethernet {
        pub connected: Cell<bool>,
        pub connection_id: RefCell<String>,
        pub device_product: RefCell<String>,
        pub state: Cell<u32>,
        pub device_path: RefCell<Option<zbus::zvariant::OwnedObjectPath>>,
        pub devices: RefCell<Vec<super::EthernetDevice>>,
    }

    impl Default for Ethernet {
        fn default() -> Self {
            Self {
                connected: Cell::new(false),
                connection_id: RefCell::new(String::new()),
                device_product: RefCell::new(String::new()),
                state: Cell::new(0),
                device_path: RefCell::new(None),
                devices: RefCell::new(Vec::new()),
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
        fn primary_connection(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

        #[zbus(property)]
        fn state(&self) -> zbus::Result<u32>;

        #[zbus(signal)]
        fn device_added(&self, device_path: zbus::zvariant::ObjectPath<'_>) -> zbus::Result<()>;

        #[zbus(signal)]
        fn device_removed(&self, device_path: zbus::zvariant::ObjectPath<'_>) -> zbus::Result<()>;

        #[zbus(signal)]
        fn active_connection_added(
            &self,
            connection: zbus::zvariant::ObjectPath<'_>,
        ) -> zbus::Result<()>;

        #[zbus(signal)]
        fn active_connection_removed(
            &self,
            connection: zbus::zvariant::ObjectPath<'_>,
        ) -> zbus::Result<()>;
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
    }

    #[proxy(
        interface = "org.freedesktop.NetworkManager.Device.Wired",
        default_service = "org.freedesktop.NetworkManager"
    )]
    trait WiredDevice {
        #[zbus(property)]
        fn hw_address(&self) -> zbus::Result<String>;
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
    }

    // NetworkManager device types
    const NM_DEVICE_TYPE_ETHERNET: u32 = 1;

    #[glib::object_subclass]
    impl ObjectSubclass for Ethernet {
        const NAME: &'static str = "Ethernet";
        type Type = super::Ethernet;
    }

    impl ObjectImpl for Ethernet {
        fn constructed(&self) {
            self.parent_constructed();

            self.update_ethernet_state();
            self.setup_event_listener();
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: OnceLock<Vec<ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![
                    glib::ParamSpecBoolean::builder("connected")
                        .read_only()
                        .build(),
                    glib::ParamSpecString::builder("connection-id")
                        .read_only()
                        .build(),
                    glib::ParamSpecString::builder("device-product")
                        .read_only()
                        .build(),
                    glib::ParamSpecUInt::builder("state").read_only().build(),
                    glib::ParamSpecString::builder("icon-name")
                        .read_only()
                        .build(),
                    glib::ParamSpecPointer::builder("devices")
                        .read_only()
                        .build(),
                ]
            })
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "connected" => self.connected.get().to_value(),
                "connection-id" => self.connection_id.borrow().to_value(),
                "device-product" => self.device_product.borrow().to_value(),
                "state" => self.state.get().to_value(),
                "icon-name" => self.obj().icon_name().to_value(),
                "devices" => glib::Value::from_type(glib::Type::POINTER),
                _ => unimplemented!(),
            }
        }
    }

    impl Ethernet {
        fn setup_event_listener(&self) {
            let obj = self.obj().clone();

            glib::MainContext::default().spawn_local(async move {
                let connection = match Connection::system().await {
                    Ok(conn) => conn,
                    Err(e) => {
                        let err_msg = format!("Failed to connect to D-Bus: {}", e);
                        crate::service::idle_add_safe(move || {
                            crate::service::notifications::Notifications::instance()
                                .send_notification("rusty-de", "Ethernet Error", &err_msg);
                        });
                        return;
                    }
                };

                let nm_proxy = match NetworkManagerProxy::new(&connection).await {
                    Ok(p) => p,
                    Err(e) => {
                        let err_msg = format!("Failed to create NetworkManager proxy: {}", e);
                        crate::service::idle_add_safe(move || {
                            crate::service::notifications::Notifications::instance()
                                .send_notification("rusty-de", "Ethernet Error", &err_msg);
                        });
                        return;
                    }
                };

                let ethernet_devices =
                    Self::discover_ethernet_devices(&connection, &nm_proxy).await;

                let (all_state_streams, all_active_conn_streams) =
                    Self::create_property_streams(&connection, ethernet_devices.clone()).await;

                Self::handle_device_events(
                    obj,
                    connection,
                    nm_proxy,
                    all_state_streams,
                    all_active_conn_streams,
                )
                .await;
            });
        }

        async fn discover_ethernet_devices(
            connection: &Connection,
            nm_proxy: &NetworkManagerProxy<'_>,
        ) -> Vec<zbus::zvariant::OwnedObjectPath> {
            let devices =
                match nm_proxy.devices().await {
                    Ok(d) => d,
                    Err(e) => {
                        let err_msg = format!("Failed to get devices: {}", e);
                        crate::service::idle_add_safe(move || {
                            crate::service::notifications::Notifications::instance()
                                .send_notification("rusty-de", "Ethernet Error", &err_msg);
                        });
                        return Vec::new();
                    }
                };

            let mut ethernet_devices = Vec::new();
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
                    if device_type == NM_DEVICE_TYPE_ETHERNET {
                        ethernet_devices.push(device_path);
                    }
                }
            }

            ethernet_devices
        }

        async fn create_property_streams(
            connection: &Connection,
            ethernet_devices: Vec<zbus::zvariant::OwnedObjectPath>,
        ) -> (
            futures_util::stream::SelectAll<zbus::PropertyStream<'static, u32>>,
            futures_util::stream::SelectAll<
                zbus::PropertyStream<'static, zbus::zvariant::OwnedObjectPath>,
            >,
        ) {
            let mut streams = Vec::new();

            for device_path in ethernet_devices {
                let device_proxy = match DeviceProxy::builder(connection).path(device_path.clone())
                {
                    Ok(builder) => match builder.build().await {
                        Ok(p) => p,
                        Err(_) => continue,
                    },
                    Err(_) => continue,
                };

                let state_stream = device_proxy.receive_state_changed().await;
                let active_conn_stream = device_proxy.receive_active_connection_changed().await;

                streams.push((device_path, state_stream, active_conn_stream));
            }

            let (state_streams, active_conn_streams): (Vec<_>, Vec<_>) = streams
                .into_iter()
                .map(|(_, state, active)| (state, active))
                .unzip();

            let all_state_streams = futures_util::stream::select_all(state_streams);
            let all_active_conn_streams = futures_util::stream::select_all(active_conn_streams);

            (all_state_streams, all_active_conn_streams)
        }

        async fn handle_device_events(
            obj: super::Ethernet,
            connection: Connection,
            nm_proxy: NetworkManagerProxy<'_>,
            mut all_state_streams: futures_util::stream::SelectAll<
                zbus::PropertyStream<'static, u32>,
            >,
            mut all_active_conn_streams: futures_util::stream::SelectAll<
                zbus::PropertyStream<'static, zbus::zvariant::OwnedObjectPath>,
            >,
        ) {
            use futures_util::StreamExt;

            let mut device_added_stream = nm_proxy
                .receive_device_added()
                .await
                .expect("Failed to create device_added signal stream");
            let mut device_removed_stream = nm_proxy
                .receive_device_removed()
                .await
                .expect("Failed to create device_removed signal stream");
            let mut active_conn_added_stream = nm_proxy
                .receive_active_connection_added()
                .await
                .expect("Failed to create active_connection_added signal stream");
            let mut active_conn_removed_stream = nm_proxy
                .receive_active_connection_removed()
                .await
                .expect("Failed to create active_connection_removed signal stream");

            loop {
                tokio::select! {
                    result = all_state_streams.next() => {
                        if result.is_none() {
                            break;
                        }
                        let changes = obj.imp().update_ethernet_state();
                        Self::notify_changes(&obj, changes);
                    }
                    result = all_active_conn_streams.next() => {
                        if result.is_none() {
                            break;
                        }
                        let changes = obj.imp().update_ethernet_state();
                        Self::notify_changes(&obj, changes);
                    }
                    result = device_added_stream.next() => {
                        if result.is_none() {
                            break;
                        }
                        if let Some(signal) = result {
                            if let Ok(args) = signal.args() {
                                let device_path = args.device_path.to_owned();

                                // Check if it's an ethernet device and set up listeners
                                let conn_clone = connection.clone();
                                let obj_clone = obj.clone();
                                glib::timeout_add_local_once(
                                    std::time::Duration::from_secs(1),
                                    move || {
                                        glib::MainContext::default().spawn_local(async move {
                                            let device_proxy = match DeviceProxy::builder(&conn_clone)
                                                .path(device_path.clone())
                                            {
                                                Ok(builder) => match builder.build().await {
                                                    Ok(p) => p,
                                                    Err(_) => return,
                                                },
                                                Err(_) => return,
                                            };

                                            if let Ok(device_type) = device_proxy.device_type().await {
                                                if device_type == 1 { // NM_DEVICE_TYPE_ETHERNET
                                                    // Spawn a task to listen to this device's property changes
                                                    Self::listen_to_device(conn_clone, device_path.into(), obj_clone).await;
                                                }
                                            }
                                        });
                                    },
                                );
                            }
                        }
                        let obj_clone = obj.clone();
                        glib::timeout_add_local_once(std::time::Duration::from_secs(1), move || {
                            let changes = obj_clone.imp().update_ethernet_state();
                            Self::notify_changes(&obj_clone, changes);
                        });
                    }
                    result = device_removed_stream.next() => {
                        if result.is_none() {
                            break;
                        }
                        let obj_clone = obj.clone();
                        glib::timeout_add_local_once(std::time::Duration::from_secs(1), move || {
                            let changes = obj_clone.imp().update_ethernet_state();
                            Self::notify_changes(&obj_clone, changes);
                        });
                    }
                    result = active_conn_added_stream.next() => {
                        if result.is_none() {
                            break;
                        }
                        let changes = obj.imp().update_ethernet_state();
                        Self::notify_changes(&obj, changes);
                    }
                    result = active_conn_removed_stream.next() => {
                        if result.is_none() {
                            break;
                        }
                        let changes = obj.imp().update_ethernet_state();
                        Self::notify_changes(&obj, changes);
                    }
                }
            }
        }

        async fn listen_to_device(
            connection: Connection,
            device_path: zbus::zvariant::OwnedObjectPath,
            obj: super::Ethernet,
        ) {
            use futures_util::StreamExt;

            let device_proxy = match DeviceProxy::builder(&connection).path(device_path.clone()) {
                Ok(builder) => match builder.build().await {
                    Ok(p) => p,
                    Err(_) => return,
                },
                Err(_) => return,
            };

            let mut state_stream = device_proxy.receive_state_changed().await;
            let mut active_conn_stream = device_proxy.receive_active_connection_changed().await;

            loop {
                tokio::select! {
                    result = state_stream.next() => {
                        if result.is_none() {
                            break;
                        }
                        let changes = obj.imp().update_ethernet_state();
                        Self::notify_changes(&obj, changes);
                    }
                    result = active_conn_stream.next() => {
                        if result.is_none() {
                            break;
                        }
                        let changes = obj.imp().update_ethernet_state();
                        Self::notify_changes(&obj, changes);
                    }
                }
            }
        }

        fn notify_changes(obj: &super::Ethernet, changes: EthernetChanges) {
            if changes.connected_changed {
                obj.notify("connected");
            }
            if changes.connection_id_changed {
                obj.notify("connection-id");
            }
            if changes.device_product_changed {
                obj.notify("device-product");
            }
            if changes.state_changed {
                obj.notify("state");
                obj.notify("icon-name");
            }
            if changes.devices_changed {
                obj.notify("devices");
            }
        }

        fn update_ethernet_state(&self) -> EthernetChanges {
            let mut changes = EthernetChanges::default();

            let runtime =
                match tokio::runtime::Runtime::new() {
                    Ok(rt) => rt,
                    Err(e) => {
                        let err_msg = format!("Failed to create async runtime: {}", e);
                        crate::service::idle_add_safe(move || {
                            crate::service::notifications::Notifications::instance()
                                .send_notification("rusty-de", "Ethernet Error", &err_msg);
                        });
                        return changes;
                    }
                };

            runtime.block_on(async {
                let connection = match Connection::system().await {
                    Ok(conn) => conn,
                    Err(e) => {
                        let err_msg = format!("Failed to connect to D-Bus: {}", e);
                        crate::service::idle_add_safe(move || {
                            crate::service::notifications::Notifications::instance()
                                .send_notification("rusty-de", "Ethernet Error", &err_msg);
                        });
                        return;
                    }
                };

                let nm_proxy = match NetworkManagerProxy::new(&connection).await {
                    Ok(p) => p,
                    Err(e) => {
                        let err_msg = format!("Failed to create NetworkManager proxy: {}", e);
                        crate::service::idle_add_safe(move || {
                            crate::service::notifications::Notifications::instance()
                                .send_notification("rusty-de", "Ethernet Error", &err_msg);
                        });
                        return;
                    }
                };

                let devices = match nm_proxy.devices().await {
                    Ok(d) => d,
                    Err(e) => {
                        let err_msg = format!("Failed to get devices: {}", e);
                        crate::service::idle_add_safe(move || {
                            crate::service::notifications::Notifications::instance()
                                .send_notification("rusty-de", "Ethernet Error", &err_msg);
                        });
                        return;
                    }
                };

                let ethernet_devices =
                    Self::get_all_ethernet_devices_with_connections(&connection, devices.clone())
                        .await;

                let old_devices = self.devices.borrow().clone();
                let devices_changed =
                    Self::has_devices_list_changed(&old_devices, &ethernet_devices);
                if devices_changed {
                    *self.devices.borrow_mut() = ethernet_devices.clone();
                    changes.devices_changed = true;
                }

                let active_device = Self::find_active_ethernet_device(&connection, devices).await;

                if let Some((device_path, device_state, active_conn_path, is_default)) =
                    active_device
                {
                    *self.device_path.borrow_mut() = Some(device_path);

                    let conn_info =
                        Self::get_ethernet_connection_info(&connection, &active_conn_path).await;

                    Self::apply_ethernet_state_changes(
                        self,
                        &mut changes,
                        device_state,
                        conn_info,
                        is_default,
                    );
                } else {
                    Self::apply_disconnected_state(self, &mut changes);
                }
            });

            changes
        }

        async fn find_active_ethernet_device(
            connection: &Connection,
            devices: Vec<zbus::zvariant::OwnedObjectPath>,
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

                if device_type != NM_DEVICE_TYPE_ETHERNET {
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

                if active_conn_path.as_str() == "/" {
                    continue;
                }

                let is_default =
                    Self::check_if_default_connection(connection, &active_conn_path).await;

                if device_state >= 40 {
                    return Some((device_path, device_state, active_conn_path, is_default));
                }
            }
            None
        }

        async fn check_if_default_connection(
            connection: &Connection,
            active_conn_path: &zbus::zvariant::OwnedObjectPath,
        ) -> bool {
            let active_conn_proxy =
                match ActiveConnectionProxy::builder(connection).path(active_conn_path.clone()) {
                    Ok(builder) => match builder.build().await {
                        Ok(p) => p,
                        Err(_) => return false,
                    },
                    Err(_) => return false,
                };

            active_conn_proxy.default().await.unwrap_or(false)
        }

        async fn get_all_ethernet_devices_with_connections(
            connection: &Connection,
            devices: Vec<zbus::zvariant::OwnedObjectPath>,
        ) -> Vec<super::EthernetDevice> {
            let mut ethernet_devices = Vec::new();

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

                if device_type != NM_DEVICE_TYPE_ETHERNET {
                    continue;
                }

                let available_connections = match super::Ethernet::get_available_connections(
                    connection,
                    &device_path,
                )
                .await
                {
                    Ok(conns) => conns,
                    Err(_) => continue,
                };

                if available_connections.is_empty() {
                    continue;
                }

                // Get device state and active connection
                let device_state = device_proxy.state().await.unwrap_or(0);
                let active_conn_path = device_proxy.active_connection().await.ok();

                let (connected, is_default, id) = if let Some(ref conn_path) = active_conn_path {
                    if conn_path.as_str() != "/" && device_state >= 40 {
                        let is_default =
                            Self::check_if_default_connection(connection, conn_path).await;
                        let id = Self::get_ethernet_connection_info(connection, conn_path).await;
                        let conn_id_str = id.clone().unwrap_or_default();
                        (
                            device_state >= 40 && !conn_id_str.is_empty(),
                            is_default,
                            id.unwrap_or_else(|| "Wired Connection".to_string()),
                        )
                    } else {
                        (
                            false,
                            false,
                            Self::get_connection_id_from_path(
                                connection,
                                &available_connections[0],
                            )
                            .await,
                        )
                    }
                } else {
                    (
                        false,
                        false,
                        Self::get_connection_id_from_path(connection, &available_connections[0])
                            .await,
                    )
                };

                // Get product name
                let product = Self::get_device_product(connection, &device_path).await;

                ethernet_devices.push(super::EthernetDevice {
                    device_path,
                    id,
                    product,
                    connected,
                    is_default,
                });
            }

            ethernet_devices
        }

        async fn get_connection_id_from_path(
            connection: &Connection,
            conn_path: &zbus::zvariant::OwnedObjectPath,
        ) -> String {
            #[zbus::proxy(
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
            }

            let conn_proxy =
                match SettingsConnectionProxy::builder(connection).path(conn_path.clone()) {
                    Ok(builder) => match builder.build().await {
                        Ok(p) => p,
                        Err(_) => return "Wired Connection".to_string(),
                    },
                    Err(_) => return "Wired Connection".to_string(),
                };

            match conn_proxy.get_settings().await {
                Ok(settings) => {
                    if let Some(conn_settings) = settings.get("connection") {
                        if let Some(id_value) = conn_settings.get("id") {
                            if let Ok(id_str) = id_value.downcast_ref::<zbus::zvariant::Str>() {
                                return id_str.to_string();
                            }
                        }
                    }
                    "Wired Connection".to_string()
                }
                Err(_) => "Wired Connection".to_string(),
            }
        }

        async fn get_device_product(
            connection: &Connection,
            device_path: &zbus::zvariant::OwnedObjectPath,
        ) -> String {
            #[zbus::proxy(
                interface = "org.freedesktop.NetworkManager.Device",
                default_service = "org.freedesktop.NetworkManager"
            )]
            trait DeviceProduct {
                #[zbus(property)]
                fn product(&self) -> zbus::Result<String>;
            }

            let device_proxy =
                match DeviceProductProxy::builder(connection).path(device_path.clone()) {
                    Ok(builder) => match builder.build().await {
                        Ok(p) => p,
                        Err(_) => return String::new(),
                    },
                    Err(_) => return String::new(),
                };

            device_proxy.product().await.unwrap_or_default()
        }

        async fn get_ethernet_connection_info(
            connection: &Connection,
            active_conn_path: &zbus::zvariant::OwnedObjectPath,
        ) -> Option<String> {
            let active_conn_proxy =
                match ActiveConnectionProxy::builder(connection).path(active_conn_path.clone()) {
                    Ok(builder) => match builder.build().await {
                        Ok(p) => p,
                        Err(_) => return None,
                    },
                    Err(_) => return None,
                };

            active_conn_proxy.id().await.ok()
        }

        fn apply_ethernet_state_changes(
            ethernet: &Ethernet,
            changes: &mut EthernetChanges,
            device_state: u32,
            conn_id: Option<String>,
            _is_default: bool,
        ) {
            let old_state = ethernet.state.get();
            if device_state != old_state {
                ethernet.state.set(device_state);
                changes.state_changed = true;
            }

            let conn_id_str = conn_id.unwrap_or_default();
            let old_connection_id = ethernet.connection_id.borrow().clone();
            if conn_id_str != old_connection_id {
                ethernet.connection_id.replace(conn_id_str.clone());
                changes.connection_id_changed = true;
            }

            let old_connected = ethernet.connected.get();
            // Consider connected when state >= 40 (Preparing and onwards) with active connection
            let new_connected = device_state >= 40 && !conn_id_str.is_empty();
            if new_connected != old_connected {
                ethernet.connected.set(new_connected);
                changes.connected_changed = true;
            }
        }

        fn apply_disconnected_state(ethernet: &Ethernet, changes: &mut EthernetChanges) {
            let old_connected = ethernet.connected.get();
            if old_connected {
                ethernet.connected.set(false);
                changes.connected_changed = true;
            }

            let old_state = ethernet.state.get();
            if old_state != 30 {
                // NM_DEVICE_STATE_DISCONNECTED
                ethernet.state.set(30);
                changes.state_changed = true;
            }

            let old_connection_id = ethernet.connection_id.borrow().clone();
            if !old_connection_id.is_empty() {
                ethernet.connection_id.replace(String::new());
                changes.connection_id_changed = true;
            }
        }

        fn has_devices_list_changed(
            old: &[super::EthernetDevice],
            new: &[super::EthernetDevice],
        ) -> bool {
            if old.len() != new.len() {
                return true;
            }

            for (old_dev, new_dev) in old.iter().zip(new.iter()) {
                if old_dev.device_path != new_dev.device_path
                    || old_dev.id != new_dev.id
                    || old_dev.product != new_dev.product
                    || old_dev.connected != new_dev.connected
                    || old_dev.is_default != new_dev.is_default
                {
                    return true;
                }
            }

            false
        }
    }

    #[derive(Default)]
    struct EthernetChanges {
        connected_changed: bool,
        connection_id_changed: bool,
        device_product_changed: bool,
        state_changed: bool,
        devices_changed: bool,
    }
}
