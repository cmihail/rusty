use glib::clone;
use gtk4::gdk::{ContentProvider, Display, DragAction, Monitor};
use gtk4::prelude::*;
use gtk4::{Align, DrawingArea, Orientation, Overlay, Separator, Widget};
use gtk4::{ApplicationWindow, Box, Button, EventControllerMotion, GestureClick, IconTheme};
use gtk4::{DragSource, DropTarget, Image, Label, Popover, PositionType, WidgetPaintable};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;
use std::rc::Rc;
use std::time::Duration;

use crate::config::{moved_pin, toggled_pin, Config};
use crate::service::desktop_entry;
use crate::service::hyprland::{address_arg, Client, Hyprland, Workspace};
use crate::service::network::{Network, NetworkState};
use crate::service::notifications::Notifications;

pub const MINIMIZED_WORKSPACE: &str = "special:minimized";

const TRIGGER_LENGTH: i32 = 400;
const HIDE_DELAY: Duration = Duration::from_millis(300);
const POPOVER_DELAY: Duration = Duration::from_millis(400);
// Longer than HIDE_DELAY because the pointer only "enters" the dock once it moves again.
const SHOW_GRACE: Duration = Duration::from_millis(1500);
const ICON_SIZE: i32 = 32;
const FALLBACK_ICON: &str = "application-x-executable";
const LAUNCHER: &str = ".local/bin/rusty-universal-search-wrapped";
const LAUNCHER_ICONS: &[&str] = &["view-app-grid-symbolic", "system-search-symbolic"];
const RESIZE_EVENTS: &[&str] = &["openwindow", "closewindow", "movewindowv2"];
const LAYOUT_EVENTS: &[&str] = &["workspacev2", "moveworkspacev2", "configreloaded"];
pub const SCROLLING_LAYOUT: &str = "scrolling";
const FALLBACK_TILING_LAYOUT: &str = "dwindle";
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

/// A pinned app: its first window in dock order, or its desktop id while it has none.
#[derive(Debug, Clone, PartialEq)]
pub enum PinnedSlot {
    Running(DockEntry),
    Idle(String),
}

/// Splits the dock entries into the pinned slots, in pin order, and the other windows. A pin
/// takes only the first window of its app, the rest stay with the other windows.
pub fn pin_entries(
    entries: Vec<DockEntry>,
    pinned: &[String],
    app_id: impl Fn(&Client) -> Option<String>,
) -> (Vec<PinnedSlot>, Vec<DockEntry>) {
    let ids: Vec<Option<String>> = entries.iter().map(|entry| app_id(&entry.client)).collect();
    let mut remaining: Vec<Option<DockEntry>> = entries.into_iter().map(Some).collect();
    let mut seen = HashSet::new();
    let slots = pinned
        .iter()
        .filter(|pin| seen.insert(pin.as_str()))
        .map(|pin| {
            ids.iter()
                .position(|id| id.as_deref() == Some(pin.as_str()))
                .and_then(|index| remaining[index].take())
                .map_or_else(|| PinnedSlot::Idle(pin.clone()), PinnedSlot::Running)
        })
        .collect();
    (slots, remaining.into_iter().flatten().collect())
}

