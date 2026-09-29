use gtk4::gdk::Key;
use gtk4::prelude::*;
use gtk4::{Align, Box, Button, EventControllerKey, Image, Orientation, Scale};
use std::cell::Cell;
use std::rc::Rc;

pub struct Slider {
    widget: Box,
    scale: Scale,
    icon: Rc<Image>,
    expand_icon: Option<Rc<Image>>,
    is_expanded: Rc<Cell<bool>>,
    #[allow(dead_code)]
    updating_from_keyboard: Rc<Cell<bool>>,
}

impl Slider {
    #[allow(dead_code)]
    pub fn new<F, G>(
        icon_name: &str,
        value: f64,
        min: f64,
        max: f64,
        on_value_changed: F,
        on_icon_clicked: Option<G>,
    ) -> Self
    where
        F: Fn(f64) + 'static,
        G: Fn() + 'static,
    {
        Self::new_with_expander::<F, G, fn(bool)>(
            icon_name,
            value,
            min,
            max,
            on_value_changed,
            on_icon_clicked,
            None,
        )
    }

    pub fn new_with_expander<F, G, H>(
        icon_name: &str,
        value: f64,
        min: f64,
        max: f64,
        on_value_changed: F,
        on_icon_clicked: Option<G>,
        on_expander_clicked: Option<H>,
    ) -> Self
    where
        F: Fn(f64) + 'static,
        G: Fn() + 'static,
        H: Fn(bool) + 'static,
    {
        let container = Box::new(Orientation::Horizontal, 0);
        container.add_css_class("Slider");

        // Icon button
        let icon = Rc::new(Image::from_icon_name(icon_name));

        if let Some(click_handler) = on_icon_clicked {
            let icon_button = Button::new();
            icon_button.set_child(Some(icon.as_ref()));
            icon_button.set_valign(Align::Center);
            icon_button.add_css_class("SliderIcon");
            icon_button.connect_clicked(move |_| {
                click_handler();
            });
            container.append(&icon_button);
        } else {
            let icon_box = Box::new(Orientation::Horizontal, 0);
            icon_box.set_valign(Align::Center);
            icon_box.append(icon.as_ref());
            container.append(&icon_box);
        }

        // Scale (slider)
        let scale = Scale::with_range(Orientation::Horizontal, min, max, 0.01);
        scale.set_draw_value(false);
        scale.set_hexpand(true);
        scale.set_value(value);
        scale.set_valign(Align::Center);
        scale.set_can_focus(true);
        scale.set_focusable(true);

        let updating_from_keyboard = Rc::new(Cell::new(false));

        scale.connect_value_changed(glib::clone!(
            #[strong]
            updating_from_keyboard,
            move |scale| {
                if updating_from_keyboard.get() {
                    return;
                }
                on_value_changed(scale.value());
            }
        ));

        // Add keyboard controller to handle navigation and value adjustment
        // Arrow keys: navigate to adjacent widgets
        // Home/End keys: adjust slider value
        let key_controller = EventControllerKey::new();
        key_controller.set_propagation_phase(gtk4::PropagationPhase::Capture);
        key_controller.connect_key_pressed(glib::clone!(
            #[weak]
            scale,
            #[strong]
            updating_from_keyboard,
            #[upgrade_or]
            glib::Propagation::Proceed,
            move |_, key, _, _| {
                match key {
                    Key::Up | Key::Down | Key::Left | Key::Right => {
                        // Use arrow keys for navigation, not for adjusting slider value
                        let direction = match key {
                            Key::Up => gtk4::DirectionType::Up,
                            Key::Down => gtk4::DirectionType::Down,
                            Key::Left => gtk4::DirectionType::Left,
                            Key::Right => gtk4::DirectionType::Right,
                            _ => return glib::Propagation::Proceed,
                        };

                        // Get the toplevel window and move focus
                        if let Some(root) = scale.root() {
                            if let Some(window) = root.downcast_ref::<gtk4::Window>() {
                                window.child_focus(direction);
                            }
                        }

                        // Stop propagation to prevent GTK from adjusting slider value
                        glib::Propagation::Stop
                    }
                    Key::Home => {
                        // Home: decrease value
                        let current = scale.value();
                        let step = scale.adjustment().step_increment();
                        let lower = scale.adjustment().lower();
                        let new_value = (current - step).max(lower);

                        // Set flag to prevent service feedback loop
                        updating_from_keyboard.set(true);
                        scale.set_value(new_value);

                        // Clear flag after service has processed the change
                        glib::idle_add_local_once(glib::clone!(
                            #[strong]
                            updating_from_keyboard,
                            move || {
                                updating_from_keyboard.set(false);
                            }
                        ));

                        glib::Propagation::Stop
                    }
                    Key::End => {
                        // End: increase value
                        let current = scale.value();
                        let step = scale.adjustment().step_increment();
                        let upper = scale.adjustment().upper();
                        let new_value = (current + step).min(upper);

                        // Set flag to prevent service feedback loop
                        updating_from_keyboard.set(true);
                        scale.set_value(new_value);

                        // Clear flag after service has processed the change
                        glib::idle_add_local_once(glib::clone!(
                            #[strong]
                            updating_from_keyboard,
                            move || {
                                updating_from_keyboard.set(false);
                            }
                        ));

                        glib::Propagation::Stop
                    }
                    Key::Tab => {
                        // Handle Tab navigation
                        scale.keynav_failed(gtk4::DirectionType::TabForward);
                        glib::Propagation::Stop
                    }
                    Key::ISO_Left_Tab => {
                        // Handle Shift+Tab navigation
                        scale.keynav_failed(gtk4::DirectionType::TabBackward);
                        glib::Propagation::Stop
                    }
                    _ => glib::Propagation::Proceed,
                }
            }
        ));
        scale.add_controller(key_controller);

        container.append(&scale);

        let is_expanded = Rc::new(Cell::new(false));

        // Optional expand button
        let expand_icon = if let Some(expander_handler) = on_expander_clicked {
            let expand_img = Rc::new(Image::from_icon_name("pan-end-symbolic"));
            let expand_button = Button::new();
            expand_button.set_child(Some(expand_img.as_ref()));
            expand_button.set_valign(Align::Center);
            expand_button.add_css_class("SliderExpander");

            let is_expanded_clone = is_expanded.clone();
            let expand_img_clone = expand_img.clone();
            expand_button.connect_clicked(move |_| {
                let new_state = !is_expanded_clone.get();
                is_expanded_clone.set(new_state);
                expand_img_clone.set_icon_name(Some(if new_state {
                    "pan-up-symbolic"
                } else {
                    "pan-end-symbolic"
                }));
                expander_handler(new_state);
            });

            container.append(&expand_button);
            Some(expand_img)
        } else {
            None
        };

        Self {
            widget: container,
            scale,
            icon,
            expand_icon,
            is_expanded,
            updating_from_keyboard,
        }
    }

    pub fn widget(&self) -> &Box {
        &self.widget
    }

    pub fn set_value(&self, value: f64) {
        self.scale.set_value(value);
    }

    pub fn set_icon_name(&self, icon_name: &str) {
        self.icon.set_icon_name(Some(icon_name));
    }

    pub fn set_expanded(&self, expanded: bool) {
        self.is_expanded.set(expanded);
        if let Some(ref expand_icon) = self.expand_icon {
            expand_icon.set_icon_name(Some(if expanded {
                "pan-up-symbolic"
            } else {
                "pan-end-symbolic"
            }));
        }
    }
}
