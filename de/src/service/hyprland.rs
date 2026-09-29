use async_channel::{Receiver, Sender};
use glib::subclass::prelude::*;
use glib::Object;
use gtk4::glib;
use gtk4::prelude::*;
use serde::Deserialize;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};

glib::wrapper! {
    pub struct Hyprland(ObjectSubclass<imp::Hyprland>);
}

thread_local! {
    static HYPRLAND_INSTANCE: RefCell<Option<Hyprland>> = const { RefCell::new(None) };
}

/// The `address:` argument hyprctl dispatchers expect. `Client` stores addresses with the
/// `0x` stripped, so it has to be put back.
pub fn address_arg(address: &str) -> String {
    format!("address:0x{}", address)
}

impl Hyprland {
    pub fn instance() -> Option<Hyprland> {
        HYPRLAND_INSTANCE.with(|instance| {
            let mut instance_mut = instance.borrow_mut();
            if instance_mut.is_none() {
                let his = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").ok()?;
                if his.is_empty() {
                    return None;
                }

                let obj: Hyprland = Object::builder().build();
                obj.imp().start_socket_listener(&his);
                *instance_mut = Some(obj);
            }
            instance_mut.clone()
        })
    }

    fn new() -> Option<Self> {
        let his = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").ok()?;
        if his.is_empty() {
            return None;
        }

        let obj: Hyprland = Object::builder().build();
        obj.imp().start_socket_listener(&his);
        Some(obj)
    }

    pub fn monitors(&self) -> Vec<Monitor> {
        let monitors = self.imp().monitors.lock().unwrap();
        monitors.values().cloned().collect()
    }

    pub fn focused_monitor(&self) -> Option<Monitor> {
        self.imp().focused_monitor.lock().unwrap().clone()
    }

    pub fn focused_client(&self) -> Option<Client> {
        self.imp().focused_client.lock().unwrap().clone()
    }

    pub fn clients(&self) -> Vec<Client> {
        let runtime_dir =
            std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/run/user/1000".to_string());
        let his = match std::env::var("HYPRLAND_INSTANCE_SIGNATURE") {
            Ok(sig) => sig,
            Err(_) => return Vec::new(),
        };
        let socket_path = format!("{}/hypr/{}/.socket.sock", runtime_dir, his);

        match imp::Hyprland::message(&socket_path, "j/clients") {
            Ok(json) => {
                if let Ok(clients_data) = serde_json::from_str::<Vec<ClientJson>>(&json) {
                    clients_data
                        .iter()
                        .map(imp::Hyprland::create_client_from_json)
                        .collect()
                } else {
                    Vec::new()
                }
            }
            Err(_) => Vec::new(),
        }
    }

    /// Focus the first open window matching any of `classes`, best guess first.
    pub fn focus_window_by_class(&self, classes: &[String]) -> bool {
        if classes.is_empty() {
            return false;
        }

        let clients = self.clients();

        for class in classes {
            let Some(client) = clients.iter().find(|c| c.class.eq_ignore_ascii_case(class)) else {
                continue;
            };

            return self
                .dispatch(&["focuswindow", &address_arg(&client.address)])
                .is_ok();
        }

        false
    }

