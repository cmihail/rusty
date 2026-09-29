use gtk4::pango::EllipsizeMode;
use gtk4::prelude::*;
use gtk4::{Align, Box, Image, Label, Orientation, ToggleButton};
use std::cell::Cell;
use std::rc::Rc;

pub struct Toggle {
    widget: Box,
    toggle_button: ToggleButton,
    expander_button: Option<gtk4::Button>,
    main_label: Rc<Label>,
    small_label: Rc<Label>,
    icon: Rc<Image>,
    programmatic_change: Rc<Cell<bool>>,
    expander_state: Option<Rc<Cell<bool>>>,
}

impl Toggle {
    pub fn new<F>(
        icon_name: &str,
        main_text: &str,
        small_text: Option<&str>,
        active: bool,
        on_toggled: Option<F>,
    ) -> Self
    where
        F: Fn(bool) + 'static,
    {
        let container = Box::new(Orientation::Horizontal, 0);
        container.add_css_class("Toggle");

        // Create toggle button
        let toggle_button = ToggleButton::new();
        toggle_button.add_css_class("Toggle");
        toggle_button.set_active(active);

        let button_box = Box::new(Orientation::Horizontal, 0);

        // Icon
        let icon = Rc::new(Image::from_icon_name(icon_name));
        icon.set_pixel_size(13);
        button_box.append(icon.as_ref());

        // Text container
        let text_box = Box::new(Orientation::Vertical, 0);

        // Main label
        let main_label = Rc::new(Label::new(Some(main_text)));
        main_label.set_xalign(0.0);
        main_label.set_vexpand(true);
        main_label.set_halign(Align::Start);
        main_label.set_ellipsize(EllipsizeMode::End);
        main_label.set_max_width_chars(11);
        main_label.add_css_class("LargeText");
        text_box.append(main_label.as_ref());

        // Small label
        let small_label = Rc::new(Label::new(small_text));
        small_label.set_xalign(0.0);
        small_label.set_halign(Align::Start);
        small_label.set_ellipsize(EllipsizeMode::Middle);
        small_label.set_max_width_chars(12);
        small_label.add_css_class("SmallText");
        small_label.set_visible(small_text.is_some());
        text_box.append(small_label.as_ref());

        button_box.append(&text_box);
        toggle_button.set_child(Some(&button_box));

        // Set initial tooltip
        Self::update_tooltip_static(&toggle_button, main_text, small_text);

        let programmatic_change = Rc::new(Cell::new(false));

        container.append(&toggle_button);

        // Always create expander button and state
        let expander_state = Rc::new(Cell::new(false));

        let expander = gtk4::Button::new();
        expander.add_css_class("Expander");
        if active {
            expander.add_css_class("Active");
        }
        expander.set_icon_name("pan-end-symbolic");

        container.append(&expander);

        // Setup toggle callback after expander is created so we can update its CSS
        if let Some(handler) = on_toggled {
            let programmatic_change_clone = programmatic_change.clone();
            let expander_clone = expander.clone();
            toggle_button.connect_toggled(move |btn| {
                if !programmatic_change_clone.get() {
                    let is_active = btn.is_active();
                    handler(is_active);

                    // Update expander button CSS to match toggle state
                    if is_active {
                        expander_clone.add_css_class("Active");
                    } else {
                        expander_clone.remove_css_class("Active");
                    }
                }
            });
        }

        Self {
            widget: container,
            toggle_button,
            expander_button: Some(expander),
            main_label,
            small_label,
            icon,
            programmatic_change,
            expander_state: Some(expander_state),
        }
    }

    pub fn widget(&self) -> &Box {
        &self.widget
    }

    pub fn set_active(&self, active: bool) {
        self.programmatic_change.set(true);
        self.toggle_button.set_active(active);
        self.programmatic_change.set(false);

        if let Some(expander) = &self.expander_button {
            if active {
                expander.add_css_class("Active");
            } else {
                expander.remove_css_class("Active");
            }
        }
    }

    pub fn set_icon_name(&self, icon_name: &str) {
        self.icon.set_icon_name(Some(icon_name));
    }

    pub fn set_small_text(&self, text: Option<&str>) {
        self.small_label.set_label(text.unwrap_or(""));
        self.small_label.set_visible(text.is_some());
        self.update_tooltip();
    }

    pub fn set_expanded(&self, expanded: bool) {
        // Update internal state
        if let Some(state) = &self.expander_state {
            state.set(expanded);
        }

        // Update icon
        if let Some(expander) = &self.expander_button {
            expander.set_icon_name(if expanded {
                "pan-up-symbolic"
            } else {
                "pan-end-symbolic"
            });
        }
    }

    pub fn set_expander_visible(&self, visible: bool) {
        if let Some(expander) = &self.expander_button {
            expander.set_visible(visible);
        }
        // Update CSS classes based on visibility
        if visible {
            self.toggle_button.remove_css_class("NoExpander");
        } else {
            self.toggle_button.add_css_class("NoExpander");
        }

        // Update max width chars based on expander visibility
        self.main_label
            .set_max_width_chars(if visible { 11 } else { 14 });
        self.small_label
            .set_max_width_chars(if visible { 12 } else { 15 });
    }

    pub fn connect_expander_toggled<F>(&self, handler: F)
    where
        F: Fn(bool) + 'static,
    {
        if let (Some(expander), Some(expanded_state)) =
            (&self.expander_button, &self.expander_state)
        {
            let expanded = expanded_state.clone();
            expander.connect_clicked(move |btn| {
                let is_expanded = !expanded.get();
                expanded.set(is_expanded);
                btn.set_icon_name(if is_expanded {
                    "pan-up-symbolic"
                } else {
                    "pan-end-symbolic"
                });
                handler(is_expanded);
            });
        }
    }

    fn update_tooltip(&self) {
        let main_text = self.main_label.text();
        let small_text = self.small_label.text();
        let small_text_opt = if small_text.is_empty() {
            None
        } else {
            Some(small_text.as_str())
        };

        Self::update_tooltip_static(&self.toggle_button, &main_text, small_text_opt);
    }

    fn update_tooltip_static(button: &ToggleButton, main_text: &str, small_text: Option<&str>) {
        let tooltip = if let Some(small) = small_text {
            format!("{}\n{}", main_text, small)
        } else {
            main_text.to_string()
        };

        button.set_tooltip_text(Some(&tooltip));
    }

    pub fn grab_focus_expander_if_visible(&self) {
        if let Some(expander) = &self.expander_button {
            if expander.is_visible() {
                expander.grab_focus();
                return;
            }
        }
        self.toggle_button.grab_focus();
    }
}
