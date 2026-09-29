use glib::clone;
use gtk4::prelude::*;
use gtk4::Box;
use std::process::Command;
use std::rc::Rc;

use crate::service::wifi::{AccessPoint, Wifi};
use crate::widget::switch_list::{SwitchEntry, SwitchList};

pub struct WifiContent {
    widget: Box,
    _header_label: Rc<gtk4::Label>,
}

impl WifiContent {
    pub fn new(popover: &gtk4::Popover) -> Self {
        let container = Box::new(gtk4::Orientation::Vertical, 0);
        let wifi = Wifi::instance();

        // Create a dummy label for header updates (will be replaced by SwitchList header)
        let header_label = Rc::new(gtk4::Label::new(Some("Available Networks")));

        Self::rebuild_ui(&container, popover, &wifi, &header_label);

        // Listen to access-points property changes
        wifi.connect_notify_local(
            Some("access-points"),
            clone!(
                #[weak]
                container,
                #[weak]
                popover,
                #[strong]
                header_label,
                move |w, _| {
                    Self::rebuild_ui(&container, &popover, w, &header_label);
                }
            ),
        );

        // Listen to connected state changes
        wifi.connect_notify_local(
            Some("connected"),
            clone!(
                #[weak]
                container,
                #[weak]
                popover,
                #[strong]
                header_label,
                move |w, _| {
                    Self::rebuild_ui(&container, &popover, w, &header_label);
                }
            ),
        );

        // Listen to ssid changes
        wifi.connect_notify_local(
            Some("ssid"),
            clone!(
                #[weak]
                container,
                #[weak]
                popover,
                #[strong]
                header_label,
                move |w, _| {
                    Self::rebuild_ui(&container, &popover, w, &header_label);
                }
            ),
        );

        // Listen to scanning state changes and rebuild UI to show/hide refresh button
        wifi.connect_notify_local(
            Some("scanning"),
            clone!(
                #[weak]
                container,
                #[weak]
                popover,
                #[strong]
                header_label,
                move |w, _| {
                    Self::rebuild_ui(&container, &popover, w, &header_label);
                }
            ),
        );

        Self {
            widget: container,
            _header_label: header_label,
        }
    }

