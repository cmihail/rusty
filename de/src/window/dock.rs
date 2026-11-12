use glib::clone;
use gtk4::gdk::{Display, Monitor};
use gtk4::prelude::*;
use gtk4::{Align, DrawingArea, Orientation, Overlay, Separator, Widget};
use gtk4::{ApplicationWindow, Box, Button, EventControllerMotion, GestureClick, IconTheme};
use gtk4::{Image, Label};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::path::Path;
use std::process::Command;
use std::rc::Rc;
use std::time::Duration;

use crate::service::desktop_entry;
use crate::service::hyprland::{address_arg, Client, Hyprland, Workspace};
use crate::service::network::{Network, NetworkState};
use crate::service::notifications::Notifications;

pub const MINIMIZED_WORKSPACE: &str = "special:minimized";

const TRIGGER_LENGTH: i32 = 400;
const HIDE_DELAY: Duration = Duration::from_millis(300);
// Longer than HIDE_DELAY because the pointer only "enters" the dock once it moves again.
const SHOW_GRACE: Duration = Duration::from_millis(1500);
const ICON_SIZE: i32 = 32;
const FALLBACK_ICON: &str = "application-x-executable";
const LAUNCHER: &str = ".local/bin/rusty-universal-search-wrapped";
const LAUNCHER_ICONS: &[&str] = &["view-app-grid-symbolic", "system-search-symbolic"];
const RESIZE_EVENTS: &[&str] = &["openwindow", "closewindow", "movewindowv2"];
const HANDLE_THICKNESS: i32 = 8;
const HANDLE_EDGE: f64 = 1.0;
const BADGE_INSET: i32 = 5;
// The theme colors the badges use in style.css.
const HANDLE_COLOR: &str = "theme_selected_bg_color";
const HANDLE_ALERT_COLOR: &str = "warning_color";
const HANDLE_OFFLINE_COLOR: &str = "error_color";

#[derive(Debug, Clone, PartialEq)]
pub struct DockEntry {
    pub client: Client,
    pub minimized: bool,
}

/// Regular workspaces first, then other special workspaces, then minimized windows.
/// Windows on the same workspace keep their on-screen order.
pub fn dock_entries(clients: Vec<Client>) -> Vec<DockEntry> {
    let mut entries: Vec<DockEntry> = clients
        .into_iter()
        .map(|client| DockEntry {
            minimized: client.workspace.name == MINIMIZED_WORKSPACE,
            client,
        })
        .collect();

    entries.sort_by_key(|entry| {
        let client = &entry.client;
        (
            entry.minimized,
            client.workspace.id < 0,
            client.workspace.id,
            client.x,
            client.y,
        )
    });
    entries
}

/// Whether `cursor` is in the right-center area covered by the dock and its hover strip.
/// `monitor` is (x, y, width, height) and `dock_size` is (width, height), in layout pixels.
pub fn in_dock_area(
    cursor: (i32, i32),
    monitor: (i32, i32, i32, i32),
    dock_size: (i32, i32),
) -> bool {
    let (x, y, width, height) = monitor;
    let area_height = dock_size.1.max(TRIGGER_LENGTH);
    let top = y + (height - area_height) / 2;
    let bottom = top + area_height;
    let right = x + width;
    let left = right - dock_size.0.max(1);
    (left..right).contains(&cursor.0) && (top..bottom).contains(&cursor.1)
}

/// The workspace badge: its number, "+" past 9 like the notification count, and "S" for
/// special workspaces. Minimized windows get none since they come back on the active one.
pub fn workspace_badge(workspace: &Workspace) -> Option<String> {
    match workspace.id {
        _ if workspace.name == MINIMIZED_WORKSPACE => None,
        id if id < 0 => Some("S".to_string()),
        id if id > 9 => Some("+".to_string()),
        id => Some(id.to_string()),
    }
}

/// Updates the windows asking for attention from a Hyprland event. They stay urgent until
/// focused or closed. Returns whether the set changed.
pub fn track_urgent(urgent: &mut HashSet<String>, event: &str, address: &str) -> bool {
    match event {
        "urgent" => urgent.insert(address.to_string()),
        "activewindowv2" | "closewindow" => urgent.remove(address),
        _ => false,
    }
}

