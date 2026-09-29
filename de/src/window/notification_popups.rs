use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box, Orientation};
use gtk4_layer_shell::{Edge, Layer, LayerShell};

use crate::widget::notifications::Notifications;

pub struct NotificationPopups {
    _window: ApplicationWindow,
}

impl NotificationPopups {
    pub fn new(app: &Application) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .decorated(false)
            .visible(false)
            .build();

        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Right, true);
        window.set_margin(Edge::Top, 55);
        window.set_margin(Edge::Right, 3);
        window.set_namespace(Some("NotificationPopups"));
        window.add_css_class("NotificationPopups");

        let container = Box::new(Orientation::Vertical, 0);

        let window_weak = window.downgrade();
        let notifications = Notifications::new(
            |notification| notification.id.to_string(),
            false,
            true,
            None::<fn()>,
            Some({
                let window_weak = window_weak.clone();
                move || {
                    if let Some(window) = window_weak.upgrade() {
                        // Ensure window is realized before making visible
                        if !window.is_realized() {
                            WidgetExt::realize(&window);
                        }
                        window.set_visible(true);
                    }
                }
            }),
            Some({
                move || {
                    if let Some(window) = window_weak.upgrade() {
                        window.set_visible(false);
                    }
                }
            }),
        );

        container.append(notifications.widget());
        window.set_child(Some(&container));

        Self { _window: window }
    }
}
