use clap::Parser;
use gdk4::Display;
use gio::prelude::*;
use gtk4::prelude::*;
use gtk4::Application;

mod config;
mod service;
mod widget;
mod window;

const APP_ID: &str = "com.cmihail.rusty-de";
const DBUS_NAME: &str = "com.cmihail.RustyDE";
const DBUS_PATH: &str = "/com/cmihail/RustyDE";

#[derive(Parser, Debug)]
#[command(name = "rusty-de")]
#[command(about = "GTK4 Desktop Environment Bar for Hyprland", long_about = None)]
struct Args {
    /// Toggle the control center
    #[arg(long)]
    toggle_control_center: bool,
}

fn main() {
    env_logger::init();

    let args = Args::parse();

    // If toggle flag is set, send D-Bus message and exit
    if args.toggle_control_center {
        if let Err(e) = toggle_control_center_via_dbus() {
            eprintln!("Failed to toggle control center: {}", e);
            std::process::exit(1);
        }
        return;
    }

    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(|app| {
        // Load and register resources
        let resources_bytes = include_bytes!(concat!(env!("OUT_DIR"), "/resources.gresource"));
        let resource_data = glib::Bytes::from_static(resources_bytes);
        let resources = gio::Resource::from_data(&resource_data).expect("Failed to load resources");
        gio::resources_register(&resources);

        // Add custom icon resource path
        if let Some(display) = Display::default() {
            let icon_theme = gtk4::IconTheme::for_display(&display);
            // Add resource path so GTK can find icons in our gresource bundle
            icon_theme.add_resource_path("/com/rusty-de/icons");
        }

        let provider = gtk4::CssProvider::new();
        provider.load_from_data(include_str!("style.css"));
        gtk4::style_context_add_provider_for_display(
            &Display::default().expect("Could not get default display"),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        if let Some(_hyprland) = service::hyprland::Hyprland::instance() {
            // Hyprland service initialized successfully
        }

        setup_config_file_monitor();

        setup_on_screen_display(app);
        setup_notification_popups(app);

        create_bars_for_monitors(app);

        setup_power_profile_handler();
        setup_suspend_listener(app);
    });

    app.run();
}

fn setup_power_profile_handler() {
    let battery = service::battery::Battery::instance();
    let power_profiles = service::power_profiles::PowerProfiles::instance();

    // Set initial power profile based on current battery state
    update_power_profile_based_on_battery(&battery, &power_profiles);

    // Only react to a real AC <-> battery transition, so a manual pick is not overridden
    let was_charging = std::cell::Cell::new(is_charging(&battery));

    battery.connect_notify_local(Some("state"), move |battery, _| {
        let charging = is_charging(battery);
        if charging == was_charging.get() {
            return;
        }
        was_charging.set(charging);
        update_power_profile_based_on_battery(battery, &power_profiles);
    });
}

fn setup_suspend_listener(app: &Application) {
    let app_weak = app.downgrade();

    glib::spawn_future_local(async move {
        let connection = match zbus::Connection::system().await {
            Ok(conn) => conn,
            Err(_) => {
                return;
            }
        };

        let proxy = match zbus::Proxy::new(
            &connection,
            "org.freedesktop.login1",
            "/org/freedesktop/login1",
            "org.freedesktop.login1.Manager",
        )
        .await
        {
            Ok(p) => p,
            Err(_) => {
                return;
            }
        };

        let mut stream = match proxy.receive_signal("PrepareForSleep").await {
            Ok(s) => s,
            Err(_) => {
                return;
            }
        };

        use futures_util::StreamExt;
        while let Some(signal) = stream.next().await {
            let body = signal.body();
            if let Ok((going_to_sleep,)) = body.deserialize::<(bool,)>() {
                if going_to_sleep {
                    if let Some(app) = app_weak.upgrade() {
                        app.quit();
                    }
                    break;
                }
            }
        }
    });
}

fn is_charging(battery: &service::battery::Battery) -> bool {
    use service::battery::BatteryState;

    match battery.state() {
        BatteryState::FullyCharged | BatteryState::PendingCharge | BatteryState::Charging => true,
        BatteryState::Unknown
        | BatteryState::Empty
        | BatteryState::PendingDischarge
        | BatteryState::Discharging => false,
    }
}

fn update_power_profile_based_on_battery(
    battery: &service::battery::Battery,
    power_profiles: &service::power_profiles::PowerProfiles,
) {
    let profile = if is_charging(battery) {
        "balanced"
    } else {
        "power-saver"
    };

    power_profiles.set_active_profile(profile);
}

fn setup_on_screen_display(app: &Application) {
    let _osd = window::on_screen_display::OnScreenDisplay::new(app);
}

fn setup_notification_popups(app: &Application) {
    let _popups = window::notification_popups::NotificationPopups::new(app);
}

fn attach_monitor_property_listener(
    monitor: &gtk4::gdk::Monitor,
    _monitor_index: u32,
    app: &gtk4::Application,
    bars: &std::rc::Rc<std::cell::RefCell<Vec<window::bar::Bar>>>,
) {
    // Listen specifically to connector property changes
    let app_clone = app.clone();
    let bars_clone = bars.clone();
    monitor.connect_notify_local(Some("connector"), move |monitor, _| {
        let connector = monitor.connector().map(|s| s.to_string());

        // If connector is now available, create the bar
        if let Some(connector_str) = connector {
            if !connector_str.is_empty() {
                // Check if we already have a bar for this connector
                let has_bar = bars_clone
                    .borrow()
                    .iter()
                    .any(|bar| bar.connector() == connector_str);

                if !has_bar {
                    let bar = window::bar::Bar::new(&app_clone, monitor);
                    bars_clone.borrow_mut().push(bar);
                }
            }
        }
    });
}

fn create_bars_for_monitors(app: &Application) {
    let display = match Display::default() {
        Some(d) => d,
        None => {
            service::notifications::Notifications::instance().send_notification(
                "rusty-de",
                "Display Error",
                "Could not get default display",
            );
            return;
        }
    };

    let bars = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let monitors = display.monitors();
    let monitor_count = monitors.n_items();

    // Create initial bars
    for i in 0..monitor_count {
        if let Some(monitor) = monitors
            .item(i)
            .and_then(|obj| obj.downcast::<gtk4::gdk::Monitor>().ok())
        {
            let connector = monitor
                .connector()
                .map(|s| s.to_string())
                .unwrap_or_default();

            // Skip monitors without connector (invalid/being removed)
            if connector.is_empty() {
                attach_monitor_property_listener(&monitor, i, app, &bars);
                continue;
            }

            let bar = window::bar::Bar::new(app, &monitor);
            bars.borrow_mut().push(bar);
        }
    }

    // Setup D-Bus server for IPC
    setup_dbus_server(bars.clone());

    // Listen for monitor changes
    monitors.connect_items_changed(glib::clone!(
        #[strong]
        bars,
        #[weak]
        app,
        #[weak]
        display,
        move |_, _position, _removed, _added| {
            handle_monitor_change(&display, &app, &bars);
        }
    ));
}

fn get_current_monitor_connectors(display: &Display) -> Vec<String> {
    let monitors = display.monitors();
    let mut connectors = Vec::new();

    for i in 0..monitors.n_items() {
        if let Some(monitor) = monitors
            .item(i)
            .and_then(|obj| obj.downcast::<gtk4::gdk::Monitor>().ok())
        {
            if let Some(connector) = monitor.connector() {
                let connector_str = connector.to_string();
                if !connector_str.is_empty() {
                    connectors.push(connector_str);
                }
            }
        }
    }

    connectors
}

fn remove_disconnected_bars(
    bars: &std::rc::Rc<std::cell::RefCell<Vec<window::bar::Bar>>>,
    current_connectors: &[String],
) {
    let mut bars_mut = bars.borrow_mut();

    // Use retain to keep only bars whose monitors still exist
    bars_mut.retain(|bar| {
        let bar_connector = bar.connector();
        current_connectors
            .iter()
            .any(|connector| connector == bar_connector)
    });
}

fn handle_monitor_change(
    display: &Display,
    app: &gtk4::Application,
    bars: &std::rc::Rc<std::cell::RefCell<Vec<window::bar::Bar>>>,
) {
    // Get current monitor connectors
    let current_connectors = get_current_monitor_connectors(display);

    // Remove bars for monitors that no longer exist
    remove_disconnected_bars(bars, &current_connectors);

    // Create bars for new monitors (including attaching listeners to empty connectors)
    create_missing_bars(&display.monitors(), app, bars);
}

fn create_missing_bars(
    gtk_monitors: &gio::ListModel,
    app: &gtk4::Application,
    bars: &std::rc::Rc<std::cell::RefCell<Vec<window::bar::Bar>>>,
) {
    for i in 0..gtk_monitors.n_items() {
        if let Some(monitor) = gtk_monitors
            .item(i)
            .and_then(|obj| obj.downcast::<gtk4::gdk::Monitor>().ok())
        {
            let connector = monitor
                .connector()
                .map(|s| s.to_string())
                .unwrap_or_default();

            if connector.is_empty() {
                attach_monitor_property_listener(&monitor, i, app, bars);
                continue;
            }

            let has_bar = bars.borrow().iter().any(|bar| bar.connector() == connector);

            if !has_bar {
                let bar = window::bar::Bar::new(app, &monitor);
                bars.borrow_mut().push(bar);
            }
        }
    }
}

fn setup_config_file_monitor() {
    let config_path = config::Config::config_path();
    let config_filename = config_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();

    let watch_dir = match config_path.parent() {
        Some(p) => p.to_path_buf(),
        None => return,
    };

    let (tx, rx) = async_channel::unbounded::<()>();

    let watcher = create_config_watcher(config_filename, tx);
    if watcher.is_err() {
        return;
    }

    start_watching_config_directory(watcher.unwrap(), watch_dir);
    spawn_config_reload_task(rx);
}

fn create_config_watcher(
    config_filename: String,
    tx: async_channel::Sender<()>,
) -> notify::Result<notify::RecommendedWatcher> {
    notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if let Ok(event) = res {
            let is_our_file = event.paths.iter().any(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n == config_filename.as_str())
                    .unwrap_or(false)
            });

            if is_our_file {
                let _ = tx.send_blocking(());
            }
        }
    })
}

