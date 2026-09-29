use glib::clone;
use gtk4::prelude::*;
use gtk4::{
    Align, ApplicationWindow, Box, Button, Image, Label, LevelBar, Orientation, Revealer,
    RevealerTransitionType,
};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::cell::RefCell;
use std::rc::Rc;

use crate::service::audio::Audio;
use crate::service::brightness::Brightness;
use crate::service::hyprland::Hyprland;
use crate::service::power_profiles::PowerProfiles;

const HIDE_TIMEOUT_MS: u32 = 2000;
const TRANSITION_DURATION_MS: u32 = 500;

pub struct OnScreenDisplay {
    window: ApplicationWindow,
    revealer: Revealer,
    container: Box,
    hide_count: Rc<RefCell<i32>>,
}

impl OnScreenDisplay {
    pub fn new(app: &gtk4::Application) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .decorated(false)
            .build();

        window.init_layer_shell();
        window.set_layer(Layer::Top);
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Right, true);
        window.set_margin(Edge::Top, 5);
        window.set_margin(Edge::Right, 55);
        window.set_namespace(Some("OnScreenDisplay"));
        window.add_css_class("OnScreenDisplay");

        let container = Box::new(Orientation::Horizontal, 10);

        let button = Button::builder().build();
        button.set_child(Some(&container));

        let revealer = Revealer::builder()
            .transition_type(RevealerTransitionType::SlideLeft)
            .transition_duration(TRANSITION_DURATION_MS)
            .reveal_child(false)
            .child(&button)
            .build();

        window.set_child(Some(&revealer));

        let hide_count = Rc::new(RefCell::new(0));

        let osd = Self {
            window: window.clone(),
            revealer: revealer.clone(),
            container: container.clone(),
            hide_count: hide_count.clone(),
        };

        button.connect_clicked(clone!(
            #[weak]
            revealer,
            #[weak]
            window,
            move |_| {
                hide(&revealer, &window, &hide_count);
            }
        ));

        osd.setup_brightness();
        osd.setup_power_profiles();
        osd.setup_audio();
        osd.setup_monitor_switch();

        osd
    }

    fn setup_brightness(&self) {
        let brightness = Brightness::instance();
        let first_notify = Rc::new(RefCell::new(true));

        brightness.connect_notify_local(
            Some("screen"),
            clone!(
                #[weak(rename_to = osd_window)]
                self.window,
                #[weak(rename_to = osd_revealer)]
                self.revealer,
                #[weak(rename_to = osd_container)]
                self.container,
                #[weak(rename_to = osd_hide_count)]
                self.hide_count,
                #[strong]
                first_notify,
                move |brightness, _| {
                    if *first_notify.borrow() {
                        *first_notify.borrow_mut() = false;
                        return;
                    }

                    let value = brightness.screen();
                    let icon = brightness.icon_name();
                    show_osd(
                        &osd_window,
                        &osd_revealer,
                        &osd_container,
                        &osd_hide_count,
                        value,
                        &icon,
                        1.0,
                    );
                }
            ),
        );
    }

    fn setup_power_profiles(&self) {
        let power_profiles = PowerProfiles::instance();
        let first_notify = Rc::new(RefCell::new(true));

        power_profiles.connect_notify_local(
            Some("active-profile"),
            clone!(
                #[weak(rename_to = osd_window)]
                self.window,
                #[weak(rename_to = osd_revealer)]
                self.revealer,
                #[weak(rename_to = osd_container)]
                self.container,
                #[weak(rename_to = osd_hide_count)]
                self.hide_count,
                #[strong]
                first_notify,
                move |power_profiles, _| {
                    if *first_notify.borrow() {
                        *first_notify.borrow_mut() = false;
                        return;
                    }

                    let profile = power_profiles.active_profile();
                    let formatted_profile: String = profile
                        .split('-')
                        .map(|p| {
                            let mut chars = p.chars();
                            match chars.next() {
                                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                                None => String::new(),
                            }
                        })
                        .collect();
                    let icon = power_profiles.icon_name();
                    show_osd_text(
                        &osd_window,
                        &osd_revealer,
                        &osd_container,
                        &osd_hide_count,
                        &formatted_profile,
                        &icon,
                    );
                }
            ),
        );
    }

    fn setup_audio(&self) {
        let audio = Audio::instance();

        audio.connect_notify_local(
            Some("volume"),
            clone!(
                #[weak(rename_to = osd_window)]
                self.window,
                #[weak(rename_to = osd_revealer)]
                self.revealer,
                #[weak(rename_to = osd_container)]
                self.container,
                #[weak(rename_to = osd_hide_count)]
                self.hide_count,
                move |audio, _| {
                    let value = audio.volume();
                    let icon = audio.icon_name();
                    show_osd(
                        &osd_window,
                        &osd_revealer,
                        &osd_container,
                        &osd_hide_count,
                        value,
                        &icon,
                        1.5,
                    );
                }
            ),
        );

        audio.connect_notify_local(
            Some("muted"),
            clone!(
                #[weak(rename_to = osd_window)]
                self.window,
                #[weak(rename_to = osd_revealer)]
                self.revealer,
                #[weak(rename_to = osd_container)]
                self.container,
                #[weak(rename_to = osd_hide_count)]
                self.hide_count,
                move |audio, _| {
                    let value = audio.volume();
                    let icon = audio.icon_name();
                    show_osd(
                        &osd_window,
                        &osd_revealer,
                        &osd_container,
                        &osd_hide_count,
                        value,
                        &icon,
                        1.5,
                    );
                }
            ),
        );

        audio.connect_notify_local(
            Some("sink-name"),
            clone!(
                #[weak(rename_to = osd_window)]
                self.window,
                #[weak(rename_to = osd_revealer)]
                self.revealer,
                #[weak(rename_to = osd_container)]
                self.container,
                #[weak(rename_to = osd_hide_count)]
                self.hide_count,
                move |audio, _| {
                    let value = audio.volume();
                    let icon = audio.icon_name();
                    show_osd(
                        &osd_window,
                        &osd_revealer,
                        &osd_container,
                        &osd_hide_count,
                        value,
                        &icon,
                        1.5,
                    );
                }
            ),
        );

        audio.connect_notify_local(
            Some("mic-volume"),
            clone!(
                #[weak(rename_to = osd_window)]
                self.window,
                #[weak(rename_to = osd_revealer)]
                self.revealer,
                #[weak(rename_to = osd_container)]
                self.container,
                #[weak(rename_to = osd_hide_count)]
                self.hide_count,
                move |audio, _| {
                    let value = audio.mic_volume();
                    let icon = audio.mic_icon_name();
                    show_osd(
                        &osd_window,
                        &osd_revealer,
                        &osd_container,
                        &osd_hide_count,
                        value,
                        &icon,
                        1.0,
                    );
                }
            ),
        );

        audio.connect_notify_local(
            Some("mic-muted"),
            clone!(
                #[weak(rename_to = osd_window)]
                self.window,
                #[weak(rename_to = osd_revealer)]
                self.revealer,
                #[weak(rename_to = osd_container)]
                self.container,
                #[weak(rename_to = osd_hide_count)]
                self.hide_count,
                move |audio, _| {
                    let value = audio.mic_volume();
                    let icon = audio.mic_icon_name();
                    show_osd(
                        &osd_window,
                        &osd_revealer,
                        &osd_container,
                        &osd_hide_count,
                        value,
                        &icon,
                        1.0,
                    );
                }
            ),
        );

        audio.connect_notify_local(
            Some("source-name"),
            clone!(
                #[weak(rename_to = osd_window)]
                self.window,
                #[weak(rename_to = osd_revealer)]
                self.revealer,
                #[weak(rename_to = osd_container)]
                self.container,
                #[weak(rename_to = osd_hide_count)]
                self.hide_count,
                move |audio, _| {
                    let value = audio.mic_volume();
                    let icon = audio.mic_icon_name();
                    show_osd(
                        &osd_window,
                        &osd_revealer,
                        &osd_container,
                        &osd_hide_count,
                        value,
                        &icon,
                        1.0,
                    );
                }
            ),
        );
    }

    fn setup_monitor_switch(&self) {
        if let Some(hyprland) = Hyprland::instance() {
            hyprland.connect_local(
                "event",
                false,
                clone!(
                    #[weak(rename_to = osd_revealer)]
                    self.revealer,
                    #[weak(rename_to = osd_window)]
                    self.window,
                    #[weak(rename_to = osd_hide_count)]
                    self.hide_count,
                    #[upgrade_or]
                    None,
                    move |args| {
                        let event = args[1].get::<String>().ok()?;
                        if event == "focusedmon" {
                            // Immediately hide OSD when switching monitors
                            hide(&osd_revealer, &osd_window, &osd_hide_count);
                        }
                        None
                    }
                ),
            );
        }
    }
}

