use glib::clone;
use gtk4::prelude::*;
use gtk4::{Box, Orientation};

use crate::service::brightness::Brightness;
use crate::widget::switch_list::{SwitchEntry, SwitchList};

pub struct KeyboardBacklightContent {
    widget: Box,
}

impl KeyboardBacklightContent {
    pub fn new() -> Self {
        let container = Box::new(Orientation::Vertical, 0);

        let brightness = Brightness::instance();
        let current_level = brightness.kbd();

        let levels = [
            (0, "Off", "keyboard-brightness-off-symbolic"),
            (1, "Medium", "keyboard-brightness-medium-symbolic"),
            (2, "Max", "keyboard-brightness-max-symbolic"),
        ];

        let entries: Vec<SwitchEntry<i32>> = levels
            .iter()
            .map(|&(level, name, icon)| {
                let is_active = current_level == level;

                let brightness_clone = brightness.clone();
                let brightness_clone_deactivate = brightness_clone.clone();
                SwitchEntry::new(
                    level,
                    name.to_string(),
                    Some(icon.to_string()),
                    None, // no battery icon for keyboard backlight
                    None, // no battery percentage for keyboard backlight
                    None, // no peripheral battery icon
                    None, // no peripheral battery percentage
                    None,
                    is_active,
                    true,  // switch_enabled
                    false, // No password required
                    move |&lvl| {
                        brightness_clone.set_kbd(lvl);
                    },
                    move |&lvl| {
                        // When deactivating, select first different entry
                        let levels = [0, 1, 2];
                        if let Some(&first_different) = levels.iter().find(|&&l| l != lvl) {
                            brightness_clone_deactivate.set_kbd(first_different);
                        }
                    },
                )
            })
            .collect();

        let switch_list =
            SwitchList::new(Some("Keyboard backlight".to_string()), None, None, entries);
        container.append(switch_list.widget());

        // Update switches when keyboard brightness changes
        brightness.connect_notify_local(
            Some("kbd"),
            clone!(
                #[weak]
                container,
                move |b, _| {
                    // Remove old content
                    while let Some(child) = container.first_child() {
                        container.remove(&child);
                    }

                    // Rebuild with new active level
                    let current_level = b.kbd();
                    let levels = [
                        (0, "Off", "keyboard-brightness-off-symbolic"),
                        (1, "Medium", "keyboard-brightness-medium-symbolic"),
                        (2, "Max", "keyboard-brightness-max-symbolic"),
                    ];

                    let entries: Vec<SwitchEntry<i32>> = levels
                        .iter()
                        .map(|&(level, name, icon)| {
                            let is_active = current_level == level;

                            let b_clone = b.clone();
                            let b_clone_deactivate = b_clone.clone();
                            SwitchEntry::new(
                                level,
                                name.to_string(),
                                Some(icon.to_string()),
                                None, // no battery icon for keyboard backlight
                                None, // no battery percentage for keyboard backlight
                                None, // no peripheral battery icon
                                None, // no peripheral battery percentage
                                None,
                                is_active,
                                true, // switch_enabled
                                false,
                                move |&lvl| {
                                    b_clone.set_kbd(lvl);
                                },
                                move |&lvl| {
                                    // When deactivating, select first different entry
                                    let levels = [0, 1, 2];
                                    if let Some(&first_different) =
                                        levels.iter().find(|&&l| l != lvl)
                                    {
                                        b_clone_deactivate.set_kbd(first_different);
                                    }
                                },
                            )
                        })
                        .collect();

                    let switch_list = SwitchList::new(
                        Some("Keyboard backlight".to_string()),
                        None,
                        None,
                        entries,
                    );
                    container.append(switch_list.widget());
                }
            ),
        );

        Self { widget: container }
    }

    pub fn widget(&self) -> &Box {
        &self.widget
    }
}

impl Default for KeyboardBacklightContent {
    fn default() -> Self {
        Self::new()
    }
}
