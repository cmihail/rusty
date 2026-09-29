use glib::subclass::prelude::*;
use glib::Object;
use gtk4::glib;
use std::cell::RefCell;
use std::sync::OnceLock;
use zbus::Connection;

use super::ethernet::Ethernet;
use super::wifi::Wifi;

glib::wrapper! {
    pub struct Network(ObjectSubclass<imp::Network>);
}

thread_local! {
    static NETWORK_INSTANCE: RefCell<Option<Network>> = const { RefCell::new(None) };
}

impl Network {
    pub fn instance() -> Network {
        NETWORK_INSTANCE.with(|instance| {
            let mut instance_mut = instance.borrow_mut();
            if instance_mut.is_none() {
                let obj: Network = Object::builder().build();
                *instance_mut = Some(obj);
            }
            instance_mut.as_ref().unwrap().clone()
        })
    }

    fn new() -> Self {
        Object::builder().build()
    }

    pub fn primary(&self) -> NetworkType {
        NetworkType::from_string(&self.imp().primary.borrow())
    }

    pub fn state(&self) -> NetworkState {
        NetworkState::from_u32(self.imp().state.get())
    }

    pub fn ethernet(&self) -> &Ethernet {
        &self.imp().ethernet
    }

    pub fn wifi(&self) -> &Wifi {
        &self.imp().wifi
    }

    pub fn icon_name(&self) -> String {
        let state = self.state();
        let primary = self.primary();

        match primary {
            NetworkType::Wired => self.ethernet().icon_name(),
            NetworkType::Wifi => self.wifi().icon_name(),
            NetworkType::Unknown => match state {
                NetworkState::ConnectedGlobal => "network-transmit-receive-symbolic".to_string(),
                NetworkState::ConnectedSite => "network-receive-symbolic".to_string(),
                NetworkState::Connecting => "network-transmit-symbolic".to_string(),
                NetworkState::Disconnecting => "network-idle-symbolic".to_string(),
                NetworkState::ConnectedLocal | NetworkState::Disconnected => {
                    "network-error-symbolic".to_string()
                }
                NetworkState::Unknown | NetworkState::Asleep => {
                    "network-error-symbolic".to_string()
                }
            },
        }
    }
}