    pub fn dispatch(&self, args: &[&str]) -> Result<(), String> {
        use std::process::Command;

        let output = Command::new("hyprctl")
            .arg("dispatch")
            .args(args)
            .output()
            .map_err(|e| format!("Failed to run hyprctl: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("hyprctl dispatch failed: {}", stderr));
        }

        Ok(())
    }
}

impl Default for Hyprland {
    fn default() -> Self {
        Self::new().expect("Hyprland is not running")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Monitor {
    pub id: i32,
    pub name: String,
    pub model: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub active_workspace: Workspace,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Workspace {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Client {
    pub address: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub title: String,
    pub class: String,
    pub workspace: Workspace,
    pub monitor: i32,
}

#[derive(Debug, Deserialize)]
struct MonitorJson {
    id: i32,
    name: String,
    model: String,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    #[serde(rename = "activeWorkspace")]
    active_workspace: WorkspaceJson,
    focused: bool,
}

#[derive(Debug, Deserialize)]
struct WorkspaceJson {
    id: i32,
    name: String,
}

#[derive(Debug, Deserialize)]
struct ClientJson {
    address: String,
    at: Vec<i32>,
    size: Vec<i32>,
    title: String,
    class: String,
    workspace: WorkspaceJson,
    monitor: i32,
}

mod imp {
    use super::*;
    use glib::subclass::Signal;
    use std::sync::OnceLock;

    #[derive(Default)]
    pub struct Hyprland {
        pub monitors: Arc<Mutex<HashMap<i32, Monitor>>>,
        pub focused_monitor: Arc<Mutex<Option<Monitor>>>,
        pub focused_client: Arc<Mutex<Option<Client>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Hyprland {
        const NAME: &'static str = "Hyprland";
        type Type = super::Hyprland;
    }

    impl ObjectImpl for Hyprland {
        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder("event")
                        .param_types([String::static_type(), String::static_type()])
                        .build(),
                    Signal::builder("monitor-added").build(),
                    Signal::builder("monitor-removed").build(),
                    Signal::builder("focused-monitor-changed").build(),
                    Signal::builder("focused-client-changed").build(),
                ]
            })
        }
    }

    impl Hyprland {
        fn setup_event_streams(
            event_rx: Receiver<(String, String)>,
            monitor_rx: Receiver<Option<Monitor>>,
            client_rx: Receiver<Option<Client>>,
            obj: &super::Hyprland,
            focused_monitor: Arc<Mutex<Option<Monitor>>>,
            focused_client: Arc<Mutex<Option<Client>>>,
        ) {
            let obj_clone = obj.clone();
            glib::spawn_future_local(async move {
                while let Ok((event, args)) = event_rx.recv().await {
                    obj_clone.emit_by_name::<()>("event", &[&event, &args]);
                }
            });

            let obj_clone = obj.clone();
            glib::spawn_future_local(async move {
                while let Ok(monitor) = monitor_rx.recv().await {
                    *focused_monitor.lock().unwrap() = monitor;
                    obj_clone.emit_by_name::<()>("focused-monitor-changed", &[]);
                }
            });

            let obj_clone = obj.clone();
            glib::spawn_future_local(async move {
                while let Ok(client) = client_rx.recv().await {
                    *focused_client.lock().unwrap() = client;
                    obj_clone.emit_by_name::<()>("focused-client-changed", &[]);
                }
            });
        }

        fn start_socket_thread(
            socket2_path: String,
            socket_path: String,
            monitors: Arc<Mutex<HashMap<i32, Monitor>>>,
            event_tx: Sender<(String, String)>,
            monitor_tx: Sender<Option<Monitor>>,
            client_tx: Sender<Option<Client>>,
        ) {
            std::thread::spawn(move || {
                let start_time = std::time::Instant::now();
                let max_duration = std::time::Duration::from_secs(10);

                loop {
                    match UnixStream::connect(&socket2_path) {
                        Ok(stream) => {
                            Self::process_socket_events(
                                stream,
                                &socket_path,
                                &monitors,
                                &event_tx,
                                &monitor_tx,
                                &client_tx,
                            );
                        }
                        Err(e) => {
                            let err_msg = format!("Failed to connect: {}", e);
                            Self::send_error_notification(&err_msg);
                        }
                    }

                    if start_time.elapsed() >= max_duration {
                        Self::send_error_notification(
                            "Failed to connect to Hyprland socket after 10 seconds, giving up",
                        );
                        break;
                    }

                    std::thread::sleep(std::time::Duration::from_secs(2));
                }
            });
        }

        fn process_socket_events(
            stream: UnixStream,
            socket_path: &str,
            monitors: &Arc<Mutex<HashMap<i32, Monitor>>>,
            event_tx: &Sender<(String, String)>,
            monitor_tx: &Sender<Option<Monitor>>,
            client_tx: &Sender<Option<Client>>,
        ) {
            let reader = BufReader::new(stream);
            for line in reader.lines() {
                match line {
                    Ok(event_line) => {
                        if let Err(e) = Self::handle_event(
                            &event_line,
                            socket_path,
                            monitors,
                            event_tx,
                            monitor_tx,
                            client_tx,
                        ) {
                            let err_msg = format!("Error handling event: {}", e);
                            Self::send_error_notification(&err_msg);
                        }
                    }
                    Err(_) => {
                        break;
                    }
                }
            }
        }

        fn send_error_notification(err_msg: &str) {
            let msg = err_msg.to_string();
            crate::service::idle_add_safe(move || {
                crate::service::notifications::Notifications::instance().send_notification(
                    "rusty-de",
                    "Hyprland Error",
                    &msg,
                );
            });
        }

        fn try_init_with_retry(&self, socket_path: &str) -> Result<(), Box<dyn std::error::Error>> {
            let mut last_error = None;
            for attempt in 1..=5 {
                match self.init_data(socket_path) {
                    Ok(_) => return Ok(()),
                    Err(e) => {
                        last_error = Some(e);
                        if attempt < 5 {
                            std::thread::sleep(std::time::Duration::from_secs(2));
                        }
                    }
                }
            }
            Err(last_error.unwrap())
        }

        pub fn start_socket_listener(&self, his: &str) {
            let runtime_dir =
                std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/run/user/1000".to_string());

            let socket2_path = format!("{}/hypr/{}/.socket2.sock", runtime_dir, his);
            let socket_path = format!("{}/hypr/{}/.socket.sock", runtime_dir, his);

            if let Err(e) = self.try_init_with_retry(&socket_path) {
                let err_msg = format!("Failed to initialize data after 5 attempts: {}", e);
                Self::send_error_notification(&err_msg);
                return;
            }

            let (event_tx, event_rx) = async_channel::unbounded();
            let (monitor_tx, monitor_rx) = async_channel::unbounded();
            let (client_tx, client_rx) = async_channel::unbounded();

            let obj = self.obj();
            Self::setup_event_streams(
                event_rx,
                monitor_rx,
                client_rx,
                &obj,
                self.focused_monitor.clone(),
                self.focused_client.clone(),
            );

            Self::start_socket_thread(
                socket2_path,
                socket_path,
                self.monitors.clone(),
                event_tx,
                monitor_tx,
                client_tx,
            );
        }

        fn init_data(&self, socket_path: &str) -> Result<(), Box<dyn std::error::Error>> {
            let monitors_json = Self::message(socket_path, "j/monitors")?;
            let monitors_data: Vec<MonitorJson> = serde_json::from_str(&monitors_json)?;
            Self::populate_monitors(&monitors_data, &self.monitors, &self.focused_monitor);

            let active_window_json = Self::message(socket_path, "j/activewindow")?;
            Self::populate_focused_client(&active_window_json, &self.focused_client);

            Ok(())
        }

        fn populate_monitors(
            monitors_data: &[MonitorJson],
            monitors: &Arc<Mutex<HashMap<i32, Monitor>>>,
            focused_monitor: &Arc<Mutex<Option<Monitor>>>,
        ) {
            let mut monitors_lock = monitors.lock().unwrap();

            for mon_data in monitors_data {
                let monitor = Self::create_monitor_from_json(mon_data);
                monitors_lock.insert(mon_data.id, monitor.clone());

                if mon_data.focused {
                    *focused_monitor.lock().unwrap() = Some(monitor);
                }
            }
        }

        fn create_monitor_from_json(mon_data: &MonitorJson) -> Monitor {
            Monitor {
                id: mon_data.id,
                name: mon_data.name.clone(),
                model: mon_data.model.clone(),
                x: mon_data.x,
                y: mon_data.y,
                width: mon_data.width,
                height: mon_data.height,
                active_workspace: Workspace {
                    id: mon_data.active_workspace.id,
                    name: mon_data.active_workspace.name.clone(),
                },
            }
        }

        fn populate_focused_client(
            active_window_json: &str,
            focused_client: &Arc<Mutex<Option<Client>>>,
        ) {
            if active_window_json.trim().is_empty() || active_window_json == "{}" {
                return;
            }

            if let Ok(client_data) = serde_json::from_str::<ClientJson>(active_window_json) {
                let client = Self::create_client_from_json(&client_data);
                *focused_client.lock().unwrap() = Some(client);
            }
        }

        pub(super) fn create_client_from_json(client_data: &ClientJson) -> Client {
            Client {
                address: client_data.address.replace("0x", ""),
                x: client_data.at.first().copied().unwrap_or(0),
                y: client_data.at.get(1).copied().unwrap_or(0),
                width: client_data.size.first().copied().unwrap_or(0),
                height: client_data.size.get(1).copied().unwrap_or(0),
                title: client_data.title.clone(),
                class: client_data.class.clone(),
                workspace: Workspace {
                    id: client_data.workspace.id,
                    name: client_data.workspace.name.clone(),
                },
                monitor: client_data.monitor,
            }
        }

        pub(super) fn message(
            socket_path: &str,
            command: &str,
        ) -> Result<String, Box<dyn std::error::Error>> {
            let mut stream = UnixStream::connect(socket_path)?;
            stream.write_all(command.as_bytes())?;

            let mut response = String::new();
            let mut reader = BufReader::new(stream);

            loop {
                let mut buf = vec![0u8; 4096];
                let n = std::io::Read::read(&mut reader, &mut buf)?;
                if n == 0 {
                    break;
                }

                let chunk = String::from_utf8_lossy(&buf[..n]);
                response.push_str(&chunk);

                if chunk.contains('\x04') {
                    response = response.replace('\x04', "");
                    break;
                }
            }

            Ok(response)
        }

        fn handle_workspace_event(
            _args: &str,
            socket_path: &str,
            monitors: &Arc<Mutex<HashMap<i32, Monitor>>>,
        ) -> Result<(), Box<dyn std::error::Error>> {
            Self::sync_monitors(socket_path, monitors)
        }

        fn handle_focusedmon_event(
            args: &str,
            socket_path: &str,
            monitors: &Arc<Mutex<HashMap<i32, Monitor>>>,
            monitor_tx: &Sender<Option<Monitor>>,
        ) -> Result<(), Box<dyn std::error::Error>> {
            let argv: Vec<&str> = args.split(',').collect();
            if argv.is_empty() {
                return Ok(());
            }

            Self::sync_monitors(socket_path, monitors)?;
            Self::send_focused_monitor(argv[0], monitors, monitor_tx);
            Ok(())
        }

        fn send_focused_monitor(
            monitor_name: &str,
            monitors: &Arc<Mutex<HashMap<i32, Monitor>>>,
            monitor_tx: &Sender<Option<Monitor>>,
        ) {
            let monitors_lock = monitors.lock().unwrap();
            for monitor in monitors_lock.values() {
                if monitor.name == monitor_name {
                    let _ = monitor_tx.send_blocking(Some(monitor.clone()));
                    break;
                }
            }
        }

        fn handle_activewindow_event(
            socket_path: &str,
            client_tx: &Sender<Option<Client>>,
        ) -> Result<(), Box<dyn std::error::Error>> {
            let active_window_json = Self::message(socket_path, "j/activewindow")?;
            if active_window_json.trim().is_empty() || active_window_json == "{}" {
                let _ = client_tx.send_blocking(None);
                return Ok(());
            }

            if let Ok(client_data) = serde_json::from_str::<ClientJson>(&active_window_json) {
                let client = Self::create_client_from_json(&client_data);
                let _ = client_tx.send_blocking(Some(client));
            }
            Ok(())
        }

        fn handle_monitor_changed_event(
            socket_path: &str,
            monitors: &Arc<Mutex<HashMap<i32, Monitor>>>,
        ) -> Result<(), Box<dyn std::error::Error>> {
            Self::sync_monitors(socket_path, monitors)
        }

        fn handle_event(
            line: &str,
            socket_path: &str,
            monitors: &Arc<Mutex<HashMap<i32, Monitor>>>,
            event_tx: &Sender<(String, String)>,
            monitor_tx: &Sender<Option<Monitor>>,
            client_tx: &Sender<Option<Client>>,
        ) -> Result<(), Box<dyn std::error::Error>> {
            let parts: Vec<&str> = line.split(">>").collect();
            if parts.len() < 2 {
                return Ok(());
            }

            let event = parts[0];
            let args = parts[1];

            // Update internal state FIRST, then notify listeners
            match event {
                "workspace" | "workspacev2" => {
                    Self::handle_workspace_event(args, socket_path, monitors)?;
                }
                "focusedmon" => {
                    Self::handle_focusedmon_event(args, socket_path, monitors, monitor_tx)?;
                }
                "activewindow" | "activewindowv2" => {
                    Self::handle_activewindow_event(socket_path, client_tx)?;
                }
                "monitoradded" | "monitoraddedv2" => {
                    Self::handle_monitor_changed_event(socket_path, monitors)?;
                }
                "monitorremoved" => {
                    Self::handle_monitor_changed_event(socket_path, monitors)?;
                }
                _ => {}
            }

            // Notify listeners AFTER state has been updated
            let _ = event_tx.send_blocking((event.to_string(), args.to_string()));

            Ok(())
        }

        fn sync_monitors(
            socket_path: &str,
            monitors: &Arc<Mutex<HashMap<i32, Monitor>>>,
        ) -> Result<(), Box<dyn std::error::Error>> {
            let monitors_json = Self::message(socket_path, "j/monitors")?;
            let monitors_data: Vec<MonitorJson> = serde_json::from_str(&monitors_json)?;

            let mut monitors_lock = monitors.lock().unwrap();
            monitors_lock.clear();

            for mon_data in monitors_data {
                let monitor = Self::create_monitor_from_json(&mon_data);
                monitors_lock.insert(mon_data.id, monitor);
            }

            Ok(())
        }
    }
}
