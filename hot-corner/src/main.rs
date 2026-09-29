use gdk4::Display;
use gtk4::prelude::*;
use gtk4::Application;

mod config;
mod window;

const APP_ID: &str = "com.cmihail.hot-corner";

fn main() {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(|app| {
        let provider = gtk4::CssProvider::new();
        provider.load_from_data(include_str!("style.css"));
        gtk4::style_context_add_provider_for_display(
            &Display::default().expect("Could not get default display"),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        create_hot_corners_for_monitors(app);
    });

    app.run();
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

fn get_leftmost_monitor(display: &Display) -> Option<gtk4::gdk::Monitor> {
    let monitors = display.monitors();
    let mut leftmost: Option<(gtk4::gdk::Monitor, i32)> = None;

    for i in 0..monitors.n_items() {
        if let Some(monitor) = monitors
            .item(i)
            .and_then(|obj| obj.downcast::<gtk4::gdk::Monitor>().ok())
        {
            let geometry = monitor.geometry();
            let x = geometry.x();

            if let Some((_, leftmost_x)) = &leftmost {
                if x < *leftmost_x {
                    leftmost = Some((monitor.clone(), x));
                }
            } else {
                leftmost = Some((monitor.clone(), x));
            }
        }
    }

    leftmost.map(|(monitor, _)| monitor)
}

fn get_rightmost_monitor(display: &Display) -> Option<gtk4::gdk::Monitor> {
    let monitors = display.monitors();
    let mut rightmost: Option<(gtk4::gdk::Monitor, i32)> = None;

    for i in 0..monitors.n_items() {
        if let Some(monitor) = monitors
            .item(i)
            .and_then(|obj| obj.downcast::<gtk4::gdk::Monitor>().ok())
        {
            let geometry = monitor.geometry();
            let x = geometry.x() + geometry.width();

            if let Some((_, rightmost_x)) = &rightmost {
                if x > *rightmost_x {
                    rightmost = Some((monitor.clone(), x));
                }
            } else {
                rightmost = Some((monitor.clone(), x));
            }
        }
    }

    rightmost.map(|(monitor, _)| monitor)
}

fn should_create_hot_corner_for_monitor(
    monitor: &gtk4::gdk::Monitor,
    display: &Display,
    corner_type: window::hot_corner::CornerType,
) -> bool {
    let config = config::Config::instance();

    match corner_type {
        window::hot_corner::CornerType::TopLeft => {
            // Check if command is configured
            if config.top_left_corner_command().is_none() {
                return false;
            }

            if config.top_left_corner_on_all_monitors() {
                return true;
            }

            // Only create on leftmost monitor
            if let Some(leftmost) = get_leftmost_monitor(display) {
                if let (Some(connector), Some(leftmost_connector)) =
                    (monitor.connector(), leftmost.connector())
                {
                    return connector == leftmost_connector;
                }
            }

            false
        }
        window::hot_corner::CornerType::TopRight => {
            // Check if command is configured
            if config.top_right_corner_command().is_none() {
                return false;
            }

            if config.top_right_corner_on_all_monitors() {
                return true;
            }

            // Only create on rightmost monitor
            if let Some(rightmost) = get_rightmost_monitor(display) {
                if let (Some(connector), Some(rightmost_connector)) =
                    (monitor.connector(), rightmost.connector())
                {
                    return connector == rightmost_connector;
                }
            }

            false
        }
        window::hot_corner::CornerType::BottomLeft => {
            // Check if command is configured
            if config.bottom_left_corner_command().is_none() {
                return false;
            }

            if config.bottom_left_corner_on_all_monitors() {
                return true;
            }

            // Only create on leftmost monitor
            if let Some(leftmost) = get_leftmost_monitor(display) {
                if let (Some(connector), Some(leftmost_connector)) =
                    (monitor.connector(), leftmost.connector())
                {
                    return connector == leftmost_connector;
                }
            }

            false
        }
        window::hot_corner::CornerType::BottomRight => {
            // Check if command is configured
            if config.bottom_right_corner_command().is_none() {
                return false;
            }

            if config.bottom_right_corner_on_all_monitors() {
                return true;
            }

            // Only create on rightmost monitor
            if let Some(rightmost) = get_rightmost_monitor(display) {
                if let (Some(connector), Some(rightmost_connector)) =
                    (monitor.connector(), rightmost.connector())
                {
                    return connector == rightmost_connector;
                }
            }

            false
        }
    }
}

fn remove_disconnected_hot_corners(
    hot_corners: &std::rc::Rc<std::cell::RefCell<Vec<window::hot_corner::HotCorner>>>,
    current_connectors: &[String],
) {
    let mut corners_mut = hot_corners.borrow_mut();

    // Use retain to keep only hot corners whose monitors still exist
    corners_mut.retain(|corner| {
        let corner_connector = corner.connector();
        current_connectors
            .iter()
            .any(|connector| connector == corner_connector)
    });
}

fn attach_monitor_property_listener(
    monitor: &gtk4::gdk::Monitor,
    _monitor_index: u32,
    app: &Application,
    hot_corners: &std::rc::Rc<std::cell::RefCell<Vec<window::hot_corner::HotCorner>>>,
) {
    // Listen specifically to connector property changes
    let app_clone = app.clone();
    let corners_clone = hot_corners.clone();
    monitor.connect_notify_local(Some("connector"), move |monitor, _| {
        let connector = monitor.connector().map(|s| s.to_string());

        // If connector is now available, create hot corners
        if let Some(connector_str) = connector {
            if !connector_str.is_empty() {
                // Get the display to check if we should create hot corners
                if let Some(display) = Display::default() {
                    // Try to create all four corners
                    for corner_type in [
                        window::hot_corner::CornerType::TopLeft,
                        window::hot_corner::CornerType::TopRight,
                        window::hot_corner::CornerType::BottomLeft,
                        window::hot_corner::CornerType::BottomRight,
                    ] {
                        if !should_create_hot_corner_for_monitor(monitor, &display, corner_type) {
                            continue;
                        }

                        // Check if we already have this corner type for this connector
                        let has_corner = corners_clone.borrow().iter().any(|corner| {
                            corner.connector() == connector_str
                                && corner.corner_type() == corner_type
                        });

                        if !has_corner {
                            let hot_corner = window::hot_corner::HotCorner::new(
                                &app_clone,
                                monitor,
                                corner_type,
                            );
                            corners_clone.borrow_mut().push(hot_corner);
                        }
                    }
                }
            }
        }
    });
}

fn create_missing_hot_corners(
    gtk_monitors: &gtk4::gio::ListModel,
    app: &Application,
    hot_corners: &std::rc::Rc<std::cell::RefCell<Vec<window::hot_corner::HotCorner>>>,
    display: &Display,
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
                attach_monitor_property_listener(&monitor, i, app, hot_corners);
                continue;
            }

            // Try to create all four corners
            for corner_type in [
                window::hot_corner::CornerType::TopLeft,
                window::hot_corner::CornerType::TopRight,
                window::hot_corner::CornerType::BottomLeft,
                window::hot_corner::CornerType::BottomRight,
            ] {
                // Check if we should create a hot corner for this monitor
                if !should_create_hot_corner_for_monitor(&monitor, display, corner_type) {
                    continue;
                }

                let has_corner = hot_corners.borrow().iter().any(|corner| {
                    corner.connector() == connector && corner.corner_type() == corner_type
                });

                if !has_corner {
                    let hot_corner = window::hot_corner::HotCorner::new(app, &monitor, corner_type);
                    hot_corners.borrow_mut().push(hot_corner);
                }
            }
        }
    }
}

