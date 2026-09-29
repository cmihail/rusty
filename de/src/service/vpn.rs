use glib::subclass::prelude::*;
use glib::Object;
use gtk4::glib;
use std::cell::RefCell;
use std::sync::OnceLock;
use zbus::Connection;

#[derive(Debug, Clone)]
pub struct VpnConnection {
    pub connection_path: zbus::zvariant::OwnedObjectPath,
    pub id: String,
    pub connected: bool,
    pub vpn_type: String,
    pub requires_password: bool,
    pub has_saved_password: bool,
}

glib::wrapper! {
    pub struct Vpn(ObjectSubclass<imp::Vpn>);
}

impl Vpn {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn instance() -> Self {
        use std::cell::OnceCell;
        thread_local! {
            static INSTANCE: OnceCell<Vpn> = const { OnceCell::new() };
        }

        INSTANCE.with(|cell| cell.get_or_init(Self::new).clone())
    }

    pub fn connected(&self) -> bool {
        self.imp().connected.get()
    }

    pub fn connection_id(&self) -> String {
        self.imp().connection_id.borrow().clone()
    }

    pub fn connections(&self) -> Vec<VpnConnection> {
        self.imp().connections.borrow().clone()
    }

    pub fn icon_name(&self) -> String {
        if self.connected() {
            "network-vpn-symbolic".to_string()
        } else {
            "network-vpn-disabled-symbolic".to_string()
        }
    }

    pub fn connect_vpn<F>(
        &self,
        connection_path: &zbus::zvariant::OwnedObjectPath,
        password: Option<String>,
        on_complete: F,
    ) where
        F: Fn(Result<(), String>) + 'static,
    {
        let path = connection_path.clone();

        let on_complete_clone = std::rc::Rc::new(on_complete);
        let on_complete_for_timeout = on_complete_clone.clone();
        let timed_out = std::rc::Rc::new(std::cell::Cell::new(false));
        let timed_out_clone = timed_out.clone();

        // Set up timeout
        glib::timeout_add_local_once(std::time::Duration::from_secs(30), move || {
            if !timed_out_clone.get() {
                timed_out_clone.set(true);
                let error_msg = "VPN connection timed out after 30 seconds".to_string();
                on_complete_for_timeout(Err(error_msg));
            }
        });

        glib::MainContext::default().spawn_local(async move {
            match Self::connect_vpn_async(&path, password).await {
                Ok(()) => {
                    if !timed_out.get() {
                        timed_out.set(true);
                        on_complete_clone(Ok(()));
                    }
                }
                Err(e) => {
                    if !timed_out.get() {
                        timed_out.set(true);
                        on_complete_clone(Err(format!("Failed to connect VPN: {}", e)));
                    }
                }
            }
        });
    }

    pub fn disconnect_vpn<F>(&self, connection_path: &zbus::zvariant::OwnedObjectPath, on_error: F)
    where
        F: Fn(String) + 'static,
    {
        let path = connection_path.clone();

        glib::MainContext::default().spawn_local(async move {
            if let Err(e) = Self::disconnect_vpn_async(&path).await {
                on_error(format!("Failed to disconnect VPN: {}", e));
            }
        });
    }

    pub fn connect_all<F>(&self, on_error: F)
    where
        F: Fn(String) + 'static + Clone,
    {
        let connections = self.connections();

        for connection in connections {
            let error_handler = on_error.clone();
            self.connect_vpn(&connection.connection_path, None, move |result| {
                if let Err(err) = result {
                    error_handler(err);
                }
            });
        }
    }

    pub fn disconnect_all<F>(&self, on_error: F)
    where
        F: Fn(String) + 'static + Clone,
    {
        let connections = self.connections();

        for connection in connections {
            let error_handler = on_error.clone();
            self.disconnect_vpn(&connection.connection_path, move |err| {
                error_handler(err);
            });
        }
    }

    async fn connect_vpn_async(
        connection_path: &zbus::zvariant::OwnedObjectPath,
        password: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let connection = Connection::system().await?;
        // If password is provided, save it as secrets before activating
        if let Some(pw) = password {
            Self::save_vpn_secrets(&connection, connection_path, &pw).await?;
        }

        Self::activate_vpn_connection(&connection, connection_path).await?;
        Ok(())
    }