fn start_watching_config_directory(
    mut watcher: notify::RecommendedWatcher,
    watch_dir: std::path::PathBuf,
) {
    use notify::{RecursiveMode, Watcher};

    if let Err(e) = watcher.watch(&watch_dir, RecursiveMode::NonRecursive) {
        eprintln!("Failed to watch config directory: {}", e);
        return;
    }

    std::mem::forget(watcher);
}

fn spawn_config_reload_task(rx: async_channel::Receiver<()>) {
    glib::spawn_future_local(async move {
        while let Ok(()) = rx.recv().await {
            let config = config::Config::instance();
            config.reload();
        }
    });
}

fn toggle_control_center_via_dbus() -> Result<(), Box<dyn std::error::Error>> {
    use zbus::blocking::Connection;

    let connection = Connection::session()?;
    let proxy = zbus::blocking::Proxy::new(
        &connection,
        DBUS_NAME,
        DBUS_PATH,
        "com.cmihail.RustyDE.ControlCenter",
    )?;

    proxy.call_method("Toggle", &())?;
    Ok(())
}

fn setup_dbus_server(bars: std::rc::Rc<std::cell::RefCell<Vec<window::bar::Bar>>>) {
    use zbus::interface;

    struct ControlCenterInterface {
        toggle_tx: async_channel::Sender<()>,
    }

    #[interface(name = "com.cmihail.RustyDE.ControlCenter")]
    impl ControlCenterInterface {
        fn toggle(&self) {
            let _ = self.toggle_tx.send_blocking(());
        }
    }

    let (toggle_tx, toggle_rx) = async_channel::unbounded::<()>();

    // Listen for toggle requests on GTK main thread
    glib::spawn_future_local(async move {
        while toggle_rx.recv().await.is_ok() {
            // Get focused monitor from Hyprland
            if let Some(hyprland) = service::hyprland::Hyprland::instance() {
                if let Some(focused_monitor) = hyprland.focused_monitor() {
                    // Toggle control center only on the focused monitor
                    for bar in bars.borrow().iter() {
                        if bar.connector() == focused_monitor.name {
                            bar.toggle_control_center();
                            break;
                        }
                    }
                }
            } else {
                // Fallback: toggle on first bar if Hyprland is not available
                if let Some(bar) = bars.borrow().first() {
                    bar.toggle_control_center();
                }
            }
        }
    });

    // Setup D-Bus server on separate thread
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            let interface = ControlCenterInterface { toggle_tx };

            let connection = match zbus::Connection::session().await {
                Ok(conn) => conn,
                Err(e) => {
                    eprintln!("Failed to connect to D-Bus session: {}", e);
                    return;
                }
            };

            if let Err(e) = connection.object_server().at(DBUS_PATH, interface).await {
                eprintln!("Failed to register D-Bus object: {}", e);
                return;
            }

            if let Err(e) = connection.request_name(DBUS_NAME).await {
                eprintln!("Failed to request D-Bus name: {}", e);
                return;
            }

            // Keep the connection alive
            std::future::pending::<()>().await;
        });
    });
}
