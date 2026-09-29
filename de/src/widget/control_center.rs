use glib::clone;
use gtk4::prelude::*;
use gtk4::{
    Box, Calendar, CenterBox, Label, MenuButton, Orientation, Popover, PositionType, Revealer,
};
use std::rc::Rc;

use crate::service::audio::Audio;
use crate::service::brightness::Brightness;
use crate::service::system_info::SystemInfo;
use crate::widget::audio_devices_content::AudioDevicesContent;
use crate::widget::clients_content::ClientsContent;
use crate::widget::keyboard_backlight_content::KeyboardBacklightContent;
use crate::widget::mic_devices_content::MicDevicesContent;
use crate::widget::slider::Slider;
use crate::widget::system_info_content::SystemInfoContent;
use crate::widget::toggles::Toggles;
use crate::window::confirmation_dialog::ConfirmationDialog;

pub struct ControlCenter {
    widget: Box,
    toggles: Rc<Toggles>,
}

impl ControlCenter {
    pub fn new(popover: &Popover) -> Self {
        let container = Box::new(Orientation::Vertical, 0);
        container.add_css_class("ControlCenter");

        // Add QuickControls
        let quick_controls = Self::create_quick_controls(popover);
        container.append(&quick_controls);

        // Create shared state for revealers and sliders
        let keyboard_revealer = Rc::new(Revealer::new());
        let audio_revealer = Rc::new(Revealer::new());
        let mic_revealer = Rc::new(Revealer::new());
        let brightness_slider = Rc::new(std::cell::OnceCell::new());
        let audio_slider = Rc::new(std::cell::OnceCell::new());
        let mic_slider = Rc::new(std::cell::OnceCell::new());

        // Add Toggles first (need references for sliders)
        let toggles = Rc::new(Toggles::new(
            popover,
            &keyboard_revealer,
            &brightness_slider,
            &audio_revealer,
            &audio_slider,
            &mic_revealer,
            &mic_slider,
        ));

        // Add Sliders with reference to toggles
        let sliders = Self::create_sliders(
            &toggles,
            &keyboard_revealer,
            &audio_revealer,
            &mic_revealer,
            &brightness_slider,
            &audio_slider,
            &mic_slider,
        );
        container.append(&sliders);

        // Append toggles widget
        container.append(toggles.widget());

        Self {
            widget: container,
            toggles: toggles.clone(),
        }
    }

    pub fn widget(&self) -> &Box {
        &self.widget
    }

    pub fn focus_notifications_toggle(&self) {
        self.toggles.focus_notifications_toggle();
    }

    fn create_quick_controls(popover: &Popover) -> Box {
        let container = Box::new(Orientation::Vertical, 0);
        container.add_css_class("QuickControls");

        // First row: Close, Calendar | Date | Suspend, Shutdown
        let first_row = Self::create_first_row(popover);
        container.append(&first_row);

        // Second row: System Info, Clients | Logout, Reboot
        let second_row = Self::create_second_row(popover);
        container.append(&second_row);

        container
    }

    fn create_first_row(popover: &Popover) -> CenterBox {
        let first_row = CenterBox::new();

        // Left: Lock, Calendar
        let left_box = Box::new(Orientation::Horizontal, 0);

        let lock_button = gtk4::Button::from_icon_name("system-lock-screen-symbolic");
        lock_button.set_tooltip_text(Some("Lock Screen"));
        lock_button.connect_clicked(clone!(
            #[weak]
            popover,
            move |_| {
                popover.popdown();
                // Use timeout to ensure popover closes before executing command
                glib::timeout_add_local_once(std::time::Duration::from_millis(1), || {
                    use std::env;
                    // Get HOME to construct library path and PATH for hyprlock
                    if let Ok(home) = env::var("HOME") {
                        let lib_path = format!("{}/.local/lib64", home);
                        let local_bin = format!("{}/.local/bin", home);
                        let current_path = env::var("PATH").unwrap_or_default();
                        let extended_path = if current_path.is_empty() {
                            local_bin
                        } else {
                            format!("{}:{}", local_bin, current_path)
                        };
                        // Use systemd-run to move hyprlock to independent cgroup
                        let _ = std::process::Command::new("systemd-run")
                            .arg("--user")
                            .arg("--scope")
                            .arg("--")
                            .arg("hyprlock")
                            .env("PATH", extended_path)
                            .env("LD_LIBRARY_PATH", lib_path)
                            .spawn();
                    }
                });
            }
        ));
        left_box.append(&lock_button);

        let calendar_button = Self::create_calendar_button();
        left_box.append(&calendar_button);

        first_row.set_start_widget(Some(&left_box));

        // Center: Date
        let date_label = Self::create_date_label();
        first_row.set_center_widget(Some(&date_label));

        // Right: Suspend, Shutdown
        let right_box = Box::new(Orientation::Horizontal, 0);

        let suspend_button = gtk4::Button::from_icon_name("night-light-symbolic");
        suspend_button.set_tooltip_text(Some("Suspend"));
        suspend_button.connect_clicked(|_| {
            ConfirmationDialog::new(
                "Suspend",
                "This will suspend your system",
                "systemctl suspend",
            );
        });
        right_box.append(&suspend_button);

        let shutdown_button = gtk4::Button::from_icon_name("system-shutdown-symbolic");
        shutdown_button.set_tooltip_text(Some("Shutdown"));
        shutdown_button.connect_clicked(|_| {
            ConfirmationDialog::new(
                "Shutdown",
                "This will shutdown your system",
                "hyprshutdown -t 'Shutting down...' --post-cmd 'shutdown -P 0'",
            );
        });
        right_box.append(&shutdown_button);

        first_row.set_end_widget(Some(&right_box));

        first_row
    }