    async fn disconnect_vpn_async(
        connection_path: &zbus::zvariant::OwnedObjectPath,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let connection = Connection::system().await?;
        let nm_proxy = imp::NetworkManagerProxy::new(&connection).await?;
        let active_conn_path =
            Self::find_active_connection_for_vpn(&connection, connection_path).await?;

        if active_conn_path.as_str() == "/" {
            return Ok(()); // Not connected
        }
        Self::deactivate_connection(&connection, &nm_proxy, &active_conn_path).await?;
        Ok(())
    }

    async fn find_active_connection_for_vpn(
        connection: &Connection,
        vpn_connection_path: &zbus::zvariant::OwnedObjectPath,
    ) -> Result<zbus::zvariant::OwnedObjectPath, Box<dyn std::error::Error>> {
        let nm_proxy = imp::NetworkManagerProxy::new(connection).await?;
        let active_connections = nm_proxy.active_connections().await?;

        for active_conn_path in active_connections {
            let active_conn_proxy = imp::ActiveConnectionProxy::builder(connection)
                .path(active_conn_path.clone())?
                .build()
                .await?;

            if let Ok(conn_path) = active_conn_proxy.connection().await {
                if conn_path == *vpn_connection_path {
                    return Ok(active_conn_path);
                }
            }
        }
        Ok(zbus::zvariant::ObjectPath::try_from("/")?.into())
    }

    async fn activate_vpn_connection(
        connection: &Connection,
        conn_path: &zbus::zvariant::OwnedObjectPath,
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
        let active_conn = proxy
            .activate_connection(
                conn_path.as_ref(),
                zbus::zvariant::ObjectPath::try_from("/")?,
                zbus::zvariant::ObjectPath::try_from("/")?,
            )
            .await?;
        // Wait for the connection to be fully activated (or fail)
        Self::wait_for_activation(connection, &active_conn).await?;

        Ok(())
    }

    async fn wait_for_activation(
        connection: &Connection,
        active_conn_path: &zbus::zvariant::OwnedObjectPath,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use futures_util::StreamExt;
        // Try to build the proxy - if this fails, the connection was rejected immediately
        let active_conn_proxy = match imp::ActiveConnectionProxy::builder(connection)
            .path(active_conn_path.clone())
        {
            Ok(builder) => match builder.build().await {
                Ok(proxy) => proxy,
                Err(e) => {
                    if e.to_string().contains("Object does not exist") {
                        return Err(
                            "VPN connection failed: Wrong password or authentication error".into(),
                        );
                    }
                    return Err(format!("Failed to monitor VPN connection: {}", e).into());
                }
            },
            Err(e) => {
                return Err(format!("Failed to monitor VPN connection: {}", e).into());
            }
        };

        // Check initial state
        let initial_state = match active_conn_proxy.state().await {
            Ok(state) => state,
            Err(e) => {
                if e.to_string().contains("Object does not exist") {
                    return Err(
                        "VPN connection failed: Wrong password or authentication error".into(),
                    );
                }
                return Err(format!("Failed to check VPN connection state: {}", e).into());
            }
        };

        if initial_state == 2 {
            return Ok(());
        }

        if initial_state == 4 {
            return Err("VPN connection failed: Wrong password or authentication error".into());
        }

        // Listen to StateChanged signal (which includes both state and reason)
        let mut state_changed_stream = match active_conn_proxy.receive_state_changed_signal().await
        {
            Ok(stream) => stream,
            Err(e) => {
                if e.to_string().contains("Object does not exist") {
                    return Err(
                        "VPN connection failed: Wrong password or authentication error".into(),
                    );
                }
                return Err(format!("Failed to monitor VPN connection: {}", e).into());
            }
        };

        // Wait up to 30 seconds for activation
        let max_iterations = 300; // 30 seconds at 100ms per iteration
        let mut iterations = 0;

        while let Some(signal) = state_changed_stream.next().await {
            let args = signal.args()?;
            let state = args.state;
            let reason = args.reason;
            match state {
                2 => {
                    return Ok(());
                }
                4 => {
                    let reason_str = Self::state_reason_to_string(reason);
                    return Err(format!("VPN activation failed: {}", reason_str).into());
                }
                _ => {}
            }

            iterations += 1;
            if iterations >= max_iterations {
                return Err("VPN activation timed out".into());
            }
        }
        Err("VPN activation monitoring stream ended unexpectedly".into())
    }

