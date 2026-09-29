use glib::clone;
use gtk4::prelude::*;
use gtk4::{Box, Orientation};
use std::rc::Rc;

use crate::service::fcitx::Fcitx;
use crate::widget::switch_list::{SwitchEntry, SwitchList};

pub struct LanguageContent {
    widget: Box,
}

impl LanguageContent {
    pub fn new(popover: &gtk4::Popover) -> Self {
        let container = Box::new(Orientation::Vertical, 0);

        let fcitx = Fcitx::instance();
        let current_im = fcitx.current_im();
        let available_ims = fcitx.available_ims();

        let enabled_ims: Vec<_> = available_ims
            .iter()
            .filter(|im| im.enabled)
            .cloned()
            .collect();

        let entries: Vec<SwitchEntry<String>> = enabled_ims
            .iter()
            .enumerate()
            .map(|(index, im)| {
                let is_active = current_im == im.unique_name;

                let fcitx_clone = fcitx.clone();
                let fcitx_clone_deactivate = fcitx.clone();
                let im_name = im.unique_name.clone();
                let enabled_ims_clone = enabled_ims.clone();
                SwitchEntry::new(
                    im.unique_name.clone(),
                    im.name.clone(),
                    None,
                    None, // no battery icon for language
                    None, // no battery percentage for language
                    None, // no peripheral battery icon
                    None, // no peripheral battery percentage
                    None,
                    is_active,
                    true, // switch_enabled
                    false,
                    move |_| {
                        fcitx_clone.set_current_im(&im_name);
                    },
                    move |_| {
                        // When deactivating the active language, switch to the next one
                        if enabled_ims_clone.len() > 1 {
                            let next_index = (index + 1) % enabled_ims_clone.len();
                            let next_im = &enabled_ims_clone[next_index];
                            fcitx_clone_deactivate.set_current_im(&next_im.unique_name);
                        } else {
                            // Single language, re-select the same one
                            fcitx_clone_deactivate
                                .set_current_im(&enabled_ims_clone[0].unique_name);
                        }
                    },
                )
            })
            .collect();

        let popover_clone = popover.clone();
        let settings_callback: Option<Rc<dyn Fn()>> = Some(Rc::new(move || {
            popover_clone.popdown();
            glib::timeout_add_local_once(std::time::Duration::from_millis(1), || {
                use std::process::Stdio;
                let _ = std::process::Command::new("fcitx5-config-qt")
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn();
            });
        }));

        let header_text = Some("Languages".to_string());

        let switch_list = SwitchList::new(header_text, None, settings_callback, entries);
        container.append(switch_list.widget());

        // Update switches when current input method changes
        let popover_for_callback = popover.clone();
        let popover_for_callback2 = popover.clone();
        fcitx.connect_notify_local(
            Some("current-im"),
            clone!(
                #[weak]
                container,
                move |fcitx, _| {
                    // Remove old content
                    while let Some(child) = container.first_child() {
                        container.remove(&child);
                    }

                    // Rebuild with new current input method
                    let current_im = fcitx.current_im();
                    let available_ims = fcitx.available_ims();

                    let enabled_ims: Vec<_> = available_ims
                        .iter()
                        .filter(|im| im.enabled)
                        .cloned()
                        .collect();

                    let entries: Vec<SwitchEntry<String>> = enabled_ims
                        .iter()
                        .enumerate()
                        .map(|(index, im)| {
                            let is_active = current_im == im.unique_name;

                            let fcitx_clone = fcitx.clone();
                            let fcitx_clone_deactivate = fcitx.clone();
                            let im_name = im.unique_name.clone();
                            let enabled_ims_clone = enabled_ims.clone();
                            SwitchEntry::new(
                                im.unique_name.clone(),
                                im.name.clone(),
                                None,
                                None, // no battery icon for language
                                None, // no battery percentage for language
                                None, // no peripheral battery icon
                                None, // no peripheral battery percentage
                                None,
                                is_active,
                                true, // switch_enabled
                                false,
                                move |_| {
                                    fcitx_clone.set_current_im(&im_name);
                                },
                                move |_| {
                                    // When deactivating the active language, switch to the next one
                                    if enabled_ims_clone.len() > 1 {
                                        let next_index = (index + 1) % enabled_ims_clone.len();
                                        let next_im = &enabled_ims_clone[next_index];
                                        fcitx_clone_deactivate.set_current_im(&next_im.unique_name);
                                    } else {
                                        // Single language, re-select the same one
                                        fcitx_clone_deactivate
                                            .set_current_im(&enabled_ims_clone[0].unique_name);
                                    }
                                },
                            )
                        })
                        .collect();

                    let popover_clone2 = popover_for_callback.clone();
                    let settings_callback: Option<Rc<dyn Fn()>> = Some(Rc::new(move || {
                        popover_clone2.popdown();
                        glib::timeout_add_local_once(std::time::Duration::from_millis(1), || {
                            use std::process::Stdio;
                            let _ = std::process::Command::new("fcitx5-config-qt")
                                .stdout(Stdio::null())
                                .stderr(Stdio::null())
                                .spawn();
                        });
                    }));

                    let header_text = Some("Languages".to_string());

                    let switch_list =
                        SwitchList::new(header_text, None, settings_callback, entries);
                    container.append(switch_list.widget());
                }
            ),
        );

        // Update switches when available languages list changes
        fcitx.connect_notify_local(
            Some("available-ims-count"),
            clone!(
                #[weak]
                container,
                move |fcitx, _| {
                    // Remove old content
                    while let Some(child) = container.first_child() {
                        container.remove(&child);
                    }

                    // Rebuild with new language list
                    let current_im = fcitx.current_im();
                    let available_ims = fcitx.available_ims();

                    let enabled_ims: Vec<_> = available_ims
                        .iter()
                        .filter(|im| im.enabled)
                        .cloned()
                        .collect();

                    let entries: Vec<SwitchEntry<String>> = enabled_ims
                        .iter()
                        .enumerate()
                        .map(|(index, im)| {
                            let is_active = current_im == im.unique_name;

                            let fcitx_clone = fcitx.clone();
                            let fcitx_clone_deactivate = fcitx.clone();
                            let im_name = im.unique_name.clone();
                            let enabled_ims_clone = enabled_ims.clone();
                            SwitchEntry::new(
                                im.unique_name.clone(),
                                im.name.clone(),
                                None,
                                None, // no battery icon for language
                                None, // no battery percentage for language
                                None, // no peripheral battery icon
                                None, // no peripheral battery percentage
                                None,
                                is_active,
                                true, // switch_enabled
                                false,
                                move |_| {
                                    fcitx_clone.set_current_im(&im_name);
                                },
                                move |_| {
                                    // When deactivating the active language, switch to the next one
                                    if enabled_ims_clone.len() > 1 {
                                        let next_index = (index + 1) % enabled_ims_clone.len();
                                        let next_im = &enabled_ims_clone[next_index];
                                        fcitx_clone_deactivate.set_current_im(&next_im.unique_name);
                                    } else {
                                        // Single language, re-select the same one
                                        fcitx_clone_deactivate
                                            .set_current_im(&enabled_ims_clone[0].unique_name);
                                    }
                                },
                            )
                        })
                        .collect();

                    let popover_clone3 = popover_for_callback2.clone();
                    let settings_callback: Option<Rc<dyn Fn()>> = Some(Rc::new(move || {
                        popover_clone3.popdown();
                        glib::timeout_add_local_once(std::time::Duration::from_millis(1), || {
                            use std::process::Stdio;
                            let _ = std::process::Command::new("fcitx5-config-qt")
                                .stdout(Stdio::null())
                                .stderr(Stdio::null())
                                .spawn();
                        });
                    }));

                    let header_text = Some("Languages".to_string());

                    let switch_list =
                        SwitchList::new(header_text, None, settings_callback, entries);
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