/// Whether `class` matches one of the notified window classes, ignoring case.
pub fn is_notified(class: &str, notified_classes: &[String]) -> bool {
    notified_classes
        .iter()
        .any(|c| class.eq_ignore_ascii_case(c))
}

/// Ids of the notifications, each given with its window classes, that belong to `class`.
pub fn notifications_of(class: &str, notifications: &[(u32, Vec<String>)]) -> Vec<u32> {
    notifications
        .iter()
        .filter(|(_, classes)| is_notified(class, classes))
        .map(|(id, _)| *id)
        .collect()
}

fn dismiss_notifications_of(class: &str) {
    let service = Notifications::instance();
    let notifications: Vec<(u32, Vec<String>)> = service
        .notifications()
        .iter()
        .map(|notification| (notification.id, notification.window_classes()))
        .collect();
    for id in notifications_of(class, &notifications) {
        service.dismiss(id);
    }
}

/// Window classes of the apps with open notifications. None while Do Not Disturb hides them.
fn notification_classes() -> Vec<String> {
    let notifications = Notifications::instance();
    if notifications.dont_disturb() {
        return Vec::new();
    }
    notifications
        .notifications()
        .iter()
        .flat_map(|notification| notification.window_classes())
        .collect()
}

/// Open notifications, or none while Do Not Disturb hides them.
fn visible_notification_count() -> usize {
    let notifications = Notifications::instance();
    if notifications.dont_disturb() {
        return 0;
    }
    notifications.notifications().len()
}

/// The theme color of the hidden dock's handle. Being offline outranks the alert for an
/// open notification or an urgent window.
pub fn handle_color(online: bool, notifications: usize, urgent: usize) -> &'static str {
    if !online {
        HANDLE_OFFLINE_COLOR
    } else if notifications > 0 || urgent > 0 {
        HANDLE_ALERT_COLOR
    } else {
        HANDLE_COLOR
    }
}

/// Cubic Bézier points for the handle's top edge: `HANDLE_EDGE` tall at both ends, rising
/// to the top of `height` in the middle. The handle fills everything below it.
pub fn handle_curve(width: f64, height: f64) -> [(f64, f64); 4] {
    let end_y = height - HANDLE_EDGE;
    // A quadratic curve peaks halfway to its control point, raised as a cubic.
    let control_y = -end_y;
    let toward = |from: f64, to: f64| from + 2.0 / 3.0 * (to - from);
    [
        (0.0, end_y),
        (toward(0.0, width / 2.0), toward(end_y, control_y)),
        (toward(width, width / 2.0), toward(end_y, control_y)),
        (width, end_y),
    ]
}

struct DockState {
    window: ApplicationWindow,
    items: Box,
    status: Widget,
    status_separator: Separator,
    held: Cell<bool>,
    handle: DrawingArea,
    monitor: Monitor,
    connector: String,
    hide_generation: Cell<u64>,
    urgent: RefCell<HashSet<String>>,
}

pub struct Dock {
    _trigger: ApplicationWindow,
    _state: Rc<DockState>,
}

impl Dock {
    pub fn new(app: &gtk4::Application, monitor: &Monitor, status: Widget) -> Self {
        let connector = monitor
            .connector()
            .map(|s| s.to_string())
            .unwrap_or_default();

        let window = Self::layer_window(app, monitor, "Dock");
        let items = Box::new(Orientation::Vertical, 6);
        items.add_css_class("DockItems");
        window.set_child(Some(&items));

        let handle = DrawingArea::builder()
            .content_width(HANDLE_THICKNESS)
            .content_height(1)
            .build();
        handle.set_draw_func(draw_handle);

        let state = Rc::new(DockState {
            window,
            items,
            status,
            status_separator: Separator::new(Orientation::Horizontal),
            held: Cell::new(false),
            handle,
            monitor: monitor.clone(),
            connector,
            hide_generation: Cell::new(0),
            urgent: RefCell::new(HashSet::new()),
        });

        let trigger = Self::create_trigger(app, monitor, &state);
        rebuild(&state);
        Self::connect_hover(&state);
        Self::connect_hyprland_events(&state);
        Self::connect_notifications(&state);
        Self::connect_network(&state);

        Self {
            _trigger: trigger,
            _state: state,
        }
    }

