use glib::clone;
use gtk4::prelude::*;
use gtk4::Box;

use crate::service::audio::Audio;
use crate::widget::switch_list::{SwitchEntry, SwitchList};

pub struct MicDevicesContent {
    widget: Box,
}

impl Default for MicDevicesContent {
    fn default() -> Self {
        Self::new()
    }
}

impl MicDevicesContent {
    pub fn new() -> Self {
        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        let audio = Audio::instance();

        Self::rebuild_ui(&container, &audio);

        // Listen for device changes
        audio.connect_notify_local(
            Some("sources"),
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

        let sources = audio.sources();

        let entries: Vec<SwitchEntry<_>> = sources
            .into_iter()
            .filter(|source| source.is_available)
            .map(|source| {
                let is_default = source.is_default;
                SwitchEntry::new(
                    source.clone(),
                    source.description.clone(),
                    None, // No icon for microphone devices
                    None, // no battery icon for microphone devices
                    None, // no battery percentage for microphone devices
                    None, // no peripheral battery icon
                    None, // no peripheral battery percentage
                    None, // No tooltip
                    is_default,
                    true,  // switch_enabled
                    false, // Microphone devices don't require passwords
                    move |s| {
                        Audio::instance().set_default_source(&s.name);
                    },
                    move |s| {
                        // If trying to deactivate the current default
                        if s.is_default {
                            let audio = Audio::instance();
                            let available_sources: Vec<_> = audio
                                .sources()
                                .into_iter()
                                .filter(|source| source.is_available)
                                .collect();

                            // Only switch if there are other available devices
                            if available_sources.len() > 1 {
                                if let Some(first_source) = available_sources
                                    .into_iter()
                                    .find(|source| source.name != s.name)
                                {
                                    audio.set_default_source(&first_source.name);
                                }
                            } else {
                                // Only one device, keep it selected
                                audio.set_default_source(&s.name);
                            }
                        }
                    },
                )
            })
            .collect();

        let switch_list = SwitchList::new(
            Some("Microphone devices".to_string()),
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
