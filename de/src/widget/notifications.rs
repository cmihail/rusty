use glib::clone;
use gtk4::prelude::*;
use gtk4::{Box, Orientation, Revealer, RevealerTransitionType};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::service::notifications::{
    Notification as NotificationData, Notifications as NotificationsService,
};
use crate::widget::notification_group::NotificationGroup;

const ANIMATION_DURATION_MS: u32 = 500;

type CallbackOption = Rc<RefCell<Option<Rc<dyn Fn()>>>>;

struct GroupFields {
    revealer: Revealer,
    group: NotificationGroup,
    notifications: Vec<NotificationData>,
}

pub struct Notifications {
    widget: Box,
}

impl Notifications {
    pub fn new<F, G, H, I>(
        group_key: F,
        with_existing_notifications: bool,
        enable_timeout: bool,
        on_action_invoked: Option<G>,
        on_first_notification_added: Option<H>,
        on_last_notification_removed: Option<I>,
    ) -> Self
    where
        F: Fn(&NotificationData) -> String + 'static,
        G: Fn() + 'static,
        H: Fn() + 'static,
        I: Fn() + 'static,
    {
        let container = Box::new(Orientation::Vertical, 0);
        container.add_css_class("Notifications");

        let groups = Rc::new(RefCell::new(HashMap::new()));
        let group_key_fn: Rc<dyn Fn(&NotificationData) -> String> = Rc::new(group_key);
        let on_action_invoked_rc = Rc::new(RefCell::new(
            on_action_invoked.map(|f| Rc::new(f) as Rc<dyn Fn()>),
        ));
        let on_first_notification_added_rc = Rc::new(RefCell::new(
            on_first_notification_added.map(|f| Rc::new(f) as Rc<dyn Fn()>),
        ));
        let on_last_notification_removed_rc = Rc::new(RefCell::new(
            on_last_notification_removed.map(|f| Rc::new(f) as Rc<dyn Fn()>),
        ));

        let notifications = NotificationsService::instance();

        // Connect to notified signal
        notifications.connect_local(
            "notified",
            false,
            clone!(
                #[strong]
                container,
                #[strong]
                groups,
                #[strong]
                group_key_fn,
                #[strong]
                on_action_invoked_rc,
                #[strong]
                on_first_notification_added_rc,
                #[strong]
                on_last_notification_removed_rc,
                move |values| {
                    let id = values[1].get::<u32>().unwrap();
                    let replaced = values[2].get::<bool>().unwrap();
                    Self::handle_new_notification(
                        id,
                        replaced,
                        &container,
                        &groups,
                        &group_key_fn,
                        &on_action_invoked_rc,
                        &on_first_notification_added_rc,
                        &on_last_notification_removed_rc,
                        enable_timeout,
                    );
                    None
                }
            ),
        );

        // Connect to resolved signal
        notifications.connect_local(
            "resolved",
            false,
            clone!(
                #[strong]
                container,
                #[strong]
                groups,
                #[strong]
                on_action_invoked_rc,
                #[strong]
                on_last_notification_removed_rc,
                move |values| {
                    let id = values[1].get::<u32>().unwrap();
                    Self::handle_removed_notification(
                        id,
                        &container,
                        &groups,
                        &on_action_invoked_rc,
                        &on_last_notification_removed_rc,
                        true,
                    );
                    None
                }
            ),
        );

        // Connect to dont-disturb changes
        notifications.connect_notify_local(
            Some("dont-disturb"),
            clone!(
                #[strong]
                container,
                #[strong]
                groups,
                #[strong]
                group_key_fn,
                #[strong]
                on_action_invoked_rc,
                #[strong]
                on_first_notification_added_rc,
                #[strong]
                on_last_notification_removed_rc,
                move |n, _| {
                    if n.dont_disturb() {
                        Self::remove_existing_notifications(
                            &container,
                            &groups,
                            &on_action_invoked_rc,
                            &on_last_notification_removed_rc,
                        );
                    } else if with_existing_notifications {
                        Self::add_existing_notifications(
                            &container,
                            &groups,
                            &group_key_fn,
                            &on_action_invoked_rc,
                            &on_first_notification_added_rc,
                            &on_last_notification_removed_rc,
                            enable_timeout,
                        );
                    }
                }
            ),
        );

        // Add existing notifications with a delay
        if with_existing_notifications {
            glib::timeout_add_local_once(
                std::time::Duration::from_millis(1),
                clone!(
                    #[strong]
                    container,
                    #[strong]
                    groups,
                    #[strong]
                    group_key_fn,
                    #[strong]
                    on_action_invoked_rc,
                    #[strong]
                    on_first_notification_added_rc,
                    #[strong]
                    on_last_notification_removed_rc,
                    move || {
                        Self::add_existing_notifications(
                            &container,
                            &groups,
                            &group_key_fn,
                            &on_action_invoked_rc,
                            &on_first_notification_added_rc,
                            &on_last_notification_removed_rc,
                            enable_timeout,
                        );
                    }
                ),
            );
        }

        Self { widget: container }
    }

