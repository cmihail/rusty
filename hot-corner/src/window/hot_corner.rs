use glib::clone;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, EventControllerMotion, Label};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::cell::Cell;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CornerType {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

pub struct HotCorner {
    _window: ApplicationWindow,
    _connector: String,
    _corner_type: CornerType,
}

impl HotCorner {
    pub fn connector(&self) -> &str {
        &self._connector
    }

    pub fn corner_type(&self) -> CornerType {
        self._corner_type
    }

    pub fn new(app: &Application, monitor: &gtk4::gdk::Monitor, corner_type: CornerType) -> Self {
        let connector = monitor
            .connector()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let window = ApplicationWindow::builder()
            .application(app)
            .decorated(false)
            .build();

        window.init_layer_shell();
        // Use Overlay layer to ensure hot corners are above all other windows
        // Layer order from bottom to top: Background, Bottom, Top, Overlay
        window.set_layer(Layer::Overlay);
        window.set_monitor(Some(monitor));

        // Set vertical and horizontal anchors based on corner type
        let namespace = match corner_type {
            CornerType::TopLeft => {
                window.set_anchor(Edge::Top, true);
                window.set_anchor(Edge::Left, true);
                "HotCornerTopLeft"
            }
            CornerType::TopRight => {
                window.set_anchor(Edge::Top, true);
                window.set_anchor(Edge::Right, true);
                "HotCornerTopRight"
            }
            CornerType::BottomLeft => {
                window.set_anchor(Edge::Bottom, true);
                window.set_anchor(Edge::Left, true);
                "HotCornerBottomLeft"
            }
            CornerType::BottomRight => {
                window.set_anchor(Edge::Bottom, true);
                window.set_anchor(Edge::Right, true);
                "HotCornerBottomRight"
            }
        };

        window.set_namespace(Some(namespace));
        window.add_css_class("HotCorner");
        window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::None);

        // Create invisible label with 4px width
        let label = Label::builder().label(".").width_request(4).build();
        label.add_css_class("Invisible");

        let open_expo = Rc::new(Cell::new(false));

        let motion_controller = EventControllerMotion::new();

        // On mouse enter
        motion_controller.connect_enter(clone!(
            #[strong]
            open_expo,
            #[strong]
            connector,
            move |_, _, _| {
                open_expo.set(true);

                let open_expo_clone = open_expo.clone();
                let connector_clone = connector.clone();
                let corner_type_clone = corner_type; // Copy the corner_type
                let config = crate::config::Config::instance();
                let delay_ms = config.trigger_delay_ms();
                glib::timeout_add_local_once(
                    std::time::Duration::from_millis(delay_ms as u64),
                    move || {
                        if !open_expo_clone.get() {
                            return;
                        }

                        trigger_action(&connector_clone, corner_type_clone);
                    },
                );
            }
        ));

        // On mouse leave
        motion_controller.connect_leave(clone!(
            #[strong]
            open_expo,
            move |_| {
                open_expo.set(false);
            }
        ));

        window.add_controller(motion_controller);
        window.set_child(Some(&label));

        // Realize the window to ensure widget hierarchy is fully initialized
        // before making it visible, preventing GTK assertion failures during
        // rapid window recreation (e.g., monitor hotplug)
        gtk4::prelude::WidgetExt::realize(&window);
        window.set_visible(true);

        Self {
            _window: window,
            _connector: connector,
            _corner_type: corner_type,
        }
    }
}

impl Drop for HotCorner {
    fn drop(&mut self) {
        // Hide and close the window before dropping
        self._window.set_visible(false);
        self._window.close();
    }
}

fn trigger_action(monitor_connector: &str, corner_type: CornerType) {
    let focused_monitor = get_focused_monitor();

    // Only trigger action if the mouse is on the focused monitor
    if let Some(focused) = focused_monitor {
        if focused == monitor_connector {
            let config = crate::config::Config::instance();
            let command = match corner_type {
                CornerType::TopLeft => config.top_left_corner_command(),
                CornerType::TopRight => config.top_right_corner_command(),
                CornerType::BottomLeft => config.bottom_left_corner_command(),
                CornerType::BottomRight => config.bottom_right_corner_command(),
            };

            if let Some(cmd) = command {
                execute_command(&cmd);
            }
        }
    }
}

fn get_focused_monitor() -> Option<String> {
    let output = std::process::Command::new("hyprctl")
        .args(["monitors", "-j"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let json_str = String::from_utf8(output.stdout).ok()?;
    let monitors: serde_json::Value = serde_json::from_str(&json_str).ok()?;

    monitors
        .as_array()?
        .iter()
        .find(|m| m["focused"].as_bool() == Some(true))
        .and_then(|m| m["name"].as_str())
        .map(|s| s.to_string())
}

fn execute_command(command: &str) {
    // Execute command via shell to properly handle quotes and complex commands
    // Redirect stdout to null to suppress output
    let _ = std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdout(std::process::Stdio::null())
        .spawn();
}