fn show_osd(
    window: &ApplicationWindow,
    revealer: &Revealer,
    container: &Box,
    hide_count: &Rc<RefCell<i32>>,
    value: f64,
    icon: &str,
    max_value: f64,
) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }

    let image = Image::builder()
        .icon_name(icon)
        .halign(Align::Start)
        .build();

    container.append(&image);

    let level_bar = LevelBar::builder()
        .width_request(100)
        .halign(Align::Center)
        .valign(Align::Center)
        .value(value)
        .max_value(max_value)
        .build();
    container.append(&level_bar);

    let percentage = (value * 100.0).floor() as i32;
    let label = Label::builder()
        .label(format!("{}%", percentage))
        .halign(Align::End)
        .build();
    container.append(&label);

    // Ensure window is realized before making visible
    if !window.is_realized() {
        WidgetExt::realize(window);
    }
    window.set_visible(true);

    revealer.set_reveal_child(true);

    *hide_count.borrow_mut() += 1;

    glib::timeout_add_local_once(
        std::time::Duration::from_millis(HIDE_TIMEOUT_MS as u64),
        clone!(
            #[weak]
            revealer,
            #[weak]
            window,
            #[weak]
            hide_count,
            move || {
                *hide_count.borrow_mut() -= 1;
                if *hide_count.borrow() == 0 {
                    hide(&revealer, &window, &hide_count);
                }
            }
        ),
    );
}