    /// Keeps the dock open and focusable while the control center is open on it.
    pub fn hold(&self, held: bool) {
        let state = &self._state;
        if state.held.replace(held) == held {
            return;
        }
        if held {
            state.window.set_keyboard_mode(KeyboardMode::OnDemand);
            cancel_hide(state);
        } else {
            state.window.set_keyboard_mode(KeyboardMode::None);
            schedule_hide(state, HIDE_DELAY);
        }
    }

    fn layer_window(app: &gtk4::Application, monitor: &Monitor, name: &str) -> ApplicationWindow {
        // Without a tiny default size GTK falls back to 200x200 and the window never shrinks.
        let window = ApplicationWindow::builder()
            .application(app)
            .decorated(false)
            .default_width(1)
            .default_height(1)
            .build();

        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_anchor(Edge::Right, true);
        window.set_monitor(Some(monitor));
        window.set_keyboard_mode(KeyboardMode::None);
        window.set_exclusive_zone(0);
        window.set_namespace(Some(name));
        window.add_css_class(name);
        window
    }

    fn create_trigger(
        app: &gtk4::Application,
        monitor: &Monitor,
        state: &Rc<DockState>,
    ) -> ApplicationWindow {
        let trigger = Self::layer_window(app, monitor, "DockTrigger");
        trigger.set_child(Some(&state.handle));

        update_handle(state);

        let motion = EventControllerMotion::new();
        motion.connect_enter(clone!(
            #[weak]
            state,
            move |_, _, _| show(&state)
        ));
        trigger.add_controller(motion);
        WidgetExt::realize(&trigger);
        trigger.set_visible(true);
        trigger
    }

    fn connect_hover(state: &Rc<DockState>) {
        let motion = EventControllerMotion::new();
        motion.connect_enter(clone!(
            #[weak]
            state,
            move |_, _, _| cancel_hide(&state)
        ));
        motion.connect_leave(clone!(
            #[weak]
            state,
            move |_| schedule_hide(&state, HIDE_DELAY)
        ));
        state.window.add_controller(motion);
    }

    fn connect_notifications(state: &Rc<DockState>) {
        let notifications = Notifications::instance();
        for signal in ["notified", "resolved"] {
            let weak = Rc::downgrade(state);
            notifications.connect_local(signal, false, move |_| {
                let state = weak.upgrade()?;
                update_handle(&state);
                if state.window.is_visible() {
                    rebuild(&state);
                }
                None
            });
        }

        let weak = Rc::downgrade(state);
        notifications.connect_notify_local(Some("dont-disturb"), move |_, _| {
            if let Some(state) = weak.upgrade() {
                update_handle(&state);
            }
        });
    }

    fn connect_network(state: &Rc<DockState>) {
        let weak = Rc::downgrade(state);
        Network::instance().connect_notify_local(Some("state"), move |_, _| {
            if let Some(state) = weak.upgrade() {
                update_handle(&state);
            }
        });
    }

    fn connect_hyprland_events(state: &Rc<DockState>) {
        let Some(hyprland) = Hyprland::instance() else {
            return;
        };

        let weak = Rc::downgrade(state);
        hyprland.connect_local("event", false, move |args| {
            let state = weak.upgrade()?;
            let event = args.get(1).and_then(|v| v.get::<String>().ok())?;
            let data = args.get(2).and_then(|v| v.get::<String>().ok());
            let urgent_changed = track_urgent(
                &mut state.urgent.borrow_mut(),
                &event,
                data.as_deref().unwrap_or_default(),
            );
            if urgent_changed {
                update_handle(&state);
            }
            // The hidden dock still follows window changes so the handle keeps its width.
            let resize = RESIZE_EVENTS.contains(&event.as_str());
            let relabel = event == "windowtitlev2";
            if resize || (relabel && state.window.is_visible()) {
                rebuild(&state);
            }
            if resize {
                update_handle(&state);
            }
            None
        });
    }
}

