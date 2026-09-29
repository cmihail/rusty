use gtk4::gdk::BUTTON_SECONDARY;
use gtk4::pango::{EllipsizeMode, WrapMode};
use gtk4::prelude::*;
use gtk4::{Align, Box, Button, GestureClick, Label, Orientation, Separator};

use crate::service::hyprland::Hyprland;
use crate::service::notifications::{
    Notification as NotificationData, Notifications, DEFAULT_ACTION,
};

const ON_INVOKE_TIMEOUT_DISMISS: u64 = 100;

fn invoke_notification_action<F>(
    notification_id: u32,
    action_id: &str,
    on_action_invoked: Option<F>,
) where
    F: Fn() + 'static,
{
    let notifications = Notifications::instance();
    if let Some(notification) = notifications.get_notification(notification_id) {
        // Invoke the action via D-Bus
        notifications.invoke_action(notification_id, action_id);

        // A Wayland client cannot raise itself in response to the default action: the
        // click happened in our surface, so it has no activation token. Focus it here.
        if action_id == DEFAULT_ACTION {
            if let Some(hyprland) = Hyprland::instance() {
                hyprland.focus_window_by_class(&notification.window_classes());
            }
        }

        if let Some(handler) = on_action_invoked {
            handler();
        }

        // Dismiss the notification after invoking the action for
        // apps that do not dismiss the notification themselves.
        glib::timeout_add_local_once(
            std::time::Duration::from_millis(ON_INVOKE_TIMEOUT_DISMISS),
            move || {
                // Check if notification still exists before dismissing
                if Notifications::instance()
                    .get_notification(notification_id)
                    .is_some()
                {
                    Notifications::instance().dismiss(notification_id);
                }
            },
        );
    }
}

fn create_header<F, G>(
    notification: &NotificationData,
    notifications_count: usize,
    on_dismiss: F,
    on_dismiss_all: G,
) -> Box
where
    F: Fn() + 'static,
    G: Fn() + 'static,
{
    let header_box = Box::new(Orientation::Horizontal, 0);
    header_box.add_css_class("Header");

    let app_name = Label::new(Some(&notification.display_app_name()));
    app_name.set_halign(Align::Start);
    app_name.set_hexpand(true);
    app_name.set_ellipsize(EllipsizeMode::End);
    header_box.append(&app_name);

    let right_box = Box::new(Orientation::Horizontal, 4);
    right_box.set_halign(Align::End);

    // Time label
    let time_str = glib::DateTime::from_unix_local(notification.time)
        .and_then(|dt| dt.format("%H:%M"))
        .unwrap_or_else(|_| glib::GString::from(""));
    let time_label = Label::new(Some(&time_str));
    right_box.append(&time_label);

    // Count label (only visible if count > 1)
    let count_label = Label::new(Some(&notifications_count.to_string()));
    count_label.set_visible(notifications_count > 1);
    right_box.append(&count_label);

    // Dismiss all button (only visible if count > 1)
    let dismiss_all_btn = Button::new();
    dismiss_all_btn.set_icon_name("edit-clear-all-symbolic");
    dismiss_all_btn.set_visible(notifications_count > 1);
    dismiss_all_btn.connect_clicked(move |_| {
        on_dismiss_all();
    });
    right_box.append(&dismiss_all_btn);

    // Close button
    let close_btn = Button::new();
    close_btn.set_icon_name("window-close-symbolic");
    close_btn.connect_clicked(move |_| {
        on_dismiss();
    });
    right_box.append(&close_btn);

    header_box.append(&right_box);
    header_box
}

