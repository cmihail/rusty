use glib::clone;
use gtk4::prelude::*;
use gtk4::Box;
use std::process::Command;
use std::rc::Rc;

use crate::service::battery::Battery;
use crate::service::bluetooth::{Bluetooth, BluetoothDevice};
use crate::widget::switch_list::{SwitchEntry, SwitchList};

pub struct BluetoothContent {
    widget: Box,
    _header_label: Rc<gtk4::Label>,
}

impl BluetoothContent {
    pub fn new(popover: &gtk4::Popover) -> Self {
        let container = Box::new(gtk4::Orientation::Vertical, 0);
        let bluetooth = Bluetooth::instance();

        let header_label = Rc::new(gtk4::Label::new(Some("Available Devices")));

        Self::rebuild_ui(&container, popover, &bluetooth, &header_label);

        // Listen to devices property changes
        bluetooth.connect_notify_local(
            Some("devices"),
            clone!(
                #[weak]
                container,
                #[weak]
                popover,
                #[strong]
                header_label,
                move |bt, _| {
                    Self::rebuild_ui(&container, &popover, bt, &header_label);
                }
            ),
        );

        // Listen to scanning state changes
        bluetooth.connect_notify_local(
            Some("scanning"),
            clone!(
                #[weak]
                container,
                #[weak]
                popover,
                #[strong]
                header_label,
                move |bt, _| {
                    Self::rebuild_ui(&container, &popover, bt, &header_label);
                }
            ),
        );

        Self {
            widget: container,
            _header_label: header_label,
        }
    }

    fn rebuild_ui(
        container: &Box,
        popover: &gtk4::Popover,
        bluetooth: &Bluetooth,
        header_label: &gtk4::Label,
    ) {
        while let Some(child) = container.first_child() {
            container.remove(&child);
        }

        let devices = bluetooth.devices();
        let sorted_devices = Self::sort_devices(devices);
        let scanning = bluetooth.scanning();
        let header_text = Self::get_header_text(scanning);
        header_label.set_text(&header_text);

        let entries = Self::create_device_entries(sorted_devices);
        let on_refresh = Self::create_refresh_callback(bluetooth, scanning);
        let on_settings_open = Self::create_settings_callback(popover);

        let switch_list = SwitchList::new(
            Some(header_text),
            on_refresh,
            Some(on_settings_open),
            entries,
        );

        container.append(switch_list.widget());
    }

    fn get_header_text(scanning: bool) -> String {
        if scanning {
            "Scanning...".to_string()
        } else {
            "Bluetooth devices".to_string()
        }
    }

    fn sort_devices(mut devices: Vec<BluetoothDevice>) -> Vec<BluetoothDevice> {
        devices.sort_by_key(|device| {
            // Priority order (lower number = shown first):
            // 0: Connected + nearby
            // 1: Nearby + paired + not connected
            // 2: Nearby + not paired + not connected
            // 3: Paired + not nearby
            // 4: Other (edge cases)
            if device.connected && device.nearby {
                0
            } else if device.nearby && device.paired && !device.connected {
                1
            } else if device.nearby && !device.paired {
                2
            } else if device.paired && !device.nearby {
                3
            } else {
                4
            }
        });
        devices
    }

    fn create_device_entries(devices: Vec<BluetoothDevice>) -> Vec<SwitchEntry<BluetoothDevice>> {
        devices
            .into_iter()
            .filter(|device| device.nearby)
            .map(Self::create_device_entry)
            .collect()
    }

    fn create_device_entry(device: BluetoothDevice) -> SwitchEntry<BluetoothDevice> {
        let is_connected = device.connected;
        let switch_enabled = device.nearby;
        let icon_name = Self::get_icon_for_device(&device);
        let battery_icon = Self::get_battery_icon(&device);
        let device_path_connect = device.path.clone();
        let device_path_disconnect = device.path.clone();

        // Check if this is a split keyboard with multiple batteries
        let battery_service = Battery::instance();
        let keyboard_batteries = battery_service.keyboard_batteries();
        for _kb in &keyboard_batteries {}

        let keyboard_battery = keyboard_batteries.iter().find(|kb| {
            let kb_addr = kb.address.replace(":", "_");
            let dev_addr = device.address.replace(":", "_");
            kb_addr == dev_addr
        });

        let (peripheral_battery_icon, peripheral_battery_percentage) =
            if let Some(kb) = keyboard_battery {
                (
                    kb.peripheral_percentage
                        .map(Self::get_battery_icon_from_percentage),
                    kb.peripheral_percentage,
                )
            } else {
                (None, None)
            };

        let tooltip = Self::create_device_tooltip(&device, peripheral_battery_percentage);

        // Check if this device is currently pairing
        let bluetooth = Bluetooth::instance();
        let pairing_device = bluetooth.get_pairing_device();
        let is_pairing = pairing_device.as_ref() == Some(&device.path);

        // Get any stored error for this device
        let initial_error = bluetooth.get_device_error(&device.path);

        if is_pairing {
            let pairing_code = bluetooth.get_pairing_code();
            let device_path_approval = device.path.clone();

            SwitchEntry::with_pairing_confirmation(
                device.clone(),
                device.name.clone(),
                Some(icon_name),
                battery_icon,
                device.battery_percentage,
                peripheral_battery_icon,
                peripheral_battery_percentage,
                Some(tooltip),
                is_connected,
                switch_enabled,
                pairing_code,
                move |_, on_complete| Self::handle_connect(&device_path_connect, on_complete),
                move |_, on_complete| Self::handle_disconnect(&device_path_disconnect, on_complete),
                move |_, approved| {
                    bluetooth.respond_to_pairing(&device_path_approval, approved);
                },
            )
        } else {
            SwitchEntry::with_completion_callback(
                device.clone(),
                device.name.clone(),
                Some(icon_name),
                battery_icon,
                device.battery_percentage,
                peripheral_battery_icon,
                peripheral_battery_percentage,
                Some(tooltip),
                is_connected,
                switch_enabled,
                initial_error,
                move |_, on_complete| {
                    // Clear all errors in the list when any switch is toggled
                    Bluetooth::instance().clear_all_errors();
                    Self::handle_connect(&device_path_connect, on_complete);
                },
                move |_, on_complete| {
                    // Clear all errors in the list when any switch is toggled
                    Bluetooth::instance().clear_all_errors();
                    Self::handle_disconnect(&device_path_disconnect, on_complete);
                },
            )
        }
    }

