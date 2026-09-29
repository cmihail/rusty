use gdk4::Key;
use gtk4::prelude::*;
use gtk4::{
    glib, Application, Box, Button, Image, Label, Orientation, Separator, ToggleButton, Window,
};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::cell::RefCell;
use std::rc::Rc;

use rusty_screenshot::notify;
use rusty_screenshot::recording::*;
use rusty_screenshot::screenshot::*;

const APP_ID: &str = "com.github.cmihail.screenshot";

// Must outlast the compositor's window close animation, or the fading menu
// still shows up in the capture.
const CAPTURE_DELAY: std::time::Duration = std::time::Duration::from_millis(600);

fn main() -> glib::ExitCode {
    // A detached child of a previous run: handled before GTK exists, so it
    // never claims the single-instance name and never blocks the GUI.
    if let Some((kind, output_file)) = notify::parse_notify_args(std::env::args().skip(1)) {
        notify::run_notification(kind, &output_file);
        return glib::ExitCode::SUCCESS;
    }

    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    // If a window already exists, close it (toggle behavior like de app)
    if let Some(window) = app.active_window() {
        window.close();
        return;
    }

    // Capture focused window and monitor BEFORE creating the UI
    let focused_monitor = rusty_screenshot::get_focused_monitor().ok();
    let focused_window = rusty_screenshot::get_active_window().ok();

    let window = create_layer_shell_window(app);
    load_css();

    let main_box = create_main_content_box(&window, focused_monitor, focused_window);
    window.set_child(Some(&main_box));

    setup_window_controllers(&window, &main_box);
    window.present();
}

fn create_layer_shell_window(app: &Application) -> Window {
    let window = Window::builder()
        .application(app)
        .title("Screenshot")
        .decorated(false)
        .build();

    window.init_layer_shell();
    window.set_layer(Layer::Top);
    window.set_namespace(Some("Screenshot"));
    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive);

    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Bottom, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);

    window.set_css_classes(&["Screenshot"]);
    window
}