fn update_handle(state: &DockState) {
    let online = Network::instance().state() == NetworkState::ConnectedGlobal;
    let color = handle_color(
        online,
        visible_notification_count(),
        state.urgent.borrow().len(),
    );
    state.handle.set_css_classes(&[color]);
    state.handle.queue_draw();
}

fn draw_handle(area: &DrawingArea, cr: &gtk4::cairo::Context, width: i32, height: i32) {
    let classes = area.css_classes();
    let name = classes.first().map_or(HANDLE_COLOR, |name| name.as_str());
    // GtkWidget::color needs GTK 4.10, so the color comes from the theme by name.
    let Some(color) = area.style_context().lookup_color(name) else {
        log::warn!("dock: the theme defines no {}", name);
        return;
    };
    // The curve is built for a bottom handle, so swapping the axes stands it on the right edge.
    let (width, height) = (height as f64, width as f64);
    let [start, first, second, end] = handle_curve(width, height);
    cr.transform(gtk4::cairo::Matrix::new(0.0, 1.0, 1.0, 0.0, 0.0, 0.0));
    cr.set_source_rgb(
        color.red() as f64,
        color.green() as f64,
        color.blue() as f64,
    );
    cr.move_to(0.0, height);
    cr.line_to(start.0, start.1);
    cr.curve_to(first.0, first.1, second.0, second.1, end.0, end.1);
    cr.line_to(width, height);
    cr.close_path();
    if let Err(e) = cr.fill() {
        log::warn!("dock: failed to draw the handle: {}", e);
    }
}

fn show(state: &Rc<DockState>) {
    rebuild(state);
    state.window.present();
    set_trigger_visible(state, false);
    schedule_hide(state, SHOW_GRACE);
}

fn hide(state: &DockState) {
    cancel_hide(state);
    state.window.set_visible(false);
    set_trigger_visible(state, true);
}

/// The open dock sits over the handle, which would peek out beside it. Hyprland skips
/// redrawing the covered handle, so the whole trigger window is unmapped instead.
fn set_trigger_visible(state: &DockState, visible: bool) {
    if let Some(trigger) = state.handle.root() {
        trigger.set_visible(visible);
    }
}

fn cancel_hide(state: &DockState) {
    state.hide_generation.set(state.hide_generation.get() + 1);
}

/// GTK only reports the pointer entering once it moves, so a still pointer over the dock
/// looks absent. The compositor's cursor position is checked before hiding.
fn schedule_hide(state: &Rc<DockState>, delay: Duration) {
    cancel_hide(state);
    let generation = state.hide_generation.get();
    let weak = Rc::downgrade(state);
    glib::timeout_add_local_once(delay, move || {
        let Some(state) = weak.upgrade() else {
            return;
        };
        if state.hide_generation.get() != generation || state.held.get() {
            return;
        }
        if pointer_in_dock_area(&state) {
            schedule_hide(&state, HIDE_DELAY);
        } else {
            hide(&state);
        }
    });
}

fn pointer_in_dock_area(state: &DockState) -> bool {
    let Some(cursor) = Hyprland::instance().and_then(|hyprland| hyprland.cursor_position()) else {
        return false;
    };
    let geometry = state.monitor.geometry();
    let monitor = (
        geometry.x(),
        geometry.y(),
        geometry.width(),
        geometry.height(),
    );
    let dock_size = (state.window.width(), state.window.height());
    in_dock_area(cursor, monitor, dock_size)
}

