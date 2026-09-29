use gtk4::pango::EllipsizeMode;
use gtk4::prelude::*;
use gtk4::{
    Align, Box, Entry, EntryIconPosition, Label, Orientation, PolicyType, Revealer, ScrolledWindow,
    Separator, Spinner, Switch,
};
use std::boxed::Box as StdBox;
use std::cell::Cell;
use std::rc::Rc;

pub struct SwitchList {
    widget: Box,
}

type PasswordCallback<T> = Rc<dyn Fn(&T, String, StdBox<dyn Fn(Result<(), String>)>)>;
type CompletionCallback<T> = Rc<dyn Fn(&T, StdBox<dyn Fn(Result<(), String>)>)>;
type PairingApprovalCallback<T> = Rc<dyn Fn(&T, bool)>;

pub struct SwitchEntry<T: Clone + 'static> {
    pub entry: T,
    pub text: String,
    pub icon_name: Option<String>,
    pub battery_icon: Option<String>,
    pub battery_percentage: Option<u8>,
    pub peripheral_battery_icon: Option<String>,
    pub peripheral_battery_percentage: Option<u8>,
    pub tooltip_text: Option<String>,
    pub active: bool,
    pub switch_enabled: bool,
    pub requires_password: bool,
    pub min_password_length: usize,
    pub pairing_code: Option<u32>,
    pub initial_error: Option<String>,
    pub on_activate: Rc<dyn Fn(&T)>,
    pub on_deactivate: Rc<dyn Fn(&T)>,
    pub on_password_required: Option<PasswordCallback<T>>,
    pub on_activate_with_completion: Option<CompletionCallback<T>>,
    pub on_deactivate_with_completion: Option<CompletionCallback<T>>,
    pub on_pairing_approval: Option<PairingApprovalCallback<T>>,
}