    #[allow(clippy::too_many_arguments)]
    fn handle_new_notification(
        id: u32,
        _replaced: bool,
        container: &Box,
        groups: &Rc<RefCell<HashMap<String, GroupFields>>>,
        group_key_fn: &Rc<dyn Fn(&NotificationData) -> String>,
        on_action_invoked: &CallbackOption,
        on_first_notification_added: &CallbackOption,
        on_last_notification_removed: &CallbackOption,
        enable_timeout: bool,
    ) {
        let notifications = NotificationsService::instance();
        if notifications.dont_disturb() {
            return;
        }

        let groups_map = groups.borrow();
        if groups_map.is_empty() {
            drop(groups_map);
            if let Some(ref handler) = *on_first_notification_added.borrow() {
                handler();
            }
        } else {
            drop(groups_map);
        }

        // Check if notification already exists
        for group_fields in groups.borrow().values() {
            if group_fields.notifications.iter().any(|n| n.id == id) {
                return;
            }
        }

        let notification = match notifications.get_notification(id) {
            Some(n) => n,
            None => return,
        };

        let key = group_key_fn(&notification);

        // Create new group if it doesn't exist
        if !groups.borrow().contains_key(&key) {
            Self::create_group(container, groups, &key, on_action_invoked);
        }

        // Add notification to group
        let mut groups_map = groups.borrow_mut();
        if let Some(group_fields) = groups_map.get_mut(&key) {
            let mut notifs = group_fields.notifications.clone();
            notifs.insert(0, notification.clone());

            // Re-reveal the group if it was hidden (being removed)
            if !group_fields.revealer.reveals_child() || group_fields.notifications.is_empty() {
                group_fields.revealer.set_reveal_child(true);
            }

            group_fields.notifications = notifs.clone();

            let on_action_invoked_clone = on_action_invoked.borrow().clone();
            group_fields.group.update_notifications(
                notifs,
                on_action_invoked_clone.map(|f| move || f()),
                None::<fn(u32)>,
            );
        }
        drop(groups_map);

        // Setup auto-dismiss if timeout is enabled
        if enable_timeout {
            glib::timeout_add_local_once(
                std::time::Duration::from_millis(5000),
                clone!(
                    #[strong]
                    container,
                    #[strong]
                    groups,
                    #[strong]
                    on_action_invoked,
                    #[strong]
                    on_last_notification_removed,
                    move || {
                        Self::handle_removed_notification(
                            id,
                            &container,
                            &groups,
                            &on_action_invoked,
                            &on_last_notification_removed,
                            true,
                        );
                    }
                ),
            );
        }
    }

    fn handle_removed_notification(
        id: u32,
        container: &Box,
        groups: &Rc<RefCell<HashMap<String, GroupFields>>>,
        on_action_invoked: &CallbackOption,
        on_last_notification_removed: &CallbackOption,
        with_delay_on_last: bool,
    ) {
        let key_to_remove = Self::process_notification_removal(
            id,
            container,
            groups,
            on_action_invoked,
            on_last_notification_removed,
            with_delay_on_last,
        );

        if let Some(key) = key_to_remove {
            Self::remove_group(container, groups, &key);

            if groups.borrow().is_empty() {
                if let Some(ref handler) = *on_last_notification_removed.borrow() {
                    handler();
                }
            }
        }
    }

    fn process_notification_removal(
        id: u32,
        container: &Box,
        groups: &Rc<RefCell<HashMap<String, GroupFields>>>,
        on_action_invoked: &CallbackOption,
        on_last_notification_removed: &CallbackOption,
        with_delay_on_last: bool,
    ) -> Option<String> {
        let mut groups_map = groups.borrow_mut();

        for (key, group_fields) in groups_map.iter_mut() {
            let initial_len = group_fields.notifications.len();
            let new_notifs: Vec<_> = group_fields
                .notifications
                .iter()
                .filter(|n| n.id != id)
                .cloned()
                .collect();

            if new_notifs.len() == initial_len {
                continue;
            }

            return Self::update_group_after_removal(
                key,
                group_fields,
                new_notifs,
                container,
                groups,
                on_action_invoked,
                on_last_notification_removed,
                with_delay_on_last,
            );
        }

        None
    }

