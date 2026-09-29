use glib::clone;
use gtk4::prelude::*;
use gtk4::Box;
use std::process::Command;
use std::rc::Rc;

use crate::service::notifications::Notifications;
use crate::service::vpn::Vpn;
use crate::widget::switch_list::{SwitchEntry, SwitchList};

pub struct VpnContent {
    widget: Box,
}

impl VpnContent {
    pub fn new(popover: &gtk4::Popover) -> Self {
        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        let vpn = Vpn::instance();

        Self::rebuild_ui(&container, popover, &vpn);

        vpn.connect_notify_local(
            Some("connections"),
            clone!(
                #[weak]
                container,
                #[weak]
                popover,
                move |vpn_service, _| {
                    Self::rebuild_ui(&container, &popover, vpn_service);
                }
            ),
        );

        Self { widget: container }
    }

    fn rebuild_ui(container: &Box, popover: &gtk4::Popover, vpn: &Vpn) {
        while let Some(child) = container.first_child() {
            container.remove(&child);
        }

        let connections = vpn.connections();

        let entries: Vec<SwitchEntry<_>> = connections
            .into_iter()
            .map(|connection| {
                let connection_path = connection.connection_path.clone();
                let connection_path_activate = connection_path.clone();
                let connection_path_deactivate = connection_path.clone();
                let connection_path_password = connection_path.clone();
                let connection_id = connection.id.clone();
                let connection_id_for_disconnect = connection_id.clone();
                let connection_id_for_connect = connection_id.clone();
                let connection_id_for_disconnect2 = connection_id.clone();

                let needs_password_input =
                    connection.requires_password && !connection.has_saved_password;

                if needs_password_input {
                    SwitchEntry::with_password_support(
                        connection.clone(),
                        connection.id.clone(),
                        Some("network-vpn-symbolic".to_string()),
                        None, // no battery icon for VPN
                        None, // no battery percentage for VPN
                        None, // no peripheral battery icon
                        None, // no peripheral battery percentage
                        Some(format!("Type: {}", connection.vpn_type)),
                        connection.connected,
                        true, // switch_enabled
                        true, // requires_password flag for UI
                        1,    // minimum password length for VPN (allow any non-empty)
                        move |_| {
                            // This shouldn't be called for password-protected VPNs
                            // without saved password
                        },
                        move |_| {
                            let conn_id = connection_id_for_disconnect.clone();
                            Vpn::instance().disconnect_vpn(
                                &connection_path_deactivate,
                                move |error| {
                                    Notifications::instance().send_notification(
                                        "rusty-de",
                                        &format!("VPN Disconnect Failed: {}", conn_id),
                                        &error,
                                    );
                                },
                            );
                        },
                        move |_, password, on_complete| {
                            Vpn::instance().connect_vpn(
                                &connection_path_password,
                                Some(password),
                                move |result| match result {
                                    Ok(()) => {
                                        on_complete(Ok(()));
                                    }
                                    Err(error) => {
                                        let message = if error.contains("invalid credentials")
                                            || error.contains("Wrong password")
                                        {
                                            "Wrong password - Please try again"
                                        } else {
                                            &error
                                        };
                                        on_complete(Err(message.to_string()));
                                    }
                                },
                            );
                        },
                    )
                } else {
                    SwitchEntry::new(
                        connection.clone(),
                        connection.id.clone(),
                        Some("network-vpn-symbolic".to_string()),
                        None, // no battery icon for VPN
                        None, // no battery percentage for VPN
                        None, // no peripheral battery icon
                        None, // no peripheral battery percentage
                        Some(format!("Type: {}", connection.vpn_type)),
                        connection.connected,
                        true, // switch_enabled
                        false,
                        move |_| {
                            let conn_id = connection_id_for_connect.clone();
                            Vpn::instance().connect_vpn(
                                &connection_path_activate,
                                None,
                                move |result| {
                                    if let Err(error) = result {
                                        Notifications::instance().send_notification(
                                            "rusty-de",
                                            &format!("VPN Connection Failed: {}", conn_id),
                                            &error,
                                        );
                                    }
                                },
                            );
                        },
                        move |_| {
                            let conn_id = connection_id_for_disconnect2.clone();
                            Vpn::instance().disconnect_vpn(
                                &connection_path_deactivate,
                                move |error| {
                                    Notifications::instance().send_notification(
                                        "rusty-de",
                                        &format!("VPN Disconnect Failed: {}", conn_id),
                                        &error,
                                    );
                                },
                            );
                        },
                    )
                }
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
            Some("VPN connections".to_string()),
            None,
            Some(on_settings_open),
            entries,
        );

        container.append(switch_list.widget());
    }

    pub fn widget(&self) -> &Box {
        &self.widget
    }
}