fn handle_monitor_change(
    display: &Display,
    app: &Application,
    hot_corners: &std::rc::Rc<std::cell::RefCell<Vec<window::hot_corner::HotCorner>>>,
) {
    // Get current monitor connectors
    let current_connectors = get_current_monitor_connectors(display);

    // Remove hot corners for monitors that no longer exist
    remove_disconnected_hot_corners(hot_corners, &current_connectors);

    // Create hot corners for new monitors (including attaching listeners to empty connectors)
    create_missing_hot_corners(&display.monitors(), app, hot_corners, display);
}

fn create_hot_corners_for_monitors(app: &Application) {
    let display = match Display::default() {
        Some(d) => d,
        None => {
            eprintln!("Could not get default display");
            return;
        }
    };

    // Initialize config singleton
    let _config = config::Config::instance();

    let hot_corners = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let monitors = display.monitors();

    // Create initial hot corners
    for i in 0..monitors.n_items() {
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
                attach_monitor_property_listener(&monitor, i, app, &hot_corners);
                continue;
            }

            // Try to create all four corners
            for corner_type in [
                window::hot_corner::CornerType::TopLeft,
                window::hot_corner::CornerType::TopRight,
                window::hot_corner::CornerType::BottomLeft,
                window::hot_corner::CornerType::BottomRight,
            ] {
                if should_create_hot_corner_for_monitor(&monitor, &display, corner_type) {
                    let hot_corner = window::hot_corner::HotCorner::new(app, &monitor, corner_type);
                    hot_corners.borrow_mut().push(hot_corner);
                }
            }
        }
    }

    // Setup file watcher for config file changes
    setup_config_file_monitor();

    // Listen for configuration changes to recreate hot corners
    let config = config::Config::instance();

    // Helper function to recreate all corners
    let recreate_corners =
        |hot_corners: &std::rc::Rc<std::cell::RefCell<Vec<window::hot_corner::HotCorner>>>,
         display: &Display,
         app: &Application| {
            // Clear all existing hot corners
            hot_corners.borrow_mut().clear();

            // Recreate hot corners based on new configuration
            let monitors = display.monitors();
            for i in 0..monitors.n_items() {
                if let Some(monitor) = monitors
                    .item(i)
                    .and_then(|obj| obj.downcast::<gtk4::gdk::Monitor>().ok())
                {
                    let connector = monitor
                        .connector()
                        .map(|s| s.to_string())
                        .unwrap_or_default();

                    if connector.is_empty() {
                        continue;
                    }

                    // Try to create all four corners
                    for corner_type in [
                        window::hot_corner::CornerType::TopLeft,
                        window::hot_corner::CornerType::TopRight,
                        window::hot_corner::CornerType::BottomLeft,
                        window::hot_corner::CornerType::BottomRight,
                    ] {
                        if should_create_hot_corner_for_monitor(&monitor, display, corner_type) {
                            let hot_corner =
                                window::hot_corner::HotCorner::new(app, &monitor, corner_type);
                            hot_corners.borrow_mut().push(hot_corner);
                        }
                    }
                }
            }
        };

    // Listen for top-left corner changes
    config.connect_notify_local(
        Some("top-left-corner-on-all-monitors"),
        glib::clone!(
            #[strong]
            hot_corners,
            #[strong]
            display,
            #[weak]
            app,
            move |_, _| {
                recreate_corners(&hot_corners, &display, &app);
            }
        ),
    );

    // Listen for top-right corner command changes
    config.connect_notify_local(
        Some("top-right-corner-command"),
        glib::clone!(
            #[strong]
            hot_corners,
            #[strong]
            display,
            #[weak]
            app,
            move |_, _| {
                recreate_corners(&hot_corners, &display, &app);
            }
        ),
    );

    // Listen for top-right corner on-all-monitors changes
    config.connect_notify_local(
        Some("top-right-corner-on-all-monitors"),
        glib::clone!(
            #[strong]
            hot_corners,
            #[strong]
            display,
            #[weak]
            app,
            move |_, _| {
                recreate_corners(&hot_corners, &display, &app);
            }
        ),
    );

    // Listen for bottom-left corner command changes
    config.connect_notify_local(
        Some("bottom-left-corner-command"),
        glib::clone!(
            #[strong]
            hot_corners,
            #[strong]
            display,
            #[weak]
            app,
            move |_, _| {
                recreate_corners(&hot_corners, &display, &app);
            }
        ),
    );

    // Listen for bottom-left corner on-all-monitors changes
    config.connect_notify_local(
        Some("bottom-left-corner-on-all-monitors"),
        glib::clone!(
            #[strong]
            hot_corners,
            #[strong]
            display,
            #[weak]
            app,
            move |_, _| {
                recreate_corners(&hot_corners, &display, &app);
            }
        ),
    );

    // Listen for bottom-right corner command changes
    config.connect_notify_local(
        Some("bottom-right-corner-command"),
        glib::clone!(
            #[strong]
            hot_corners,
            #[strong]
            display,
            #[weak]
            app,
            move |_, _| {
                recreate_corners(&hot_corners, &display, &app);
            }
        ),
    );

    // Listen for bottom-right corner on-all-monitors changes
    config.connect_notify_local(
        Some("bottom-right-corner-on-all-monitors"),
        glib::clone!(
            #[strong]
            hot_corners,
            #[strong]
            display,
            #[weak]
            app,
            move |_, _| {
                recreate_corners(&hot_corners, &display, &app);
            }
        ),
    );

    // Listen for monitor changes
    monitors.connect_items_changed(glib::clone!(
        #[strong]
        hot_corners,
        #[strong]
        display,
        #[weak]
        app,
        move |_, _, _, _| {
            handle_monitor_change(&display, &app, &hot_corners);
        }
    ));
}

