use glib::clone;
use gtk4::prelude::*;
use gtk4::{Box, Orientation, Revealer};
use std::rc::Rc;

use crate::service::bluetooth::Bluetooth;
use crate::service::charge_threshold::ChargeThreshold;
use crate::service::ethernet::Ethernet;
use crate::service::fcitx::Fcitx;
use crate::service::notifications::Notifications as NotificationsService;
use crate::service::power_profiles::PowerProfiles;
use crate::service::vpn::Vpn;
use crate::service::wifi::Wifi;
use crate::widget::bluetooth_content::BluetoothContent;
use crate::widget::language_content::LanguageContent;
use crate::widget::notifications::Notifications as NotificationsWidget;
use crate::widget::power_profiles_content::PowerProfilesContent;
use crate::widget::toggle::Toggle;
use crate::widget::vpn_content::VpnContent;
use crate::widget::wifi_content::WifiContent;
use crate::widget::wired_content::WiredContent;

fn upcase_profile(profile: &str) -> String {
    profile
        .split('-')
        .map(|p| {
            let mut chars = p.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect::<Vec<_>>()
        .join("")
}

fn wrap_revealer_content<W: IsA<gtk4::Widget>>(content: &W) -> Box {
    let wrapper = Box::new(Orientation::Vertical, 0);
    wrapper.add_css_class("RevealerContent");
    wrapper.append(content);
    wrapper
}

fn get_bluetooth_small_text(bluetooth: &Bluetooth) -> Option<String> {
    let devices = bluetooth.devices();
    let nearby_devices: Vec<_> = devices.iter().filter(|d| d.nearby).collect();
    let connected_devices: Vec<_> = nearby_devices.iter().filter(|d| d.connected).collect();
    let connected_count = connected_devices.len();

    if connected_count == 1 {
        Some(connected_devices[0].name.clone())
    } else if connected_count > 1 {
        Some(format!("Connected: {}", connected_count))
    } else if !nearby_devices.is_empty() {
        Some(format!("Devices: {}", nearby_devices.len()))
    } else {
        None
    }
}

struct ToggleEntry {
    toggle: Rc<Toggle>,
    revealer: Rc<Revealer>,
}

pub struct Toggles {
    widget: Box,
    toggle_entries: Rc<Vec<ToggleEntry>>,
}

impl Toggles {
    pub fn new(
        popover: &gtk4::Popover,
        brightness_revealer: &Rc<Revealer>,
        brightness_slider: &Rc<std::cell::OnceCell<Rc<crate::widget::slider::Slider>>>,
        audio_revealer: &Rc<Revealer>,
        audio_slider: &Rc<std::cell::OnceCell<Rc<crate::widget::slider::Slider>>>,
        mic_revealer: &Rc<Revealer>,
        mic_slider: &Rc<std::cell::OnceCell<Rc<crate::widget::slider::Slider>>>,
    ) -> Self {
        let container = Box::new(Orientation::Vertical, 0);
        container.add_css_class("Toggles");

        let bluetooth = Bluetooth::instance();
        let ethernet = Ethernet::instance();
        let wifi = Wifi::instance();
        let vpn = Vpn::instance();
        let power_profiles = PowerProfiles::instance();
        let charge_threshold = ChargeThreshold::instance();
        let notifications_service = NotificationsService::instance();
        let fcitx = Fcitx::instance();

        // Build list of toggle entries
        let mut toggle_entries: Vec<ToggleEntry> = Vec::new();

        // 1. Wired toggle
        let wired_revealer = Rc::new(Revealer::new());
        wired_revealer.set_reveal_child(false);

        let connected = ethernet.connected();
        let connection_id = ethernet.connection_id();
        let devices_count = ethernet.devices().len();

        let wired_small_text = if connected && !connection_id.is_empty() {
            Some(connection_id.clone())
        } else if !connected {
            Some(format!("Devices: {}", devices_count))
        } else {
            None
        };

        let wired_toggle = Rc::new(Toggle::new(
            &ethernet.icon_name(),
            "Wired",
            wired_small_text.as_deref(),
            connected,
            {
                Some(move |active: bool| {
                    if active {
                        Ethernet::instance().connect_all(|_error| {});
                    } else {
                        Ethernet::instance().disconnect_all(|_error| {});
                    }
                })
            },
        ));

        wired_toggle.set_expander_visible(devices_count > 0);

        let wired_content = WiredContent::new(popover);
        let wired_wrapper = wrap_revealer_content(wired_content.widget());
        wired_revealer.set_child(Some(&wired_wrapper));

        toggle_entries.push(ToggleEntry {
            toggle: wired_toggle.clone(),
            revealer: wired_revealer.clone(),
        });

        // 2. Wifi toggle
        let wifi_revealer = Rc::new(Revealer::new());
        wifi_revealer.set_reveal_child(false);

        let wifi_small_text = if wifi.connected() {
            Some(wifi.ssid())
        } else {
            None
        };

        let wifi_toggle = Rc::new(Toggle::new(
            &wifi.icon_name(),
            "Wifi",
            wifi_small_text.as_deref(),
            wifi.enabled(),
            {
                Some(move |active: bool| {
                    Wifi::instance().set_enabled(active);
                })
            },
        ));

        wifi_toggle.set_expander_visible(wifi.enabled());

        let wifi_content = WifiContent::new(popover);
        let wifi_wrapper = wrap_revealer_content(wifi_content.widget());
        wifi_revealer.set_child(Some(&wifi_wrapper));

        toggle_entries.push(ToggleEntry {
            toggle: wifi_toggle.clone(),
            revealer: wifi_revealer.clone(),
        });

        // 3. VPN toggle
        let vpn_revealer = Rc::new(Revealer::new());
        vpn_revealer.set_reveal_child(false);

        let connected = vpn.connected();
        let connection_id = vpn.connection_id();
        let connections_count = vpn.connections().len();

        let vpn_small_text = if connected && !connection_id.is_empty() {
            Some(connection_id.clone())
        } else if !connected && connections_count > 0 {
            Some(format!("VPNs: {}", connections_count))
        } else {
            None
        };

        let vpn_toggle = Rc::new(Toggle::new(
            &vpn.icon_name(),
            "VPN",
            vpn_small_text.as_deref(),
            connected,
            {
                Some(move |active: bool| {
                    if active {
                        Vpn::instance().connect_all(|_error| {});
                    } else {
                        Vpn::instance().disconnect_all(|_error| {});
                    }
                })
            },
        ));

        vpn_toggle.set_expander_visible(connections_count > 0);

        let vpn_content = VpnContent::new(popover);
        let vpn_wrapper = wrap_revealer_content(vpn_content.widget());
        vpn_revealer.set_child(Some(&vpn_wrapper));

        toggle_entries.push(ToggleEntry {
            toggle: vpn_toggle.clone(),
            revealer: vpn_revealer.clone(),
        });

        // 4. Bluetooth toggle
        let bluetooth_revealer = Rc::new(Revealer::new());
        bluetooth_revealer.set_reveal_child(false);

        let bluetooth_enabled = bluetooth.enabled();

        let bluetooth_small_text = get_bluetooth_small_text(&bluetooth);

        let bluetooth_toggle = Rc::new(Toggle::new(
            &bluetooth.icon_name(),
            "Bluetooth",
            bluetooth_small_text.as_deref(),
            bluetooth_enabled,
            Some(move |active: bool| {
                Bluetooth::instance().set_enabled(active);
            }),
        ));

        bluetooth_toggle.set_expander_visible(bluetooth_enabled);

        let bluetooth_content = BluetoothContent::new(popover);
        let bluetooth_wrapper = wrap_revealer_content(bluetooth_content.widget());
        bluetooth_revealer.set_child(Some(&bluetooth_wrapper));

        toggle_entries.push(ToggleEntry {
            toggle: bluetooth_toggle.clone(),
            revealer: bluetooth_revealer.clone(),
        });

        // 5. Battery toggle
        let battery_revealer = Rc::new(Revealer::new());
        battery_revealer.set_reveal_child(false);

        let threshold = charge_threshold.threshold();

        let battery_toggle = Rc::new(Toggle::new(
            "battery-symbolic",
            "Battery",
            Some(&format!("Max: {}%", threshold)),
            threshold == 100,
            {
                let charge_threshold_clone = charge_threshold.clone();
                Some(move |active: bool| {
                    charge_threshold_clone.set_threshold(if active { 100 } else { 80 });
                })
            },
        ));

        battery_toggle.set_expander_visible(false);

        toggle_entries.push(ToggleEntry {
            toggle: battery_toggle.clone(),
            revealer: battery_revealer.clone(),
        });

        // 6. Power profile toggle
        let power_profile_revealer = Rc::new(Revealer::new());
        power_profile_revealer.set_reveal_child(false);

        let active_profile = power_profiles.active_profile();

        let profile_text = if !power_profiles.available() {
            "Unavailable".to_string()
        } else if active_profile.is_empty() {
            "Loading...".to_string()
        } else {
            upcase_profile(&active_profile)
        };

        let power_profile_toggle_holder: Rc<std::cell::OnceCell<Rc<Toggle>>> =
            Rc::new(std::cell::OnceCell::new());
        let power_profile_toggle_holder_clone = power_profile_toggle_holder.clone();

        let power_profile_toggle = Rc::new(Toggle::new(
            &power_profiles.icon_name(),
            "Profile",
            Some(&profile_text),
            active_profile != "power-saver",
            {
                Some(move |_active: bool| {
                    let power_profiles = PowerProfiles::instance();
                    let current_profile = power_profiles.active_profile();

                    // Cycle only between power-saver and balanced
                    // Performance must be selected manually via switch list
                    let next_profile = match current_profile.as_str() {
                        "power-saver" => "balanced",
                        "balanced" => "power-saver",
                        "performance" => "power-saver",
                        _ => "balanced",
                    };

                    power_profiles.set_active_profile(next_profile);

                    // Update toggle state after a brief delay to allow service to update
                    if let Some(toggle) = power_profile_toggle_holder_clone.get() {
                        let toggle_clone = toggle.clone();
                        glib::idle_add_local_once(move || {
                            let pp = PowerProfiles::instance();
                            let profile = pp.active_profile();
                            toggle_clone.set_active(profile != "power-saver");
                        });
                    }
                })
            },
        ));

        power_profile_toggle_holder
            .set(power_profile_toggle.clone())
            .ok();
        power_profile_toggle.set_expander_visible(power_profiles.available());

        let power_content = PowerProfilesContent::new();
        let power_wrapper = wrap_revealer_content(power_content.widget());
        power_profile_revealer.set_child(Some(&power_wrapper));

        toggle_entries.push(ToggleEntry {
            toggle: power_profile_toggle.clone(),
            revealer: power_profile_revealer.clone(),
        });

        // 7. Language toggle
        let language_revealer = Rc::new(Revealer::new());
        language_revealer.set_reveal_child(false);

        let available_ims = fcitx.available_ims();
        let enabled_ims: Vec<_> = available_ims.iter().filter(|im| im.enabled).collect();

        let current_im_name = if enabled_ims.is_empty() {
            "No languages".to_string()
        } else {
            let current_im = fcitx.current_im();
            available_ims
                .iter()
                .find(|im| im.unique_name == current_im)
                .map(|im| im.name.clone())
                .unwrap_or_else(|| "Unknown".to_string())
        };

        let is_not_first_language = enabled_ims
            .first()
            .map(|first| {
                let current_im = fcitx.current_im();
                first.unique_name != current_im
            })
            .unwrap_or(false);

        let has_languages = !enabled_ims.is_empty();

        let language_toggle_holder: Rc<std::cell::OnceCell<Rc<Toggle>>> =
            Rc::new(std::cell::OnceCell::new());
        let language_toggle_holder_clone = language_toggle_holder.clone();

        let language_toggle = Rc::new(Toggle::new(
            "input-keyboard-symbolic",
            "Language",
            Some(&current_im_name),
            is_not_first_language,
            Some(move |_active: bool| {
                let fcitx = Fcitx::instance();
                let available_ims = fcitx.available_ims();
                let enabled_ims: Vec<_> = available_ims.iter().filter(|im| im.enabled).collect();

                if enabled_ims.len() <= 1 {
                    if let Some(toggle) = language_toggle_holder_clone.get() {
                        let toggle_clone = toggle.clone();
                        glib::idle_add_local_once(move || {
                            toggle_clone.set_active(false);
                        });
                    }
                    return;
                }

                let current_im = fcitx.current_im();

                if let Some(first) = enabled_ims.first() {
                    if current_im == first.unique_name {
                        if enabled_ims.len() > 1 {
                            fcitx.set_current_im(&enabled_ims[1].unique_name);
                        }
                    } else {
                        fcitx.set_current_im(&first.unique_name);
                    }
                }
            }),
        ));

        language_toggle_holder.set(language_toggle.clone()).ok();
        language_toggle.set_expander_visible(has_languages);

        let language_content = LanguageContent::new(popover);
        let language_wrapper = wrap_revealer_content(language_content.widget());
        language_revealer.set_child(Some(&language_wrapper));

        toggle_entries.push(ToggleEntry {
            toggle: language_toggle.clone(),
            revealer: language_revealer.clone(),
        });

        // 8. Notifications toggle
        let notifications_revealer = Rc::new(Revealer::new());
        notifications_revealer.set_reveal_child(false);

        let notifications_count = notifications_service.notifications().len();
        let dont_disturb = notifications_service.dont_disturb();

        let notifications_small_text = if notifications_count > 0 {
            Some(format!("Count: {}", notifications_count))
        } else {
            None
        };

        let notifications_toggle = Rc::new(Toggle::new(
            if dont_disturb {
                "notifications-disabled-symbolic"
            } else {
                "preferences-system-notifications-symbolic"
            },
            "Notifications",
            notifications_small_text.as_deref(),
            !dont_disturb,
            {
                Some(move |active: bool| {
                    NotificationsService::instance().set_dont_disturb(!active);
                })
            },
        ));

        notifications_toggle.set_expander_visible(!dont_disturb && notifications_count > 0);

        let notifications_content = gtk4::ScrolledWindow::new();
        notifications_content.set_hscrollbar_policy(gtk4::PolicyType::Never);
        notifications_content.set_vscrollbar_policy(gtk4::PolicyType::Automatic);
        notifications_content.set_propagate_natural_height(true);
        notifications_content.set_max_content_height(700);

        let popover_clone = popover.clone();
        let _notifications_widget = NotificationsWidget::new(
            |notification| notification.app_name.clone(),
            true,
            false,
            Some(move || {
                popover_clone.popdown();
            }),
            None::<fn()>,
            None::<fn()>,
        );

        notifications_content.set_child(Some(_notifications_widget.widget()));
        let notifications_wrapper = wrap_revealer_content(&notifications_content);
        notifications_revealer.set_child(Some(&notifications_wrapper));

        toggle_entries.push(ToggleEntry {
            toggle: notifications_toggle.clone(),
            revealer: notifications_revealer.clone(),
        });

        // Build rows from toggle entries (2 per row)
        for (index, entry) in toggle_entries.iter().enumerate() {
            if index % 2 == 0 {
                // Start a new row
                let row = Box::new(Orientation::Horizontal, 0);
                row.append(entry.toggle.widget());

                // Check if there's a next entry for the right side
                if index + 1 < toggle_entries.len() {
                    row.append(toggle_entries[index + 1].toggle.widget());
                }

                container.append(&row);
            }

            // Add revealer for this entry
            container.append(entry.revealer.as_ref());
        }

        // Wrap toggle_entries in Rc for use in callbacks
        let toggle_entries_rc = Rc::new(toggle_entries);

        // Set up expander callbacks for each toggle
        let brightness_revealer_clone = brightness_revealer.clone();
        let brightness_slider_clone = brightness_slider.clone();
        let audio_revealer_clone = audio_revealer.clone();
        let audio_slider_clone = audio_slider.clone();
        let mic_revealer_clone = mic_revealer.clone();
        let mic_slider_clone = mic_slider.clone();
        for (current_index, entry) in toggle_entries_rc.iter().enumerate() {
            let entries_clone = toggle_entries_rc.clone();
            let brightness_revealer_clone2 = brightness_revealer_clone.clone();
            let brightness_slider_clone2 = brightness_slider_clone.clone();
            let audio_revealer_clone2 = audio_revealer_clone.clone();
            let audio_slider_clone2 = audio_slider_clone.clone();
            let mic_revealer_clone2 = mic_revealer_clone.clone();
            let mic_slider_clone2 = mic_slider_clone.clone();
            entry.toggle.connect_expander_toggled(move |is_expanded| {
                if is_expanded {
                    // Close all slider expanders when any toggle expands
                    brightness_revealer_clone2.set_reveal_child(false);
                    if let Some(slider) = brightness_slider_clone2.get() {
                        slider.set_expanded(false);
                    }
                    audio_revealer_clone2.set_reveal_child(false);
                    if let Some(slider) = audio_slider_clone2.get() {
                        slider.set_expanded(false);
                    }
                    mic_revealer_clone2.set_reveal_child(false);
                    if let Some(slider) = mic_slider_clone2.get() {
                        slider.set_expanded(false);
                    }

                    // Collapse all other toggles
                    for (i, e) in entries_clone.iter().enumerate() {
                        if i != current_index {
                            e.revealer.set_reveal_child(false);
                            e.toggle.set_expanded(false);
                        } else {
                            e.revealer.set_reveal_child(true);
                        }
                    }
                } else {
                    // Collapse current toggle
                    entries_clone[current_index]
                        .revealer
                        .set_reveal_child(false);
                }
            });
        }

        // Update wired toggle when connection changes
        ethernet.connect_notify_local(
            Some("connected"),
            clone!(
                #[strong]
                wired_toggle,
                #[strong]
                wired_revealer,
                move |eth, _| {
                    let is_connected = eth.connected();
                    let devices_count = eth.devices().len();
                    wired_toggle.set_active(is_connected);
                    wired_toggle.set_expander_visible(devices_count > 0);
                    wired_toggle.set_icon_name(&eth.icon_name());

                    // Update small text based on connection state
                    let connection_id = eth.connection_id();
                    if is_connected && !connection_id.is_empty() {
                        wired_toggle.set_small_text(Some(&connection_id));
                    } else if !is_connected {
                        wired_toggle.set_small_text(Some(&format!("Devices: {}", devices_count)));
                    } else {
                        wired_toggle.set_small_text(None);
                    }

                    // Collapse revealer only if wired is disconnected AND no devices available
                    if !is_connected && devices_count == 0 {
                        wired_revealer.set_reveal_child(false);
                        wired_toggle.set_expanded(false);
                    }
                }
            ),
        );

        ethernet.connect_notify_local(
            Some("connection-id"),
            clone!(
                #[strong]
                wired_toggle,
                move |eth, _| {
                    let is_connected = eth.connected();
                    let connection_id = eth.connection_id();
                    let devices_count = eth.devices().len();
                    if is_connected && !connection_id.is_empty() {
                        wired_toggle.set_small_text(Some(&connection_id));
                    } else if !is_connected {
                        wired_toggle.set_small_text(Some(&format!("Devices: {}", devices_count)));
                    } else {
                        wired_toggle.set_small_text(None);
                    }
                }
            ),
        );

        ethernet.connect_notify_local(
            Some("state"),
            clone!(
                #[strong]
                wired_toggle,
                move |eth, _| {
                    wired_toggle.set_icon_name(&eth.icon_name());
                }
            ),
        );

        ethernet.connect_notify_local(
            Some("devices"),
            clone!(
                #[strong]
                wired_toggle,
                move |eth, _| {
                    let devices_count = eth.devices().len();
                    wired_toggle.set_expander_visible(devices_count > 0);

                    // Update small text when not connected
                    if !eth.connected() {
                        wired_toggle.set_small_text(Some(&format!("Devices: {}", devices_count)));
                    }
                }
            ),
        );

        // Update wifi toggle small text when connection changes
        wifi.connect_notify_local(
            Some("connected"),
            clone!(
                #[strong]
                wifi_toggle,
                move |w, _| {
                    if w.connected() {
                        wifi_toggle.set_small_text(Some(&w.ssid()));
                    } else {
                        wifi_toggle.set_small_text(None);
                    }
                }
            ),
        );

        wifi.connect_notify_local(
            Some("ssid"),
            clone!(
                #[strong]
                wifi_toggle,
                move |w, _| {
                    if w.connected() {
                        wifi_toggle.set_small_text(Some(&w.ssid()));
                    } else {
                        wifi_toggle.set_small_text(None);
                    }
                }
            ),
        );

        wifi.connect_notify_local(
            Some("state"),
            clone!(
                #[strong]
                wifi_toggle,
                move |w, _| {
                    wifi_toggle.set_icon_name(&w.icon_name());
                }
            ),
        );

        wifi.connect_notify_local(
            Some("strength"),
            clone!(
                #[strong]
                wifi_toggle,
                move |w, _| {
                    wifi_toggle.set_icon_name(&w.icon_name());
                }
            ),
        );

        wifi.connect_notify_local(
            Some("enabled"),
            clone!(
                #[strong]
                wifi_toggle,
                #[strong]
                wifi_revealer,
                move |w, _| {
                    let is_enabled = w.enabled();
                    wifi_toggle.set_active(is_enabled);
                    wifi_toggle.set_expander_visible(is_enabled);
                    wifi_toggle.set_icon_name(&w.icon_name());

                    // Collapse revealer if wifi is disabled
                    if !is_enabled {
                        wifi_revealer.set_reveal_child(false);
                        wifi_toggle.set_expanded(false);
                    }
                }
            ),
        );

        // Update VPN toggle when connection changes
        vpn.connect_notify_local(
            Some("connected"),
            clone!(
                #[strong]
                vpn_toggle,
                #[strong]
                vpn_revealer,
                move |v, _| {
                    let is_connected = v.connected();
                    let connections_count = v.connections().len();
                    vpn_toggle.set_active(is_connected);
                    vpn_toggle.set_expander_visible(connections_count > 0);
                    vpn_toggle.set_icon_name(&v.icon_name());

                    let connection_id = v.connection_id();
                    if is_connected && !connection_id.is_empty() {
                        vpn_toggle.set_small_text(Some(&connection_id));
                    } else if !is_connected && connections_count > 0 {
                        vpn_toggle.set_small_text(Some(&format!("VPNs: {}", connections_count)));
                    } else {
                        vpn_toggle.set_small_text(None);
                    }

                    if !is_connected && connections_count == 0 {
                        vpn_revealer.set_reveal_child(false);
                        vpn_toggle.set_expanded(false);
                    }
                }
            ),
        );

        vpn.connect_notify_local(
            Some("connection-id"),
            clone!(
                #[strong]
                vpn_toggle,
                move |v, _| {
                    let is_connected = v.connected();
                    let connection_id = v.connection_id();
                    let connections_count = v.connections().len();
                    if is_connected && !connection_id.is_empty() {
                        vpn_toggle.set_small_text(Some(&connection_id));
                    } else if !is_connected && connections_count > 0 {
                        vpn_toggle.set_small_text(Some(&format!("VPNs: {}", connections_count)));
                    } else {
                        vpn_toggle.set_small_text(None);
                    }
                }
            ),
        );

        vpn.connect_notify_local(
            Some("connections"),
            clone!(
                #[strong]
                vpn_toggle,
                move |v, _| {
                    let connections_count = v.connections().len();
                    vpn_toggle.set_expander_visible(connections_count > 0);

                    if !v.connected() && connections_count > 0 {
                        vpn_toggle.set_small_text(Some(&format!("VPNs: {}", connections_count)));
                    } else if connections_count == 0 {
                        vpn_toggle.set_small_text(None);
                    }
                }
            ),
        );

        // Update bluetooth toggle when enabled state changes
        bluetooth.connect_notify_local(
            Some("enabled"),
            clone!(
                #[strong]
                bluetooth_toggle,
                #[strong]
                bluetooth_revealer,
                move |bt, _| {
                    let enabled = bt.enabled();
                    bluetooth_toggle.set_active(enabled);
                    bluetooth_toggle.set_icon_name(&bt.icon_name());
                    bluetooth_toggle.set_expander_visible(enabled);

                    // Collapse revealer if bluetooth is disabled
                    if !enabled {
                        bluetooth_revealer.set_reveal_child(false);
                        bluetooth_toggle.set_expanded(false);
                    }
                }
            ),
        );

        // Update bluetooth toggle icon when expected devices connection state changes
        bluetooth.connect_notify_local(
            Some("missing-expected-devices"),
            clone!(
                #[strong]
                bluetooth_toggle,
                move |bt, _| {
                    bluetooth_toggle.set_icon_name(&bt.icon_name());
                }
            ),
        );

        // Update bluetooth toggle when devices change
        bluetooth.connect_notify_local(
            Some("devices"),
            clone!(
                #[strong]
                bluetooth_toggle,
                move |bt, _| {
                    bluetooth_toggle.set_icon_name(&bt.icon_name());
                    let small_text = get_bluetooth_small_text(bt);
                    bluetooth_toggle.set_small_text(small_text.as_deref());
                }
            ),
        );

        // Update battery toggle when threshold changes
        charge_threshold.connect_notify_local(
            Some("threshold"),
            clone!(
                #[strong]
                battery_toggle,
                move |t, _| {
                    let threshold = t.threshold();
                    battery_toggle.set_small_text(Some(&format!("Max: {}%", threshold)));
                    battery_toggle.set_active(threshold == 100);
                }
            ),
        );

        // Update power profile toggle when active profile changes
        power_profiles.connect_notify_local(
            Some("active-profile"),
            clone!(
                #[strong]
                power_profile_toggle,
                move |pp, _| {
                    let profile = pp.active_profile();
                    power_profile_toggle.set_icon_name(&pp.icon_name());
                    power_profile_toggle.set_small_text(Some(&upcase_profile(&profile)));
                    power_profile_toggle.set_active(profile != "power-saver");
                }
            ),
        );

        // Update power profile toggle when the daemon appears or goes away
        power_profiles.connect_notify_local(
            Some("available"),
            clone!(
                #[strong]
                power_profile_toggle,
                #[strong]
                power_profile_revealer,
                move |pp, _| {
                    let available = pp.available();
                    power_profile_toggle.set_expander_visible(available);

                    // Collapse revealer if there is no daemon to talk to
                    if !available {
                        power_profile_toggle.set_small_text(Some("Unavailable"));
                        power_profile_revealer.set_reveal_child(false);
                        power_profile_toggle.set_expanded(false);
                    }
                }
            ),
        );

        // Update notifications toggle when dont-disturb changes
        notifications_service.connect_notify_local(
            Some("dont-disturb"),
            clone!(
                #[strong]
                notifications_toggle,
                #[strong]
                notifications_revealer,
                move |n, _| {
                    let dont_disturb = n.dont_disturb();
                    notifications_toggle.set_active(!dont_disturb);
                    notifications_toggle.set_icon_name(if dont_disturb {
                        "notifications-disabled-symbolic"
                    } else {
                        "preferences-system-notifications-symbolic"
                    });

                    let notifications_count = n.notifications().len();
                    notifications_toggle
                        .set_expander_visible(!dont_disturb && notifications_count > 0);

                    // Collapse revealer if dont-disturb is enabled
                    if dont_disturb {
                        notifications_revealer.set_reveal_child(false);
                        notifications_toggle.set_expanded(false);
                    }
                }
            ),
        );

        // Update notifications toggle when notification is added
        notifications_service.connect_local(
            "notified",
            false,
            clone!(
                #[strong]
                notifications_toggle,
                move |_values| {
                    let notifications = NotificationsService::instance();
                    let notifications_count = notifications.notifications().len();
                    let dont_disturb = notifications.dont_disturb();

                    if notifications_count > 0 {
                        notifications_toggle
                            .set_small_text(Some(&format!("Count: {}", notifications_count)));
                    } else {
                        notifications_toggle.set_small_text(None);
                    }

                    notifications_toggle
                        .set_expander_visible(!dont_disturb && notifications_count > 0);
                    None
                }
            ),
        );

        // Update notifications toggle when notification is resolved
        notifications_service.connect_local(
            "resolved",
            false,
            clone!(
                #[strong]
                notifications_toggle,
                #[strong]
                notifications_revealer,
                move |_values| {
                    let notifications = NotificationsService::instance();
                    let notifications_count = notifications.notifications().len();
                    let dont_disturb = notifications.dont_disturb();

                    if notifications_count > 0 {
                        notifications_toggle
                            .set_small_text(Some(&format!("Count: {}", notifications_count)));
                    } else {
                        notifications_toggle.set_small_text(None);
                    }

                    notifications_toggle
                        .set_expander_visible(!dont_disturb && notifications_count > 0);

                    // Collapse revealer if no notifications
                    if notifications_count == 0 {
                        notifications_revealer.set_reveal_child(false);
                        notifications_toggle.set_expanded(false);
                    }
                    None
                }
            ),
        );

        // Update language toggle when current input method changes
        fcitx.connect_notify_local(
            Some("current-im"),
            clone!(
                #[strong]
                language_toggle,
                move |fcitx, _| {
                    let available_ims = fcitx.available_ims();
                    let enabled_ims: Vec<_> =
                        available_ims.iter().filter(|im| im.enabled).collect();

                    // Display "No languages" if no languages available
                    let current_im_name = if enabled_ims.is_empty() {
                        "No languages".to_string()
                    } else {
                        let current_im = fcitx.current_im();
                        available_ims
                            .iter()
                            .find(|im| im.unique_name == current_im)
                            .map(|im| im.name.clone())
                            .unwrap_or_else(|| "Unknown".to_string())
                    };

                    language_toggle.set_small_text(Some(&current_im_name));

                    // Update toggle state: enabled when NOT on first language
                    let is_not_first_language = enabled_ims
                        .first()
                        .map(|first| {
                            let current_im = fcitx.current_im();
                            first.unique_name != current_im
                        })
                        .unwrap_or(false);

                    language_toggle.set_active(is_not_first_language);

                    // Update expander visibility: show as long as there is at least 1 language
                    let has_languages = !enabled_ims.is_empty();
                    language_toggle.set_expander_visible(has_languages);
                }
            ),
        );

        // Update language toggle when available languages list changes
        fcitx.connect_notify_local(
            Some("available-ims-count"),
            clone!(
                #[strong]
                language_toggle,
                move |fcitx, _| {
                    let available_ims = fcitx.available_ims();
                    let enabled_ims: Vec<_> =
                        available_ims.iter().filter(|im| im.enabled).collect();

                    // Display "No languages" if no languages available
                    let current_im_name = if enabled_ims.is_empty() {
                        "No languages".to_string()
                    } else {
                        let current_im = fcitx.current_im();
                        available_ims
                            .iter()
                            .find(|im| im.unique_name == current_im)
                            .map(|im| im.name.clone())
                            .unwrap_or_else(|| "Unknown".to_string())
                    };

                    language_toggle.set_small_text(Some(&current_im_name));

                    // Update toggle state: enabled when NOT on first language
                    let is_not_first_language = enabled_ims
                        .first()
                        .map(|first| {
                            let current_im = fcitx.current_im();
                            first.unique_name != current_im
                        })
                        .unwrap_or(false);

                    language_toggle.set_active(is_not_first_language);

                    // Update expander visibility: show as long as there is at least 1 language
                    let has_languages = !enabled_ims.is_empty();
                    language_toggle.set_expander_visible(has_languages);
                }
            ),
        );

        // When popover opens, close all expanded contents and open notifications
        let entries_for_popover = toggle_entries_rc.clone();
        popover.connect_show(move |_| {
            let notifications = NotificationsService::instance();
            let notifications_count = notifications.notifications().len();
            let dont_disturb = notifications.dont_disturb();

            // Close all revealers first
            for entry in entries_for_popover.iter() {
                entry.revealer.set_reveal_child(false);
                entry.toggle.set_expanded(false);
            }

            // Only expand notifications (index 7) if there are notifications and DND is off
            if !dont_disturb && notifications_count > 0 {
                entries_for_popover[7].revealer.set_reveal_child(true);
                entries_for_popover[7].toggle.set_expanded(true);
            }
        });

        Self {
            widget: container,
            toggle_entries: toggle_entries_rc,
        }
    }

    pub fn widget(&self) -> &Box {
        &self.widget
    }

    pub fn close_all_expanders(&self) {
        for entry in self.toggle_entries.iter() {
            entry.revealer.set_reveal_child(false);
            entry.toggle.set_expanded(false);
        }
    }

    pub fn focus_notifications_toggle(&self) {
        // Notifications toggle is at index 7 (8th toggle)
        if let Some(notifications_entry) = self.toggle_entries.get(7) {
            notifications_entry.toggle.grab_focus_expander_if_visible();
        }
    }
}
