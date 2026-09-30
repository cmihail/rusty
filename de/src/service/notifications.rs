use async_channel::Sender;
use glib::subclass::prelude::*;
use glib::Object;
use gtk4::glib;
use gtk4::prelude::*;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use zbus::Connection;

glib::wrapper! {
    pub struct Notifications(ObjectSubclass<imp::Notifications>);
}

thread_local! {
    static NOTIFICATIONS_INSTANCE: RefCell<Option<Notifications>> = const { RefCell::new(None) };
}

impl Notifications {
    pub fn instance() -> Notifications {
        NOTIFICATIONS_INSTANCE.with(|instance| {
            let mut instance_mut = instance.borrow_mut();
            if instance_mut.is_none() {
                let obj: Notifications = Object::builder().build();
                *instance_mut = Some(obj);
            }
            instance_mut.as_ref().unwrap().clone()
        })
    }

    fn new() -> Self {
        Object::builder().build()
    }

    pub fn ignore_timeout(&self) -> bool {
        self.imp().ignore_timeout.get()
    }

    pub fn set_ignore_timeout(&self, value: bool) {
        self.imp().ignore_timeout.set(value);
        self.notify("ignore-timeout");
    }

    pub fn dont_disturb(&self) -> bool {
        self.imp().dont_disturb.get()
    }

    pub fn set_dont_disturb(&self, value: bool) {
        self.imp().dont_disturb.set(value);
        self.notify("dont-disturb");
    }

    pub fn notifications(&self) -> Vec<Notification> {
        let notifs = self.imp().notifications.lock().unwrap();
        notifs.values().cloned().collect()
    }

    pub fn get_notification(&self, id: u32) -> Option<Notification> {
        let notifs = self.imp().notifications.lock().unwrap();
        notifs.get(&id).cloned()
    }

    pub fn dismiss(&self, id: u32) {
        self.imp()
            .close_notification(id, ClosedReason::DismissedByUser);
    }

    pub fn invoke_action(&self, id: u32, action_key: &str) {
        self.imp().invoke_action(id, action_key);
    }

    pub fn send_notification(&self, app_name: &str, summary: &str, body: &str) -> u32 {
        let imp = self.imp();
        let mut notifs = imp.notifications.lock().unwrap();
        let mut next_id = imp.next_id.lock().unwrap();

        let id = *next_id;
        *next_id += 1;

        let notification = Notification {
            id,
            app_name: app_name.to_string(),
            app_icon: "dialog-error".to_string(),
            summary: summary.to_string(),
            body: body.to_string(),
            actions: Vec::new(),
            urgency: Urgency::Critical,
            time: chrono::Local::now().timestamp(),
            expire_timeout: 5000,
            transient: false,
            category: None,
            desktop_entry: None,
        };

        notifs.insert(id, notification);
        drop(notifs);
        drop(next_id);

        imp.write_state();

        self.emit_by_name::<()>("notified", &[&id, &false]);

        id
    }
}

impl Default for Notifications {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Notification {
    pub id: u32,
    pub app_name: String,
    pub app_icon: String,
    pub summary: String,
    pub body: String,
    pub actions: Vec<NotificationAction>,
    pub urgency: Urgency,
    pub time: i64,
    pub expire_timeout: i32,
    #[serde(default)]
    pub transient: bool,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub desktop_entry: Option<String>,
}

/// The action a client expects when the notification body itself is clicked.
pub const DEFAULT_ACTION: &str = "default";

impl Notification {
    /// The name to show in the header. Chromium-based clients (Slack, Discord) send an
    /// empty `app_name` and identify themselves only through the `desktop-entry` hint.
    pub fn display_app_name(&self) -> String {
        // For app names like "Claude Code: session-id", show only "Claude Code".
        let trimmed = match self.app_name.find(':') {
            Some(colon_pos) => self.app_name[..colon_pos].trim(),
            None => self.app_name.trim(),
        };

        if !trimmed.is_empty() {
            return trimmed.to_string();
        }

        let Some(entry_id) = self.desktop_entry.as_deref() else {
            return String::new();
        };

        crate::service::desktop_entry::lookup(entry_id)
            .and_then(|entry| entry.name)
            .unwrap_or_else(|| entry_id.to_string())
    }

