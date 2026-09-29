use glib::clone;
use gtk4::prelude::*;
use gtk4::{Box, Label, Orientation};

use crate::service::power_profiles::PowerProfiles;
use crate::widget::switch_list::{SwitchEntry, SwitchList};

const PROFILES: &[&str] = &["power-saver", "balanced", "performance"];

pub struct PowerProfilesContent {
    widget: Box,
}

fn upcase_profile(profile: &str) -> String {
    profile
        .split('-')
        .map(|p| {
            let mut chars = p.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect::<Vec<_>>()
        .join("")
}

fn build_entries(power_profiles: &PowerProfiles) -> Vec<SwitchEntry<String>> {
    let active_profile = power_profiles.active_profile();

    PROFILES
        .iter()
        .map(|&profile| {
            let profile_str = profile.to_string();
            let is_active = active_profile == profile_str;
            let icon_name = format!("power-profile-{}-symbolic", profile);

            let power_profiles_clone = power_profiles.clone();
            let profile_str_activate = profile_str.clone();
            SwitchEntry::new(
                profile_str.clone(),
                upcase_profile(profile),
                Some(icon_name),
                None, // no battery icon for power profiles
                None, // no battery percentage for power profiles
                None, // no peripheral battery icon
                None, // no peripheral battery percentage
                None,
                is_active,
                true,  // switch_enabled
                false, // Power profiles don't require passwords
                move |_| {
                    power_profiles_clone.set_active_profile(&profile_str_activate);
                },
                |_| {
                    // No deactivate for power profiles
                },
            )
        })
        .collect()
}

fn rebuild_content(container: &Box, power_profiles: &PowerProfiles) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }

    if !power_profiles.available() {
        let label = Label::new(Some("Power profiles daemon unavailable"));
        label.add_css_class("dim-label");
        container.append(&label);
        return;
    }

    let switch_list = SwitchList::new(None, None, None, build_entries(power_profiles));
    container.append(switch_list.widget());
}

impl PowerProfilesContent {
    pub fn new() -> Self {
        let container = Box::new(Orientation::Vertical, 0);

        let power_profiles = PowerProfiles::instance();
        rebuild_content(&container, &power_profiles);

        // Rebuild when the active profile changes or the daemon appears
        for property in ["active-profile", "available"] {
            power_profiles.connect_notify_local(
                Some(property),
                clone!(
                    #[weak]
                    container,
                    move |pp, _| {
                        rebuild_content(&container, pp);
                    }
                ),
            );
        }

        Self { widget: container }
    }

    pub fn widget(&self) -> &Box {
        &self.widget
    }
}

impl Default for PowerProfilesContent {
    fn default() -> Self {
        Self::new()
    }
}
