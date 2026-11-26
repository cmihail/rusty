use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Box, Image, Label, Orientation, ProgressBar};
use gtk4_layer_shell::{KeyboardMode, Layer, LayerShell};
use std::cell::{Cell, OnceCell};
use std::rc::Rc;
use std::time::Duration;

use crate::service::desktop_entry::{self, DesktopEntry};
use crate::service::hyprland::Hyprland;
use crate::window::workspace_indicator::focused_monitor;

/// Covers single-instance apps that only focus an existing window, and failed launches.
const TIMEOUT: Duration = Duration::from_secs(10);
const ICON_SIZE: i32 = 64;
const FALLBACK_ICON: &str = "application-x-executable";
const PULSE_INTERVAL: Duration = Duration::from_millis(100);
const PULSE_STEP: f64 = 0.15;
const BAR_WIDTH: i32 = 160;

thread_local! {
    static SPINNER: OnceCell<LaunchSpinner> = const { OnceCell::new() };
}

/// Whether a Hyprland event ends the launch feedback.
pub fn hides_on(event: &str) -> bool {
    event == "openwindow"
}

/// The entry's name, or the desktop id without its `.desktop` suffix.
pub fn launch_label(desktop_id: &str, entry: Option<&DesktopEntry>) -> String {
    entry
        .and_then(|e| e.name.clone())
        .unwrap_or_else(|| desktop_id.trim_end_matches(".desktop").to_string())
}

struct LaunchSpinner {
    window: ApplicationWindow,
    icon: Image,
    label: Label,
    bar: ProgressBar,
    generation: Rc<Cell<u64>>,
}

/// Creates the window and hides it when Hyprland opens a window.
pub fn init(app: &gtk4::Application) {
    let Some(hyprland) = Hyprland::instance() else {
        return;
    };
    let spinner = LaunchSpinner::new(app);
    let (window, generation) = (spinner.window.clone(), spinner.generation.clone());
    hyprland.connect_local("event", false, move |args| {
        let event = args.get(1).and_then(|v| v.get::<String>().ok())?;
        if hides_on(&event) {
            generation.set(generation.get() + 1);
            window.set_visible(false);
        }
        None
    });
    SPINNER.with(|cell| {
        let _ = cell.set(spinner);
    });
}

/// Shows the spinner for `desktop_id` on the focused monitor until a window opens.
pub fn show(desktop_id: &str) {
    SPINNER.with(|cell| {
        if let Some(spinner) = cell.get() {
            spinner.show(desktop_id);
        }
    });
}

impl LaunchSpinner {
    fn new(app: &gtk4::Application) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .decorated(false)
            .default_width(1)
            .default_height(1)
            .build();
        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_keyboard_mode(KeyboardMode::None);
        window.set_namespace(Some("LaunchSpinner"));
        window.add_css_class("LaunchSpinner");
        // An empty input region lets clicks reach the windows underneath.
        window.connect_map(|window| {
            if let Some(surface) = window.surface() {
                surface.set_input_region(&gtk4::cairo::Region::create());
            }
        });

        let icon = Image::new();
        icon.set_pixel_size(ICON_SIZE);
        let label = Label::new(None);
        let bar = ProgressBar::new();
        bar.set_pulse_step(PULSE_STEP);
        bar.set_size_request(BAR_WIDTH, -1);
        let content = Box::new(Orientation::Vertical, 12);
        content.append(&icon);
        content.append(&label);
        content.append(&bar);
        window.set_child(Some(&content));

        Self {
            window,
            icon,
            label,
            bar,
            generation: Rc::new(Cell::new(0)),
        }
    }

    fn show(&self, desktop_id: &str) {
        let entry = desktop_entry::lookup(desktop_id);
        self.label
            .set_label(&launch_label(desktop_id, entry.as_ref()));
        set_icon(&self.icon, entry.and_then(|e| e.icon).as_deref());

        self.window.set_monitor(focused_monitor().as_ref());
        self.window.set_visible(true);

        self.generation.set(self.generation.get() + 1);
        let shown = self.generation.get();
        self.pulse_while_shown(shown);
        let (window, generation) = (self.window.clone(), self.generation.clone());
        glib::timeout_add_local_once(TIMEOUT, move || {
            if generation.get() == shown {
                window.set_visible(false);
            }
        });
    }

    fn pulse_while_shown(&self, shown: u64) {
        let (window, bar) = (self.window.clone(), self.bar.clone());
        let generation = self.generation.clone();
        glib::timeout_add_local(PULSE_INTERVAL, move || {
            if generation.get() != shown || !window.is_visible() {
                return glib::ControlFlow::Break;
            }
            bar.pulse();
            glib::ControlFlow::Continue
        });
    }
}

fn set_icon(image: &Image, icon: Option<&str>) {
    match icon {
        Some(path) if path.starts_with('/') => image.set_from_file(Some(path)),
        Some(name) => image.set_icon_name(Some(name)),
        None => image.set_icon_name(Some(FALLBACK_ICON)),
    }
}
