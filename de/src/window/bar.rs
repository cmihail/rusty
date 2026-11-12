use glib::clone;
use gtk4::gdk::{Key, ModifierType, Monitor, RGBA};
use gtk4::prelude::*;
use gtk4::{
    Align, ApplicationWindow, ArrowType, Box, EventControllerKey, Image, Label, MenuButton,
    Orientation, Overlay, Popover, PositionType,
};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::service::audio::Audio;
use crate::service::battery::Battery;
use crate::service::bluetooth::Bluetooth;
use crate::service::network::{Network, NetworkState};
use crate::service::notifications::Notifications;
use crate::widget::circular_progress::CircularProgress;
use crate::widget::control_center::ControlCenter;
use crate::window::dock::Dock;

const CIRCULAR_SIZE: i32 = 17;
const CIRCULAR_IMAGE_SIZE: i32 = 11;
// The dock's app icon size, and its item spacing plus the inset on both sides of an icon.
const DOCK_STATUS_SIZE: i32 = 32;
const DOCK_STATUS_SPACING: i32 = 16;
const BAR_SPACING: i32 = 4;
// One pixel over CIRCULAR_SIZE so the circle and the digits get even gaps in the same height.
const BAR_BATTERY_SIZE: i32 = 18;

const BAR_ALERT_CLASSES: [&str; 2] = ["Offline", "Notified"];

/// The bar's border class, if any. Being offline outranks open notifications.
pub fn bar_alert_class(online: bool, notifications: usize) -> Option<&'static str> {
    if !online {
        Some("Offline")
    } else if notifications > 0 {
        Some("Notified")
    } else {
        None
    }
}

type ProgressCalculator = Rc<RefCell<Option<std::boxed::Box<dyn Fn()>>>>;

static BAR_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

pub struct Bar {
    _window: ApplicationWindow,
    _monitor_info: String,
    _connector: String,
    _id: u64,
    _popover: Popover,
    _bar_button: MenuButton,
    _control_center: Rc<ControlCenter>,
    _dock: Rc<Dock>,
}

impl Bar {
    #[allow(dead_code)]
    pub fn monitor_info(&self) -> &str {
        &self._monitor_info
    }

    pub fn connector(&self) -> &str {
        &self._connector
    }

    pub fn toggle_control_center(&self) {
        if self._popover.is_visible() {
            self._popover.popdown();
        } else {
            self._bar_button.popup();
        }
    }

    pub fn new(app: &gtk4::Application, monitor: &Monitor) -> Self {
        let bar_id = BAR_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        let monitor_connector = monitor
            .connector()
            .map(|s| s.to_string())
            .unwrap_or_default();
        let monitor_info = format!("id={}, connector='{}'", bar_id, monitor_connector);

        let window = Self::create_window(app, monitor);

        // Create bar button with contents - use MenuButton instead of Button
        let bar_button = MenuButton::new();
        bar_button.add_css_class("Bar");

        let bar_contents = Self::create_bar_contents();
        bar_button.set_child(Some(&bar_contents));

        // Create popover with ControlCenter
        let popover = Popover::new();
        popover.set_position(PositionType::Bottom);
        let control_center = Rc::new(ControlCenter::new(&popover));
        popover.set_child(Some(control_center.widget()));
        bar_button.set_popover(Some(&popover));

        let dock_button = Self::create_dock_status();
        Self::share_popover(&popover, &bar_button, &dock_button);
        let dock = Rc::new(Dock::new(app, monitor, dock_button.clone().upcast()));

        Self::connect_popover_visibility(&popover, &window, &dock_button, &control_center, &dock);

        // Add keyboard controller to popover
        popover.add_controller(Self::create_popover_key_controller(&popover));

        Self::fill_window(&window, &bar_button);

        Self {
            _window: window,
            _monitor_info: monitor_info,
            _connector: monitor_connector,
            _id: bar_id,
            _popover: popover.clone(),
            _bar_button: bar_button,
            _control_center: control_center,
            _dock: dock,
        }
    }