impl Default for Network {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkType {
    Unknown,
    Wired,
    Wifi,
}

impl NetworkType {
    pub fn from_string(value: &str) -> Self {
        match value {
            "802-3-ethernet" => NetworkType::Wired,
            "802-11-wireless" => NetworkType::Wifi,
            _ => NetworkType::Unknown,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            NetworkType::Wired => "wired",
            NetworkType::Wifi => "wifi",
            NetworkType::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkState {
    Unknown = 0,
    Asleep = 10,
    Disconnected = 20,
    Disconnecting = 30,
    Connecting = 40,
    ConnectedLocal = 50,
    ConnectedSite = 60,
    ConnectedGlobal = 70,
}

impl NetworkState {
    pub fn from_u32(value: u32) -> Self {
        match value {
            10 => NetworkState::Asleep,
            20 => NetworkState::Disconnected,
            30 => NetworkState::Disconnecting,
            40 => NetworkState::Connecting,
            50 => NetworkState::ConnectedLocal,
            60 => NetworkState::ConnectedSite,
            70 => NetworkState::ConnectedGlobal,
            _ => NetworkState::Unknown,
        }
    }
}

mod imp {
    use super::*;
    use glib::prelude::*;
    use glib::subclass::prelude::*;
    use glib::{ParamSpec, ParamSpecBuilderExt, Value};
    use zbus::proxy;

    pub struct Network {
        pub primary: RefCell<String>,
        pub state: std::cell::Cell<u32>,
        pub ethernet: Ethernet,
        pub wifi: Wifi,
    }

    impl Default for Network {
        fn default() -> Self {
            Self {
                primary: RefCell::new(String::new()),
                state: std::cell::Cell::new(0),
                ethernet: Ethernet::instance(),
                wifi: Wifi::instance(),
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
        fn primary_connection(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

        #[zbus(property)]
        fn activating_connection(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

        #[zbus(property)]
        fn state(&self) -> zbus::Result<u32>;
    }

    #[proxy(
        interface = "org.freedesktop.NetworkManager.Connection.Active",
        default_service = "org.freedesktop.NetworkManager"
    )]
    trait ActiveConnection {
        #[zbus(property, name = "Type")]
        fn connection_type(&self) -> zbus::Result<String>;
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Network {
        const NAME: &'static str = "Network";
        type Type = super::Network;
    }

    impl ObjectImpl for Network {
        fn constructed(&self) {
            self.parent_constructed();

            // Initialize current primary network and state
            self.update_primary();
            self.update_state();

            // Setup event listener
            self.setup_event_listener();
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: OnceLock<Vec<ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![
                    glib::ParamSpecString::builder("primary")
                        .read_only()
                        .build(),
                    glib::ParamSpecUInt::builder("state").read_only().build(),
                    glib::ParamSpecString::builder("icon-name")
                        .read_only()
                        .build(),
                ]
            })
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "primary" => self.primary.borrow().to_value(),
                "state" => self.state.get().to_value(),
                "icon-name" => self.obj().icon_name().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl Network {
        fn setup_event_listener(&self) {
            let obj = self.obj().clone();

            // Listen to ethernet and wifi icon changes
            self.ethernet.connect_notify_local(
                Some("icon-name"),
                glib::clone!(
                    #[weak]
                    obj,
                    move |_, _| {
                        obj.notify("icon-name");
                    }
                ),
            );

            self.wifi.connect_notify_local(
                Some("icon-name"),
                glib::clone!(
                    #[weak]
                    obj,
                    move |_, _| {
                        obj.notify("icon-name");
                    }
                ),
            );

            // Spawn async D-Bus listener on glib main context - fully event-driven!
            glib::MainContext::default().spawn_local(async move {
                let connection = match Connection::system().await {
                    Ok(conn) => conn,
                    Err(_) => return,
                };

                let proxy = match NetworkManagerProxy::new(&connection).await {
                    Ok(p) => p,
                    Err(_) => return,
                };

                // Listen to PrimaryConnection and State property changes
                use futures_util::StreamExt;
                let mut primary_stream = proxy.receive_primary_connection_changed().await;
                let mut state_stream = proxy.receive_state_changed().await;

                loop {
                    tokio::select! {
                        result = primary_stream.next() => {
                            if result.is_some() {
                                let old_primary = obj.imp().primary.borrow().clone();
                                obj.imp().update_primary();
                                let new_primary = obj.imp().primary.borrow().clone();

                                if old_primary != new_primary {
                                    obj.notify("primary");
                                    obj.notify("icon-name");
                                }
                            } else {
                                break;
                            }
                        }
                        result = state_stream.next() => {
                            if result.is_some() {
                                let old_state = obj.imp().state.get();
                                obj.imp().update_state();
                                let new_state = obj.imp().state.get();

                                if old_state != new_state {
                                    obj.notify("state");
                                    obj.notify("icon-name");
                                }
                            } else {
                                break;
                            }
                        }
                    }
                }
            });
        }

        fn update_primary(&self) {
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

                let active_conn_path = match self.get_primary_connection_path(&nm_proxy).await {
                    Some(path) => path,
                    None => {
                        self.primary.replace(String::new());
                        return;
                    }
                };

                let conn_type = self
                    .determine_connection_type(&connection, active_conn_path)
                    .await;
                self.primary.replace(conn_type);
            });
        }

        async fn get_primary_connection_path(
            &self,
            nm_proxy: &NetworkManagerProxy<'_>,
        ) -> Option<zbus::zvariant::OwnedObjectPath> {
            if let Ok(path) = nm_proxy.primary_connection().await {
                if path.as_str() != "/" {
                    return Some(path);
                }
            }

            if let Ok(path) = nm_proxy.activating_connection().await {
                if path.as_str() != "/" {
                    return Some(path);
                }
            }

            None
        }

        async fn determine_connection_type(
            &self,
            connection: &Connection,
            active_conn_path: zbus::zvariant::OwnedObjectPath,
        ) -> String {
            let active_conn_proxy =
                match ActiveConnectionProxy::builder(connection).path(active_conn_path) {
                    Ok(builder) => match builder.build().await {
                        Ok(p) => p,
                        Err(_) => return String::new(),
                    },
                    Err(_) => return String::new(),
                };

            active_conn_proxy
                .connection_type()
                .await
                .unwrap_or_default()
        }

        fn update_state(&self) {
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

                if let Ok(state) = nm_proxy.state().await {
                    self.state.set(state);
                }
            });
        }
    }
}
