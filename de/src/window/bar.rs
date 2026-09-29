use glib::clone;
use gtk4::gdk::{Key, ModifierType, Monitor, RGBA};
use gtk4::prelude::*;
use gtk4::{
    Align, ApplicationWindow, Box, CenterBox, EventControllerKey, EventControllerScroll,
    EventControllerScrollFlags, Image, Label, MenuButton, Orientation, Overlay, Popover,
    PositionType, Separator, Stack, StackTransitionType,
};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::service::audio::Audio;
use crate::service::battery::Battery;
use crate::service::bluetooth::Bluetooth;
use crate::service::hyprland::Hyprland;
use crate::service::network::Network;
use crate::service::notifications::Notifications;
use crate::widget::circular_progress::CircularProgress;
use crate::widget::control_center::ControlCenter;

const CIRCULAR_SIZE: i32 = 17;
const CIRCULAR_IMAGE_SIZE: i32 = 11;

type ProgressCalculator = Rc<RefCell<Option<std::boxed::Box<dyn Fn()>>>>;

static BAR_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

pub struct Bar {
    _window: ApplicationWindow,
    _monitor_info: String,
    _connector: String,
    _id: u64,
    _popover: Popover,
    _control_center: Rc<ControlCenter>,
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
            self._popover.popup();
        }
    }

    pub fn new(app: &gtk4::Application, monitor: &Monitor) -> Self {
        let bar_id = BAR_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        let monitor_connector = monitor
            .connector()
            .map(|s| s.to_string())
            .unwrap_or_default();
        let monitor_info = format!("id={}, connector='{}'", bar_id, monitor_connector);

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

        // Main container overlay
        let main_overlay = Overlay::new();

        // Create bar button with contents - use MenuButton instead of Button
        let bar_button = MenuButton::new();
        bar_button.add_css_class("Bar");

        let (bar_contents, left_stack, right_stack) = Self::create_bar_contents(&monitor_connector);
        bar_button.set_child(Some(&bar_contents));

        // Setup scroll event handling on the stacks themselves
        Self::setup_scroll_events(&left_stack, &right_stack);

        // Create popover with ControlCenter
        let popover = Popover::new();
        popover.set_position(PositionType::Bottom);
        let control_center = Rc::new(ControlCenter::new(&popover));
        popover.set_child(Some(control_center.widget()));
        bar_button.set_popover(Some(&popover));

        // Setup keyboard mode based on popover visibility and refresh languages
        popover.connect_show(clone!(
            #[weak]
            window,
            #[strong]
            control_center,
            move |_| {
                window.set_keyboard_mode(KeyboardMode::OnDemand);
                // Refresh language list when opening control center
                crate::service::fcitx::Fcitx::instance().refresh_languages();
                // Focus notifications toggle (or expander if visible) for keyboard navigation
                control_center.focus_notifications_toggle();
            }
        ));

        popover.connect_hide(clone!(
            #[weak]
            window,
            move |_| {
                window.set_keyboard_mode(KeyboardMode::None);
            }
        ));

        // Add keyboard controller to popover
        let key_controller = EventControllerKey::new();
        key_controller.connect_key_pressed(clone!(
            #[weak]
            popover,
            #[weak]
            left_stack,
            #[weak]
            right_stack,
            #[upgrade_or]
            glib::Propagation::Proceed,
            move |controller, key, _, modifiers| {
                let has_ctrl = modifiers.contains(ModifierType::CONTROL_MASK);
                let has_shift = modifiers.contains(ModifierType::SHIFT_MASK);

                // Handle Ctrl+Arrow and Shift+Arrow for stack cycling
                if matches!(key, Key::Up | Key::Down) && (has_ctrl || has_shift) {
                    let is_up = key == Key::Up;

                    if has_ctrl {
                        Self::cycle_right_stack(&right_stack, is_up);
                    }
                    if has_shift {
                        Self::cycle_left_stack(&left_stack, is_up);
                    }

                    return glib::Propagation::Stop;
                }

                // Check if focus is in a text input widget
                if let Some(focus_widget) = controller.widget().and_then(|w| w.focus_child()) {
                    let type_name = focus_widget.type_().name();
                    let is_text_input = type_name == "GtkEntry"
                        || type_name == "GtkText"
                        || type_name == "GtkTextView"
                        || type_name == "GtkPasswordEntry"
                        || type_name == "GtkSearchEntry";

                    if is_text_input {
                        return glib::Propagation::Proceed;
                    }
                }

                // Define navigation keys that should NOT close the popover
                let is_navigation_key = matches!(
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
                );

                // Define modifier keys that should NOT close the popover
                let is_modifier_key = matches!(
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
                );

                // Check if Ctrl or Shift modifiers are active
                let has_modifiers = has_ctrl || has_shift;

                // Close popover on Escape or non-navigation/non-modifier key (unless modifiers active)
                if key == Key::Escape || (!is_navigation_key && !is_modifier_key && !has_modifiers)
                {
                    popover.popdown();
                    glib::Propagation::Stop
                } else {
                    glib::Propagation::Proceed
                }
            }
        ));
        popover.add_controller(key_controller);

        // Box for spacing overlay (mimics .Overlay padding)
        let overlay_box = Box::new(Orientation::Horizontal, 0);
        overlay_box.add_css_class("Overlay");
        overlay_box.append(&bar_button);

        main_overlay.set_child(Some(&overlay_box));

        // Notifications count overlay
        let notifications_count = Self::create_notifications_count();
        main_overlay.add_overlay(&notifications_count);

        // Setup tooltip
        Self::setup_tooltip(&bar_button);

        window.set_child(Some(&main_overlay));

        // Realize the window to ensure widget hierarchy is fully initialized
        // before making it visible, preventing GTK assertion failures during
        // rapid window recreation (e.g., after resume from suspend)
        WidgetExt::realize(&window);
        window.set_visible(true);

        Self {
            _window: window,
            _monitor_info: monitor_info,
            _connector: monitor_connector,
            _id: bar_id,
            _popover: popover.clone(),
            _control_center: control_center,
        }
    }

    fn create_bar_contents(monitor_connector: &str) -> (Box, Stack, Stack) {
        let container = Box::new(Orientation::Vertical, 0);

        let top_row = Box::new(Orientation::Horizontal, 0);
        top_row.set_halign(Align::Center);

        let time_label = Label::new(Some("--:--"));
        time_label.add_css_class("TimeLabel");
        top_row.append(&time_label);

        container.append(&top_row);

        // Create the circular progress widgets
        let network_state = Self::create_network_state();
        let bluetooth_state = Self::create_bluetooth_state();
        let workspace_digit = Self::create_workspace_digit(monitor_connector);
        let battery_state = Self::create_battery_state();

        // Create left stack for network, workspace digit, or bluetooth
        let left_stack = Stack::new();
        left_stack.add_css_class("LeftStack");
        left_stack.set_transition_duration(300);
        left_stack.add_named(&network_state, Some("network"));
        left_stack.add_named(&workspace_digit, Some("workspace-digit"));
        left_stack.add_named(&bluetooth_state, Some("bluetooth"));

        // Create right stack for microphone, volume, or battery
        let right_stack = Stack::new();
        right_stack.add_css_class("RightStack");
        right_stack.set_transition_duration(300);

        let microphone_state = Self::create_microphone_state();
        right_stack.add_named(&microphone_state, Some("microphone"));

        let speaker_state_right = Self::create_speaker_state();
        right_stack.add_named(&speaker_state_right, Some("volume"));

        right_stack.add_named(&battery_state, Some("battery"));

        // Use CenterBox with left stack on left and right stack on right
        let bottom_row = CenterBox::new();
        bottom_row.add_css_class("BarStacks");
        bottom_row.set_start_widget(Some(&left_stack));

        // Add middle dot separator
        let separator = Separator::new(Orientation::Vertical);
        separator.add_css_class("BarDotSeparator");
        bottom_row.set_center_widget(Some(&separator));

        bottom_row.set_end_widget(Some(&right_stack));

        container.append(&bottom_row);

        Self::setup_time_updates(&time_label);
        Self::setup_stack_updates(&left_stack, &right_stack);

        (container, left_stack, right_stack)
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

    fn setup_scroll_events(left_stack: &Stack, right_stack: &Stack) {
        // Left stack scroll controller - scrolls through network -> workspace-digit -> bluetooth
        // Stops at edges (no infinite loop)
        let left_scroll = EventControllerScroll::new(EventControllerScrollFlags::VERTICAL);
        left_scroll.connect_scroll(clone!(
            #[weak]
            left_stack,
            #[upgrade_or]
            glib::Propagation::Proceed,
            move |_, _, dy| {
                let current = left_stack.visible_child_name();
                if let Some(name) = current.as_ref() {
                    let name_str = name.as_str();
                    if dy < 0.0 {
                        // Scroll up: previous in sequence (stop at network)
                        match name_str {
                            "network" => {
                                // Already at first item, do nothing
                            }
                            "workspace-digit" => {
                                left_stack.set_transition_type(StackTransitionType::SlideDown);
                                left_stack.set_visible_child_name("network");
                            }
                            "bluetooth" => {
                                left_stack.set_transition_type(StackTransitionType::SlideDown);
                                left_stack.set_visible_child_name("workspace-digit");
                            }
                            _ => {}
                        }
                    } else {
                        // Scroll down: next in sequence (stop at bluetooth)
                        match name_str {
                            "network" => {
                                left_stack.set_transition_type(StackTransitionType::SlideUp);
                                left_stack.set_visible_child_name("workspace-digit");
                            }
                            "workspace-digit" => {
                                left_stack.set_transition_type(StackTransitionType::SlideUp);
                                left_stack.set_visible_child_name("bluetooth");
                            }
                            "bluetooth" => {
                                // Already at last item, do nothing
                            }
                            _ => {}
                        }
                    }
                }
                glib::Propagation::Stop
            }
        ));
        left_stack.add_controller(left_scroll);

        // Right stack scroll controller - scrolls through microphone -> volume -> battery
        // Stops at edges (no infinite loop)
        let right_scroll = EventControllerScroll::new(EventControllerScrollFlags::VERTICAL);
        right_scroll.connect_scroll(clone!(
            #[weak]
            right_stack,
            #[upgrade_or]
            glib::Propagation::Proceed,
            move |_, _, dy| {
                let current = right_stack.visible_child_name();
                if let Some(name) = current.as_ref() {
                    let name_str = name.as_str();
                    if dy < 0.0 {
                        // Scroll up: previous in sequence (stop at microphone)
                        match name_str {
                            "microphone" => {
                                // Already at first item, do nothing
                            }
                            "volume" => {
                                right_stack.set_transition_type(StackTransitionType::SlideDown);
                                right_stack.set_visible_child_name("microphone");
                            }
                            "battery" => {
                                right_stack.set_transition_type(StackTransitionType::SlideDown);
                                right_stack.set_visible_child_name("volume");
                            }
                            _ => {}
                        }
                    } else {
                        // Scroll down: next in sequence (stop at battery)
                        match name_str {
                            "microphone" => {
                                right_stack.set_transition_type(StackTransitionType::SlideUp);
                                right_stack.set_visible_child_name("volume");
                            }
                            "volume" => {
                                right_stack.set_transition_type(StackTransitionType::SlideUp);
                                right_stack.set_visible_child_name("battery");
                            }
                            "battery" => {
                                // Already at last item, do nothing
                            }
                            _ => {}
                        }
                    }
                }
                glib::Propagation::Stop
            }
        ));
        right_stack.add_controller(right_scroll);
    }

    fn cycle_left_stack(stack: &Stack, is_up: bool) {
        let current = stack.visible_child_name();
        if let Some(name) = current.as_ref() {
            let name_str = name.as_str();
            if is_up {
                // Up: previous in sequence (stop at network)
                match name_str {
                    "network" => {
                        // Already at first item, do nothing
                    }
                    "workspace-digit" => {
                        stack.set_transition_type(StackTransitionType::SlideDown);
                        stack.set_visible_child_name("network");
                    }
                    "bluetooth" => {
                        stack.set_transition_type(StackTransitionType::SlideDown);
                        stack.set_visible_child_name("workspace-digit");
                    }
                    _ => {}
                }
            } else {
                // Down: next in sequence (stop at bluetooth)
                match name_str {
                    "network" => {
                        stack.set_transition_type(StackTransitionType::SlideUp);
                        stack.set_visible_child_name("workspace-digit");
                    }
                    "workspace-digit" => {
                        stack.set_transition_type(StackTransitionType::SlideUp);
                        stack.set_visible_child_name("bluetooth");
                    }
                    "bluetooth" => {
                        // Already at last item, do nothing
                    }
                    _ => {}
                }
            }
        }
    }

    fn cycle_right_stack(stack: &Stack, is_up: bool) {
        let current = stack.visible_child_name();
        if let Some(name) = current.as_ref() {
            let name_str = name.as_str();
            if is_up {
                // Up: previous in sequence (stop at microphone)
                match name_str {
                    "microphone" => {
                        // Already at first item, do nothing
                    }
                    "volume" => {
                        stack.set_transition_type(StackTransitionType::SlideDown);
                        stack.set_visible_child_name("microphone");
                    }
                    "battery" => {
                        stack.set_transition_type(StackTransitionType::SlideDown);
                        stack.set_visible_child_name("volume");
                    }
                    _ => {}
                }
            } else {
                // Down: next in sequence (stop at battery)
                match name_str {
                    "microphone" => {
                        stack.set_transition_type(StackTransitionType::SlideUp);
                        stack.set_visible_child_name("volume");
                    }
                    "volume" => {
                        stack.set_transition_type(StackTransitionType::SlideUp);
                        stack.set_visible_child_name("battery");
                    }
                    "battery" => {
                        // Already at last item, do nothing
                    }
                    _ => {}
                }
            }
        }
    }

    fn setup_stack_updates(left_stack: &Stack, right_stack: &Stack) {
        // Update stacks based on current state
        Self::update_stacks(left_stack, right_stack);

        // Listen to battery state changes
        let battery = Battery::instance();
        battery.connect_notify_local(
            Some("state"),
            clone!(
                #[weak]
                left_stack,
                #[weak]
                right_stack,
                move |_, _| {
                    Self::update_stacks(&left_stack, &right_stack);
                }
            ),
        );

        // Listen to network type changes (to detect connection/disconnection)
        let network = Network::instance();
        network.connect_notify_local(
            Some("primary"),
            clone!(
                #[weak]
                left_stack,
                #[weak]
                right_stack,
                move |_, _| {
                    Self::update_stacks(&left_stack, &right_stack);
                }
            ),
        );

        // Listen to bluetooth enabled changes
        let bluetooth = Bluetooth::instance();
        bluetooth.connect_notify_local(
            Some("enabled"),
            clone!(
                #[weak]
                left_stack,
                #[weak]
                right_stack,
                move |_, _| {
                    Self::update_stacks(&left_stack, &right_stack);
                }
            ),
        );

        // Listen to bluetooth missing expected devices changes
        bluetooth.connect_notify_local(
            Some("missing-expected-devices"),
            clone!(
                #[weak]
                left_stack,
                #[weak]
                right_stack,
                move |_, _| {
                    Self::update_stacks(&left_stack, &right_stack);
                }
            ),
        );

        // Listen to audio mic-muted changes
        let audio = Audio::instance();
        audio.connect_notify_local(
            Some("mic-muted"),
            clone!(
                #[weak]
                left_stack,
                #[weak]
                right_stack,
                move |_, _| {
                    Self::update_stacks(&left_stack, &right_stack);
                }
            ),
        );
    }

    fn update_stacks(left_stack: &Stack, right_stack: &Stack) {
        let battery = Battery::instance();
        let network = Network::instance();
        let bluetooth = Bluetooth::instance();
        let audio = Audio::instance();

        // Check if battery is connected but not charging (capped to preserve battery life)
        use crate::service::battery::BatteryState;
        let battery_state = battery.state();
        let battery_not_charging = matches!(
            battery_state,
            BatteryState::FullyCharged | BatteryState::PendingCharge
        );

        // Check if network is fully connected (not just connecting or disconnecting)
        use crate::service::network::NetworkState;
        let network_fully_connected = matches!(network.state(), NetworkState::ConnectedGlobal);

        // Check bluetooth state
        let bluetooth_enabled = bluetooth.enabled();
        let bluetooth_has_missing_devices = bluetooth.has_missing_expected_devices();

        // Update right stack priority:
        // 1. Battery needs attention → show battery
        // 2. Microphone muted → show microphone
        // 3. Default → show volume (displays muted icon if muted)
        let mic_muted = audio.mic_muted();

        if !battery_not_charging {
            // Battery needs attention (charging, discharging, low)
            right_stack.set_transition_type(StackTransitionType::SlideUp);
            right_stack.set_visible_child_name("battery");
        } else if mic_muted {
            // Battery ok, but microphone is muted
            right_stack.set_transition_type(StackTransitionType::SlideDown);
            right_stack.set_visible_child_name("microphone");
        } else {
            // Normal state - show volume (will show muted icon if muted)
            right_stack.set_transition_type(StackTransitionType::SlideDown);
            right_stack.set_visible_child_name("volume");
        }

        // Update left stack with priority:
        // 1. Network not fully connected → show network (highest priority)
        // 2. Bluetooth off OR expected device missing OR battery critically low → show bluetooth
        // 3. Otherwise → show workspace digit (default)
        let current = left_stack.visible_child_name();
        let current_str = current.as_ref().map(|s| s.as_str()).unwrap_or("");

        let lowest_battery = bluetooth.lowest_connected_expected_battery();
        let battery_critically_low = lowest_battery.is_some_and(|level| level < 20);

        let target = if !network_fully_connected {
            "network"
        } else if !bluetooth_enabled || bluetooth_has_missing_devices || battery_critically_low {
            "bluetooth"
        } else {
            "workspace-digit"
        };

        if current_str != target {
            // Determine transition direction based on sequence: network -> workspace-digit -> bluetooth
            let transition = match (current_str, target) {
                ("network", "workspace-digit") | ("workspace-digit", "bluetooth") => {
                    StackTransitionType::SlideUp
                }
                ("bluetooth", "workspace-digit") | ("workspace-digit", "network") => {
                    StackTransitionType::SlideDown
                }
                ("network", "bluetooth") => StackTransitionType::SlideUp,
                ("bluetooth", "network") => StackTransitionType::SlideDown,
                _ => StackTransitionType::SlideUp,
            };
            left_stack.set_transition_type(transition);
            left_stack.set_visible_child_name(target);
        }
    }

    fn create_network_state() -> Overlay {
        let overlay = Overlay::new();
        overlay.set_halign(Align::Start);

        let circular = CircularProgress::new(CIRCULAR_SIZE, 1.0, true);
        overlay.set_child(Some(circular.widget()));

        let image = Image::from_icon_name("network-offline-symbolic");
        image.set_pixel_size(CIRCULAR_IMAGE_SIZE);
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

    fn create_bluetooth_state() -> Overlay {
        let overlay = Overlay::new();
        overlay.set_halign(Align::Start);

        let circular = CircularProgress::new(CIRCULAR_SIZE, 1.0, true);
        overlay.set_child(Some(circular.widget()));

        let image = Image::from_icon_name("bluetooth-active-symbolic");
        image.set_pixel_size(CIRCULAR_IMAGE_SIZE);
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

    fn create_workspace_digit(monitor_connector: &str) -> Overlay {
        let overlay = Overlay::new();
        overlay.set_halign(Align::Start);

        let circular = CircularProgress::new(CIRCULAR_SIZE, 1.0, false);
        overlay.set_child(Some(circular.widget()));

        let image = Image::from_icon_name("1-symbolic");
        image.set_pixel_size(CIRCULAR_IMAGE_SIZE);
        overlay.add_overlay(&image);

        // Initial color setup
        let color = Self::color_with_alpha(&image);
        circular.set_color(color);

        // Update workspace number based on Hyprland events
        if let Some(hyprland) = Hyprland::instance() {
            let monitor_connector = monitor_connector.to_string();

            let update_workspace = {
                let hyprland = hyprland.clone();
                let image = image.clone();
                let circular = circular.clone();
                let monitor_connector = monitor_connector.clone();
                move || {
                    let monitors = hyprland.monitors();
                    if let Some(monitor) = monitors.iter().find(|m| m.name == monitor_connector) {
                        let workspace_id = monitor.active_workspace.id;
                        let icon_name = if workspace_id > 9 {
                            "plus-symbolic"
                        } else {
                            &format!("{}-symbolic", workspace_id)
                        };
                        image.set_icon_name(Some(icon_name));

                        // Set percentage based on workspace number
                        let percentage = if workspace_id >= 10 {
                            1.0
                        } else {
                            (workspace_id as f64) * 0.1
                        };
                        circular.set_percentage(percentage);
                    }
                }
            };

            // Listen to workspace change events (matching control center implementation)
            hyprland.connect_local(
                "event",
                false,
                clone!(
                    #[strong]
                    update_workspace,
                    move |args| {
                        // Event name is in args[1], not args[0]
                        if let Some(event) = args.get(1).and_then(|v| v.get::<String>().ok()) {
                            if matches!(
                                event.as_str(),
                                "workspace" | "workspacev2" | "focusedmon" | "moveworkspace"
                            ) {
                                update_workspace();
                            }
                        }
                        None
                    }
                ),
            );

            update_workspace();
        }

        overlay
    }

    fn create_microphone_state() -> Overlay {
        let overlay = Overlay::new();
        overlay.set_halign(Align::Start);

        let circular = CircularProgress::new(CIRCULAR_SIZE, 1.0, true);
        overlay.set_child(Some(circular.widget()));

        let image = Image::from_icon_name("microphone-sensitivity-high-symbolic");
        image.set_pixel_size(CIRCULAR_IMAGE_SIZE);
        overlay.add_overlay(&image);

        let audio = Audio::instance();

        let update_microphone = {
            let audio = audio.clone();
            let circular = circular.clone();
            let image = image.clone();
            move || {
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
                let color = Self::color_with_alpha(&image);
                circular.set_color(color);
            }
        };

        audio.connect_notify_local(Some("mic-volume"), {
            let update_microphone = update_microphone.clone();
            move |_, _| update_microphone()
        });

        audio.connect_notify_local(Some("mic-muted"), {
            let update_microphone = update_microphone.clone();
            move |_, _| update_microphone()
        });

        audio.connect_notify_local(Some("source-name"), {
            let update_microphone = update_microphone.clone();
            move |_, _| update_microphone()
        });

        update_microphone();

        overlay
    }

    fn create_speaker_state() -> Overlay {
        let overlay = Overlay::new();
        overlay.set_halign(Align::Start);

        let circular = CircularProgress::new(CIRCULAR_SIZE, 1.0, true);
        overlay.set_child(Some(circular.widget()));

        let image = Image::from_icon_name("audio-speakers-symbolic");
        image.set_pixel_size(CIRCULAR_IMAGE_SIZE);
        overlay.add_overlay(&image);

        let audio = Audio::instance();

        let update_speaker = {
            let audio = audio.clone();
            let circular = circular.clone();
            let image = image.clone();
            move || {
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
                let color = Self::color_with_alpha(&image);
                circular.set_color(color);
            }
        };

        audio.connect_notify_local(Some("volume"), {
            let update_speaker = update_speaker.clone();
            move |_, _| update_speaker()
        });

        audio.connect_notify_local(Some("muted"), {
            let update_speaker = update_speaker.clone();
            move |_, _| update_speaker()
        });

        audio.connect_notify_local(Some("sink-name"), {
            let update_speaker = update_speaker.clone();
            move |_, _| update_speaker()
        });

        update_speaker();

        overlay
    }

    fn create_battery_state() -> Overlay {
        let overlay = Overlay::new();
        overlay.set_halign(Align::End);

        let circular = CircularProgress::new(CIRCULAR_SIZE, 1.0, true);
        overlay.set_child(Some(circular.widget()));

        let image = Image::from_icon_name("battery-symbolic");
        image.set_pixel_size(CIRCULAR_IMAGE_SIZE);
        overlay.add_overlay(&image);

        let battery = Battery::instance();

        let update_battery = {
            let battery = battery.clone();
            let circular = circular.clone();
            let image = image.clone();
            move || {
                let percentage = battery.percentage();
                let state = battery.state();

                // Update icon
                let icon = battery.icon_name();
                image.set_icon_name(Some(&icon));

                // Set CSS classes based on battery state
                image.remove_css_class("Warning");
                image.remove_css_class("Critical");
                image.remove_css_class("Active");

                use crate::service::battery::BatteryState;
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

                // Update progress
                circular.set_percentage(percentage);

                // Update color
                let color = Self::color_with_alpha(&image);
                circular.set_color(color);
            }
        };

        battery.connect_notify_local(Some("percentage"), {
            let update_battery = update_battery.clone();
            move |_, _| update_battery()
        });

        battery.connect_notify_local(Some("state"), {
            let update_battery = update_battery.clone();
            move |_, _| update_battery()
        });

        update_battery();

        overlay
    }

    fn create_notifications_count() -> Box {
        let container = Box::new(Orientation::Horizontal, 0);
        container.set_halign(Align::End);
        container.set_valign(Align::Start);

        let label = Label::new(Some("0"));
        label.add_css_class("NotificationsCount");
        label.set_visible(false);
        container.append(&label);

        let notifications = Notifications::instance();

        let update_count = Rc::new({
            let notifications = notifications.clone();
            let label = label.clone();
            move || {
                let count = if notifications.dont_disturb() {
                    0
                } else {
                    notifications.notifications().len()
                };

                if count > 0 {
                    let label_text = if count > 9 {
                        "+".to_string()
                    } else {
                        count.to_string()
                    };
                    label.set_label(&label_text);
                    label.set_visible(true);
                } else {
                    label.set_visible(false);
                }
            }
        });

        notifications.connect_local("notified", false, {
            let update_count = update_count.clone();
            move |_| {
                update_count();
                None
            }
        });

        notifications.connect_local("resolved", false, {
            let update_count = update_count.clone();
            move |_| {
                update_count();
                None
            }
        });

        notifications.connect_notify_local(Some("dont-disturb"), {
            let update_count = update_count.clone();
            move |_, _| {
                update_count();
            }
        });

        update_count();

        container
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