fn create_contents(notification: &NotificationData) -> Box {
    let contents_box = Box::new(Orientation::Vertical, 0);

    // Summary
    let summary_text = glib::markup_escape_text(&notification.summary);
    let summary_label = Label::new(Some(&summary_text));
    summary_label.add_css_class("Summary");
    summary_label.set_xalign(0.0);
    summary_label.set_use_markup(true);
    summary_label.set_max_width_chars(20);
    summary_label.set_ellipsize(EllipsizeMode::End);
    summary_label.set_justify(gtk4::Justification::Left);
    contents_box.append(&summary_label);

    // Body (if present)
    if !notification.body.is_empty() {
        let body_text = glib::markup_escape_text(&notification.body);
        let body_label = Label::new(Some(&body_text));
        body_label.add_css_class("Body");
        body_label.set_xalign(0.0);
        body_label.set_hexpand(true);
        body_label.set_wrap(true);
        body_label.set_use_markup(true);
        body_label.set_lines(3);
        body_label.set_max_width_chars(20);
        body_label.set_ellipsize(EllipsizeMode::End);
        body_label.set_wrap_mode(WrapMode::WordChar);
        body_label.set_justify(gtk4::Justification::Left);
        contents_box.append(&body_label);
    }

    contents_box
}

fn create_actions<F>(notification: &NotificationData, on_action_invoked: Option<F>) -> Box
where
    F: Fn() + Clone + 'static,
{
    let actions_box = Box::new(Orientation::Horizontal, 8);

    for action in notification.button_actions() {
        let action_btn = Button::new();
        action_btn.add_css_class("Action");
        action_btn.set_hexpand(false);

        let action_label = Label::new(Some(&action.label));
        action_label.set_halign(Align::Center);
        action_label.set_hexpand(true);
        action_btn.set_child(Some(&action_label));

        let notification_id = notification.id;
        let action_id = action.id.clone();
        let on_action_invoked_clone = on_action_invoked.clone();
        action_btn.connect_clicked(move |_| {
            invoke_notification_action(
                notification_id,
                &action_id,
                on_action_invoked_clone.clone(),
            );
        });

        actions_box.append(&action_btn);
    }

    actions_box
}

pub struct Notification {
    widget: Box,
}

impl Notification {
    pub fn new<F, G, H, I>(
        notification: &NotificationData,
        notifications_count: usize,
        on_action_invoked: Option<F>,
        on_hover_lost: Option<G>,
        on_dismiss: H,
        on_dismiss_all: I,
    ) -> Self
    where
        F: Fn() + Clone + 'static,
        G: Fn(u32) + 'static,
        H: Fn() + Clone + 'static,
        I: Fn() + 'static,
    {
        let container = Box::new(Orientation::Vertical, 0);
        container.add_css_class("Notification");

        // Add urgency CSS class
        use crate::service::notifications::Urgency;
        match notification.urgency {
            Urgency::Critical => container.add_css_class("Critical"),
            Urgency::Low => container.add_css_class("Low"),
            Urgency::Normal => {}
        }

        // Header
        let header = create_header(
            notification,
            notifications_count,
            on_dismiss.clone(),
            on_dismiss_all,
        );
        container.append(&header);

        // Separator
        let separator = Separator::new(Orientation::Horizontal);
        separator.set_visible(true);
        container.append(&separator);

        // Contents
        let contents = create_contents(notification);
        container.append(&contents);

        // Actions
        let actions = create_actions(notification, on_action_invoked.clone());
        container.append(&actions);

        // Handle hover lost event
        if let Some(hover_handler) = on_hover_lost {
            let motion_controller = gtk4::EventControllerMotion::new();
            let notification_id = notification.id;
            motion_controller.connect_leave(move |_| {
                hover_handler(notification_id);
            });
            container.add_controller(motion_controller);
        }

        // Handle primary click to invoke the default action, or the first one if the
        // client did not send a default.
        let left_click = GestureClick::new();
        left_click.set_button(gtk4::gdk::BUTTON_PRIMARY);
        let notification_id = notification.id;
        let click_action = notification.click_action().map(|a| a.id.clone());
        left_click.connect_released(move |_, _, _, _| {
            if let Some(ref action_id) = click_action {
                invoke_notification_action(notification_id, action_id, on_action_invoked.clone());
            }
        });
        container.add_controller(left_click);

        // Handle right click to dismiss
        let right_click = GestureClick::new();
        right_click.set_button(BUTTON_SECONDARY);
        right_click.connect_released(move |_, _, _, _| {
            on_dismiss();
        });
        container.add_controller(right_click);

        Self { widget: container }
    }

    pub fn widget(&self) -> &Box {
        &self.widget
    }
}