    fn create_device_tooltip(device: &BluetoothDevice, peripheral_battery: Option<u8>) -> String {
        let status_text = if device.paired {
            if device.connected {
                "Connected"
            } else {
                "Paired"
            }
        } else {
            "Not Paired"
        };
        let battery_text = match (device.battery_percentage, peripheral_battery) {
            (Some(central), Some(peripheral)) => {
                format!(
                    "\nBattery: {}% (Central) / {}% (Peripheral)",
                    central, peripheral
                )
            }
            (Some(percentage), None) => {
                format!("\nBattery: {}%", percentage)
            }
            _ => String::new(),
        };
        let nearby_text = if device.nearby { "Yes" } else { "No" };
        format!(
            "Name: {}\nAddress: {}\nStatus: {}\nNearby: {}{}",
            device.name, device.address, status_text, nearby_text, battery_text
        )
    }

    fn get_battery_icon(device: &BluetoothDevice) -> Option<String> {
        device.battery_percentage.map(|percentage| {
            match percentage {
                0..=10 => "battery-level-0-symbolic",
                11..=30 => "battery-level-10-symbolic",
                31..=50 => "battery-level-30-symbolic",
                51..=70 => "battery-level-50-symbolic",
                71..=90 => "battery-level-70-symbolic",
                _ => "battery-level-90-symbolic",
            }
            .to_string()
        })
    }

    fn get_battery_icon_from_percentage(percentage: u8) -> String {
        match percentage {
            0..=10 => "battery-level-0-symbolic",
            11..=30 => "battery-level-10-symbolic",
            31..=50 => "battery-level-30-symbolic",
            51..=70 => "battery-level-50-symbolic",
            71..=90 => "battery-level-70-symbolic",
            _ => "battery-level-90-symbolic",
        }
        .to_string()
    }

    fn handle_connect(device_path: &str, on_complete: std::boxed::Box<dyn Fn(Result<(), String>)>) {
        Bluetooth::instance().connect_device(device_path, move |result| match result {
            Ok(_) => {
                on_complete(Ok(()));
            }
            Err(e) => {
                on_complete(Err(e));
            }
        });
    }

    fn handle_disconnect(
        device_path: &str,
        on_complete: std::boxed::Box<dyn Fn(Result<(), String>)>,
    ) {
        Bluetooth::instance().disconnect_device(device_path, move |result| match result {
            Ok(_) => {
                on_complete(Ok(()));
            }
            Err(e) => {
                on_complete(Err(e));
            }
        });
    }

    fn create_refresh_callback(bluetooth: &Bluetooth, scanning: bool) -> Option<Rc<dyn Fn()>> {
        if scanning {
            None
        } else {
            let bluetooth_clone = bluetooth.clone();
            Some(Rc::new(move || {
                bluetooth_clone.scan();
            }) as Rc<dyn Fn()>)
        }
    }

    fn create_settings_callback(popover: &gtk4::Popover) -> Rc<dyn Fn()> {
        let popover_clone = popover.clone();
        Rc::new(move || {
            popover_clone.popdown();
            glib::timeout_add_local_once(std::time::Duration::from_millis(1), || {
                let _ = Command::new("blueman-manager").spawn();
            });
        })
    }

    fn get_icon_for_device(device: &BluetoothDevice) -> String {
        let bluetooth = Bluetooth::instance();
        let connecting_device = bluetooth.get_connecting_device();

        // Show acquiring icon if this device is currently connecting
        if connecting_device.as_ref() == Some(&device.path) {
            return "bluetooth-acquiring-symbolic".to_string();
        }

        if device.connected {
            "bluetooth-active-symbolic".to_string()
        } else if device.paired {
            "bluetooth-disconnected-symbolic".to_string()
        } else {
            "bluetooth-disabled-symbolic".to_string()
        }
    }

    pub fn widget(&self) -> &Box {
        &self.widget
    }
}