/// Whether `cursor` is in the left-center area covered by the dock and its hover strip.
/// `monitor` is (x, y, width, height) and `dock_size` is (width, height), in layout pixels.
pub fn in_dock_area(
    cursor: (i32, i32),
    monitor: (i32, i32, i32, i32),
    dock_size: (i32, i32),
) -> bool {
    let (x, y, _, height) = monitor;
    let area_height = dock_size.1.max(TRIGGER_LENGTH);
    let top = y + (height - area_height) / 2;
    let bottom = top + area_height;
    let right = x + dock_size.0.max(1);
    (x..right).contains(&cursor.0) && (top..bottom).contains(&cursor.1)
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

/// The layout a workspace switches to: scrolling from tiling, and back to the default tiling
/// layout, or dwindle when scrolling is the default.
pub fn toggled_layout(current: &str, default: &str) -> String {
    if current != SCROLLING_LAYOUT {
        SCROLLING_LAYOUT.to_string()
    } else if default != SCROLLING_LAYOUT {
        default.to_string()
    } else {
        FALLBACK_TILING_LAYOUT.to_string()
    }
}

pub fn layout_icon(layout: &str) -> &'static str {
    if layout == SCROLLING_LAYOUT {
        "workspace-scrolling-symbolic"
    } else {
        "workspace-tiling-symbolic"
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
    popover: RefCell<Option<Popover>>,
    popover_generation: Cell<u64>,
    rebuild_pending: Cell<bool>,
    dragging: Cell<bool>,
    pinned_end: RefCell<Option<Widget>>,
    layout_toggle: RefCell<Option<Button>>,
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
            popover: RefCell::new(None),
            popover_generation: Cell::new(0),
            rebuild_pending: Cell::new(false),
            dragging: Cell::new(false),
            pinned_end: RefCell::new(None),
            layout_toggle: RefCell::new(None),
        });

        let trigger = Self::create_trigger(app, monitor, &state);
        rebuild(&state);
        Self::connect_hover(&state);
        Self::connect_hyprland_events(&state);
        Self::connect_notifications(&state);
        Self::connect_network(&state);
        Self::connect_config(&state);
        connect_unpin_drop(&state);

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
        window.set_anchor(Edge::Left, true);
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
            move |_, x, y| {
                log::debug!("dock: trigger enter at ({:.1}, {:.1})", x, y);
                show(&state)
            }
        ));
        // The trigger is unmapped with the pointer still on it, so GTK misses the leave and
        // reports no enter on the next visit. Motion still arrives.
        motion.connect_motion(clone!(
            #[weak]
            state,
            move |_, x, y| {
                log::debug!("dock: trigger motion ({:.1}, {:.1})", x, y);
                if !state.window.is_visible() {
                    show(&state);
                }
            }
        ));
        motion.connect_leave(|_| log::debug!("dock: trigger leave"));
        trigger.add_controller(motion);
        trigger.connect_map(|_| log::debug!("dock: trigger mapped"));
        trigger.connect_unmap(|_| log::debug!("dock: trigger unmapped"));
        debug_watch_edge(state);
        WidgetExt::realize(&trigger);
        trigger.set_visible(true);
        trigger
    }

    fn connect_hover(state: &Rc<DockState>) {
        let motion = EventControllerMotion::new();
        motion.connect_enter(clone!(
            #[weak]
            state,
            move |_, _, _| {
                log::debug!("dock: dock enter");
                cancel_hide(&state)
            }
        ));
        motion.connect_leave(clone!(
            #[weak]
            state,
            move |_| {
                log::debug!("dock: dock leave");
                schedule_hide(&state, HIDE_DELAY)
            }
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

    fn connect_config(state: &Rc<DockState>) {
        let weak = Rc::downgrade(state);
        Config::instance().connect_notify_local(Some("dock-pinned"), move |_, _| {
            if let Some(state) = weak.upgrade() {
                rebuild(&state);
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
            if LAYOUT_EVENTS.contains(&event.as_str()) && state.window.is_visible() {
                update_layout_toggle(&state);
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
    // The curve is built for a bottom handle, so swapping and mirroring the axes stands it
    // on the left edge.
    let (width, height) = (height as f64, width as f64);
    let [start, first, second, end] = handle_curve(width, height);
    cr.transform(gtk4::cairo::Matrix::new(0.0, 1.0, -1.0, 0.0, height, 0.0));
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
    log::debug!("dock: show, cursor {:?}", debug_cursor());
    rebuild(state);
    state.window.present();
    set_trigger_visible(state, false);
    schedule_hide(state, SHOW_GRACE);
}

fn hide(state: &Rc<DockState>) {
    log::debug!("dock: hide, cursor {:?}", debug_cursor());
    cancel_hide(state);
    dismiss_popover(state);
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
        let busy = state.held.get() || state.popover.borrow().is_some() || state.dragging.get();
        if state.hide_generation.get() != generation || busy {
            log::debug!("dock: hide skipped, stale or busy (busy={})", busy);
            return;
        }
        if pointer_in_dock_area(&state) {
            log::debug!(
                "dock: hide deferred, cursor {:?} in dock area",
                debug_cursor()
            );
            schedule_hide(&state, HIDE_DELAY);
        } else {
            hide(&state);
        }
    });
}

fn debug_cursor() -> Option<(i32, i32)> {
    Hyprland::instance().and_then(|hyprland| hyprland.cursor_position())
}

/// Debug only: logs when the cursor rests on the handle strip while the dock stays hidden.
fn debug_watch_edge(state: &Rc<DockState>) {
    if !log::log_enabled!(log::Level::Debug) {
        return;
    }
    let weak = Rc::downgrade(state);
    let reported = Cell::new(false);
    glib::timeout_add_local(Duration::from_millis(50), move || {
        let Some(state) = weak.upgrade() else {
            return glib::ControlFlow::Break;
        };
        let geometry = state.monitor.geometry();
        let on_strip = debug_cursor().filter(|&(x, y)| {
            let left = geometry.x();
            let top = geometry.y() + (geometry.height() - state.handle.height()) / 2;
            x >= left
                && x < left + HANDLE_THICKNESS
                && (top..top + state.handle.height()).contains(&y)
        });
        let missed = on_strip.is_some() && !state.window.is_visible();
        if missed && !reported.replace(true) {
            log::debug!(
                "dock: MISSED cursor {:?} on strip, dock hidden, trigger mapped={}",
                on_strip,
                state.handle.is_mapped()
            );
        } else if !missed {
            reported.set(false);
        }
        glib::ControlFlow::Continue
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

/// The status widget stays in place so the control center open on it is not closed. An open
/// popover's item has to stay too, so the rebuild waits for the popover to close. A dragged
/// item has to stay until the drag ends.
fn rebuild(state: &Rc<DockState>) {
    if state.popover.borrow().is_some() || state.dragging.get() {
        state.rebuild_pending.set(true);
        return;
    }
    clear_items(state);

    let clients = Hyprland::instance()
        .map(|hyprland| hyprland.clients())
        .unwrap_or_default();
    let pinned = Config::instance().dock_pinned();
    let (slots, windows) = pin_entries(dock_entries(clients), &pinned, |client| {
        client_app(client).map(|(id, _)| id)
    });
    let notified_classes = notification_classes();

    state.items.append(&launcher_button(state));
    state.items.append(&layout_toggle_button(state));
    append_pinned(state, slots, &notified_classes);
    append_windows(state, windows, &notified_classes);
    place_status(state);
}

fn append_pinned(state: &Rc<DockState>, slots: Vec<PinnedSlot>, notified_classes: &[String]) {
    if !slots.is_empty() {
        state.items.append(&Separator::new(Orientation::Horizontal));
    }
    let mut last = None;
    for slot in slots {
        let (item, id) = match slot {
            PinnedSlot::Running(entry) => {
                let id = client_app(&entry.client).map(|(id, _)| id);
                let notified = is_notified(&entry.client.class, notified_classes);
                (item_button(state, entry, notified), id)
            }
            PinnedSlot::Idle(id) => (idle_button(state, id.clone()), Some(id)),
        };
        if let Some(id) = id {
            connect_pin_drag(state, &item, &id);
        }
        state.items.append(&item);
        last = Some(item);
    }
    state.pinned_end.replace(last);
}

fn clear_items(state: &DockState) {
    let mut child = state.items.first_child();
    while let Some(widget) = child {
        child = widget.next_sibling();
        if widget != state.status && widget != *state.status_separator.upcast_ref::<Widget>() {
            state.items.remove(&widget);
        }
    }
}

fn append_windows(state: &Rc<DockState>, windows: Vec<DockEntry>, notified_classes: &[String]) {
    let mut previous_minimized = false;
    for (index, entry) in windows.into_iter().enumerate() {
        if index == 0 || (entry.minimized && !previous_minimized) {
            state.items.append(&Separator::new(Orientation::Horizontal));
        }
        previous_minimized = entry.minimized;
        let notified = is_notified(&entry.client.class, notified_classes);
        state.items.append(&item_button(state, entry, notified));
    }
}

fn place_status(state: &DockState) {
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

fn layout_toggle_button(state: &Rc<DockState>) -> Button {
    let button = Button::new();
    button.add_css_class("DockItem");
    button.add_css_class("DockLayoutToggle");
    button.connect_clicked(clone!(
        #[weak]
        state,
        move |_| {
            toggle_layout(&state);
            update_layout_toggle(&state);
        }
    ));
    state.layout_toggle.replace(Some(button.clone()));
    update_layout_toggle(state);
    button
}

/// The active workspace of the dock's monitor. Named workspaces have no number to badge or
/// to match a workspace rule by, so they are left out.
fn active_workspace(state: &DockState) -> Option<Workspace> {
    Hyprland::instance()?
        .monitors()
        .into_iter()
        .find(|monitor| monitor.name == state.connector)
        .map(|monitor| monitor.active_workspace)
        .filter(|workspace| workspace.id > 0)
}

fn update_layout_toggle(state: &DockState) {
    let Some(button) = state.layout_toggle.borrow().clone() else {
        return;
    };
    let workspace = active_workspace(state);
    let layout = workspace.as_ref().and_then(|workspace| {
        Hyprland::instance().and_then(|hyprland| hyprland.workspace_layout(workspace.id))
    });
    let (Some(workspace), Some(layout)) = (workspace, layout) else {
        button.set_visible(false);
        return;
    };
    let image = Image::from_icon_name(layout_icon(&layout));
    image.set_pixel_size(ICON_SIZE);
    button.set_child(Some(&icon_with_badge(
        image,
        workspace_badge(&workspace),
        false,
    )));
    button.set_tooltip_text(Some(&format!(
        "Workspace {}: {} layout",
        workspace.id, layout
    )));
    button.set_visible(true);
}

fn toggle_layout(state: &DockState) {
    let (Some(hyprland), Some(workspace)) = (Hyprland::instance(), active_workspace(state)) else {
        return;
    };
    let Some(current) = hyprland.workspace_layout(workspace.id) else {
        return;
    };
    let default = hyprland
        .default_layout()
        .unwrap_or_else(|| FALLBACK_TILING_LAYOUT.to_string());
    let layout = toggled_layout(&current, &default);
    if let Err(e) = hyprland.set_workspace_layout(workspace.id, &layout) {
        log::warn!(
            "dock: failed to set workspace {} to {}: {}",
            workspace.id,
            layout,
            e
        );
    }
}

fn launch() {
    let Some(home) = std::env::var_os("HOME") else {
        return;
    };
    run_in_scope(Path::new(&home).join(LAUNCHER).as_os_str(), &[], &[]);
}

fn launch_app(id: &str) {
    let current = std::env::var("XDG_DATA_DIRS").ok();
    let data_dirs = desktop_entry::launch_data_dirs(current.as_deref());
    let env = [("XDG_DATA_DIRS", data_dirs.as_str())];
    run_in_scope(OsStr::new("gtk-launch"), &["--", id], &env);
}

/// Runs in its own scope so what it starts outlives a rusty-de restart.
fn run_in_scope(program: &OsStr, args: &[&str], env: &[(&str, &str)]) {
    let spawned = Command::new("systemd-run")
        .args(["--user", "--scope", "--"])
        .arg(program)
        .args(args)
        .envs(env.iter().copied())
        .spawn();
    match spawned {
        Ok(mut child) => {
            std::thread::spawn(move || child.wait());
        }
        Err(e) => log::warn!("dock: failed to start {}: {}", program.to_string_lossy(), e),
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
    let button = Button::new();
    button.add_css_class("DockItem");
    let badge = workspace_badge(&client.workspace);
    button.set_child(Some(&icon_with_badge(icon_image(client), badge, notified)));
    if entry.minimized {
        button.add_css_class("minimized");
    }

    let details = window_details(client);
    let menu_entry = entry.clone();
    connect_popover(state, &button, details, move |state| {
        window_menu(state, &menu_entry, notified)
    });
    button.connect_clicked(clone!(
        #[weak]
        state,
        move |_| {
            activate(&entry, &state.connector);
            dismiss_notifications_of(&entry.client.class);
            hide(&state);
        }
    ));
    if notified {
        let dismiss = move || dismiss_notifications_of(&class);
        button.add_controller(click_controller(state, 2, dismiss));
    }
    button.upcast()
}

/// A pinned app with no window, launched on click.
fn idle_button(state: &Rc<DockState>, id: String) -> Widget {
    let app = desktop_entry::lookup(&id);
    let name = app.as_ref().and_then(|app| app.name.clone());
    let icon = app_icon(app.and_then(|app| app.icon), &[&id]);
    let button = Button::new();
    button.add_css_class("DockItem");
    button.add_css_class("idle");
    button.set_child(Some(&icon_with_badge(icon, None, false)));

    let details = (
        name.unwrap_or_else(|| id.clone()),
        "Not running".to_string(),
    );
    let menu_id = id.clone();
    connect_popover(state, &button, details, move |state| {
        idle_menu(state, &menu_id)
    });
    button.connect_clicked(clone!(
        #[weak]
        state,
        move |_| {
            hide(&state);
            launch_app(&id);
        }
    ));
    button.upcast()
}

/// The app's name over the window title, or the class when the app has no desktop entry.
fn window_details(client: &Client) -> (String, String) {
    let name = client_app(client)
        .and_then(|(_, app)| app.name)
        .unwrap_or_else(|| client.class.clone());
    (name, client.title.clone())
}

fn details_view((name, detail): &(String, String)) -> Widget {
    let view = Box::new(Orientation::Vertical, 2);
    let title = Label::new(Some(name));
    title.add_css_class("DockDetailsName");
    let subtitle = Label::new(Some(detail));
    subtitle.add_css_class("dim-label");
    subtitle.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    subtitle.set_max_width_chars(40);
    for label in [&title, &subtitle] {
        label.set_halign(Align::Start);
        view.append(label);
    }
    view.upcast()
}

fn window_menu(state: &Rc<DockState>, entry: &DockEntry, notified: bool) -> Widget {
    let client = &entry.client;
    let mut actions = Vec::new();
    if let Some((id, _)) = client_app(client) {
        actions.push(pin_action(state, &id));
    }
    let address = client.address.clone();
    actions.push(menu_action(
        state,
        ("window-close-symbolic", "Close window"),
        false,
        move || close_window(&address),
    ));
    if notified {
        let class = client.class.clone();
        actions.push(menu_action(
            state,
            ("edit-clear-all-symbolic", "Dismiss notifications"),
            false,
            move || dismiss_notifications_of(&class),
        ));
    }
    menu(&actions)
}

fn idle_menu(state: &Rc<DockState>, id: &str) -> Widget {
    let launch_id = id.to_string();
    let launch = menu_action(
        state,
        ("media-playback-start-symbolic", "Launch"),
        true,
        move || launch_app(&launch_id),
    );
    menu(&[launch, pin_action(state, id)])
}

/// A filled, accented pin while the app is pinned, an outlined one otherwise.
fn pin_action(state: &Rc<DockState>, id: &str) -> Button {
    let pinned = Config::instance().dock_pinned().iter().any(|pin| pin == id);
    let tooltip = if pinned {
        "Unpin from dock"
    } else {
        "Pin to dock"
    };
    let id = id.to_string();
    let icon = if pinned {
        "dock-pinned-symbolic"
    } else {
        "dock-unpinned-symbolic"
    };
    let button = menu_action(state, (icon, tooltip), false, move || toggle_pin(&id));
    if pinned {
        button.add_css_class("pinned");
    }
    button
}

fn toggle_pin(id: &str) {
    save_pins(toggled_pin(Config::instance().dock_pinned(), id));
}

fn save_pins(pinned: Vec<String>) {
    if let Err(e) = Config::instance().set_dock_pinned(pinned) {
        log::warn!("dock: failed to save the pinned apps: {}", e);
    }
}

/// A pinned item is dropped on another pinned item to move next to it, or below the pinned
/// section to unpin.
fn connect_pin_drag(state: &Rc<DockState>, item: &Widget, id: &str) {
    item.add_controller(pin_drag_source(state, item, id));
    item.add_controller(pin_drop_target(id));
}

fn pin_drag_source(state: &Rc<DockState>, item: &Widget, id: &str) -> DragSource {
    let source = DragSource::builder().actions(DragAction::MOVE).build();
    source.set_content(Some(&ContentProvider::for_value(&id.to_value())));
    let paintable = WidgetPaintable::new(Some(item));
    source.connect_drag_begin(clone!(
        #[weak]
        state,
        #[weak]
        item,
        move |source, _| {
            state.dragging.set(true);
            dismiss_popover(&state);
            cancel_hide(&state);
            source.set_icon(Some(&paintable), item.width() / 2, item.height() / 2);
        }
    ));
    source.connect_drag_end(clone!(
        #[weak]
        state,
        move |_, _, _| {
            state.dragging.set(false);
            if state.rebuild_pending.replace(false) {
                rebuild(&state);
            }
            schedule_hide(&state, HIDE_DELAY);
        }
    ));
    source
}

fn pin_drop_target(id: &str) -> DropTarget {
    let target = DropTarget::new(glib::Type::STRING, DragAction::MOVE);
    target.connect_motion(|target, _, y| {
        mark_drop_side(target, Some(drops_after(target, y)));
        DragAction::MOVE
    });
    target.connect_leave(|target| mark_drop_side(target, None));
    let id = id.to_string();
    target.connect_drop(move |target, value, _, y| {
        mark_drop_side(target, None);
        let Ok(dragged) = value.get::<String>() else {
            return false;
        };
        let after = drops_after(target, y);
        save_pins(moved_pin(
            Config::instance().dock_pinned(),
            &dragged,
            &id,
            after,
        ));
        true
    });
    target
}

fn drops_after(target: &DropTarget, y: f64) -> bool {
    target
        .widget()
        .is_some_and(|item| y > f64::from(item.height()) / 2.0)
}

fn mark_drop_side(target: &DropTarget, after: Option<bool>) {
    let Some(item) = target.widget() else {
        return;
    };
    item.remove_css_class("drop-before");
    item.remove_css_class("drop-after");
    match after {
        Some(true) => item.add_css_class("drop-after"),
        Some(false) => item.add_css_class("drop-before"),
        None => {}
    }
}

/// Drops the pinned items' own targets do not take land here, and unpin below the pinned
/// section.
fn connect_unpin_drop(state: &Rc<DockState>) {
    let target = DropTarget::new(glib::Type::STRING, DragAction::MOVE);
    let weak = Rc::downgrade(state);
    target.connect_drop(move |_, value, _, y| {
        let (Some(state), Ok(id)) = (weak.upgrade(), value.get::<String>()) else {
            return false;
        };
        if !below_pinned(&state, y) {
            return false;
        }
        let mut pinned = Config::instance().dock_pinned();
        pinned.retain(|pin| *pin != id);
        save_pins(pinned);
        true
    });
    state.items.add_controller(target);
}

fn below_pinned(state: &DockState, y: f64) -> bool {
    let last = state.pinned_end.borrow();
    let Some(bounds) = last
        .as_ref()
        .and_then(|last| last.compute_bounds(&state.items))
    else {
        return false;
    };
    y > f64::from(bounds.y() + bounds.height())
}

fn menu(actions: &[Button]) -> Widget {
    let menu = Box::new(Orientation::Vertical, 8);
    for action in actions {
        menu.append(action);
    }
    menu.upcast()
}

/// Actions that start an app hide the dock like the launcher does.
fn menu_action(
    state: &Rc<DockState>,
    (icon, tooltip): (&str, &str),
    hides: bool,
    action: impl Fn() + 'static,
) -> Button {
    let button = Button::from_icon_name(icon);
    button.add_css_class("flat");
    button.add_css_class("circular");
    button.set_tooltip_text(Some(tooltip));
    let action = Rc::new(action);
    let weak = Rc::downgrade(state);
    // Deferred because closing the popover destroys this button.
    button.connect_clicked(move |_| {
        let (weak, action) = (weak.clone(), action.clone());
        glib::idle_add_local_once(move || {
            let Some(state) = weak.upgrade() else {
                return;
            };
            close_popover(&state);
            action();
            if hides {
                hide(&state);
            }
        });
    });
    button
}

/// Opens `menu` beside `button` once the pointer rests on it, or at once while another
/// item's popover is open. The popover is a child of `button`, so its events reach it too.
/// Hovering shows the app's details, a right click its actions.
fn connect_popover(
    state: &Rc<DockState>,
    button: &Button,
    details: (String, String),
    menu: impl Fn(&Rc<DockState>) -> Widget + 'static,
) {
    connect_details(state, button, details);
    let right_click = GestureClick::builder().button(3).build();
    let weak = Rc::downgrade(state);
    let target = button.downgrade();
    right_click.connect_pressed(move |gesture, _, _, _| {
        gesture.set_state(gtk4::EventSequenceState::Claimed);
        if let (Some(state), Some(button)) = (weak.upgrade(), target.upgrade()) {
            cancel_popover_timer(&state);
            open_popover(&state, &button, menu(&state), true);
        }
    });
    button.add_controller(right_click);
}

fn connect_details(state: &Rc<DockState>, button: &Button, details: (String, String)) {
    let motion = EventControllerMotion::new();
    let weak = Rc::downgrade(state);
    let target = button.downgrade();
    motion.connect_enter(move |_, _, _| {
        let (Some(state), Some(button)) = (weak.upgrade(), target.upgrade()) else {
            return;
        };
        if owns_popover(&state, &button) {
            return cancel_popover_timer(&state);
        }
        let delay = if state.popover.borrow().is_some() {
            Duration::ZERO
        } else {
            POPOVER_DELAY
        };
        let view = details_view(&details);
        schedule_popover(&state, delay, move |state| {
            open_popover(state, &button, view, false)
        });
    });
    motion.connect_motion(clone!(
        #[weak]
        state,
        #[weak]
        button,
        move |_, _, _| {
            if owns_popover(&state, &button) {
                cancel_popover_timer(&state);
            }
        }
    ));
    motion.connect_leave(clone!(
        #[weak]
        state,
        move |_| schedule_popover(&state, HIDE_DELAY, close_popover)
    ));
    button.add_controller(motion);
}

fn owns_popover(state: &DockState, button: &Button) -> bool {
    state
        .popover
        .borrow()
        .as_ref()
        .and_then(|popover| popover.parent())
        .is_some_and(|parent| parent == *button.upcast_ref::<Widget>())
}

/// Without autohide the popover takes no grab, which a dock without keyboard focus needs.
/// Details are not targetable, so the pointer moving onto them closes them like leaving.
fn open_popover(state: &Rc<DockState>, button: &Button, content: Widget, interactive: bool) {
    take_popover(state);
    if button.root().is_none() || state.dragging.get() {
        return;
    }
    let popover = Popover::builder()
        .child(&content)
        .can_target(interactive)
        .autohide(false)
        .has_arrow(true)
        .position(PositionType::Right)
        .build();
    popover.add_css_class("DockMenu");

    let motion = EventControllerMotion::new();
    motion.connect_enter(clone!(
        #[weak]
        state,
        move |_, _, _| cancel_popover_timer(&state)
    ));
    motion.connect_leave(clone!(
        #[weak]
        state,
        move |_| schedule_popover(&state, HIDE_DELAY, close_popover)
    ));
    popover.add_controller(motion);
    popover.set_parent(button);
    popover.popup();
    state.popover.replace(Some(popover));
    cancel_hide(state);
}

fn schedule_popover(
    state: &Rc<DockState>,
    delay: Duration,
    action: impl FnOnce(&Rc<DockState>) + 'static,
) {
    cancel_popover_timer(state);
    let generation = state.popover_generation.get();
    let weak = Rc::downgrade(state);
    glib::timeout_add_local_once(delay, move || {
        let Some(state) = weak.upgrade() else {
            return;
        };
        if state.popover_generation.get() == generation {
            action(&state);
        }
    });
}

fn cancel_popover_timer(state: &DockState) {
    state
        .popover_generation
        .set(state.popover_generation.get() + 1);
}

/// Closes the popover and lets the dock hide once the pointer is away from it.
fn close_popover(state: &Rc<DockState>) {
    if dismiss_popover(state) {
        schedule_hide(state, HIDE_DELAY);
    }
}

/// Closes the popover and runs the rebuild it held back. Returns whether one was open.
fn dismiss_popover(state: &Rc<DockState>) -> bool {
    cancel_popover_timer(state);
    let open = take_popover(state);
    if state.rebuild_pending.replace(false) {
        rebuild(state);
    }
    open
}

fn take_popover(state: &DockState) -> bool {
    let Some(popover) = state.popover.take() else {
        return false;
    };
    popover.popdown();
    popover.unparent();
    true
}

fn click_controller(
    state: &Rc<DockState>,
    button: u32,
    action: impl Fn() + 'static,
) -> GestureClick {
    let click = GestureClick::builder().button(button).build();
    let weak = Rc::downgrade(state);
    click.connect_pressed(move |gesture, _, _, _| {
        gesture.set_state(gtk4::EventSequenceState::Claimed);
        if let Some(state) = weak.upgrade() {
            dismiss_popover(&state);
        }
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
    let lower = class.to_lowercase();
    let entry_icon = client_app(client).and_then(|(_, entry)| entry.icon);
    app_icon(entry_icon, &[class, lower.as_str()])
}

/// The desktop entry's icon, else the first of `fallbacks` the theme has.
fn app_icon(entry_icon: Option<String>, fallbacks: &[&str]) -> Image {
    let theme = Display::default().map(|display| IconTheme::for_display(&display));
    let has_icon = |name: &str| theme.as_ref().is_some_and(|theme| theme.has_icon(name));
    let image = match entry_icon {
        Some(path) if path.starts_with('/') => Image::from_file(path),
        Some(name) if has_icon(&name) => Image::from_icon_name(&name),
        _ => {
            let name = fallbacks
                .iter()
                .copied()
                .find(|name| !name.is_empty() && has_icon(name))
                .unwrap_or(FALLBACK_ICON);
            Image::from_icon_name(name)
        }
    };
    image.set_pixel_size(ICON_SIZE);
    image
}

/// The app a window belongs to, by its class or else by the program it runs.
fn client_app(client: &Client) -> Option<desktop_entry::Resolved> {
    desktop_entry::lookup_by_window_class(&client.class).or_else(|| {
        process_program(client.pid)
            .and_then(|program| desktop_entry::lookup_by_executable(&program))
    })
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
