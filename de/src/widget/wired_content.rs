use glib::clone;
use gtk4::prelude::*;
use gtk4::Box;
use std::process::Command;
use std::rc::Rc;

use crate::service::ethernet::Ethernet;
use crate::widget::switch_list::{SwitchEntry, SwitchList};

pub struct WiredContent {
    widget: Box,
}

impl WiredContent {
    pub fn new(popover: &gtk4::Popover) -> Self {
        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        let ethernet = Ethernet::instance();

        Self::rebuild_ui(&container, popover, &ethernet);

        // Listen for device changes
        ethernet.connect_notify_local(
            Some("devices"),
            clone!(
                #[weak]
                container,
                #[weak]
                popover,
                move |eth, _| {
                    Self::rebuild_ui(&container, &popover, eth);
                }
            ),
        );

        Self { widget: container }
    }

    fn rebuild_ui(container: &Box, popover: &gtk4::Popover, ethernet: &Ethernet) {
        // Clear existing content
        while let Some(child) = container.first_child() {
            container.remove(&child);
        }

        let devices = ethernet.devices();

        let entries: Vec<SwitchEntry<_>> = devices
            .into_iter()
            .map(|device| {
                let device_path = device.device_path.clone();
                let device_path_activate = device_path.clone();
                let device_path_deactivate = device_path.clone();

                SwitchEntry::new(
                    device.clone(),
                    device.id.clone(),
                    None, // No icon for wired devices
                    None, // no battery icon for wired devices
                    None, // no battery percentage for wired devices
                    None, // no peripheral battery icon
                    None, // no peripheral battery percentage
                    if !device.product.is_empty() {
                        Some(format!("Product: {}", device.product))
                    } else {
                        None
                    },
                    device.connected,
                    true,  // switch_enabled
                    false, // Wired connections don't require passwords
                    move |_| {
                        Ethernet::instance().connect_device(&device_path_activate, |_error| {});
                    },
                    move |_| {
                        Ethernet::instance()
                            .disconnect_device(&device_path_deactivate, |_error| {});
                    },
                )
            })
            .collect();

        let popover_clone = popover.clone();
        let on_settings_open = Rc::new(move || {
            popover_clone.popdown();
            glib::timeout_add_local_once(std::time::Duration::from_millis(1), || {
                let _ = Command::new("nm-connection-editor").spawn();
            });
        });

        let switch_list = SwitchList::new(
            Some("Ethernet devices".to_string()),
            None, // No refresh for wired
            Some(on_settings_open),
            entries,
        );

        container.append(switch_list.widget());
    }

    pub fn widget(&self) -> &Box {
        &self.widget
    }
}