    fn rebuild_ui(
        container: &Box,
        popover: &gtk4::Popover,
        wifi: &Wifi,
        header_label: &gtk4::Label,
    ) {
        // Clear existing content
        while let Some(child) = container.first_child() {
            container.remove(&child);
        }

        let access_points = wifi.access_points();
        let current_ssid = wifi.ssid();
        let scanning = wifi.scanning();

        let header_text = if scanning {
            "Scanning...".to_string()
        } else {
            "Available Networks".to_string()
        };

        // Update the header label text
        header_label.set_text(&header_text);

        let entries: Vec<SwitchEntry<AccessPoint>> = access_points
            .into_iter()
            .map(|ap| {
                let is_connected = ap.ssid == current_ssid;
                let icon_name = Self::get_icon_for_strength(ap.strength);
                let ssid_clone = ap.ssid.clone();
                let ssid_clone_pw = ap.ssid.clone();
                let current_ssid_clone = current_ssid.clone();

                // Format frequencies for tooltip
                let frequency_text = Self::format_frequencies(&ap.frequencies);

                // Only ask for password if network requires it AND there's no saved connection
                let needs_password_input = ap.requires_password && !ap.has_saved_connection;

                log::debug!(
                    "WiFi: Building entry for '{}' - requires_password: {}, \
                     has_saved_connection: {}, needs_password_input: {}",
                    ap.ssid,
                    ap.requires_password,
                    ap.has_saved_connection,
                    needs_password_input
                );

                let tooltip = format!(
                    "Name: {}\nStrength: {}%\nFrequency: {}",
                    ap.ssid, ap.strength, frequency_text
                );

                if needs_password_input {
                    SwitchEntry::with_password_support(
                        ap.clone(),
                        ap.ssid.clone(),
                        Some(icon_name),
                        None, // no battery icon for WiFi
                        None, // no battery percentage for WiFi
                        None, // no peripheral battery icon
                        None, // no peripheral battery percentage
                        Some(tooltip),
                        is_connected,
                        true, // switch_enabled
                        true, // requires_password flag for UI
                        8,    // minimum password length for WiFi
                        move |_| {
                            // This shouldn't be called for password-protected
                            // networks without saved connection
                        },
                        move |ap_entry| {
                            if ap_entry.ssid == current_ssid_clone {
                                Wifi::instance().disconnect();
                            }
                        },
                        move |_ap_entry, password, on_complete| {
                            Wifi::instance().connect(Some(ssid_clone_pw.clone()), Some(password));
                            // TODO: WiFi service doesn't have completion callback yet,
                            // assume success for now
                            on_complete(Ok(()));
                        },
                    )
                } else {
                    // Network has saved connection or doesn't require password
                    log::debug!(
                        "WiFi: In else branch for '{}' - has_saved_connection: {}",
                        ap.ssid,
                        ap.has_saved_connection
                    );

                    // Validate security configuration if there's a saved connection
                    let validation_error = if ap.has_saved_connection {
                        log::info!(
                            "WiFi: Running validation for '{}' at path: {}",
                            ap.ssid,
                            ap.path
                        );
                        let error = Wifi::instance().validate_connection_security(&ap.path);
                        if let Some(ref err_msg) = error {
                            log::info!("WiFi: Validation error for '{}': {}", ap.ssid, err_msg);
                        } else {
                            log::debug!("WiFi: Validation passed for '{}'", ap.ssid);
                        }
                        error
                    } else {
                        log::debug!(
                            "WiFi: Skipping validation for '{}' - no saved connection",
                            ap.ssid
                        );
                        None
                    };

                    let validation_error_for_callback = validation_error.clone();
                    SwitchEntry::with_completion_callback(
                        ap.clone(),
                        ap.ssid.clone(),
                        Some(icon_name),
                        None, // no battery icon for WiFi
                        None, // no battery percentage for WiFi
                        None, // no peripheral battery icon
                        None, // no peripheral battery percentage
                        Some(tooltip),
                        is_connected,
                        true,             // switch_enabled
                        validation_error, // show inline error if validation failed
                        move |_, on_complete| {
                            // Don't attempt connection if validation failed
                            if let Some(ref error) = validation_error_for_callback {
                                log::warn!(
                                    "WiFi: Skipping connection attempt due to validation error: {}",
                                    error
                                );
                                on_complete(Err(error.clone()));
                                return;
                            }

                            // Connect with saved connection or without password
                            Wifi::instance().connect(Some(ssid_clone.clone()), None);
                            // TODO: WiFi service doesn't have completion callback yet,
                            // assume success for now
                            on_complete(Ok(()));
                        },
                        move |ap_entry, on_complete| {
                            if ap_entry.ssid == current_ssid_clone {
                                Wifi::instance().disconnect();
                                on_complete(Ok(()));
                            } else {
                                on_complete(Ok(()));
                            }
                        },
                    )
                }
            })
            .collect();

        let wifi_clone = wifi.clone();
        let on_refresh = if scanning {
            None
        } else {
            Some(Rc::new(move || {
                wifi_clone.scan();
            }) as Rc<dyn Fn()>)
        };

        let popover_clone = popover.clone();
        let on_settings_open = Rc::new(move || {
            popover_clone.popdown();
            glib::timeout_add_local_once(std::time::Duration::from_millis(1), || {
                let _ = Command::new("nm-connection-editor").spawn();
            });
        });

        let switch_list = SwitchList::new(
            Some(header_text),
            on_refresh,
            Some(on_settings_open),
            entries,
        );

        container.append(switch_list.widget());
    }

    fn get_icon_for_strength(strength: u8) -> String {
        if strength >= 80 {
            "network-wireless-signal-excellent-symbolic".to_string()
        } else if strength >= 60 {
            "network-wireless-signal-good-symbolic".to_string()
        } else if strength >= 40 {
            "network-wireless-signal-ok-symbolic".to_string()
        } else if strength >= 20 {
            "network-wireless-signal-weak-symbolic".to_string()
        } else {
            "network-wireless-signal-none-symbolic".to_string()
        }
    }

    fn format_frequencies(frequencies: &[u32]) -> String {
        let formatted: Vec<String> = frequencies
            .iter()
            .map(|&freq_mhz| {
                // Convert MHz to GHz with one decimal place
                let freq_ghz = freq_mhz as f64 / 1000.0;
                format!("{:.1} GHz", freq_ghz)
            })
            .collect();

        formatted.join(", ")
    }

    pub fn widget(&self) -> &Box {
        &self.widget
    }
}