fn setup_config_file_monitor() {
    use notify::{RecursiveMode, Watcher};

    let config_path = config::Config::config_path();
    let config_filename = config_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();

    // Watch the parent directory to catch file replacements by editors
    let watch_dir = match config_path.parent() {
        Some(p) => p.to_path_buf(),
        None => return,
    };

    // Create an async channel for communication between watcher and main thread
    let (tx, rx) = async_channel::unbounded::<()>();

    // Create watcher with callback that sends to the async channel
    let watcher = match notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if let Ok(event) = res {
            // Check if this event is for our config file
            let is_our_file = event.paths.iter().any(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n == config_filename.as_str())
                    .unwrap_or(false)
            });

            if is_our_file {
                // Send notification through async channel
                let _ = tx.send_blocking(());
            }
        }
    }) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Failed to create file watcher for hot-corner config: {}", e);
            return;
        }
    };

    // Watch the directory
    let mut watcher = watcher;
    if let Err(e) = watcher.watch(&watch_dir, RecursiveMode::NonRecursive) {
        eprintln!("Failed to watch hot-corner config directory: {}", e);
        return;
    }

    // Store the watcher to keep it alive
    std::mem::forget(watcher);

    // Spawn a glib async task to receive notifications
    glib::spawn_future_local(async move {
        while let Ok(()) = rx.recv().await {
            let config = config::Config::instance();
            config.reload();
        }
    });
}