    fn state_reason_to_string(reason: u32) -> String {
        match reason {
            0 => "Unknown reason".to_string(),
            1 => "None (no failure)".to_string(),
            2 => "User disconnected".to_string(),
            3 => "Device disconnected".to_string(),
            4 => "Service stopped".to_string(),
            5 => "IP configuration invalid".to_string(),
            6 => "IP configuration timeout".to_string(),
            7 => "Service start timeout".to_string(),
            8 => "Service start failed".to_string(),
            9 => "No secrets (password/key required)".to_string(),
            10 => "Login failed (invalid credentials)".to_string(),
            11 => "Connection removed".to_string(),
            12 => "Dependency failed".to_string(),
            13 => "Master failed".to_string(),
            _ => format!("Unknown reason code: {}", reason),
        }
    }

    async fn save_vpn_secrets(
        connection: &Connection,
        conn_path: &zbus::zvariant::OwnedObjectPath,
        password: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use std::collections::HashMap;
        use zbus::zvariant::{Dict, OwnedValue, Value};
        let settings_proxy = imp::SettingsConnectionProxy::builder(connection)
            .path(conn_path.clone())?
            .build()
            .await?;

        // Get existing settings
        let mut settings = settings_proxy.get_settings().await?;

        // Create VPN secrets dict - NetworkManager expects HashMap<String, String>
        let mut vpn_secrets: HashMap<String, String> = HashMap::new();
        vpn_secrets.insert("password".to_string(), password.to_string());

        // Get existing VPN settings or create new
        let mut vpn_settings: HashMap<String, OwnedValue> = HashMap::new();
        if let Some(vpn) = settings.get("vpn") {
            for (k, v) in vpn.iter() {
                if k != "secrets" {
                    // Keep everything except secrets
                    vpn_settings.insert(k.clone(), v.try_to_owned()?);
                }
            }
        }

        // Insert the new secrets as a{ss} type
        let secrets_dict = Dict::from(vpn_secrets);
        vpn_settings.insert(
            "secrets".to_string(),
            OwnedValue::try_from(Value::from(secrets_dict))?,
        );

        settings.insert("vpn".to_string(), vpn_settings);
        settings_proxy.update(settings).await?;
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

impl Default for Vpn {
    fn default() -> Self {
        Self::new()
    }
}

mod imp {
    use super::*;
    use glib::prelude::*;
    use glib::subclass::prelude::*;
    use glib::{ParamSpec, ParamSpecBuilderExt, Value};
    use std::cell::Cell;
    use zbus::proxy;

    pub struct Vpn {
        pub connected: Cell<bool>,
        pub connection_id: RefCell<String>,
        pub connections: RefCell<Vec<super::VpnConnection>>,
    }

    impl Default for Vpn {
        fn default() -> Self {
            Self {
                connected: Cell::new(false),
                connection_id: RefCell::new(String::new()),
                connections: RefCell::new(Vec::new()),
            }
        }
    }

    #[proxy(
        interface = "org.freedesktop.NetworkManager",
        default_service = "org.freedesktop.NetworkManager",
        default_path = "/org/freedesktop/NetworkManager"
    )]
    pub trait NetworkManager {
        #[zbus(property)]
        fn active_connections(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;

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
        interface = "org.freedesktop.NetworkManager.Connection.Active",
        default_service = "org.freedesktop.NetworkManager"
    )]
    pub trait ActiveConnection {
        #[zbus(property)]
        fn id(&self) -> zbus::Result<String>;

        #[zbus(property, name = "Type")]
        fn connection_type(&self) -> zbus::Result<String>;

        #[zbus(property)]
        fn vpn(&self) -> zbus::Result<bool>;

        #[zbus(property)]
        fn state(&self) -> zbus::Result<u32>;

        #[zbus(property)]
        fn connection(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

        #[zbus(signal, name = "StateChanged")]
        fn state_changed_signal(&self, state: u32, reason: u32) -> zbus::Result<()>;
    }