/// The status widget stays in place so the control center open on it is not closed.
fn rebuild(state: &Rc<DockState>) {
    let mut child = state.items.first_child();
    while let Some(widget) = child {
        child = widget.next_sibling();
        if widget != state.status && widget != *state.status_separator.upcast_ref::<Widget>() {
            state.items.remove(&widget);
        }
    }

    let clients = Hyprland::instance()
        .map(|hyprland| hyprland.clients())
        .unwrap_or_default();

    state.items.append(&launcher_button(state));
    if !clients.is_empty() {
        state.items.append(&Separator::new(Orientation::Horizontal));
    }

    let notified_classes = notification_classes();

    let mut previous_minimized = false;
    for (index, entry) in dock_entries(clients).into_iter().enumerate() {
        if entry.minimized && !previous_minimized && index > 0 {
            state.items.append(&Separator::new(Orientation::Horizontal));
        }
        previous_minimized = entry.minimized;
        let notified = is_notified(&entry.client.class, &notified_classes);
        state.items.append(&item_button(state, entry, notified));
    }

    let last = state.items.last_child();
    if state.status.parent().is_none() {
        state.items.append(&state.status_separator);
        state.items.append(&state.status);
    }
    state
        .items
        .reorder_child_after(&state.status_separator, last.as_ref());
    state
        .items
        .reorder_child_after(&state.status, Some(&state.status_separator));

    let (_, dock_height, _, _) = state.items.measure(Orientation::Vertical, -1);
    state.handle.set_content_height(dock_height.max(1));
}

/// The first launcher icon the theme has, or the generic executable icon.
pub fn launcher_icon(has_icon: impl Fn(&str) -> bool) -> &'static str {
    LAUNCHER_ICONS
        .iter()
        .copied()
        .find(|name| has_icon(name))
        .unwrap_or(FALLBACK_ICON)
}

fn launcher_button(state: &Rc<DockState>) -> Button {
    let theme = Display::default().map(|display| IconTheme::for_display(&display));
    let icon = launcher_icon(|name| theme.as_ref().is_some_and(|theme| theme.has_icon(name)));
    let image = Image::from_icon_name(icon);
    image.set_pixel_size(ICON_SIZE);

    let button = Button::new();
    button.add_css_class("DockItem");
    button.add_css_class("DockLauncher");
    button.set_child(Some(&icon_with_badge(image, None, false)));
    button.set_tooltip_text(Some("Launcher"));
    button.connect_clicked(clone!(
        #[weak]
        state,
        move |_| {
            hide(&state);
            launch();
        }
    ));
    button
}

/// Runs in its own scope so the launcher outlives a rusty-de restart.
fn launch() {
    let Some(home) = std::env::var_os("HOME") else {
        return;
    };
    let launcher = Path::new(&home).join(LAUNCHER);
    let spawned = Command::new("systemd-run")
        .args(["--user", "--scope", "--"])
        .arg(&launcher)
        .spawn();
    if let Err(e) = spawned {
        log::warn!("dock: failed to start {}: {}", launcher.display(), e);
    }
}

/// The icon is inset so the corner badge sits mostly outside it.
fn icon_with_badge(icon: Image, badge_text: Option<String>, notified: bool) -> Overlay {
    let overlay = Overlay::new();
    icon.set_margin_top(BADGE_INSET);
    icon.set_margin_bottom(BADGE_INSET);
    icon.set_margin_start(BADGE_INSET);
    icon.set_margin_end(BADGE_INSET);
    overlay.set_child(Some(&icon));

    let Some(text) = badge_text else {
        return overlay;
    };
    let badge = Label::new(Some(&text));
    badge.add_css_class("DockBadge");
    if notified {
        badge.add_css_class("notified");
    }
    badge.set_halign(Align::End);
    badge.set_valign(Align::Start);
    badge.set_can_target(false);
    overlay.add_overlay(&badge);
    overlay
}