    fn create_window(app: &gtk4::Application, monitor: &Monitor) -> ApplicationWindow {
        let window = ApplicationWindow::builder()
            .application(app)
            .decorated(false)
            .build();

        window.init_layer_shell();
        window.set_layer(Layer::Top);
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Right, true);
        window.set_namespace(Some("Bar"));
        window.add_css_class("Bar");
        window.set_monitor(Some(monitor));
        window.set_keyboard_mode(KeyboardMode::None);
        window
    }

    fn fill_window(window: &ApplicationWindow, bar_button: &MenuButton) {
        // Main container overlay
        let main_overlay = Overlay::new();

        // Box for spacing overlay (mimics .Overlay padding)
        let overlay_box = Box::new(Orientation::Horizontal, 0);
        overlay_box.add_css_class("Overlay");
        overlay_box.append(bar_button);

        main_overlay.set_child(Some(&overlay_box));

        // Setup tooltip
        Self::setup_tooltip(bar_button);
        Self::connect_border_alert(bar_button);

        window.set_child(Some(&main_overlay));

        // Realize the window to ensure widget hierarchy is fully initialized
        // before making it visible, preventing GTK assertion failures during
        // rapid window recreation (e.g., after resume from suspend)
        WidgetExt::realize(window);
        window.set_visible(true);
    }

    fn connect_popover_visibility(
        popover: &Popover,
        window: &ApplicationWindow,
        dock_button: &MenuButton,
        control_center: &Rc<ControlCenter>,
        dock: &Rc<Dock>,
    ) {
        // Setup keyboard mode based on popover visibility and refresh languages
        popover.connect_show(clone!(
            #[weak]
            window,
            #[weak]
            dock_button,
            #[strong]
            control_center,
            #[strong]
            dock,
            move |popover| {
                if popover.parent().as_ref() == Some(dock_button.upcast_ref()) {
                    dock.hold(true);
                } else {
                    window.set_keyboard_mode(KeyboardMode::OnDemand);
                }
                // Refresh language list when opening control center
                crate::service::fcitx::Fcitx::instance().refresh_languages();
                // Focus notifications toggle (or expander if visible) for keyboard navigation
                control_center.focus_notifications_toggle();
            }
        ));

        popover.connect_hide(clone!(
            #[weak]
            window,
            #[strong]
            dock,
            move |_| {
                window.set_keyboard_mode(KeyboardMode::None);
                dock.hold(false);
            }
        ));
    }

    fn create_popover_key_controller(popover: &Popover) -> EventControllerKey {
        let key_controller = EventControllerKey::new();
        key_controller.connect_key_pressed(clone!(
            #[weak]
            popover,
            #[upgrade_or]
            glib::Propagation::Proceed,
            move |controller, key, _, modifiers| {
                let has_ctrl = modifiers.contains(ModifierType::CONTROL_MASK);
                let has_shift = modifiers.contains(ModifierType::SHIFT_MASK);

                // Check if focus is in a text input widget
                if Self::is_text_input_focused(controller) {
                    return glib::Propagation::Proceed;
                }

                // Check if Ctrl or Shift modifiers are active
                let has_modifiers = has_ctrl || has_shift;

                // Close popover on Escape or non-navigation/non-modifier key
                // (unless modifiers active)
                if key == Key::Escape
                    || (!Self::is_navigation_key(key)
                        && !Self::is_modifier_key(key)
                        && !has_modifiers)
                {
                    popover.popdown();
                    glib::Propagation::Stop
                } else {
                    glib::Propagation::Proceed
                }
            }
        ));
        key_controller
    }

    fn is_text_input_focused(controller: &EventControllerKey) -> bool {
        let Some(focus_widget) = controller.widget().and_then(|w| w.focus_child()) else {
            return false;
        };
        let type_name = focus_widget.type_().name();
        type_name == "GtkEntry"
            || type_name == "GtkText"
            || type_name == "GtkTextView"
            || type_name == "GtkPasswordEntry"
            || type_name == "GtkSearchEntry"
    }

    // Define navigation keys that should NOT close the popover
    fn is_navigation_key(key: Key) -> bool {
        matches!(
            key,
            Key::Tab
                | Key::ISO_Left_Tab
                | Key::Up
                | Key::Down
                | Key::Left
                | Key::Right
                | Key::Return
                | Key::KP_Enter
                | Key::space
                | Key::Home
                | Key::End
        )
    }

    // Define modifier keys that should NOT close the popover
    fn is_modifier_key(key: Key) -> bool {
        matches!(
            key,
            Key::Control_L
                | Key::Control_R
                | Key::Shift_L
                | Key::Shift_R
                | Key::Alt_L
                | Key::Alt_R
                | Key::Super_L
                | Key::Super_R
                | Key::Meta_L
                | Key::Meta_R
        )
    }

    fn create_bar_contents() -> Box {
        let container = Box::new(Orientation::Horizontal, BAR_SPACING);
        container.set_halign(Align::Center);

        let time_label = Label::new(Some("--:--"));
        time_label.add_css_class("TimeLabel");
        container.append(&time_label);

        let battery = Self::create_battery_state(BAR_BATTERY_SIZE);
        battery.set_valign(Align::Center);
        battery.add_css_class("BarBattery");
        container.append(&battery);

        Self::setup_time_updates(&time_label);

        container
    }

    fn create_dock_status() -> MenuButton {
        let status = Box::new(Orientation::Vertical, DOCK_STATUS_SPACING);
        status.append(&Self::create_dock_notifications_count());
        for icon in [
            Self::create_microphone_state(DOCK_STATUS_SIZE),
            Self::create_speaker_state(DOCK_STATUS_SIZE),
            Self::create_bluetooth_state(DOCK_STATUS_SIZE),
            Self::create_network_state(DOCK_STATUS_SIZE),
            Self::create_battery_state(DOCK_STATUS_SIZE),
        ] {
            icon.set_halign(Align::Center);
            status.append(&icon);
        }

        let button = MenuButton::new();
        button.add_css_class("DockStatus");
        button.set_halign(Align::Center);
        // set_popover takes the popover's position from this, overriding set_position.
        button.set_direction(ArrowType::Left);
        button.set_child(Some(&status));
        button
    }

    /// One control center per monitor, moved to whichever button is clicked so it opens there.
    fn share_popover(popover: &Popover, bar_button: &MenuButton, dock_button: &MenuButton) {
        bar_button.set_create_popup_func(clone!(
            #[weak]
            popover,
            #[weak]
            dock_button,
            move |bar_button| {
                dock_button.set_popover(None::<&Popover>);
                popover.set_position(PositionType::Bottom);
                bar_button.set_popover(Some(&popover));
            }
        ));
        dock_button.set_create_popup_func(clone!(
            #[weak]
            popover,
            #[weak]
            bar_button,
            move |dock_button| {
                bar_button.set_popover(None::<&Popover>);
                popover.set_position(PositionType::Left);
                dock_button.set_popover(Some(&popover));
            }
        ));
    }

    fn line_width(size: i32) -> f64 {
        size as f64 / CIRCULAR_SIZE as f64
    }

    fn setup_time_updates(time_label: &Label) {
        let update_time = || {
            let output = std::process::Command::new("date")
                .arg("+%H:%M")
                .output()
                .ok();
            if let Some(output) = output {
                if let Ok(time_str) = String::from_utf8(output.stdout) {
                    return Some(time_str);
                }
            }
            None
        };

        if let Some(time_str) = update_time() {
            time_label.set_label(time_str.trim());
        }

        glib::timeout_add_seconds_local(
            10,
            clone!(
                #[weak]
                time_label,
                #[upgrade_or]
                glib::ControlFlow::Break,
                move || {
                    if let Some(time_str) = update_time() {
                        time_label.set_label(time_str.trim());
                    }
                    glib::ControlFlow::Continue
                }
            ),
        );
    }

    fn create_network_state(size: i32) -> Overlay {
        let overlay = Overlay::new();
        overlay.set_halign(Align::Start);

        let circular = CircularProgress::new(size, Self::line_width(size), true);
        overlay.set_child(Some(circular.widget()));

        let image = Image::from_icon_name("network-offline-symbolic");
        image.set_pixel_size(size * CIRCULAR_IMAGE_SIZE / CIRCULAR_SIZE);
        overlay.add_overlay(&image);

        let network = Network::instance();
        let progress = Rc::new(Cell::new(0.0));
        let timeout_id = Rc::new(RefCell::new(None::<glib::SourceId>));
        let calculate_progress_ref: ProgressCalculator = Rc::new(RefCell::new(None));

        Self::setup_network_progress_calculator(
            &calculate_progress_ref,
            &network,
            &circular,
            &image,
            &progress,
            &timeout_id,
        );

        Self::connect_network_signals(&network, &calculate_progress_ref);

        if let Some(func) = calculate_progress_ref.borrow().as_ref() {
            func();
        }

        overlay
    }

    fn setup_network_progress_calculator(
        calculate_progress_ref: &ProgressCalculator,
        network: &Network,
        circular: &CircularProgress,
        image: &Image,
        progress: &Rc<Cell<f64>>,
        timeout_id: &Rc<RefCell<Option<glib::SourceId>>>,
    ) {
        let network = network.clone();
        let circular = circular.clone();
        let image = image.clone();
        let progress = progress.clone();
        let timeout_id = timeout_id.clone();
        let calculate_progress_weak = Rc::downgrade(calculate_progress_ref);

        let closure = move || {
            if let Some(id) = timeout_id.borrow_mut().take() {
                id.remove();
            }

            let state = network.state();
            Self::update_network_css_classes(&image, state);

            let icon = network.icon_name();
            image.set_icon_name(Some(&icon));

            Self::calculate_network_progress(
                state,
                &network,
                &circular,
                &progress,
                &timeout_id,
                &calculate_progress_weak,
            );

            circular.set_percentage(progress.get());
            let color = Self::color_with_alpha(&image);
            circular.set_color(color);
        };

        *calculate_progress_ref.borrow_mut() = Some(std::boxed::Box::new(closure));
    }

    fn update_network_css_classes(image: &Image, state: crate::service::network::NetworkState) {
        use crate::service::network::NetworkState;

        image.remove_css_class("Warning");
        image.remove_css_class("Critical");

        if matches!(
            state,
            NetworkState::ConnectedSite | NetworkState::Connecting | NetworkState::Disconnecting
        ) {
            image.add_css_class("Warning");
        }
    }

    #[allow(clippy::type_complexity)]
    fn calculate_network_progress(
        state: crate::service::network::NetworkState,
        network: &Network,
        circular: &CircularProgress,
        progress: &Rc<Cell<f64>>,
        timeout_id: &Rc<RefCell<Option<glib::SourceId>>>,
        calculate_progress_weak: &std::rc::Weak<RefCell<Option<std::boxed::Box<dyn Fn()>>>>,
    ) {
        use crate::service::network::NetworkState;

        match state {
            NetworkState::ConnectedGlobal => {
                let progress_value = match network.primary() {
                    crate::service::network::NetworkType::Wifi => {
                        network.wifi().strength() as f64 / 100.0
                    }
                    _ => 1.0,
                };
                progress.set(progress_value);
            }
            NetworkState::ConnectedSite | NetworkState::Connecting => {
                Self::animate_network_progress_upward(
                    network,
                    circular,
                    progress,
                    timeout_id,
                    calculate_progress_weak,
                );
            }
            NetworkState::Disconnecting => {
                Self::animate_network_progress_downward(
                    network,
                    circular,
                    progress,
                    timeout_id,
                    calculate_progress_weak,
                );
            }
            NetworkState::ConnectedLocal
            | NetworkState::Disconnected
            | NetworkState::Unknown
            | NetworkState::Asleep => {
                progress.set(0.0);
            }
        }
    }

    #[allow(clippy::type_complexity)]
    fn animate_network_progress_upward(
        network: &Network,
        circular: &CircularProgress,
        progress: &Rc<Cell<f64>>,
        timeout_id: &Rc<RefCell<Option<glib::SourceId>>>,
        calculate_progress_weak: &std::rc::Weak<RefCell<Option<std::boxed::Box<dyn Fn()>>>>,
    ) {
        use crate::service::network::NetworkState;

        const NETWORK_PROGRESS_TIMEOUT: u64 = 300;
        const NETWORK_PROGRESS_STEP: f64 = 0.1;

        let network = network.clone();
        let circular = circular.clone();
        let progress = progress.clone();
        let calculate_progress_weak = calculate_progress_weak.clone();

        let id = glib::timeout_add_local(
            std::time::Duration::from_millis(NETWORK_PROGRESS_TIMEOUT),
            move || {
                let current = progress.get();
                let new_progress = if current > 1.0 - NETWORK_PROGRESS_STEP {
                    0.0
                } else {
                    current + NETWORK_PROGRESS_STEP
                };
                progress.set(new_progress);
                circular.set_percentage(new_progress);

                let state = network.state();
                if matches!(
                    state,
                    NetworkState::ConnectedSite | NetworkState::Connecting
                ) {
                    return glib::ControlFlow::Continue;
                }

                if let Some(rc) = calculate_progress_weak.upgrade() {
                    if let Some(func) = rc.borrow().as_ref() {
                        func();
                    }
                }
                glib::ControlFlow::Break
            },
        );
        *timeout_id.borrow_mut() = Some(id);
    }

    #[allow(clippy::type_complexity)]
    fn animate_network_progress_downward(
        network: &Network,
        circular: &CircularProgress,
        progress: &Rc<Cell<f64>>,
        timeout_id: &Rc<RefCell<Option<glib::SourceId>>>,
        calculate_progress_weak: &std::rc::Weak<RefCell<Option<std::boxed::Box<dyn Fn()>>>>,
    ) {
        use crate::service::network::NetworkState;

        const NETWORK_PROGRESS_TIMEOUT: u64 = 300;
        const NETWORK_PROGRESS_STEP: f64 = 0.1;

        let network = network.clone();
        let circular = circular.clone();
        let progress = progress.clone();
        let calculate_progress_weak = calculate_progress_weak.clone();

        let id = glib::timeout_add_local(
            std::time::Duration::from_millis(NETWORK_PROGRESS_TIMEOUT),
            move || {
                let current = progress.get();
                let new_progress = if current < NETWORK_PROGRESS_STEP {
                    1.0
                } else {
                    current - NETWORK_PROGRESS_STEP
                };
                progress.set(new_progress);
                circular.set_percentage(new_progress);

                let state = network.state();
                if matches!(state, NetworkState::Disconnecting) {
                    return glib::ControlFlow::Continue;
                }

                if let Some(rc) = calculate_progress_weak.upgrade() {
                    if let Some(func) = rc.borrow().as_ref() {
                        func();
                    }
                }
                glib::ControlFlow::Break
            },
        );
        *timeout_id.borrow_mut() = Some(id);
    }

    fn connect_network_signals(network: &Network, calculate_progress_ref: &ProgressCalculator) {
        network.connect_notify_local(Some("state"), {
            let calculate_progress_ref = calculate_progress_ref.clone();
            move |_, _| {
                if let Some(func) = calculate_progress_ref.borrow().as_ref() {
                    func();
                }
            }
        });

        network.connect_notify_local(Some("primary"), {
            let calculate_progress_ref = calculate_progress_ref.clone();
            move |_, _| {
                if let Some(func) = calculate_progress_ref.borrow().as_ref() {
                    func();
                }
            }
        });

        network.wifi().connect_notify_local(Some("strength"), {
            let calculate_progress_ref = calculate_progress_ref.clone();
            move |_, _| {
                if let Some(func) = calculate_progress_ref.borrow().as_ref() {
                    func();
                }
            }
        });

        network.connect_notify_local(Some("icon-name"), {
            let calculate_progress_ref = calculate_progress_ref.clone();
            move |_, _| {
                if let Some(func) = calculate_progress_ref.borrow().as_ref() {
                    func();
                }
            }
        });
    }

    fn update_bluetooth_widget(bluetooth: &Bluetooth, image: &Image, circular: &CircularProgress) {
        let enabled = bluetooth.enabled();
        let has_missing = bluetooth.has_missing_expected_devices();
        let lowest_battery = bluetooth.lowest_connected_expected_battery();

        // Update icon
        let icon = bluetooth.icon_name();
        image.set_icon_name(Some(&icon));

        // Set CSS classes with priority: Critical > Warning > Active > Normal
        image.remove_css_class("Warning");
        image.remove_css_class("Critical");
        image.remove_css_class("Active");

        if !enabled {
            image.add_css_class("Critical");
        } else if let Some(battery) = lowest_battery {
            // Apply battery level colors (same thresholds as switch_list)
            if battery < 20 {
                image.add_css_class("Critical");
            } else if battery < 40 {
                image.add_css_class("Warning");
            } else if battery < 60 {
                image.add_css_class("Active");
            }
        } else if has_missing {
            image.add_css_class("Warning");
        }

        // Update progress: connected / expected devices (100% if no expected devices)
        circular.set_percentage(bluetooth.expected_devices_percentage());

        // Update color
        let color = Self::color_with_alpha(image);
        circular.set_color(color);
    }

    fn create_bluetooth_state(size: i32) -> Overlay {
        let overlay = Overlay::new();
        overlay.set_halign(Align::Start);

        let circular = CircularProgress::new(size, Self::line_width(size), true);
        overlay.set_child(Some(circular.widget()));

        let image = Image::from_icon_name("bluetooth-active-symbolic");
        image.set_pixel_size(size * CIRCULAR_IMAGE_SIZE / CIRCULAR_SIZE);
        overlay.add_overlay(&image);

        let bluetooth = Bluetooth::instance();

        let update_bluetooth = {
            let bluetooth = bluetooth.clone();
            let circular = circular.clone();
            let image = image.clone();
            move || Self::update_bluetooth_widget(&bluetooth, &image, &circular)
        };

        bluetooth.connect_notify_local(Some("enabled"), {
            let update_bluetooth = update_bluetooth.clone();
            move |_, _| update_bluetooth()
        });

        bluetooth.connect_notify_local(Some("missing-expected-devices"), {
            let update_bluetooth = update_bluetooth.clone();
            move |_, _| update_bluetooth()
        });

        bluetooth.connect_notify_local(Some("devices"), {
            let update_bluetooth = update_bluetooth.clone();
            move |_, _| update_bluetooth()
        });

        update_bluetooth();

        overlay
    }

    fn update_microphone_widget(audio: &Audio, image: &Image, circular: &CircularProgress) {
        let mic_volume = audio.mic_volume();
        let mic_muted = audio.mic_muted();

        // Update icon
        let icon = audio.mic_icon_name();
        image.set_icon_name(Some(&icon));

        // Set CSS classes
        image.remove_css_class("Warning");
        image.remove_css_class("Critical");
        if mic_muted {
            image.add_css_class("Critical");
        }

        // Update progress
        circular.set_percentage(mic_volume);

        // Update color
        let color = Self::color_with_alpha(image);
        circular.set_color(color);
    }

    fn create_microphone_state(size: i32) -> Overlay {
        let overlay = Overlay::new();
        overlay.set_halign(Align::Start);

        let circular = CircularProgress::new(size, Self::line_width(size), true);
        overlay.set_child(Some(circular.widget()));

        let image = Image::from_icon_name("microphone-sensitivity-high-symbolic");
        image.set_pixel_size(size * CIRCULAR_IMAGE_SIZE / CIRCULAR_SIZE);
        overlay.add_overlay(&image);

        let audio = Audio::instance();

        let update_microphone = {
            let audio = audio.clone();
            let circular = circular.clone();
            let image = image.clone();
            move || Self::update_microphone_widget(&audio, &image, &circular)
        };

        for property in ["mic-volume", "mic-muted", "source-name"] {
            let update_microphone = update_microphone.clone();
            audio.connect_notify_local(Some(property), move |_, _| update_microphone());
        }

        update_microphone();

        overlay
    }

    fn update_speaker_widget(audio: &Audio, image: &Image, circular: &CircularProgress) {
        let volume = audio.volume();
        let muted = audio.muted();

        // Update icon
        let icon = audio.icon_name();
        image.set_icon_name(Some(&icon));

        // Set CSS classes
        image.remove_css_class("Warning");
        image.remove_css_class("Critical");
        if muted {
            image.add_css_class("Critical");
        } else if volume > 1.0 {
            image.add_css_class("Warning");
        }

        // Update progress (volume / 1.5 like in AGS)
        circular.set_percentage(volume / 1.5);

        // Update color
        let color = Self::color_with_alpha(image);
        circular.set_color(color);
    }

    fn create_speaker_state(size: i32) -> Overlay {
        let overlay = Overlay::new();
        overlay.set_halign(Align::Start);

        let circular = CircularProgress::new(size, Self::line_width(size), true);
        overlay.set_child(Some(circular.widget()));

        let image = Image::from_icon_name("audio-speakers-symbolic");
        image.set_pixel_size(size * CIRCULAR_IMAGE_SIZE / CIRCULAR_SIZE);
        overlay.add_overlay(&image);

        let audio = Audio::instance();

        let update_speaker = {
            let audio = audio.clone();
            let circular = circular.clone();
            let image = image.clone();
            move || Self::update_speaker_widget(&audio, &image, &circular)
        };

        for property in ["volume", "muted", "sink-name"] {
            let update_speaker = update_speaker.clone();
            audio.connect_notify_local(Some(property), move |_, _| update_speaker());
        }

        update_speaker();

        overlay
    }

    fn create_battery_state(size: i32) -> Overlay {
        let overlay = Overlay::new();
        overlay.set_halign(Align::End);

        let circular = CircularProgress::new(size, Self::line_width(size), true);
        overlay.set_child(Some(circular.widget()));

        let image = Image::from_icon_name("battery-symbolic");
        image.set_pixel_size(size * CIRCULAR_IMAGE_SIZE / CIRCULAR_SIZE);
        overlay.add_overlay(&image);

        let battery = Battery::instance();
        for property in ["percentage", "state"] {
            battery.connect_notify_local(
                Some(property),
                clone!(
                    #[weak]
                    image,
                    #[strong]
                    circular,
                    move |battery, _| Self::update_battery_widget(battery, &image, &circular)
                ),
            );
        }
        Self::update_battery_widget(&battery, &image, &circular);

        overlay
    }

    fn update_battery_widget(battery: &Battery, image: &Image, circular: &CircularProgress) {
        use crate::service::battery::BatteryState;

        let percentage = battery.percentage();
        let state = battery.state();
        image.set_icon_name(Some(&battery.icon_name()));

        image.remove_css_class("Warning");
        image.remove_css_class("Critical");
        image.remove_css_class("Active");
        if state != BatteryState::Charging && state != BatteryState::FullyCharged {
            if percentage >= 0.60 {
                // Normal, no class
            } else if percentage >= 0.40 {
                image.add_css_class("Active");
            } else if percentage >= 0.20 {
                image.add_css_class("Warning");
            } else {
                image.add_css_class("Critical");
            }
        }

        circular.set_percentage(percentage);
        circular.set_color(Self::color_with_alpha(image));
    }

    fn create_dock_notifications_count() -> Label {
        let label = Label::new(Some("0"));
        label.add_css_class("DockNotificationsCount");
        label.set_valign(Align::Center);

        Self::connect_notification_count(clone!(
            #[weak]
            label,
            move |count| {
                label.set_label(&Self::notification_count_text(count));
                label.set_visible(count > 0);
            }
        ));

        label
    }

    fn notification_count_text(count: usize) -> String {
        if count > 9 {
            "+".to_string()
        } else {
            count.to_string()
        }
    }

    /// Calls `update` now and on every change with the count shown to the user, 0 under DND.
    /// Colors the bar's border like the dock's hint: warning for notifications, error offline.
    fn connect_border_alert(bar_button: &MenuButton) {
        let weak = bar_button.downgrade();
        let count = Rc::new(Cell::new(0));
        let update = Rc::new({
            let count = count.clone();
            move || {
                let Some(bar_button) = weak.upgrade() else {
                    return;
                };
                let online = Network::instance().state() == NetworkState::ConnectedGlobal;
                for class in BAR_ALERT_CLASSES {
                    bar_button.remove_css_class(class);
                }
                if let Some(class) = bar_alert_class(online, count.get()) {
                    bar_button.add_css_class(class);
                }
            }
        });

        Network::instance().connect_notify_local(Some("state"), {
            let update = update.clone();
            move |_, _| update()
        });
        Self::connect_notification_count(move |notifications| {
            count.set(notifications);
            update();
        });
    }

    fn connect_notification_count(update: impl Fn(usize) + 'static) {
        let notifications = Notifications::instance();

        let update_count = Rc::new({
            let notifications = notifications.clone();
            move || {
                let count = if notifications.dont_disturb() {
                    0
                } else {
                    notifications.notifications().len()
                };
                update(count);
            }
        });

        for signal in ["notified", "resolved"] {
            let update_count = update_count.clone();
            notifications.connect_local(signal, false, move |_| {
                update_count();
                None
            });
        }

        notifications.connect_notify_local(Some("dont-disturb"), {
            let update_count = update_count.clone();
            move |_, _| update_count()
        });

        update_count();
    }

    fn setup_tooltip(button: &MenuButton) {
        let battery = Battery::instance();

        let update_tooltip = {
            let battery = battery.clone();
            let button = button.clone();
            move || {
                let percentage = (battery.percentage() * 100.0).round() as i32;
                let mut text = format!("Battery: {}%", percentage);

                let time_to_full = battery.time_to_full();
                if time_to_full != 0 {
                    text.push_str(&format!(
                        "\nCharging: {}",
                        Self::seconds_to_pretty_time(time_to_full)
                    ));
                }

                let time_to_empty = battery.time_to_empty();
                if time_to_empty != 0 {
                    text.push_str(&format!(
                        "\nRemaining: {}",
                        Self::seconds_to_pretty_time(time_to_empty)
                    ));
                }

                button.set_tooltip_text(Some(&text));
            }
        };

        battery.connect_notify_local(Some("percentage"), {
            let update_tooltip = update_tooltip.clone();
            move |_, _| update_tooltip()
        });

        battery.connect_notify_local(Some("time-to-full"), {
            let update_tooltip = update_tooltip.clone();
            move |_, _| update_tooltip()
        });

        battery.connect_notify_local(Some("time-to-empty"), {
            let update_tooltip = update_tooltip.clone();
            move |_, _| update_tooltip()
        });

        update_tooltip();
    }

    fn seconds_to_pretty_time(seconds: i64) -> String {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        format!("{}h:{}m", hours, minutes)
    }

    fn color_with_alpha(widget: &impl WidgetExt) -> RGBA {
        let style_context = widget.style_context();
        let mut color = style_context.color();
        color.set_alpha(color.alpha() / 1.3);
        color
    }
}

impl Drop for Bar {
    fn drop(&mut self) {
        // Hide and close the window before dropping
        self._window.set_visible(false);
        self._window.close();
    }
}