    /// The action a click on the notification body invokes.
    pub fn click_action(&self) -> Option<&NotificationAction> {
        self.actions
            .iter()
            .find(|action| action.id == DEFAULT_ACTION)
            .or_else(|| self.actions.first())
    }

    /// The actions that get a button of their own.
    pub fn button_actions(&self) -> impl Iterator<Item = &NotificationAction> {
        self.actions
            .iter()
            .filter(|action| action.id != DEFAULT_ACTION)
    }

    /// Window classes that may belong to the sending app, best guess first.
    pub fn window_classes(&self) -> Vec<String> {
        let mut classes = Vec::new();
        let mut push = |value: &str| {
            let value = value.trim();
            if !value.is_empty() && !classes.iter().any(|c: &String| c == value) {
                classes.push(value.to_string());
            }
        };

        if let Some(entry_id) = self.desktop_entry.as_deref() {
            if let Some(entry) = crate::service::desktop_entry::lookup(entry_id) {
                if let Some(wm_class) = entry.startup_wm_class {
                    push(&wm_class);
                }
            }
            push(entry_id);
            // Snap desktop files are named "<instance>_<app>", while the window class is
            // usually just the app part.
            if let Some((_, app)) = entry_id.rsplit_once('_') {
                push(app);
            }
        }

        push(&self.display_app_name());

        classes
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotificationAction {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Urgency {
    Low = 0,
    Normal = 1,
    Critical = 2,
}

impl Urgency {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Urgency::Low,
            1 => Urgency::Normal,
            2 => Urgency::Critical,
            _ => Urgency::Normal,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClosedReason {
    Expired = 1,
    DismissedByUser = 2,
    Closed = 3,
    Undefined = 4,
}

mod imp {
    use super::*;
    use glib::subclass::prelude::*;
    use glib::{ParamSpec, ParamSpecBuilderExt, Value};
    use std::cell::Cell;
    use std::fs;
    use std::io::Write;
    use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
    use std::path::PathBuf;
    use zbus::interface;

    type ActionSender = Arc<Mutex<Option<tokio::sync::mpsc::UnboundedSender<(u32, String)>>>>;
    type CloseSender = Arc<Mutex<Option<tokio::sync::mpsc::UnboundedSender<(u32, ClosedReason)>>>>;

    #[derive(Serialize, Deserialize)]
    struct NotificationsState {
        notifications: Vec<Notification>,
        ignore_timeout: bool,
        dont_disturb: bool,
    }

    pub struct Notifications {
        pub ignore_timeout: Cell<bool>,
        pub dont_disturb: Cell<bool>,
        pub notifications: Arc<Mutex<HashMap<u32, Notification>>>,
        pub next_id: Arc<Mutex<u32>>,
        pub action_tx: ActionSender,
        pub close_tx: CloseSender,
        pub state_file: PathBuf,
    }

    impl Default for Notifications {
        fn default() -> Self {
            let state_dir = if let Some(state_home) = std::env::var_os("XDG_STATE_HOME") {
                PathBuf::from(state_home).join("rusty-de")
            } else if let Some(home) = std::env::var_os("HOME") {
                PathBuf::from(home).join(".local/state/rusty-de")
            } else if let Some(runtime_dir) = std::env::var_os("XDG_RUNTIME_DIR") {
                PathBuf::from(runtime_dir).join("rusty-de")
            } else {
                PathBuf::from("/tmp/rusty-de")
            };

            let state_file = state_dir.join("notifications.json");

            Self {
                ignore_timeout: Cell::new(false),
                dont_disturb: Cell::new(false),
                notifications: Arc::new(Mutex::new(HashMap::new())),
                next_id: Arc::new(Mutex::new(1)),
                action_tx: Arc::new(Mutex::new(None)),
                close_tx: Arc::new(Mutex::new(None)),
                state_file,
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Notifications {
        const NAME: &'static str = "Notifications";
        type Type = super::Notifications;
    }

    impl ObjectImpl for Notifications {
        fn constructed(&self) {
            self.parent_constructed();

            // Load persisted state
            self.load_state();

            // Start the notification daemon
            self.start_daemon();
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: OnceLock<Vec<ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![
                    glib::ParamSpecBoolean::builder("ignore-timeout")
                        .read_only()
                        .build(),
                    glib::ParamSpecBoolean::builder("dont-disturb")
                        .read_only()
                        .build(),
                ]
            })
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "ignore-timeout" => self.ignore_timeout.get().to_value(),
                "dont-disturb" => self.dont_disturb.get().to_value(),
                _ => unimplemented!(),
            }
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: OnceLock<Vec<glib::subclass::Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![
                    glib::subclass::Signal::builder("notified")
                        .param_types([u32::static_type(), bool::static_type()])
                        .build(),
                    glib::subclass::Signal::builder("resolved")
                        .param_types([u32::static_type(), u32::static_type()])
                        .build(),
                ]
            })
        }
    }

    type NotifiedSender = Arc<Mutex<Option<Sender<(u32, bool)>>>>;
    type ResolvedSender = Arc<Mutex<Option<Sender<(u32, ClosedReason)>>>>;
    type WriteStateSender = Arc<Mutex<Option<Sender<()>>>>;

    struct NotificationsDaemon {
        notifications: Arc<Mutex<HashMap<u32, Notification>>>,
        next_id: Arc<Mutex<u32>>,
        ignore_timeout: Arc<Mutex<bool>>,
        notified_tx: NotifiedSender,
        resolved_tx: ResolvedSender,
        write_state_tx: WriteStateSender,
    }

    impl NotificationsDaemon {
        #[allow(clippy::too_many_arguments)]
        fn create_notification(
            app_name: String,
            app_icon: String,
            summary: String,
            body: String,
            actions: Vec<String>,
            hints: &HashMap<String, zbus::zvariant::Value<'_>>,
            expire_timeout: i32,
            id: u32,
        ) -> Notification {
            let mut notification_actions = Vec::new();
            let mut i = 0;
            while i + 1 < actions.len() {
                notification_actions.push(NotificationAction {
                    id: actions[i].clone(),
                    label: actions[i + 1].clone(),
                });
                i += 2;
            }

            let urgency = hints
                .get("urgency")
                .and_then(|v| v.downcast_ref::<u8>().ok())
                .map(Urgency::from_u8)
                .unwrap_or(Urgency::Normal);

            let transient = hints
                .get("transient")
                .and_then(|v| v.downcast_ref::<bool>().ok())
                .unwrap_or(false);

            let category = hints
                .get("category")
                .and_then(|v| v.downcast_ref::<String>().ok())
                .map(|s| s.to_string());

            let desktop_entry = hints
                .get("desktop-entry")
                .and_then(|v| v.downcast_ref::<String>().ok())
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty());

            Notification {
                id,
                app_name,
                app_icon,
                summary,
                body,
                actions: notification_actions,
                urgency,
                time: chrono::Local::now().timestamp(),
                expire_timeout,
                transient,
                category,
                desktop_entry,
            }
        }

        fn setup_notification_timeout(
            id: u32,
            expire_timeout: i32,
            notifications: Arc<Mutex<HashMap<u32, Notification>>>,
            ignore_timeout: Arc<Mutex<bool>>,
            resolved_tx: ResolvedSender,
        ) {
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(expire_timeout as u64));

                let still_ignore = *ignore_timeout.lock().unwrap();
                let notifs = notifications.lock().unwrap();
                let is_auto_dismiss =
                    notifs.get(&id).and_then(|n| n.category.as_deref()) == Some("x.clipboard");
                let exists = notifs.contains_key(&id);
                drop(notifs);

                if still_ignore && !is_auto_dismiss {
                    return;
                }

                if !exists {
                    return;
                }

                if let Ok(tx) = resolved_tx.lock() {
                    if let Some(sender) = tx.as_ref() {
                        let _ = sender.send_blocking((id, ClosedReason::Expired));
                    }
                }
            });
        }
    }

    #[interface(name = "org.freedesktop.Notifications")]
    impl NotificationsDaemon {
        #[allow(clippy::too_many_arguments)]
        async fn notify(
            &self,
            app_name: String,
            replaces_id: u32,
            app_icon: String,
            summary: String,
            body: String,
            actions: Vec<String>,
            hints: HashMap<String, zbus::zvariant::Value<'_>>,
            expire_timeout: i32,
        ) -> u32 {
            let (id, replaced) = {
                let mut notifs = self.notifications.lock().unwrap();
                let mut next_id = self.next_id.lock().unwrap();

                let id = if replaces_id > 0 && notifs.contains_key(&replaces_id) {
                    replaces_id
                } else {
                    let id = *next_id;
                    *next_id += 1;
                    id
                };

                let replaced = notifs.contains_key(&id);

                let notification = Self::create_notification(
                    app_name,
                    app_icon,
                    summary,
                    body,
                    actions,
                    &hints,
                    expire_timeout,
                    id,
                );

                notifs.insert(id, notification);
                (id, replaced)
            };

            // Trigger write_state
            let write_sender = self.write_state_tx.lock().ok().and_then(|tx| tx.clone());
            if let Some(sender) = write_sender {
                let _ = sender.send(()).await;
            }

            let sender = self.notified_tx.lock().ok().and_then(|tx| tx.clone());
            if let Some(sender) = sender {
                let _ = sender.send((id, replaced)).await;
            }

            let ignore = *self.ignore_timeout.lock().unwrap();
            let is_auto_dismiss = {
                let notifs = self.notifications.lock().unwrap();
                notifs.get(&id).and_then(|n| n.category.as_deref()) == Some("x.clipboard")
            };

            if (!ignore || is_auto_dismiss) && expire_timeout > 0 {
                Self::setup_notification_timeout(
                    id,
                    expire_timeout,
                    self.notifications.clone(),
                    self.ignore_timeout.clone(),
                    self.resolved_tx.clone(),
                );
            }

            id
        }

        async fn close_notification(
            &self,
            id: u32,
            #[zbus(signal_context)] ctxt: zbus::SignalContext<'_>,
        ) {
            let existed = {
                let mut notifs = self.notifications.lock().unwrap();
                notifs.remove(&id).is_some()
            };

            if existed {
                // Trigger write_state
                let write_sender = self.write_state_tx.lock().ok().and_then(|tx| tx.clone());
                if let Some(sender) = write_sender {
                    let _ = sender.send(()).await;
                }

                // Emit NotificationClosed signal
                let _ = Self::notification_closed(&ctxt, id, ClosedReason::Closed as u32).await;

                // Send to local handler
                let sender = self.resolved_tx.lock().ok().and_then(|tx| tx.clone());
                if let Some(sender) = sender {
                    let _ = sender.send((id, ClosedReason::Closed)).await;
                }
            }
        }

        async fn get_capabilities(&self) -> Vec<String> {
            vec![
                "actions".to_string(),
                "body".to_string(),
                "persistence".to_string(),
            ]
        }

        async fn get_server_information(&self) -> (String, String, String, String) {
            (
                "rusty-de".to_string(),
                "cmihail".to_string(),
                "0.1.0".to_string(),
                "1.2".to_string(),
            )
        }

        #[zbus(signal)]
        async fn notification_closed(
            ctxt: &zbus::SignalContext<'_>,
            id: u32,
            reason: u32,
        ) -> zbus::Result<()>;

        #[zbus(signal)]
        async fn action_invoked(
            ctxt: &zbus::SignalContext<'_>,
            id: u32,
            action_key: String,
        ) -> zbus::Result<()>;
    }

    impl Notifications {
        fn load_state(&self) {
            if !self.state_file.exists() {
                return;
            }

            if let Ok(contents) = fs::read_to_string(&self.state_file) {
                if let Ok(state) = serde_json::from_str::<NotificationsState>(&contents) {
                    self.ignore_timeout.set(state.ignore_timeout);
                    self.dont_disturb.set(state.dont_disturb);

                    let mut notifs = self.notifications.lock().unwrap();
                    let mut max_id = 0;

                    for notification in state.notifications {
                        if notification.id > max_id {
                            max_id = notification.id;
                        }
                        notifs.insert(notification.id, notification);
                    }

                    if max_id > 0 {
                        *self.next_id.lock().unwrap() = max_id + 1;
                    }
                }
            }
        }

        pub fn write_state(&self) {
            let notifs = self.notifications.lock().unwrap();
            let non_transient_notifs: Vec<Notification> =
                notifs.values().filter(|n| !n.transient).cloned().collect();

            let state = NotificationsState {
                notifications: non_transient_notifs,
                ignore_timeout: self.ignore_timeout.get(),
                dont_disturb: self.dont_disturb.get(),
            };

            drop(notifs);

            // Notification bodies can hold 2FA codes and message previews, so keep them owner-only.
            if let Ok(json) = serde_json::to_string_pretty(&state) {
                if let Some(parent) = self.state_file.parent() {
                    if !parent.exists()
                        && fs::DirBuilder::new()
                            .recursive(true)
                            .mode(0o700)
                            .create(parent)
                            .is_err()
                    {
                        return;
                    }
                }

                if let Ok(mut file) = fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .mode(0o600)
                    .open(&self.state_file)
                {
                    file.set_permissions(fs::Permissions::from_mode(0o600)).ok();
                    file.write_all(json.as_bytes()).ok();
                }
            }
        }

        #[allow(clippy::too_many_arguments)]
        async fn setup_dbus_connection(
            notifications: Arc<Mutex<HashMap<u32, Notification>>>,
            next_id: Arc<Mutex<u32>>,
            ignore_timeout: Arc<Mutex<bool>>,
            notified_tx: NotifiedSender,
            resolved_tx: ResolvedSender,
            write_state_tx: WriteStateSender,
            mut action_rx: tokio::sync::mpsc::UnboundedReceiver<(u32, String)>,
            mut close_rx: tokio::sync::mpsc::UnboundedReceiver<(u32, ClosedReason)>,
        ) -> Result<(), Box<dyn std::error::Error>> {
            let connection = Connection::session().await?;

            let daemon = NotificationsDaemon {
                notifications,
                next_id,
                ignore_timeout,
                notified_tx,
                resolved_tx,
                write_state_tx,
            };

            let object_path = "/org/freedesktop/Notifications";

            connection.object_server().at(object_path, daemon).await?;

            connection
                .request_name("org.freedesktop.Notifications")
                .await?;

            // Spawn task to handle action invocations
            let connection_clone = connection.clone();
            tokio::spawn(async move {
                while let Some((id, action_key)) = action_rx.recv().await {
                    let iface_ref = connection_clone
                        .object_server()
                        .interface::<_, NotificationsDaemon>(object_path)
                        .await;

                    if let Ok(iface) = iface_ref {
                        let signal_ctxt = iface.signal_context();
                        let _ =
                            NotificationsDaemon::action_invoked(signal_ctxt, id, action_key).await;
                    }
                }
            });

            // Spawn task to handle notification closures
            let connection_clone = connection.clone();
            tokio::spawn(async move {
                while let Some((id, reason)) = close_rx.recv().await {
                    let iface_ref = connection_clone
                        .object_server()
                        .interface::<_, NotificationsDaemon>(object_path)
                        .await;

                    if let Ok(iface) = iface_ref {
                        let signal_ctxt = iface.signal_context();
                        let _ = NotificationsDaemon::notification_closed(
                            signal_ctxt,
                            id,
                            reason as u32,
                        )
                        .await;
                    }
                }
            });

            std::future::pending::<()>().await;
            Ok(())
        }

        fn start_daemon(&self) {
            let obj = self.obj();
            let notifications = self.notifications.clone();
            let next_id = self.next_id.clone();
            let ignore_timeout = Arc::new(Mutex::new(self.ignore_timeout.get()));

            let (notified_tx, notified_rx) = async_channel::unbounded();
            let (resolved_tx, resolved_rx) = async_channel::unbounded();
            let (write_state_tx, write_state_rx) = async_channel::unbounded();
            let (action_tx_async, action_rx_async) = tokio::sync::mpsc::unbounded_channel();
            let (close_tx_async, close_rx_async) = tokio::sync::mpsc::unbounded_channel();

            let notified_tx = Arc::new(Mutex::new(Some(notified_tx)));
            let resolved_tx = Arc::new(Mutex::new(Some(resolved_tx)));
            let write_state_tx = Arc::new(Mutex::new(Some(write_state_tx)));

            // Store action_tx and close_tx for use by invoke_action and close_notification
            *self.action_tx.lock().unwrap() = Some(action_tx_async);
            *self.close_tx.lock().unwrap() = Some(close_tx_async);

            // Spawn async tasks to handle messages from D-Bus daemon
            let obj_clone = obj.clone();
            glib::spawn_future_local(async move {
                while let Ok((id, replaced)) = notified_rx.recv().await {
                    obj_clone.emit_by_name::<()>("notified", &[&id, &replaced]);
                }
            });

            let obj_clone = obj.clone();
            glib::spawn_future_local(async move {
                while let Ok((id, reason)) = resolved_rx.recv().await {
                    obj_clone.imp().close_notification_local(id, reason);
                }
            });

            let obj_clone = obj.clone();
            glib::spawn_future_local(async move {
                while let Ok(()) = write_state_rx.recv().await {
                    obj_clone.imp().write_state();
                }
            });

            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async move {
                    if Self::setup_dbus_connection(
                        notifications,
                        next_id,
                        ignore_timeout,
                        notified_tx,
                        resolved_tx,
                        write_state_tx,
                        action_rx_async,
                        close_rx_async,
                    )
                    .await
                    .is_err()
                    {}
                });
            });
        }

        pub fn close_notification(&self, id: u32, reason: ClosedReason) {
            let mut notifs = self.notifications.lock().unwrap();
            if notifs.remove(&id).is_some() {
                drop(notifs);

                // Write state to disk
                self.write_state();

                // Emit D-Bus NotificationClosed signal
                if let Ok(tx) = self.close_tx.lock() {
                    if let Some(sender) = tx.as_ref() {
                        let _ = sender.send((id, reason));
                    }
                }

                // Emit local resolved signal
                self.close_notification_local(id, reason);
            }
        }

        fn close_notification_local(&self, id: u32, reason: ClosedReason) {
            // Remove expired notifications from the hashmap
            if reason == ClosedReason::Expired {
                let mut notifs = self.notifications.lock().unwrap();
                notifs.remove(&id);
                drop(notifs);
            }

            self.write_state();
            let obj = self.obj();
            obj.emit_by_name::<()>("resolved", &[&id, &(reason as u32)]);
        }

        pub fn invoke_action(&self, id: u32, action_key: &str) {
            if let Ok(tx) = self.action_tx.lock() {
                if let Some(sender) = tx.as_ref() {
                    let _ = sender.send((id, action_key.to_string()));
                }
            }
        }
    }
}