    #[proxy(
        interface = "org.freedesktop.NetworkManager.Settings",
        default_service = "org.freedesktop.NetworkManager",
        default_path = "/org/freedesktop/NetworkManager/Settings"
    )]
    trait Settings {
        fn list_connections(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;
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

        fn update(
            &self,
            properties: std::collections::HashMap<
                String,
                std::collections::HashMap<String, zbus::zvariant::OwnedValue>,
            >,
        ) -> zbus::Result<()>;

        fn get_secrets(
            &self,
            setting_name: &str,
        ) -> zbus::Result<
            std::collections::HashMap<
                String,
                std::collections::HashMap<String, zbus::zvariant::OwnedValue>,
            >,
        >;
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Vpn {
        const NAME: &'static str = "Vpn";
        type Type = super::Vpn;
    }

    impl ObjectImpl for Vpn {
        fn constructed(&self) {
            self.parent_constructed();

            self.update_vpn_state();
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
                    glib::ParamSpecString::builder("icon-name")
                        .read_only()
                        .build(),
                    glib::ParamSpecPointer::builder("connections")
                        .read_only()
                        .build(),
                ]
            })
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "connected" => self.connected.get().to_value(),
                "connection-id" => self.connection_id.borrow().to_value(),
                "icon-name" => self.obj().icon_name().to_value(),
                "connections" => glib::Value::from_type(glib::Type::POINTER),
                _ => unimplemented!(),
            }
        }
    }

    impl Vpn {
        fn setup_event_listener(&self) {
            let obj = self.obj().clone();

            glib::MainContext::default().spawn_local(async move {
                let connection = match Connection::system().await {
                    Ok(conn) => conn,
                    Err(e) => {
                        let err_msg = format!("Failed to connect to D-Bus: {}", e);
                        crate::service::idle_add_safe(move || {
                            crate::service::notifications::Notifications::instance()
                                .send_notification("rusty-de", "VPN Error", &err_msg);
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
                                .send_notification("rusty-de", "VPN Error", &err_msg);
                        });
                        return;
                    }
                };

                Self::handle_vpn_events(obj, connection, nm_proxy).await;
            });
        }

        async fn handle_vpn_events(
            obj: super::Vpn,
            _connection: Connection,
            nm_proxy: NetworkManagerProxy<'_>,
        ) {
            use futures_util::StreamExt;

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
                    result = active_conn_added_stream.next() => {
                        if result.is_none() {
                            break;
                        }
                        let obj_clone = obj.clone();
                        let duration = std::time::Duration::from_millis(500);
                        glib::timeout_add_local_once(duration, move || {
                            let changes = obj_clone.imp().update_vpn_state();
                            Self::notify_changes(&obj_clone, changes);
                        });
                    }
                    result = active_conn_removed_stream.next() => {
                        if result.is_none() {
                            break;
                        }
                        let obj_clone = obj.clone();
                        let duration = std::time::Duration::from_millis(500);
                        glib::timeout_add_local_once(duration, move || {
                            let changes = obj_clone.imp().update_vpn_state();
                            Self::notify_changes(&obj_clone, changes);
                        });
                    }
                }
            }
        }

        fn notify_changes(obj: &super::Vpn, changes: VpnChanges) {
            if changes.connected_changed {
                obj.notify("connected");
                obj.notify("icon-name");
            }
            if changes.connection_id_changed {
                obj.notify("connection-id");
            }
            if changes.connections_changed {
                obj.notify("connections");
            }
        }

        fn update_vpn_state(&self) -> VpnChanges {
            let mut changes = VpnChanges::default();

            let runtime = match self.create_runtime() {
                Ok(rt) => rt,
                Err(_) => return changes,
            };

            runtime.block_on(async {
                let connection = match self.get_dbus_connection().await {
                    Ok(conn) => conn,
                    Err(_) => return,
                };

                self.update_connections_list(&connection, &mut changes)
                    .await;
                self.update_connection_state(&connection, &mut changes)
                    .await;
            });

            changes
        }

        fn create_runtime(&self) -> Result<tokio::runtime::Runtime, ()> {
            tokio::runtime::Runtime::new().map_err(|e| {
                let err_msg = format!("Failed to create async runtime: {}", e);
                crate::service::idle_add_safe(move || {
                    crate::service::notifications::Notifications::instance().send_notification(
                        "rusty-de",
                        "VPN Error",
                        &err_msg,
                    );
                });
            })
        }

        async fn get_dbus_connection(&self) -> Result<Connection, ()> {
            Connection::system().await.map_err(|e| {
                let err_msg = format!("Failed to connect to D-Bus: {}", e);
                crate::service::idle_add_safe(move || {
                    crate::service::notifications::Notifications::instance().send_notification(
                        "rusty-de",
                        "VPN Error",
                        &err_msg,
                    );
                });
            })
        }

        async fn update_connections_list(&self, connection: &Connection, changes: &mut VpnChanges) {
            let vpn_connections = Self::get_all_vpn_connections(connection).await;
            let old_connections = self.connections.borrow().clone();
            let connections_changed =
                Self::has_connections_list_changed(&old_connections, &vpn_connections);
            if connections_changed {
                *self.connections.borrow_mut() = vpn_connections;
                changes.connections_changed = true;
            }
        }

        async fn update_connection_state(&self, connection: &Connection, changes: &mut VpnChanges) {
            let active_vpn = Self::find_active_vpn_connection(connection).await;

            if let Some((conn_id, _active_conn_path)) = active_vpn {
                Self::apply_connected_state(self, &conn_id, changes);
            } else {
                Self::apply_disconnected_state(self, changes);
            }
        }

        fn apply_connected_state(vpn: &Vpn, conn_id: &str, changes: &mut VpnChanges) {
            let old_connection_id = vpn.connection_id.borrow().clone();
            if conn_id != old_connection_id {
                vpn.connection_id.replace(conn_id.to_string());
                changes.connection_id_changed = true;
            }

            let old_connected = vpn.connected.get();
            if !old_connected {
                vpn.connected.set(true);
                changes.connected_changed = true;
            }
        }

        async fn get_all_vpn_connections(connection: &Connection) -> Vec<super::VpnConnection> {
            let mut vpn_connections = Vec::new();

            let settings_proxy = match SettingsProxy::new(connection).await {
                Ok(p) => p,
                Err(_) => return vpn_connections,
            };

            let connections = match settings_proxy.list_connections().await {
                Ok(c) => c,
                Err(_) => return vpn_connections,
            };

            for conn_path in connections {
                if let Some(vpn_conn) = Self::try_parse_vpn_connection(connection, conn_path).await
                {
                    vpn_connections.push(vpn_conn);
                }
            }

            vpn_connections
        }

        async fn try_parse_vpn_connection(
            connection: &Connection,
            conn_path: zbus::zvariant::OwnedObjectPath,
        ) -> Option<super::VpnConnection> {
            let conn_proxy = Self::build_connection_proxy(connection, &conn_path).await?;
            let settings = conn_proxy.get_settings().await.ok()?;

            let conn_settings = settings.get("connection")?;
            let type_value = conn_settings.get("type")?;
            let type_str = type_value.downcast_ref::<zbus::zvariant::Str>().ok()?;

            if type_str.as_str() != "vpn" {
                return None;
            }

            let id = Self::extract_connection_id(conn_settings);
            let vpn_type = Self::extract_vpn_type(&settings);
            let connected = Self::is_vpn_connected(connection, &conn_path).await;

            // Check if VPN requires a password and has saved secrets
            let (requires_password, has_saved_password) =
                Self::check_password_status(&conn_proxy, &settings).await;

            Some(super::VpnConnection {
                connection_path: conn_path,
                id,
                connected,
                vpn_type,
                requires_password,
                has_saved_password,
            })
        }

        async fn check_password_status(
            conn_proxy: &SettingsConnectionProxy<'_>,
            settings: &std::collections::HashMap<
                String,
                std::collections::HashMap<String, zbus::zvariant::OwnedValue>,
            >,
        ) -> (bool, bool) {
            // Check if password flags require user input
            let requires_password = if let Some(vpn_settings) = settings.get("vpn") {
                // Check password-flags field
                if let Some(_flags_value) = vpn_settings.get("password-flags") {
                    // Check password-flags field
                    // Flag 0 = agent-owned (requires input)
                    // Flag 1 = not saved
                    // Flag 2 = not required
                    // For now, assume password is required if flags != 2
                    true // TODO: properly check flags
                } else {
                    true // No flags means password probably required
                }
            } else {
                true // No VPN settings means password required
            };

            // Try to get secrets to see if password is saved
            let has_saved_password = if let Ok(secrets) = conn_proxy.get_secrets("vpn").await {
                if let Some(vpn_secrets) = secrets.get("vpn") {
                    vpn_secrets.contains_key("password")
                } else {
                    false
                }
            } else {
                false
            };

            (requires_password, has_saved_password)
        }

        async fn build_connection_proxy<'a>(
            connection: &'a Connection,
            conn_path: &zbus::zvariant::OwnedObjectPath,
        ) -> Option<SettingsConnectionProxy<'a>> {
            let builder = SettingsConnectionProxy::builder(connection)
                .path(conn_path.clone())
                .ok()?;
            builder.build().await.ok()
        }

        fn extract_connection_id(
            conn_settings: &std::collections::HashMap<String, zbus::zvariant::OwnedValue>,
        ) -> String {
            conn_settings
                .get("id")
                .and_then(|v| v.downcast_ref::<zbus::zvariant::Str>().ok())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "VPN Connection".to_string())
        }

        fn extract_vpn_type(
            settings: &std::collections::HashMap<
                String,
                std::collections::HashMap<String, zbus::zvariant::OwnedValue>,
            >,
        ) -> String {
            settings
                .get("vpn")
                .and_then(|vpn_settings| vpn_settings.get("service-type"))
                .and_then(|v| v.downcast_ref::<zbus::zvariant::Str>().ok())
                .map(|s| Self::format_vpn_type(s.as_str()))
                .unwrap_or_else(|| "VPN".to_string())
        }

        fn format_vpn_type(service_type: &str) -> String {
            match service_type {
                "org.freedesktop.NetworkManager.openvpn" => "OpenVPN".to_string(),
                "org.freedesktop.NetworkManager.pptp" => "PPTP".to_string(),
                "org.freedesktop.NetworkManager.l2tp" => "L2TP".to_string(),
                "org.freedesktop.NetworkManager.vpnc" => "Cisco VPN".to_string(),
                "org.freedesktop.NetworkManager.openconnect" => "OpenConnect".to_string(),
                "org.freedesktop.NetworkManager.strongswan" => "strongSwan".to_string(),
                _ => "VPN".to_string(),
            }
        }

        async fn is_vpn_connected(
            connection: &Connection,
            vpn_connection_path: &zbus::zvariant::OwnedObjectPath,
        ) -> bool {
            let nm_proxy = match NetworkManagerProxy::new(connection).await {
                Ok(p) => p,
                Err(_) => return false,
            };

            let active_connections = match nm_proxy.active_connections().await {
                Ok(ac) => ac,
                Err(_) => return false,
            };

            for active_conn_path in active_connections {
                let active_conn_proxy = match ActiveConnectionProxy::builder(connection)
                    .path(active_conn_path.clone())
                {
                    Ok(builder) => match builder.build().await {
                        Ok(p) => p,
                        Err(_) => continue,
                    },
                    Err(_) => continue,
                };

                if let Ok(conn_path) = active_conn_proxy.connection().await {
                    if conn_path == *vpn_connection_path {
                        if let Ok(state) = active_conn_proxy.state().await {
                            return state == 2; // NM_ACTIVE_CONNECTION_STATE_ACTIVATED
                        }
                    }
                }
            }

            false
        }

        async fn find_active_vpn_connection(
            connection: &Connection,
        ) -> Option<(String, zbus::zvariant::OwnedObjectPath)> {
            let nm_proxy = match NetworkManagerProxy::new(connection).await {
                Ok(p) => p,
                Err(_) => return None,
            };

            let active_connections = match nm_proxy.active_connections().await {
                Ok(ac) => ac,
                Err(_) => return None,
            };

            for active_conn_path in active_connections {
                let active_conn_proxy = match ActiveConnectionProxy::builder(connection)
                    .path(active_conn_path.clone())
                {
                    Ok(builder) => match builder.build().await {
                        Ok(p) => p,
                        Err(_) => continue,
                    },
                    Err(_) => continue,
                };

                if let Ok(is_vpn) = active_conn_proxy.vpn().await {
                    if is_vpn {
                        if let Ok(state) = active_conn_proxy.state().await {
                            if state == 2 {
                                // NM_ACTIVE_CONNECTION_STATE_ACTIVATED
                                if let Ok(id) = active_conn_proxy.id().await {
                                    return Some((id, active_conn_path));
                                }
                            }
                        }
                    }
                }
            }

            None
        }

        fn apply_disconnected_state(vpn: &Vpn, changes: &mut VpnChanges) {
            let old_connected = vpn.connected.get();
            if old_connected {
                vpn.connected.set(false);
                changes.connected_changed = true;
            }

            let old_connection_id = vpn.connection_id.borrow().clone();
            if !old_connection_id.is_empty() {
                vpn.connection_id.replace(String::new());
                changes.connection_id_changed = true;
            }
        }

        fn has_connections_list_changed(
            old: &[super::VpnConnection],
            new: &[super::VpnConnection],
        ) -> bool {
            if old.len() != new.len() {
                return true;
            }

            for (old_conn, new_conn) in old.iter().zip(new.iter()) {
                if old_conn.connection_path != new_conn.connection_path
                    || old_conn.id != new_conn.id
                    || old_conn.connected != new_conn.connected
                    || old_conn.vpn_type != new_conn.vpn_type
                    || old_conn.requires_password != new_conn.requires_password
                    || old_conn.has_saved_password != new_conn.has_saved_password
                {
                    return true;
                }
            }

            false
        }
    }

    #[derive(Default)]
    struct VpnChanges {
        connected_changed: bool,
        connection_id_changed: bool,
        connections_changed: bool,
    }
}
