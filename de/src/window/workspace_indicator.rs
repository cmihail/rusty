use gtk4::gdk::{Display, Monitor};
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Label, Revealer, RevealerTransitionType};
use gtk4_layer_shell::{KeyboardMode, Layer, LayerShell};
use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use crate::service::hyprland::Hyprland;

const SHOW_DURATION: Duration = Duration::from_millis(700);
const FADE_MS: u32 = 200;

/// The workspace name from a `workspacev2` event's `ID,NAME` data, skipping special workspaces.
pub fn indicator_text(data: &str) -> Option<String> {
    let (_, name) = data.split_once(',')?;
    (!name.is_empty() && !name.starts_with("special")).then(|| name.to_string())
}

pub struct WorkspaceIndicator;

impl WorkspaceIndicator {
    pub fn new(app: &gtk4::Application) -> Self {
        let Some(hyprland) = Hyprland::instance() else {
            return Self;
        };

        let window = ApplicationWindow::builder()
            .application(app)
            .decorated(false)
            .default_width(1)
            .default_height(1)
            .build();
        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_keyboard_mode(KeyboardMode::None);
        window.set_namespace(Some("WorkspaceIndicator"));
        window.add_css_class("WorkspaceIndicator");
        // An empty input region lets clicks reach the windows underneath.
        window.connect_map(|window| {
            if let Some(surface) = window.surface() {
                surface.set_input_region(&gtk4::cairo::Region::create());
            }
        });

        let label = Label::new(None);
        let revealer = Revealer::builder()
            .transition_type(RevealerTransitionType::Crossfade)
            .transition_duration(FADE_MS)
            .child(&label)
            .build();
        window.set_child(Some(&revealer));

        let generation = Rc::new(Cell::new(0u64));
        hyprland.connect_local("event", false, move |args| {
            let event = args.get(1).and_then(|v| v.get::<String>().ok())?;
            if event != "workspacev2" {
                return None;
            }
            let data = args.get(2).and_then(|v| v.get::<String>().ok())?;
            label.set_label(&indicator_text(&data)?);
            show(&window, &revealer, &generation);
            None
        });

        Self
    }
}

fn show(window: &ApplicationWindow, revealer: &Revealer, generation: &Rc<Cell<u64>>) {
    window.set_monitor(focused_monitor().as_ref());
    window.set_visible(true);
    revealer.set_reveal_child(true);

    generation.set(generation.get() + 1);
    let shown = generation.get();
    let (window, revealer, generation) = (window.clone(), revealer.clone(), generation.clone());
    glib::timeout_add_local_once(SHOW_DURATION, move || {
        if generation.get() != shown {
            return;
        }
        revealer.set_reveal_child(false);
        glib::timeout_add_local_once(Duration::from_millis(FADE_MS.into()), move || {
            if generation.get() == shown {
                window.set_visible(false);
            }
        });
    });
}

fn focused_monitor() -> Option<Monitor> {
    let name = Hyprland::instance()?.focused_monitor()?.name;
    let monitors = Display::default()?.monitors();
    (0..monitors.n_items())
        .filter_map(|i| monitors.item(i)?.downcast::<Monitor>().ok())
        .find(|monitor| monitor.connector().is_some_and(|c| c == name))
}