    #[allow(clippy::too_many_arguments)]
    fn update_group_after_removal(
        key: &str,
        group_fields: &mut GroupFields,
        new_notifs: Vec<NotificationData>,
        container: &Box,
        groups: &Rc<RefCell<HashMap<String, GroupFields>>>,
        on_action_invoked: &CallbackOption,
        on_last_notification_removed: &CallbackOption,
        with_delay_on_last: bool,
    ) -> Option<String> {
        if new_notifs.is_empty() {
            // Update internal state to reflect empty group
            group_fields.notifications = new_notifs.clone();
            group_fields.revealer.set_reveal_child(false);

            if with_delay_on_last {
                Self::schedule_delayed_group_removal(
                    key.to_string(),
                    container,
                    groups,
                    on_last_notification_removed,
                );
                None
            } else {
                Some(key.to_string())
            }
        } else {
            group_fields.notifications = new_notifs.clone();
            let on_action_invoked_clone = on_action_invoked.borrow().clone();
            group_fields.group.update_notifications(
                new_notifs,
                on_action_invoked_clone.map(|f| move || f()),
                None::<fn(u32)>,
            );
            None
        }
    }

    fn schedule_delayed_group_removal(
        key: String,
        container: &Box,
        groups: &Rc<RefCell<HashMap<String, GroupFields>>>,
        on_last_notification_removed: &CallbackOption,
    ) {
        let groups_clone = groups.clone();
        let container_weak = container.downgrade();
        let on_last_removed = on_last_notification_removed.clone();

        glib::timeout_add_local_once(
            std::time::Duration::from_millis(ANIMATION_DURATION_MS as u64),
            move || {
                if let Some(container) = container_weak.upgrade() {
                    Self::execute_delayed_group_removal(
                        &key,
                        &container,
                        &groups_clone,
                        &on_last_removed,
                    );
                }
            },
        );
    }

    fn execute_delayed_group_removal(
        key: &str,
        container: &Box,
        groups: &Rc<RefCell<HashMap<String, GroupFields>>>,
        on_last_notification_removed: &CallbackOption,
    ) {
        // Re-check if group still exists and is truly empty
        let should_remove = {
            let groups_map = groups.borrow();
            if let Some(group_fields) = groups_map.get(key) {
                group_fields.notifications.is_empty()
            } else {
                // Group already removed
                false
            }
        };

        if should_remove {
            Self::remove_group(container, groups, key);

            if groups.borrow().is_empty() {
                if let Some(ref handler) = *on_last_notification_removed.borrow() {
                    handler();
                }
            }
        } else {
            // Re-reveal the group if it has notifications
            if let Some(group_fields) = groups.borrow().get(key) {
                if !group_fields.notifications.is_empty() {
                    group_fields.revealer.set_reveal_child(true);
                }
            }
        }
    }

    fn create_group(
        container: &Box,
        groups: &Rc<RefCell<HashMap<String, GroupFields>>>,
        key: &str,
        on_action_invoked: &CallbackOption,
    ) {
        let revealer = Revealer::new();
        revealer.set_transition_type(RevealerTransitionType::SlideDown);
        revealer.set_transition_duration(ANIMATION_DURATION_MS);
        revealer.set_reveal_child(false);

        let group = NotificationGroup::new(
            key,
            Vec::new(),
            on_action_invoked.borrow().clone().map(|f| move || f()),
            None::<fn(u32)>,
        );

        revealer.set_child(Some(group.widget()));
        container.prepend(&revealer);

        groups.borrow_mut().insert(
            key.to_string(),
            GroupFields {
                revealer,
                group,
                notifications: Vec::new(),
            },
        );
    }

    fn remove_group(
        container: &Box,
        groups: &Rc<RefCell<HashMap<String, GroupFields>>>,
        key: &str,
    ) {
        if let Some(group_fields) = groups.borrow_mut().remove(key) {
            container.remove(&group_fields.revealer);
        }
    }

    fn add_existing_notifications(
        container: &Box,
        groups: &Rc<RefCell<HashMap<String, GroupFields>>>,
        group_key_fn: &Rc<dyn Fn(&NotificationData) -> String>,
        on_action_invoked: &CallbackOption,
        on_first_notification_added: &CallbackOption,
        on_last_notification_removed: &CallbackOption,
        enable_timeout: bool,
    ) {
        let notifications = NotificationsService::instance();
        for notification in notifications.notifications() {
            Self::handle_new_notification(
                notification.id,
                false,
                container,
                groups,
                group_key_fn,
                on_action_invoked,
                on_first_notification_added,
                on_last_notification_removed,
                enable_timeout,
            );
        }
    }

    fn remove_existing_notifications(
        container: &Box,
        groups: &Rc<RefCell<HashMap<String, GroupFields>>>,
        on_action_invoked: &CallbackOption,
        on_last_notification_removed: &CallbackOption,
    ) {
        let all_ids: Vec<u32> = groups
            .borrow()
            .values()
            .flat_map(|g| g.notifications.iter().map(|n| n.id))
            .collect();

        for id in all_ids {
            Self::handle_removed_notification(
                id,
                container,
                groups,
                on_action_invoked,
                on_last_notification_removed,
                false,
            );
        }
    }

    pub fn widget(&self) -> &Box {
        &self.widget
    }
}
