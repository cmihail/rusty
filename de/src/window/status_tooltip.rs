use gtk4::prelude::*;
use gtk4::Widget;
use std::rc::Rc;

use crate::service::audio::Audio;
use crate::service::battery::Battery;
use crate::service::bluetooth::{Bluetooth, BluetoothDevice};
use crate::service::network::{Network, NetworkState, NetworkType};

pub fn pretty_time(seconds: i64) -> String {
    format!("{}h:{}m", seconds / 3600, (seconds % 3600) / 60)
}

pub fn battery_text(percentage: f64, time_to_full: i64, time_to_empty: i64) -> String {
    let mut text = format!("Battery: {}%", (percentage * 100.0).round() as i32);
    if time_to_full != 0 {
        text.push_str(&format!("\nCharging: {}", pretty_time(time_to_full)));
    }
    if time_to_empty != 0 {
        text.push_str(&format!("\nRemaining: {}", pretty_time(time_to_empty)));
    }
    text
}

/// The connection details shown for the primary network.
pub struct NetworkInfo<'a> {
    pub state: NetworkState,
    pub primary: NetworkType,
    pub ssid: &'a str,
    pub strength: u8,
    pub frequency: u32,
    pub wired_id: &'a str,
    pub wired_product: &'a str,
}

/// Formats a Wi-Fi frequency in MHz as GHz with its band and channel.
pub fn frequency_text(frequency: u32) -> String {
    let (band, channel) = match frequency {
        2484 => ("2.4", 14),
        2412..=2472 => ("2.4", (frequency - 2407) / 5),
        5000..=5924 => ("5", (frequency - 5000) / 5),
        5925..=7125 => ("6", (frequency - 5950) / 5),
        _ => return format!("{:.2} GHz", frequency as f64 / 1000.0),
    };
    format!(
        "{:.2} GHz ({} GHz band, channel {})",
        frequency as f64 / 1000.0,
        band,
        channel
    )
}

pub fn network_text(info: &NetworkInfo) -> String {
    let status = match info.state {
        NetworkState::ConnectedGlobal => None,
        NetworkState::ConnectedLocal | NetworkState::ConnectedSite => Some("No internet"),
        NetworkState::Connecting => Some("Connecting"),
        NetworkState::Disconnecting => Some("Disconnecting"),
        NetworkState::Asleep | NetworkState::Disconnected | NetworkState::Unknown => {
            return "Network: Offline".to_string();
        }
    };
    let mut text = match info.primary {
        NetworkType::Wifi => {
            let mut text = format!("Network: Wi-Fi {}\n  Signal: {}%", info.ssid, info.strength);
            if info.frequency != 0 {
                text.push_str(&format!("\n  Frequency: {}", frequency_text(info.frequency)));
            }
            text
        }
        NetworkType::Wired => {
            let mut text = format!("Network: Wired {}", info.wired_id);
            if !info.wired_product.is_empty() {
                text.push_str(&format!("\n  Device: {}", info.wired_product));
            }
            text
        }
        NetworkType::Unknown => "Network: Connected".to_string(),
    };
    if let Some(status) = status {
        text.push_str(&format!("\n  Status: {}", status));
    }
    text
}

pub fn bluetooth_text(enabled: bool, devices: &[BluetoothDevice]) -> String {
    if !enabled {
        return "Bluetooth: Off".to_string();
    }
    let paired = devices.iter().filter(|device| device.paired).count();
    let connected: Vec<&BluetoothDevice> = devices.iter().filter(|d| d.connected).collect();
    let mut text = format!(
        "Bluetooth: On, {} connected, {} paired",
        connected.len(),
        paired
    );
    for device in connected {
        text.push_str(&format!("\n  {}", device.name));
        if let Some(battery) = device.battery_percentage {
            text.push_str(&format!("\n    Battery: {}%", battery));
        }
        if !device.address.is_empty() {
            text.push_str(&format!("\n    Address: {}", device.address));
        }
    }
    text
}

pub fn audio_text(label: &str, volume: f64, muted: bool) -> String {
    let percentage = (volume * 100.0).round() as i32;
    if muted {
        format!("{}: Muted ({}%)", label, percentage)
    } else {
        format!("{}: {}%", label, percentage)
    }
}

fn status_text() -> String {
    let battery = Battery::instance();
    let network = Network::instance();
    let wifi = network.wifi();
    let bluetooth = Bluetooth::instance();
    let audio = Audio::instance();
    [
        battery_text(
            battery.percentage(),
            battery.time_to_full(),
            battery.time_to_empty(),
        ),
        network_text(&NetworkInfo {
            state: network.state(),
            primary: network.primary(),
            ssid: &wifi.ssid(),
            strength: wifi.strength(),
            frequency: wifi.frequency(),
            wired_id: &network.ethernet().connection_id(),
            wired_product: &network.ethernet().device_product(),
        }),
        bluetooth_text(bluetooth.enabled(), &bluetooth.devices()),
        audio_text("Volume", audio.volume(), audio.muted()),
        audio_text("Microphone", audio.mic_volume(), audio.mic_muted()),
    ]
    .join("\n")
}

/// Keeps the widget's tooltip in sync with the battery, network, bluetooth and audio state.
pub fn connect(widget: &impl IsA<Widget>) {
    let widget = widget.as_ref().downgrade();
    let update: Rc<dyn Fn()> = Rc::new(move || {
        if let Some(widget) = widget.upgrade() {
            widget.set_tooltip_text(Some(&status_text()));
        }
    });
    let watch = |object: &glib::Object, properties: &[&str]| {
        for property in properties {
            let update = update.clone();
            object.connect_notify_local(Some(property), move |_, _| update());
        }
    };

    let network = Network::instance();
    watch(
        Battery::instance().upcast_ref(),
        &["percentage", "time-to-full", "time-to-empty"],
    );
    watch(network.upcast_ref(), &["state", "primary"]);
    watch(network.wifi().upcast_ref(), &["ssid", "strength", "frequency"]);
    watch(
        network.ethernet().upcast_ref(),
        &["connection-id", "device-product"],
    );
    watch(Bluetooth::instance().upcast_ref(), &["enabled", "devices"]);
    watch(
        Audio::instance().upcast_ref(),
        &["volume", "muted", "mic-volume", "mic-muted"],
    );
    update();
}