fn item_button(state: &Rc<DockState>, entry: DockEntry, notified: bool) -> Widget {
    let client = &entry.client;
    let class = client.class.clone();
    let address = client.address.clone();
    let button = Button::new();
    button.add_css_class("DockItem");
    let badge = workspace_badge(&client.workspace);
    button.set_child(Some(&icon_with_badge(icon_image(client), badge, notified)));

    let workspace = if entry.minimized {
        "minimized"
    } else {
        client.workspace.name.as_str()
    };
    button.set_tooltip_text(Some(&format!(
        "{}\n{} · {}",
        client.title, client.class, workspace
    )));
    if entry.minimized {
        button.add_css_class("minimized");
    }

    button.connect_clicked(clone!(
        #[weak]
        state,
        move |_| {
            activate(&entry, &state.connector);
            dismiss_notifications_of(&entry.client.class);
            hide(&state);
        }
    ));
    button.add_controller(click_controller(3, move || close_window(&address)));
    if notified {
        button.add_controller(click_controller(2, move || dismiss_notifications_of(&class)));
    }
    button.upcast()
}

fn click_controller(button: u32, action: impl Fn() + 'static) -> GestureClick {
    let click = GestureClick::builder().button(button).build();
    click.connect_pressed(move |gesture, _, _, _| {
        gesture.set_state(gtk4::EventSequenceState::Claimed);
        action();
    });
    click
}

fn close_window(address: &str) {
    let Some(hyprland) = Hyprland::instance() else {
        return;
    };
    let window = address_arg(address);
    if let Err(e) = hyprland.dispatch(
        &format!("hl.dsp.window.close({{ window = '{}' }})", window),
        &format!("closewindow {}", window),
    ) {
        log::warn!("dock: failed to close {}: {}", window, e);
    }
}

/// The program a process runs. A binary replaced by an upgrade reads as "name (deleted)".
fn process_program(pid: i32) -> Option<String> {
    if pid <= 0 {
        return None;
    }
    let exe = std::fs::read_link(format!("/proc/{pid}/exe")).ok()?;
    let name = exe.file_name()?.to_str()?;
    Some(name.trim_end_matches(" (deleted)").to_string())
}

fn icon_image(client: &Client) -> Image {
    let class = client.class.as_str();
    let theme = Display::default().map(|display| IconTheme::for_display(&display));
    let has_icon = |name: &str| theme.as_ref().is_some_and(|theme| theme.has_icon(name));
    let entry_icon = desktop_entry::lookup_by_window_class(class)
        .or_else(|| {
            process_program(client.pid)
                .and_then(|program| desktop_entry::lookup_by_executable(&program))
        })
        .and_then(|entry| entry.icon);

    let image = match entry_icon {
        Some(path) if path.starts_with('/') => Image::from_file(path),
        Some(name) if has_icon(&name) => Image::from_icon_name(&name),
        _ => {
            let lower = class.to_lowercase();
            let name = [class, lower.as_str()]
                .into_iter()
                .find(|name| !name.is_empty() && has_icon(name))
                .unwrap_or(FALLBACK_ICON);
            Image::from_icon_name(name)
        }
    };
    image.set_pixel_size(ICON_SIZE);
    image
}

/// Minimized windows move to this monitor's active workspace first. Focusing a window
/// elsewhere makes Hyprland switch to its workspace.
fn activate(entry: &DockEntry, connector: &str) {
    let Some(hyprland) = Hyprland::instance() else {
        return;
    };
    let window = address_arg(&entry.client.address);

    if entry.minimized {
        let Some(workspace) = hyprland
            .monitors()
            .into_iter()
            .find(|monitor| monitor.name == connector)
            .map(|monitor| monitor.active_workspace.id)
        else {
            log::warn!("dock: no Hyprland monitor named {}", connector);
            return;
        };

        let moved = hyprland.dispatch(
            &format!(
                "hl.dsp.window.move({{ workspace = '{}', follow = false, window = '{}' }})",
                workspace, window
            ),
            &format!("movetoworkspacesilent {},{}", workspace, window),
        );
        if let Err(e) = moved {
            log::warn!("dock: failed to restore {}: {}", window, e);
            return;
        }
    }

    if let Err(e) = hyprland.dispatch(
        &format!("hl.dsp.focus({{ window = '{}' }})", window),
        &format!("focuswindow {}", window),
    ) {
        log::warn!("dock: failed to focus {}: {}", window, e);
    }
}