fn load_css() {
    let provider = gtk4::CssProvider::new();
    let css_data = include_str!("style.css");
    provider.load_from_data(css_data);
    gtk4::style_context_add_provider_for_display(
        &gdk4::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn create_main_content_box(
    window: &Window,
    focused_monitor: Option<rusty_screenshot::Monitor>,
    focused_window: Option<rusty_screenshot::ActiveWindow>,
) -> Box {
    let main_box = Box::new(Orientation::Vertical, 0);
    main_box.set_halign(gtk4::Align::Center);
    main_box.set_valign(gtk4::Align::Center);
    main_box.set_css_classes(&["Centered"]);

    let contents = create_contents(window, focused_monitor, focused_window);
    main_box.append(&contents);
    main_box
}

fn setup_window_controllers(window: &Window, main_box: &Box) {
    setup_key_controller(window);
    setup_click_outside_controller(window, main_box);
}

fn setup_key_controller(window: &Window) {
    let key_controller = gtk4::EventControllerKey::new();
    let window_clone = window.clone();
    key_controller.connect_key_pressed(move |_controller, keyval, _keycode, _state| {
        if keyval == Key::Escape {
            window_clone.close();
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
    window.add_controller(key_controller);
}

fn setup_click_outside_controller(window: &Window, main_box: &Box) {
    let gesture = gtk4::GestureClick::new();
    let window_clone = window.clone();
    let main_box_clone = main_box.clone();
    gesture.connect_released(move |_gesture, _n_press, x, y| {
        let allocation = main_box_clone.allocation();
        let main_box_x = allocation.x() as f64;
        let main_box_y = allocation.y() as f64;
        let main_box_width = allocation.width() as f64;
        let main_box_height = allocation.height() as f64;

        if x < main_box_x
            || x > main_box_x + main_box_width
            || y < main_box_y
            || y > main_box_y + main_box_height
        {
            window_clone.close();
        }
    });
    window.add_controller(gesture);
}

fn create_contents(
    window: &Window,
    focused_monitor: Option<rusty_screenshot::Monitor>,
    focused_window: Option<rusty_screenshot::ActiveWindow>,
) -> Box {
    let vbox = Box::new(Orientation::Vertical, 10);

    create_screenshot_section(&vbox, window, focused_monitor, focused_window.clone());

    let separator_between = Separator::new(Orientation::Horizontal);
    vbox.append(&separator_between);

    create_recording_section(&vbox, window);

    vbox
}

fn create_screenshot_section(
    vbox: &Box,
    window: &Window,
    focused_monitor: Option<rusty_screenshot::Monitor>,
    focused_window: Option<rusty_screenshot::ActiveWindow>,
) {
    let screenshot_title = Label::new(Some("Take a screenshot"));
    screenshot_title.add_css_class("section-title");
    vbox.append(&screenshot_title);

    let screenshot_buttons_box = Box::new(Orientation::Vertical, 8);

    let all_monitors_btn = create_button(
        "video-joined-displays-symbolic",
        "All monitors",
        window,
        take_all_monitors_screenshot,
    );
    screenshot_buttons_box.append(&all_monitors_btn);

    create_focused_monitor_button(&screenshot_buttons_box, window, focused_monitor);
    create_focused_window_button(&screenshot_buttons_box, window, focused_window);

    let select_area_btn = create_button(
        "document-edit-symbolic",
        "Select area",
        window,
        take_select_area_screenshot,
    );
    screenshot_buttons_box.append(&select_area_btn);

    vbox.append(&screenshot_buttons_box);
}

fn create_focused_monitor_button(
    container: &Box,
    window: &Window,
    focused_monitor: Option<rusty_screenshot::Monitor>,
) {
    let focused_monitor_clone = Rc::new(focused_monitor);
    let focused_monitor_for_btn = focused_monitor_clone.clone();
    let focused_monitor_btn =
        create_button("computer-symbolic", "Focused monitor", window, move || {
            rusty_screenshot::take_focused_monitor_screenshot_with_monitor(
                (*focused_monitor_for_btn).clone(),
            )
        });
    container.append(&focused_monitor_btn);
}

fn create_focused_window_button(
    container: &Box,
    window: &Window,
    focused_window: Option<rusty_screenshot::ActiveWindow>,
) {
    let focused_window_clone = Rc::new(focused_window);
    let focused_window_for_btn = focused_window_clone.clone();
    let focused_window_btn = create_button(
        "focus-windows-symbolic",
        "Focused window",
        window,
        move || {
            rusty_screenshot::take_focused_window_screenshot_with_window(
                (*focused_window_for_btn).clone(),
            )
        },
    );
    container.append(&focused_window_btn);
}

fn create_recording_section(vbox: &Box, window: &Window) {
    let recording_title = Label::new(Some("Record video"));
    recording_title.add_css_class("section-title");
    vbox.append(&recording_title);

    let recording_buttons_box = Box::new(Orientation::Vertical, 8);

    let focused_monitor_row = create_recording_row(
        "computer-symbolic",
        "Focused monitor",
        window,
        record_focused_monitor,
    );
    recording_buttons_box.append(&focused_monitor_row);

    let focused_window_row = create_recording_row(
        "focus-windows-symbolic",
        "Focused window",
        window,
        record_focused_window,
    );
    recording_buttons_box.append(&focused_window_row);

    let select_area_row = create_recording_row(
        "document-edit-symbolic",
        "Select area",
        window,
        record_select_area,
    );
    recording_buttons_box.append(&select_area_row);

    vbox.append(&recording_buttons_box);
}

fn create_button<F>(icon_name: &str, label_text: &str, window: &Window, callback: F) -> Button
where
    F: Fn() + 'static,
{
    let button = Button::new();

    let hbox = Box::new(Orientation::Horizontal, 8);

    let icon = Image::from_icon_name(icon_name);
    hbox.append(&icon);

    let label = Label::new(Some(label_text));
    label.set_halign(gtk4::Align::Start);
    label.set_hexpand(true);
    label.set_wrap(false);
    label.set_ellipsize(gtk4::pango::EllipsizeMode::None);
    hbox.append(&label);

    button.set_child(Some(&hbox));

    let window_clone = window.clone();
    let callback_rc = Rc::new(callback);

    button.connect_clicked(move |_| {
        // Hide window first, then take screenshot after a delay
        window_clone.set_visible(false);

        // Clone the values for the timeout closure
        let window_for_timeout = window_clone.clone();
        let callback_for_timeout = callback_rc.clone();

        // Use glib timeout to execute screenshot after window is hidden
        glib::timeout_add_local_once(CAPTURE_DELAY, move || {
            callback_for_timeout();
            // Close window after screenshot is taken
            window_for_timeout.close();
        });
    });

    button
}

fn create_recording_row<F>(icon_name: &str, label_text: &str, window: &Window, callback: F) -> Box
where
    F: Fn(bool) + 'static,
{
    let row_box = Box::new(Orientation::Horizontal, 8);
    let button = create_recording_button(icon_name, label_text);
    let audio_toggle = Rc::new(create_audio_toggle_button());

    row_box.append(&button);
    row_box.append(&*audio_toggle);

    setup_recording_button_click(&button, audio_toggle, window, callback);

    row_box
}

fn create_recording_button(icon_name: &str, label_text: &str) -> Button {
    let button = Button::new();
    let button_content = Box::new(Orientation::Horizontal, 8);

    let icon = Image::from_icon_name(icon_name);
    button_content.append(&icon);

    let label = Label::new(Some(label_text));
    label.set_halign(gtk4::Align::Start);
    label.set_hexpand(true);
    label.set_wrap(false);
    label.set_ellipsize(gtk4::pango::EllipsizeMode::None);
    button_content.append(&label);

    button.set_child(Some(&button_content));
    button.set_hexpand(true);
    button
}

fn create_audio_toggle_button() -> ToggleButton {
    let audio_toggle = ToggleButton::new();
    let audio_icon = Rc::new(RefCell::new(Image::from_icon_name(
        "audio-volume-muted-symbolic",
    )));
    audio_toggle.set_child(Some(&*audio_icon.borrow()));
    audio_toggle.add_css_class("audio-toggle");
    audio_toggle.set_valign(gtk4::Align::Center);

    let audio_icon_clone = audio_icon.clone();
    audio_toggle.connect_toggled(move |toggle| {
        let new_icon = if toggle.is_active() {
            Image::from_icon_name("audio-volume-high-symbolic")
        } else {
            Image::from_icon_name("audio-volume-muted-symbolic")
        };
        *audio_icon_clone.borrow_mut() = new_icon.clone();
        toggle.set_child(Some(&new_icon));
    });

    audio_toggle
}

fn setup_recording_button_click<F>(
    button: &Button,
    audio_toggle: Rc<ToggleButton>,
    window: &Window,
    callback: F,
) where
    F: Fn(bool) + 'static,
{
    let window_clone = window.clone();
    let callback_rc = Rc::new(callback);

    button.connect_clicked(move |_| {
        let with_audio = audio_toggle.is_active();
        window_clone.set_visible(false);

        let window_for_timeout = window_clone.clone();
        let callback_for_timeout = callback_rc.clone();

        glib::timeout_add_local_once(std::time::Duration::from_millis(200), move || {
            callback_for_timeout(with_audio);
            window_for_timeout.close();
        });
    });
}
