use glib::clone;
use gtk4::gdk::Key;
use gtk4::graphene::Point;
use gtk4::prelude::*;
use gtk4::{Align, Box, Button, EventControllerKey, GestureClick, Label, Orientation, Window};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::process::Command;

#[allow(dead_code)]
pub struct ConfirmationDialog {
    window: Window,
}

#[allow(dead_code)]
impl ConfirmationDialog {
    pub fn new(title: &str, description: &str, command: &str) -> Self {
        let window = Window::new();

        window.init_layer_shell();
        window.set_layer(Layer::Top);
        window.set_namespace(Some("Dialog"));
        window.add_css_class("Dialog");
        window.set_keyboard_mode(KeyboardMode::Exclusive);
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Left, true);
        window.set_anchor(Edge::Bottom, true);
        window.set_anchor(Edge::Right, true);

        // Create contents box
        let contents = Box::new(Orientation::Vertical, 10);
        contents.add_css_class("Centered");
        contents.set_halign(Align::Center);
        contents.set_valign(Align::Center);

        // Title label
        let title_label = Label::new(None);
        title_label.set_markup(&format!("<b>{}</b>", title));
        contents.append(&title_label);

        // Description label
        let description_label = Label::new(Some(description));
        contents.append(&description_label);

        // Buttons box
        let buttons_box = Box::new(Orientation::Horizontal, 8);
        buttons_box.set_valign(Align::Center);

        // Cancel button
        let cancel_button = Button::with_label("Cancel");
        buttons_box.append(&cancel_button);

        // Confirm button
        let confirm_button = Button::with_label("Confirm");
        buttons_box.append(&confirm_button);

        contents.append(&buttons_box);

        let command_clone = command.to_string();
        let command_clone2 = command.to_string();

        // Connect cancel button
        cancel_button.connect_clicked(clone!(
            #[weak]
            window,
            move |_| {
                window.close();
            }
        ));

        // Connect confirm button
        confirm_button.connect_clicked(clone!(
            #[weak]
            window,
            move |_| {
                Self::spawn_command_in_scope(&command_clone);
                window.close();
            }
        ));

        // Key event handler for Escape and Enter
        let key_controller = EventControllerKey::new();
        key_controller.connect_key_pressed(clone!(
            #[weak]
            window,
            #[upgrade_or]
            glib::Propagation::Proceed,
            move |_, key, _, _| {
                match key {
                    Key::Escape => {
                        window.close();
                        glib::Propagation::Stop
                    }
                    Key::Return => {
                        Self::spawn_command_in_scope(&command_clone2);
                        window.close();
                        glib::Propagation::Stop
                    }
                    _ => glib::Propagation::Proceed,
                }
            }
        ));
        window.add_controller(key_controller);

        // Click outside to close
        let gesture = GestureClick::new();
        gesture.connect_released(clone!(
            #[weak]
            window,
            #[weak]
            contents,
            move |_, _, x, y| {
                if is_click_outside(&window, &contents, x, y) {
                    window.close();
                }
            }
        ));
        window.add_controller(gesture);

        window.set_child(Some(&contents));

        // Ensure window is realized before making visible
        WidgetExt::realize(&window);
        window.set_visible(true);

        Self { window }
    }

    pub fn close(&self) {
        self.window.close();
    }

    fn spawn_command_in_scope(command: &str) {
        if command.is_empty() {
            return;
        }

        use std::env;
        let current_path = env::var("PATH").unwrap_or_default();
        let home = env::var("HOME").unwrap_or_default();
        let local_bin = format!("{}/.local/bin", home);
        let extended_path = if current_path.is_empty() {
            local_bin
        } else {
            format!("{}:{}", local_bin, current_path)
        };
        let lib_path = format!("{}/.local/lib64", home);

        let _ = Command::new("systemd-run")
            .arg("--user")
            .arg("--scope")
            .arg("--")
            .arg("sh")
            .arg("-c")
            .arg(command)
            .env("PATH", extended_path)
            .env("LD_LIBRARY_PATH", lib_path)
            .spawn();
    }
}

#[allow(dead_code)]
pub fn is_click_outside(window: &Window, content: &Box, x: f64, y: f64) -> bool {
    if let Some(bounds) = content.compute_bounds(window) {
        let position = Point::new(x as f32, y as f32);
        !bounds.contains_point(&position)
    } else {
        false
    }
}
