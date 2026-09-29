use glib::clone;
use gtk4::prelude::*;
use gtk4::Box;

use crate::service::audio::Audio;
use crate::widget::switch_list::{SwitchEntry, SwitchList};

pub struct AudioDevicesContent {
    widget: Box,
}

impl Default for AudioDevicesContent {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioDevicesContent {
    pub fn new() -> Self {
        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        let audio = Audio::instance();

        Self::rebuild_ui(&container, &audio);

        // Listen for device changes
        audio.connect_notify_local(
            Some("sinks"),
            clone!(
                #[weak]
                container,
                move |audio, _| {
                    Self::rebuild_ui(&container, audio);
                }
            ),
        );

        Self { widget: container }
    }

    fn rebuild_ui(container: &Box, audio: &Audio) {
        // Clear existing content
        while let Some(child) = container.first_child() {
            container.remove(&child);
        }

        let sinks = audio.sinks();

        let entries: Vec<SwitchEntry<_>> = sinks
            .into_iter()
            .filter(|sink| sink.is_available)
            .map(|sink| {
                let is_default = sink.is_default;
                SwitchEntry::new(
                    sink.clone(),
                    sink.description.clone(),
                    None, // No icon for audio devices
                    None, // no battery icon for audio devices
                    None, // no battery percentage for audio devices
                    None, // no peripheral battery icon
                    None, // no peripheral battery percentage
                    None, // No tooltip
                    is_default,
                    true,  // switch_enabled
                    false, // Audio devices don't require passwords
                    move |s| {
                        Audio::instance().set_default_sink(&s.name);
                    },
                    move |s| {
                        // If trying to deactivate the current default
                        if s.is_default {
                            let audio = Audio::instance();
                            let available_sinks: Vec<_> = audio
                                .sinks()
                                .into_iter()
                                .filter(|sink| sink.is_available)
                                .collect();

                            // Only switch if there are other available devices
                            if available_sinks.len() > 1 {
                                if let Some(first_sink) =
                                    available_sinks.into_iter().find(|sink| sink.name != s.name)
                                {
                                    audio.set_default_sink(&first_sink.name);
                                }
                            } else {
                                // Only one device, keep it selected
                                audio.set_default_sink(&s.name);
                            }
                        }
                    },
                )
            })
            .collect();

        let switch_list = SwitchList::new(
            Some("Audio devices".to_string()),
            None, // No refresh
            None, // No settings
            entries,
        );

        container.append(switch_list.widget());
    }

    pub fn widget(&self) -> &Box {
        &self.widget
    }
}