    fn create_system_info_button() -> MenuButton {
        let system_info_button = MenuButton::new();
        system_info_button.set_icon_name("application-x-addon-symbolic");
        system_info_button.set_tooltip_text(Some("System Info"));

        let system_info_popover = Popover::new();
        system_info_popover.set_position(PositionType::Bottom);
        let system_info_content = SystemInfoContent::new();
        system_info_popover.set_child(Some(system_info_content.widget()));
        system_info_button.set_popover(Some(&system_info_popover));

        let refresh_source_id = std::rc::Rc::new(std::cell::RefCell::new(None));

        system_info_popover.connect_show(clone!(
            #[strong]
            refresh_source_id,
            move |_| {
                let system_info = SystemInfo::instance();
                system_info.refresh();

                let source_id = glib::timeout_add_seconds_local(
                    2,
                    clone!(
                        #[strong]
                        system_info,
                        move || {
                            system_info.refresh();
                            glib::ControlFlow::Continue
                        }
                    ),
                );

                *refresh_source_id.borrow_mut() = Some(source_id);
            }
        ));

        system_info_popover.connect_hide(clone!(
            #[strong]
            refresh_source_id,
            move |_| {
                if let Some(source_id) = refresh_source_id.borrow_mut().take() {
                    source_id.remove();
                }
            }
        ));

        system_info_button
    }

    fn create_clients_button(control_center_popover: &Popover) -> MenuButton {
        let clients_button = MenuButton::new();
        clients_button.set_icon_name("view-list-symbolic");
        clients_button.set_tooltip_text(Some("Hyprland Clients"));

        let clients_popover = Popover::new();
        clients_popover.set_position(PositionType::Bottom);
        let clients_content = ClientsContent::new();
        clients_popover.set_child(Some(clients_content.widget()));
        clients_button.set_popover(Some(&clients_popover));

        let cc_popover = control_center_popover.clone();
        let clients_pop = clients_popover.clone();
        clients_popover.connect_show(move |_| {
            clients_content.refresh();
            clients_content.set_popovers(&clients_pop, &cc_popover);
        });

        clients_button
    }

    fn create_calendar_button() -> MenuButton {
        let calendar_button = MenuButton::new();
        calendar_button.set_icon_name("x-office-calendar-symbolic");
        calendar_button.set_tooltip_text(Some("Calendar"));

        let calendar_popover = Popover::new();
        calendar_popover.set_position(PositionType::Bottom);
        let calendar = Calendar::new();
        calendar_popover.set_child(Some(&calendar));
        calendar_button.set_popover(Some(&calendar_popover));

        // Select current day when popover is shown
        calendar_popover.connect_show(clone!(
            #[weak]
            calendar,
            move |_| {
                let now = glib::DateTime::now_local().unwrap();
                calendar.select_day(&now);
            }
        ));

        calendar_button
    }

    fn create_date_label() -> Label {
        let date_output = std::process::Command::new("date")
            .arg("+%d.%m.%Y")
            .output()
            .ok();
        let date_str = if let Some(output) = date_output {
            String::from_utf8(output.stdout)
                .unwrap_or_default()
                .trim()
                .to_string()
        } else {
            String::new()
        };

        let date_label = Label::new(Some(&date_str));
        date_label.set_hexpand(true);

        // Update date every hour
        glib::timeout_add_seconds_local(
            3600,
            clone!(
                #[weak]
                date_label,
                #[upgrade_or]
                glib::ControlFlow::Break,
                move || {
                    let output = std::process::Command::new("date")
                        .arg("+%d.%m.%Y")
                        .output()
                        .ok();
                    if let Some(output) = output {
                        if let Ok(date_str) = String::from_utf8(output.stdout) {
                            date_label.set_label(date_str.trim());
                        }
                    }
                    glib::ControlFlow::Continue
                }
            ),
        );

        date_label
    }