impl<T: Clone + 'static> SwitchEntry<T> {
    #[allow(clippy::too_many_arguments)]
    pub fn new<F, G>(
        entry: T,
        text: String,
        icon_name: Option<String>,
        battery_icon: Option<String>,
        battery_percentage: Option<u8>,
        peripheral_battery_icon: Option<String>,
        peripheral_battery_percentage: Option<u8>,
        tooltip_text: Option<String>,
        active: bool,
        switch_enabled: bool,
        requires_password: bool,
        on_activate: F,
        on_deactivate: G,
    ) -> Self
    where
        F: Fn(&T) + 'static,
        G: Fn(&T) + 'static,
    {
        Self {
            entry,
            text,
            icon_name,
            battery_icon,
            battery_percentage,
            peripheral_battery_icon,
            peripheral_battery_percentage,
            tooltip_text,
            active,
            switch_enabled,
            requires_password,
            min_password_length: 0,
            pairing_code: None,
            initial_error: None,
            on_activate: Rc::new(on_activate),
            on_deactivate: Rc::new(on_deactivate),
            on_password_required: None,
            on_activate_with_completion: None,
            on_deactivate_with_completion: None,
            on_pairing_approval: None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn with_completion_callback<F, G>(
        entry: T,
        text: String,
        icon_name: Option<String>,
        battery_icon: Option<String>,
        battery_percentage: Option<u8>,
        peripheral_battery_icon: Option<String>,
        peripheral_battery_percentage: Option<u8>,
        tooltip_text: Option<String>,
        active: bool,
        switch_enabled: bool,
        initial_error: Option<String>,
        on_activate: F,
        on_deactivate: G,
    ) -> Self
    where
        F: Fn(&T, StdBox<dyn Fn(Result<(), String>)>) + 'static,
        G: Fn(&T, StdBox<dyn Fn(Result<(), String>)>) + 'static,
    {
        Self {
            entry,
            text,
            icon_name,
            battery_icon,
            battery_percentage,
            peripheral_battery_icon,
            peripheral_battery_percentage,
            tooltip_text,
            active,
            switch_enabled,
            requires_password: false,
            min_password_length: 0,
            pairing_code: None,
            initial_error,
            on_activate: Rc::new(|_| {}),   // Dummy, won't be used
            on_deactivate: Rc::new(|_| {}), // Dummy, won't be used
            on_password_required: None,
            on_activate_with_completion: Some(Rc::new(on_activate)),
            on_deactivate_with_completion: Some(Rc::new(on_deactivate)),
            on_pairing_approval: None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn with_password_support<F, G, H>(
        entry: T,
        text: String,
        icon_name: Option<String>,
        battery_icon: Option<String>,
        battery_percentage: Option<u8>,
        peripheral_battery_icon: Option<String>,
        peripheral_battery_percentage: Option<u8>,
        tooltip_text: Option<String>,
        active: bool,
        switch_enabled: bool,
        requires_password: bool,
        min_password_length: usize,
        on_activate: F,
        on_deactivate: G,
        on_password_required: H,
    ) -> Self
    where
        F: Fn(&T) + 'static,
        G: Fn(&T) + 'static,
        H: Fn(&T, String, StdBox<dyn Fn(Result<(), String>)>) + 'static,
    {
        Self {
            entry,
            text,
            icon_name,
            battery_icon,
            battery_percentage,
            peripheral_battery_icon,
            peripheral_battery_percentage,
            tooltip_text,
            active,
            switch_enabled,
            requires_password,
            min_password_length,
            pairing_code: None,
            initial_error: None,
            on_activate: Rc::new(on_activate),
            on_deactivate: Rc::new(on_deactivate),
            on_password_required: Some(Rc::new(on_password_required)),
            on_activate_with_completion: None,
            on_deactivate_with_completion: None,
            on_pairing_approval: None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn with_pairing_confirmation<F, G, H>(
        entry: T,
        text: String,
        icon_name: Option<String>,
        battery_icon: Option<String>,
        battery_percentage: Option<u8>,
        peripheral_battery_icon: Option<String>,
        peripheral_battery_percentage: Option<u8>,
        tooltip_text: Option<String>,
        active: bool,
        switch_enabled: bool,
        pairing_code: u32,
        on_activate: F,
        on_deactivate: G,
        on_pairing_approval: H,
    ) -> Self
    where
        F: Fn(&T, StdBox<dyn Fn(Result<(), String>)>) + 'static,
        G: Fn(&T, StdBox<dyn Fn(Result<(), String>)>) + 'static,
        H: Fn(&T, bool) + 'static,
    {
        Self {
            entry,
            text,
            icon_name,
            battery_icon,
            battery_percentage,
            peripheral_battery_icon,
            peripheral_battery_percentage,
            tooltip_text,
            active,
            switch_enabled,
            requires_password: false,
            min_password_length: 0,
            pairing_code: Some(pairing_code),
            initial_error: None,
            on_activate: Rc::new(|_| {}),   // Dummy, won't be used
            on_deactivate: Rc::new(|_| {}), // Dummy, won't be used
            on_password_required: None,
            on_activate_with_completion: Some(Rc::new(on_activate)),
            on_deactivate_with_completion: Some(Rc::new(on_deactivate)),
            on_pairing_approval: Some(Rc::new(on_pairing_approval)),
        }
    }
}

impl SwitchList {
    pub fn new<T: Clone + 'static>(
        header_text: Option<String>,
        on_refresh: Option<Rc<dyn Fn()>>,
        on_settings_open: Option<Rc<dyn Fn()>>,
        entries: Vec<SwitchEntry<T>>,
    ) -> Self {
        let container = Box::new(Orientation::Vertical, 0);
        container.add_css_class("SwitchList");

        // Header
        if let Some(header) = header_text {
            let header_box = Box::new(Orientation::Horizontal, 0);
            header_box.add_css_class("Header");

            let header_label = Label::new(Some(&header));
            header_box.append(&header_label);

            let quick_controls_box = Box::new(Orientation::Horizontal, 0);
            quick_controls_box.add_css_class("QuickControls");
            quick_controls_box.set_hexpand(true);
            quick_controls_box.set_halign(Align::End);

            if let Some(refresh_cb) = on_refresh {
                let refresh_button = gtk4::Button::from_icon_name("view-refresh-symbolic");
                refresh_button.connect_clicked(move |_| {
                    refresh_cb();
                });
                quick_controls_box.append(&refresh_button);
            }

            if let Some(settings_cb) = on_settings_open {
                let settings_button = gtk4::Button::from_icon_name("applications-system-symbolic");
                settings_button.connect_clicked(move |_| {
                    settings_cb();
                });
                quick_controls_box.append(&settings_button);
            }

            header_box.append(&quick_controls_box);
            container.append(&header_box);

            let separator = Separator::new(Orientation::Horizontal);
            separator.set_visible(!entries.is_empty());
            container.append(&separator);
        }

        // Scrolled window for entries
        if !entries.is_empty() {
            let scrolled = ScrolledWindow::new();
            scrolled.set_hscrollbar_policy(PolicyType::Never);
            scrolled.set_vscrollbar_policy(PolicyType::Automatic);
            scrolled.set_overlay_scrolling(true);
            scrolled.set_propagate_natural_height(true);
            scrolled.set_max_content_height(700);

            let contents_box = Box::new(Orientation::Vertical, 8);
            contents_box.add_css_class("Contents");

            for switch_entry in entries {
                // Outer vertical box for each entry
                let outer_box = Box::new(Orientation::Vertical, 0);

                // Horizontal box for icon, label, spinner, and switch
                let entry_box = Box::new(Orientation::Horizontal, 3);

                // Icon
                if let Some(icon_name) = &switch_entry.icon_name {
                    let icon = gtk4::Image::from_icon_name(icon_name);
                    icon.set_valign(Align::Center);

                    let tooltip = switch_entry
                        .tooltip_text
                        .as_ref()
                        .unwrap_or(&switch_entry.text);
                    icon.set_tooltip_text(Some(tooltip));

                    entry_box.append(&icon);
                }

                // Label
                let label = Label::new(Some(&switch_entry.text));
                label.set_halign(Align::Start);
                label.set_valign(Align::Center);
                label.set_hexpand(true);
                label.set_max_width_chars(24);
                label.set_ellipsize(EllipsizeMode::Middle);
                label.set_margin_end(4);

                let tooltip = switch_entry
                    .tooltip_text
                    .as_ref()
                    .unwrap_or(&switch_entry.text);
                label.set_tooltip_text(Some(tooltip));

                entry_box.append(&label);

                // Battery icon (if present, shown on the right before spinner and switch)
                if let Some(battery_icon_name) = &switch_entry.battery_icon {
                    let battery_icon = gtk4::Image::from_icon_name(battery_icon_name);
                    battery_icon.set_valign(Align::Center);
                    battery_icon.set_halign(Align::End);

                    // Apply CSS classes based on battery percentage
                    if let Some(percentage) = switch_entry.battery_percentage {
                        if percentage >= 60 {
                            // Normal, no class
                        } else if percentage >= 40 {
                            battery_icon.add_css_class("Active");
                        } else if percentage >= 20 {
                            battery_icon.add_css_class("Warning");
                        } else {
                            battery_icon.add_css_class("Critical");
                        }
                    }

                    let tooltip = switch_entry
                        .tooltip_text
                        .as_ref()
                        .unwrap_or(&switch_entry.text);
                    battery_icon.set_tooltip_text(Some(tooltip));

                    entry_box.append(&battery_icon);
                }

                // Peripheral battery icon (for split keyboards)
                if let Some(peripheral_battery_icon_name) = &switch_entry.peripheral_battery_icon {
                    let peripheral_battery_icon =
                        gtk4::Image::from_icon_name(peripheral_battery_icon_name);
                    peripheral_battery_icon.set_valign(Align::Center);
                    peripheral_battery_icon.set_halign(Align::End);

                    // Apply CSS classes based on peripheral battery percentage
                    if let Some(percentage) = switch_entry.peripheral_battery_percentage {
                        if percentage >= 60 {
                            // Normal, no class
                        } else if percentage >= 40 {
                            peripheral_battery_icon.add_css_class("Active");
                        } else if percentage >= 20 {
                            peripheral_battery_icon.add_css_class("Warning");
                        } else {
                            peripheral_battery_icon.add_css_class("Critical");
                        }
                    }

                    let tooltip = switch_entry
                        .tooltip_text
                        .as_ref()
                        .unwrap_or(&switch_entry.text);
                    peripheral_battery_icon.set_tooltip_text(Some(tooltip));

                    entry_box.append(&peripheral_battery_icon);
                }

                // Spinner
                let spinner = Spinner::new();
                spinner.set_visible(false);
                entry_box.append(&spinner);

                // Password entry revealer
                let password_revealer = Revealer::new();
                password_revealer.set_reveal_child(false);
                password_revealer.set_transition_type(gtk4::RevealerTransitionType::SlideDown);
                password_revealer.set_transition_duration(200);

                let password_entry = Entry::new();
                password_entry.set_hexpand(true);
                password_entry.set_placeholder_text(Some("Password"));
                password_entry.set_visibility(false);
                password_entry.set_primary_icon_name(Some("user-not-tracked-symbolic"));
                password_entry.set_secondary_icon_name(Some("pan-end-symbolic"));
                password_entry.add_css_class("PasswordEntry");

                password_revealer.set_child(Some(&password_entry));

                // Error label revealer
                let error_revealer = Revealer::new();
                error_revealer.set_reveal_child(false);
                error_revealer.set_transition_type(gtk4::RevealerTransitionType::SlideDown);
                error_revealer.set_transition_duration(200);

                let error_label = Label::new(None);
                error_label.set_xalign(0.0);
                error_label.set_hexpand(true);
                error_label.set_wrap(true);
                error_label.set_lines(3);
                error_label.set_max_width_chars(24);
                error_label.set_ellipsize(EllipsizeMode::End);
                error_label.add_css_class("ErrorLabel");

                error_revealer.set_child(Some(&error_label));

                // Display initial error if present
                if let Some(error) = &switch_entry.initial_error {
                    error_label.set_text(error);
                    error_revealer.set_reveal_child(true);
                }

                // Pairing confirmation revealer
                let pairing_revealer = Revealer::new();
                pairing_revealer.set_reveal_child(switch_entry.pairing_code.is_some());
                pairing_revealer.set_transition_type(gtk4::RevealerTransitionType::SlideDown);
                pairing_revealer.set_transition_duration(200);

                let pairing_box = Box::new(Orientation::Horizontal, 6);
                pairing_box.set_hexpand(true);
                pairing_box.add_css_class("PairingConfirmationBox");

                let code_text = if let Some(code) = switch_entry.pairing_code {
                    format!("Pair code: {:06}", code)
                } else {
                    "Pair code: ------".to_string()
                };
                let pairing_label = Label::new(Some(&code_text));
                pairing_label.set_halign(Align::Start);
                pairing_box.append(&pairing_label);

                let spacer = Box::new(Orientation::Horizontal, 0);
                spacer.set_hexpand(true);
                pairing_box.append(&spacer);

                let cancel_button = gtk4::Button::new();
                cancel_button.set_icon_name("window-close-symbolic");
                cancel_button.add_css_class("flat");
                cancel_button.add_css_class("circular");
                cancel_button.set_tooltip_text(Some("Deny"));

                let approve_button = gtk4::Button::new();
                approve_button.set_icon_name("object-select-symbolic");
                approve_button.add_css_class("flat");
                approve_button.add_css_class("circular");
                approve_button.add_css_class("suggested-action");
                approve_button.set_tooltip_text(Some("Allow"));

                pairing_box.append(&cancel_button);
                pairing_box.append(&approve_button);

                pairing_revealer.set_child(Some(&pairing_box));

                // Wire up pairing approval callbacks
                if let Some(on_approval) = &switch_entry.on_pairing_approval {
                    let entry_clone_deny = switch_entry.entry.clone();
                    let on_approval_deny = on_approval.clone();
                    cancel_button.connect_clicked(move |_| {
                        on_approval_deny(&entry_clone_deny, false);
                    });

                    let entry_clone_approve = switch_entry.entry.clone();
                    let on_approval_approve = on_approval.clone();
                    approve_button.connect_clicked(move |_| {
                        on_approval_approve(&entry_clone_approve, true);
                    });
                }

                // Switch
                let switch = Switch::new();
                switch.set_active(switch_entry.active);
                switch.set_visible(switch_entry.switch_enabled);

                let entry_clone = switch_entry.entry.clone();
                let entry_clone_for_switch = entry_clone.clone();
                let entry_clone_for_password = entry_clone.clone();
                let on_activate = switch_entry.on_activate.clone();
                let on_deactivate = switch_entry.on_deactivate.clone();
                let on_password_required = switch_entry.on_password_required.clone();
                let on_password_required_for_switch = on_password_required.clone();
                let on_activate_with_completion = switch_entry.on_activate_with_completion.clone();
                let on_activate_completion_for_switch = on_activate_with_completion.clone();
                let on_deactivate_with_completion =
                    switch_entry.on_deactivate_with_completion.clone();
                let on_deactivate_completion_for_switch = on_deactivate_with_completion.clone();
                let requires_password = switch_entry.requires_password;
                let min_password_length = switch_entry.min_password_length;
                let updating_from_user = Rc::new(Cell::new(false));
                let waiting_for_connection = Rc::new(Cell::new(false));
                let password_revealer_clone = password_revealer.clone();
                let error_revealer_clone = error_revealer.clone();
                let error_label_clone_switch = error_label.clone();
                let spinner_clone_for_switch = spinner.clone();

                let updating_flag = updating_from_user.clone();
                let waiting_flag_for_switch = waiting_for_connection.clone();
                switch.connect_active_notify(move |sw| {
                    if updating_flag.get() {
                        return;
                    }

                    updating_flag.set(true);
                    let new_state = sw.is_active();

                    // Hide error when toggling
                    error_revealer_clone.set_reveal_child(false);

                    if new_state {
                        // Check if password is required and revealer is showing
                        if password_revealer_clone.reveals_child() {
                            // Password entry is showing, close it
                            password_revealer_clone.set_reveal_child(false);
                        } else if requires_password && on_password_required_for_switch.is_some() {
                            // Show password entry, keep switch ON
                            password_revealer_clone.set_reveal_child(true);
                        } else if let Some(on_activate_completion) =
                            &on_activate_completion_for_switch
                        {
                            // Use completion callback if available
                            spinner_clone_for_switch.set_visible(true);
                            spinner_clone_for_switch.start();
                            waiting_flag_for_switch.set(true);

                            let spinner_clone_completion = spinner_clone_for_switch.clone();
                            let error_label_clone_completion = error_label_clone_switch.clone();
                            let error_revealer_clone_completion = error_revealer_clone.clone();
                            let switch_clone = sw.clone();
                            let updating_flag_completion = updating_flag.clone();
                            let waiting_flag_completion = waiting_flag_for_switch.clone();

                            on_activate_completion(
                                &entry_clone_for_switch,
                                StdBox::new(move |result: Result<(), String>| {
                                    // Check if still waiting (not cancelled by switch toggle)
                                    if !waiting_flag_completion.get() {
                                        return;
                                    }

                                    waiting_flag_completion.set(false);
                                    spinner_clone_completion.stop();
                                    spinner_clone_completion.set_visible(false);

                                    if let Err(error_msg) = result {
                                        error_label_clone_completion.set_text(&error_msg);
                                        error_revealer_clone_completion.set_reveal_child(true);

                                        // Turn off switch when connection fails
                                        updating_flag_completion.set(true);
                                        switch_clone.set_active(false);
                                        updating_flag_completion.set(false);
                                    }
                                }),
                            );
                        } else {
                            // No password required, activate directly
                            on_activate(&entry_clone_for_switch);
                        }
                    } else {
                        // If waiting for connection, stop spinner and cancel
                        if waiting_flag_for_switch.get() {
                            spinner_clone_for_switch.stop();
                            spinner_clone_for_switch.set_visible(false);
                            waiting_flag_for_switch.set(false);
                        }

                        // If password entry is showing, close it
                        if password_revealer_clone.reveals_child() {
                            password_revealer_clone.set_reveal_child(false);
                        } else if let Some(on_deactivate_completion) =
                            &on_deactivate_completion_for_switch
                        {
                            // Use completion callback if available
                            spinner_clone_for_switch.set_visible(true);
                            spinner_clone_for_switch.start();
                            waiting_flag_for_switch.set(true);

                            let spinner_clone_completion = spinner_clone_for_switch.clone();
                            let error_label_clone_completion = error_label_clone_switch.clone();
                            let error_revealer_clone_completion = error_revealer_clone.clone();
                            let switch_clone = sw.clone();
                            let updating_flag_completion = updating_flag.clone();
                            let waiting_flag_completion = waiting_flag_for_switch.clone();

                            on_deactivate_completion(
                                &entry_clone_for_switch,
                                StdBox::new(move |result: Result<(), String>| {
                                    // Check if still waiting (not cancelled by switch toggle)
                                    if !waiting_flag_completion.get() {
                                        return;
                                    }

                                    waiting_flag_completion.set(false);
                                    spinner_clone_completion.stop();
                                    spinner_clone_completion.set_visible(false);

                                    if let Err(error_msg) = result {
                                        error_label_clone_completion.set_text(&error_msg);
                                        error_revealer_clone_completion.set_reveal_child(true);

                                        // Turn on switch when disconnection fails
                                        updating_flag_completion.set(true);
                                        switch_clone.set_active(true);
                                        updating_flag_completion.set(false);
                                    }
                                }),
                            );
                        } else {
                            on_deactivate(&entry_clone_for_switch);
                        }
                    }
                    updating_flag.set(false);
                });

                // Password entry icon click
                if let Some(on_password_req) = on_password_required {
                    let entry_clone_pw = entry_clone_for_password.clone();
                    let entry_clone_pw_for_icon = entry_clone_pw.clone();
                    let entry_clone_pw_for_activate = entry_clone_pw.clone();
                    let on_password_req_for_icon = on_password_req.clone();
                    let on_password_req_for_activate = on_password_req.clone();
                    let password_revealer_clone_pw = password_revealer.clone();
                    let error_label_clone = error_label.clone();
                    let error_revealer_clone_pw = error_revealer.clone();
                    let spinner_clone_pw = spinner.clone();
                    let password_visible = Rc::new(Cell::new(false));
                    let min_password_length_for_icon = min_password_length;
                    let switch_for_icon = switch.clone();
                    let switch_for_activate = switch.clone();
                    let updating_flag_for_icon = updating_from_user.clone();
                    let updating_flag_for_activate = updating_from_user.clone();
                    let waiting_flag_for_icon = waiting_for_connection.clone();
                    let waiting_flag_for_activate = waiting_for_connection.clone();

                    let password_visible_clone = password_visible.clone();
                    password_entry.connect_icon_press(move |entry, pos| {
                        if pos == EntryIconPosition::Primary {
                            // Toggle password visibility
                            let current = password_visible_clone.get();
                            entry.set_visibility(!current);
                            password_visible_clone.set(!current);
                        } else if pos == EntryIconPosition::Secondary {
                            // Submit password
                            let password = entry.text().to_string();
                            if password.len() < min_password_length_for_icon {
                                error_label_clone.set_text(&format!(
                                    "Password must be at least {} characters long.",
                                    min_password_length_for_icon
                                ));
                                error_revealer_clone_pw.set_reveal_child(true);
                                return;
                            }

                            entry.set_text(""); // Clear password entry
                            password_revealer_clone_pw.set_reveal_child(false);
                            error_revealer_clone_pw.set_reveal_child(false);
                            spinner_clone_pw.set_visible(true);
                            spinner_clone_pw.start();
                            waiting_flag_for_icon.set(true);

                            let spinner_clone_completion = spinner_clone_pw.clone();
                            let error_label_clone_completion = error_label_clone.clone();
                            let error_revealer_clone_completion = error_revealer_clone_pw.clone();
                            let switch_clone_completion = switch_for_icon.clone();
                            let updating_flag_completion = updating_flag_for_icon.clone();
                            let waiting_flag_completion = waiting_flag_for_icon.clone();

                            on_password_req_for_icon(
                                &entry_clone_pw_for_icon,
                                password,
                                StdBox::new(move |result: Result<(), String>| {
                                    // Check if we're still waiting (not cancelled by switch toggle)
                                    if !waiting_flag_completion.get() {
                                        return;
                                    }

                                    waiting_flag_completion.set(false);
                                    spinner_clone_completion.stop();
                                    spinner_clone_completion.set_visible(false);

                                    if let Err(error_msg) = result {
                                        error_label_clone_completion.set_text(&error_msg);
                                        error_revealer_clone_completion.set_reveal_child(true);

                                        // Turn off the switch when connection fails
                                        updating_flag_completion.set(true);
                                        switch_clone_completion.set_active(false);
                                        updating_flag_completion.set(false);
                                    }
                                }),
                            );
                        }
                    });

                    // Also submit on Enter key
                    let _password_entry_clone2 = password_entry.clone();
                    let password_revealer_clone_pw2 = password_revealer.clone();
                    let error_label_clone2 = error_label.clone();
                    let error_revealer_clone_pw2 = error_revealer.clone();
                    let spinner_clone_pw2 = spinner.clone();
                    let min_password_length_for_activate = min_password_length;

                    password_entry.connect_activate(move |entry| {
                        let password = entry.text().to_string();
                        if password.len() < min_password_length_for_activate {
                            error_label_clone2.set_text(&format!(
                                "Password must be at least {} characters long.",
                                min_password_length_for_activate
                            ));
                            error_revealer_clone_pw2.set_reveal_child(true);
                            return;
                        }

                        entry.set_text(""); // Clear password entry
                        password_revealer_clone_pw2.set_reveal_child(false);
                        error_revealer_clone_pw2.set_reveal_child(false);
                        spinner_clone_pw2.set_visible(true);
                        spinner_clone_pw2.start();
                        waiting_flag_for_activate.set(true);

                        let spinner_clone_completion2 = spinner_clone_pw2.clone();
                        let error_label_clone_completion2 = error_label_clone2.clone();
                        let error_revealer_clone_completion2 = error_revealer_clone_pw2.clone();
                        let switch_clone_completion2 = switch_for_activate.clone();
                        let updating_flag_completion2 = updating_flag_for_activate.clone();
                        let waiting_flag_completion2 = waiting_flag_for_activate.clone();

                        on_password_req_for_activate(
                            &entry_clone_pw_for_activate,
                            password,
                            StdBox::new(move |result: Result<(), String>| {
                                // Check if we're still waiting (not cancelled by switch toggle)
                                if !waiting_flag_completion2.get() {
                                    return;
                                }

                                waiting_flag_completion2.set(false);
                                spinner_clone_completion2.stop();
                                spinner_clone_completion2.set_visible(false);

                                if let Err(error_msg) = result {
                                    error_label_clone_completion2.set_text(&error_msg);
                                    error_revealer_clone_completion2.set_reveal_child(true);

                                    // Turn off the switch when connection fails
                                    updating_flag_completion2.set(true);
                                    switch_clone_completion2.set_active(false);
                                    updating_flag_completion2.set(false);
                                }
                            }),
                        );
                    });
                }

                entry_box.append(&switch);
                outer_box.append(&entry_box);
                outer_box.append(&password_revealer);
                outer_box.append(&pairing_revealer);
                outer_box.append(&error_revealer);

                contents_box.append(&outer_box);
            }

            scrolled.set_child(Some(&contents_box));
            container.append(&scrolled);
        }

        Self { widget: container }
    }

    pub fn widget(&self) -> &Box {
        &self.widget
    }
}
