use glib;
use gtk4::prelude::*;
use gtk4::{EventControllerScroll, EventControllerScrollFlags, Stack, StackTransitionType};
use std::cell::RefCell;
use std::rc::Rc;

use crate::service::notifications::{Notification as NotificationData, Notifications};
use crate::widget::notification::Notification;

const ANIMATION_DURATION_MS: u32 = 500;

pub struct NotificationGroup {
    widget: Stack,
    notifications: Rc<RefCell<Vec<NotificationData>>>,
}

impl NotificationGroup {
    pub fn new<F, G>(
        _key: &str,
        notifications: Vec<NotificationData>,
        on_action_invoked: Option<F>,
        on_hover_lost: Option<G>,
    ) -> Self
    where
        F: Fn() + Clone + 'static,
        G: Fn(u32) + Clone + 'static,
    {
        let stack = Stack::new();
        stack.set_transition_type(StackTransitionType::SlideUp);
        stack.set_transition_duration(ANIMATION_DURATION_MS);

        let notifications_rc = Rc::new(RefCell::new(notifications.clone()));

        // Add scroll controller to switch between notifications
        let scroll_controller = EventControllerScroll::new(EventControllerScrollFlags::VERTICAL);
        let stack_weak = stack.downgrade();
        let notifications_clone = notifications_rc.clone();
        scroll_controller.connect_scroll(move |_, _, dy| {
            if let Some(stack) = stack_weak.upgrade() {
                let visible_child_name = stack.visible_child_name();
                if let Some(name) = visible_child_name {
                    let current_id = name.as_str().parse::<u32>().unwrap_or(0);

                    // Scroll down (positive dy) moves to next, scroll up (negative dy) to previous
                    if dy > 0.0 {
                        // Scroll down: slide up animation
                        stack.set_transition_type(StackTransitionType::SlideUp);
                        Self::switch_after(current_id, &notifications_clone.borrow(), &stack);
                    } else if dy < 0.0 {
                        // Scroll up: slide down animation
                        stack.set_transition_type(StackTransitionType::SlideDown);
                        Self::switch_before(current_id, &notifications_clone.borrow(), &stack);
                    }
                }
            }
            glib::Propagation::Stop
        });
        stack.add_controller(scroll_controller);

        // Add notifications to stack
        for notification in &notifications {
            Self::add_notification_to_stack(
                &stack,
                notification,
                &notifications_rc,
                on_action_invoked.clone(),
                on_hover_lost.clone(),
            );
        }

        // Set first notification as visible
        if let Some(first) = notifications.first() {
            stack.set_visible_child_name(&first.id.to_string());
        }

        Self {
            widget: stack,
            notifications: notifications_rc,
        }
    }

    fn switch_after(id: u32, notifications: &[NotificationData], stack: &Stack) -> bool {
        // If only one notification, there's no next notification to switch to
        if notifications.len() <= 1 {
            return false;
        }

        let mut next: Option<&NotificationData> = None;

        for (i, notification) in notifications.iter().enumerate() {
            if notification.id != id {
                continue;
            }

            if i + 1 < notifications.len() {
                next = Some(&notifications[i + 1]);
            } else {
                next = notifications.first();
            }
            break;
        }

        if let Some(next_notif) = next {
            stack.set_visible_child_name(&next_notif.id.to_string());
            true
        } else {
            false
        }
    }

    fn switch_before(id: u32, notifications: &[NotificationData], stack: &Stack) -> bool {
        // If only one notification, there's no previous notification to switch to
        if notifications.len() <= 1 {
            return false;
        }

        let mut prev: Option<&NotificationData> = None;

        for (i, notification) in notifications.iter().enumerate() {
            if notification.id != id {
                continue;
            }

            if i > 0 {
                prev = Some(&notifications[i - 1]);
            } else {
                prev = notifications.last();
            }
            break;
        }

        if let Some(prev_notif) = prev {
            stack.set_visible_child_name(&prev_notif.id.to_string());
            true
        } else {
            false
        }
    }

    fn dismiss(id: u32, notifications: &Rc<RefCell<Vec<NotificationData>>>, stack: &Stack) {
        let notifications_vec = notifications.borrow();
        let has_next = Self::switch_after(id, &notifications_vec, stack);
        drop(notifications_vec);

        if has_next {
            // Delay to allow animation when switching to next notification
            glib::timeout_add_local_once(
                std::time::Duration::from_millis(ANIMATION_DURATION_MS as u64),
                move || {
                    Notifications::instance().dismiss(id);
                },
            );
        } else {
            // Last notification in group - dismiss immediately for instant feedback
            Notifications::instance().dismiss(id);
        }
    }

    fn add_notification_to_stack<F, G>(
        stack: &Stack,
        notification: &NotificationData,
        notifications: &Rc<RefCell<Vec<NotificationData>>>,
        on_action_invoked: Option<F>,
        on_hover_lost: Option<G>,
    ) where
        F: Fn() + Clone + 'static,
        G: Fn(u32) + Clone + 'static,
    {
        let notifications_count = notifications.borrow().len();
        let notification_id = notification.id;

        let notifications_clone = notifications.clone();
        let stack_weak = stack.downgrade();
        let notification_widget = Notification::new(
            notification,
            notifications_count,
            on_action_invoked,
            on_hover_lost,
            move || {
                if let Some(stack) = stack_weak.upgrade() {
                    Self::dismiss(notification_id, &notifications_clone, &stack);
                }
            },
            {
                let notifications_clone = notifications.clone();
                move || {
                    // Collect all notification IDs first
                    let ids: Vec<u32> = notifications_clone.borrow().iter().map(|n| n.id).collect();

                    // Defer dismissals to avoid panicking in GTK callback
                    glib::idle_add_local_once(move || {
                        for id in ids {
                            Notifications::instance().dismiss(id);
                        }
                    });
                }
            },
        );

        stack.add_named(
            notification_widget.widget(),
            Some(&notification.id.to_string()),
        );
    }

    pub fn widget(&self) -> &Stack {
        &self.widget
    }

    pub fn update_notifications<F, G>(
        &self,
        notifications: Vec<NotificationData>,
        on_action_invoked: Option<F>,
        on_hover_lost: Option<G>,
    ) where
        F: Fn() + Clone + 'static,
        G: Fn(u32) + Clone + 'static,
    {
        *self.notifications.borrow_mut() = notifications.clone();

        // Remove old children
        while let Some(child) = self.widget.first_child() {
            self.widget.remove(&child);
        }

        // Add new notifications
        for notification in &notifications {
            Self::add_notification_to_stack(
                &self.widget,
                notification,
                &self.notifications,
                on_action_invoked.clone(),
                on_hover_lost.clone(),
            );
        }

        // Set first notification as visible
        if let Some(first) = notifications.first() {
            self.widget.set_visible_child_name(&first.id.to_string());
        }
    }
}