    fn create_second_row(popover: &Popover) -> Box {
        let second_row = Box::new(Orientation::Horizontal, 0);

        // Left: System Info, Clients
        let system_info_button = Self::create_system_info_button();
        second_row.append(&system_info_button);

        let clients_button = Self::create_clients_button(popover);
        second_row.append(&clients_button);

        // Add spacer
        let spacer = Box::new(Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        second_row.append(&spacer);

        // Right: Logout, Reboot
        let logout_button = gtk4::Button::from_icon_name("system-log-out-symbolic");
        logout_button.set_tooltip_text(Some("Logout"));
        logout_button.connect_clicked(|_| {
            ConfirmationDialog::new(
                "Logout",
                "This will exit Hyprland and end your session",
                "hyprshutdown",
            );
        });
        second_row.append(&logout_button);

        let reboot_button = gtk4::Button::from_icon_name("system-reboot-symbolic");
        reboot_button.set_tooltip_text(Some("Reboot"));
        reboot_button.connect_clicked(|_| {
            ConfirmationDialog::new(
                "Reboot",
                "This will reboot your system",
                "hyprshutdown -t 'Restarting...' --post-cmd 'reboot'",
            );
        });
        second_row.append(&reboot_button);

        second_row
    }

    fn create_close_others_closure(
        revealers: Vec<Rc<Revealer>>,
        slider_cells: Vec<Rc<std::cell::OnceCell<Rc<Slider>>>>,
    ) -> impl Fn() {
        move || {
            for revealer in &revealers {
                revealer.set_reveal_child(false);
            }
            for slider_cell in &slider_cells {
                if let Some(slider) = slider_cell.get() {
                    slider.set_expanded(false);
                }
            }
        }
    }

    fn create_sliders(
        toggles: &Rc<Toggles>,
        keyboard_revealer: &Rc<Revealer>,
        audio_revealer: &Rc<Revealer>,
        mic_revealer: &Rc<Revealer>,
        brightness_slider_cell: &Rc<std::cell::OnceCell<Rc<Slider>>>,
        audio_slider_cell: &Rc<std::cell::OnceCell<Rc<Slider>>>,
        mic_slider_cell: &Rc<std::cell::OnceCell<Rc<Slider>>>,
    ) -> Box {
        let container = Box::new(Orientation::Vertical, 0);
        let audio = Audio::instance();
        let brightness = Brightness::instance();

        // Helper closures to close other slider expanders (excluding current)
        let close_others_for_mic = Self::create_close_others_closure(
            vec![audio_revealer.clone(), keyboard_revealer.clone()],
            vec![audio_slider_cell.clone(), brightness_slider_cell.clone()],
        );

        // Mic slider with expander
        let mic_slider = Rc::new(Slider::new_with_expander(
            &audio.mic_icon_name(),
            audio.mic_volume(),
            0.0,
            1.0,
            clone!(
                #[strong]
                audio,
                move |value| {
                    audio.set_mic_volume(value);
                }
            ),
            Some(clone!(
                #[strong]
                audio,
                move || {
                    audio.set_mic_muted(!audio.mic_muted());
                }
            )),
            Some(clone!(
                #[strong]
                mic_revealer,
                #[strong]
                toggles,
                move |is_expanded| {
                    if is_expanded {
                        toggles.close_all_expanders();
                        close_others_for_mic();
                    }
                    mic_revealer.set_reveal_child(is_expanded);
                }
            )),
        ));

        audio.connect_notify_local(
            Some("mic-volume"),
            clone!(
                #[strong]
                mic_slider,
                move |audio, _| {
                    mic_slider.set_value(audio.mic_volume());
                    mic_slider.set_icon_name(&audio.mic_icon_name());
                }
            ),
        );

        audio.connect_notify_local(
            Some("mic-muted"),
            clone!(
                #[strong]
                mic_slider,
                move |audio, _| {
                    mic_slider.set_icon_name(&audio.mic_icon_name());
                }
            ),
        );

        audio.connect_notify_local(
            Some("source-name"),
            clone!(
                #[strong]
                mic_slider,
                move |audio, _| {
                    mic_slider.set_icon_name(&audio.mic_icon_name());
                }
            ),
        );

        let _ = mic_slider_cell.set(mic_slider.clone());
        container.append(mic_slider.widget());

        // Add mic devices content to revealer with styled wrapper
        mic_revealer.set_reveal_child(false);
        let mic_content = MicDevicesContent::new();
        let mic_wrapper = Box::new(Orientation::Vertical, 0);
        mic_wrapper.add_css_class("RevealerContent");
        mic_wrapper.append(mic_content.widget());
        mic_revealer.set_child(Some(&mic_wrapper));
        container.append(mic_revealer.as_ref());

        // Speaker slider with expander
        let close_others_for_audio = Self::create_close_others_closure(
            vec![mic_revealer.clone(), keyboard_revealer.clone()],
            vec![mic_slider_cell.clone(), brightness_slider_cell.clone()],
        );
        let speaker_slider = Rc::new(Slider::new_with_expander(
            &audio.icon_name(),
            audio.volume(),
            0.0,
            1.5,
            clone!(
                #[strong]
                audio,
                move |value| {
                    audio.set_volume(value);
                }
            ),
            Some(clone!(
                #[strong]
                audio,
                move || {
                    audio.set_muted(!audio.muted());
                }
            )),
            Some(clone!(
                #[strong]
                audio_revealer,
                #[strong]
                toggles,
                move |is_expanded| {
                    if is_expanded {
                        toggles.close_all_expanders();
                        close_others_for_audio();
                    }
                    audio_revealer.set_reveal_child(is_expanded);
                }
            )),
        ));

        audio.connect_notify_local(
            Some("volume"),
            clone!(
                #[strong]
                speaker_slider,
                move |audio, _| {
                    speaker_slider.set_value(audio.volume());
                    speaker_slider.set_icon_name(&audio.icon_name());
                }
            ),
        );

        audio.connect_notify_local(
            Some("muted"),
            clone!(
                #[strong]
                speaker_slider,
                move |audio, _| {
                    speaker_slider.set_icon_name(&audio.icon_name());
                }
            ),
        );

        audio.connect_notify_local(
            Some("sink-name"),
            clone!(
                #[strong]
                speaker_slider,
                move |audio, _| {
                    speaker_slider.set_icon_name(&audio.icon_name());
                }
            ),
        );

        let _ = audio_slider_cell.set(speaker_slider.clone());
        container.append(speaker_slider.widget());

        // Add audio devices content to revealer with styled wrapper
        audio_revealer.set_reveal_child(false);
        let audio_content = AudioDevicesContent::new();
        let audio_wrapper = Box::new(Orientation::Vertical, 0);
        audio_wrapper.add_css_class("RevealerContent");
        audio_wrapper.append(audio_content.widget());
        audio_revealer.set_child(Some(&audio_wrapper));
        container.append(audio_revealer.as_ref());

        // Brightness slider with expander
        keyboard_revealer.set_reveal_child(false);
        let close_others_for_brightness = Self::create_close_others_closure(
            vec![audio_revealer.clone(), mic_revealer.clone()],
            vec![audio_slider_cell.clone(), mic_slider_cell.clone()],
        );
        let brightness_slider = Rc::new(Slider::new_with_expander(
            &brightness.icon_name(),
            brightness.screen(),
            0.0,
            1.0,
            clone!(
                #[strong]
                brightness,
                move |value| {
                    brightness.set_screen(value);
                }
            ),
            Some(clone!(
                #[strong]
                brightness,
                move || {
                    brightness.toggle();
                }
            )),
            Some(clone!(
                #[strong]
                keyboard_revealer,
                #[strong]
                toggles,
                move |is_expanded| {
                    if is_expanded {
                        toggles.close_all_expanders();
                        close_others_for_brightness();
                    }
                    keyboard_revealer.set_reveal_child(is_expanded);
                }
            )),
        ));

        brightness.connect_notify_local(
            Some("screen"),
            clone!(
                #[strong]
                brightness_slider,
                move |brightness, _| {
                    brightness_slider.set_value(brightness.screen());
                    brightness_slider.set_icon_name(&brightness.icon_name());
                }
            ),
        );

        // Store slider in OnceCell
        let _ = brightness_slider_cell.set(brightness_slider.clone());

        container.append(brightness_slider.widget());

        // Add keyboard backlight content to revealer with styled wrapper
        let keyboard_content = KeyboardBacklightContent::new();
        let content_wrapper = Box::new(Orientation::Vertical, 0);
        content_wrapper.add_css_class("RevealerContent");
        content_wrapper.append(keyboard_content.widget());
        keyboard_revealer.set_child(Some(&content_wrapper));
        container.append(keyboard_revealer.as_ref());

        container
    }
}