fn show_osd_text(
    window: &ApplicationWindow,
    revealer: &Revealer,
    container: &Box,
    hide_count: &Rc<RefCell<i32>>,
    text: &str,
    icon: &str,
) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }

    let image = Image::builder()
        .icon_name(icon)
        .halign(Align::Start)
        .build();
    container.append(&image);

    let label = Label::builder().label(text).halign(Align::End).build();
    container.append(&label);

    // Ensure window is realized before making visible
    if !window.is_realized() {
        WidgetExt::realize(window);
    }
    window.set_visible(true);

    revealer.set_reveal_child(true);

    *hide_count.borrow_mut() += 1;

    glib::timeout_add_local_once(
        std::time::Duration::from_millis(HIDE_TIMEOUT_MS as u64),
        clone!(
            #[weak]
            revealer,
            #[weak]
            window,
            #[weak]
            hide_count,
            move || {
                *hide_count.borrow_mut() -= 1;
                if *hide_count.borrow() == 0 {
                    hide(&revealer, &window, &hide_count);
                }
            }
        ),
    );
}

fn hide(revealer: &Revealer, window: &ApplicationWindow, _hide_count: &Rc<RefCell<i32>>) {
    if revealer.reveals_child() {
        revealer.set_reveal_child(false);
        window.set_default_size(1, 1);

        glib::timeout_add_local_once(
            std::time::Duration::from_millis(TRANSITION_DURATION_MS as u64),
            clone!(
                #[weak]
                window,
                move || {
                    window.set_visible(false);
                }
            ),
        );
    }
}
