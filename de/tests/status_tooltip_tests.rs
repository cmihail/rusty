use rusty_de::service::bluetooth::BluetoothDevice;
use rusty_de::service::network::{NetworkState, NetworkType};
use rusty_de::window::status_tooltip::{
    audio_text, battery_text, bluetooth_text, frequency_text, network_text, NetworkInfo,
};

fn device(name: &str, connected: bool, battery: Option<u8>) -> BluetoothDevice {
    BluetoothDevice {
        path: format!("/org/bluez/{}", name),
        name: name.to_string(),
        address: "AA:BB:CC:DD:EE:01".to_string(),
        connected,
        paired: true,
        battery_percentage: battery,
        nearby: true,
    }
}

#[test]
fn test_battery_text_with_times() {
    assert_eq!(battery_text(0.874, 0, 0), "Battery: 87%");
    assert_eq!(battery_text(0.5, 3660, 0), "Battery: 50%\nCharging: 1h:1m");
    assert_eq!(battery_text(0.5, 0, 7320), "Battery: 50%\nRemaining: 2h:2m");
}

fn wifi(state: NetworkState, frequency: u32) -> NetworkInfo<'static> {
    NetworkInfo {
        state,
        primary: NetworkType::Wifi,
        ssid: "Home",
        strength: 72,
        frequency,
        wired_id: "",
        wired_product: "",
    }
}

#[test]
fn test_frequency_text() {
    assert_eq!(frequency_text(2437), "2.44 GHz (2.4 GHz band, channel 6)");
    assert_eq!(frequency_text(2484), "2.48 GHz (2.4 GHz band, channel 14)");
    assert_eq!(frequency_text(5180), "5.18 GHz (5 GHz band, channel 36)");
    assert_eq!(frequency_text(5975), "5.97 GHz (6 GHz band, channel 5)");
    assert_eq!(frequency_text(60480), "60.48 GHz");
}

#[test]
fn test_network_text_wifi() {
    assert_eq!(
        network_text(&wifi(NetworkState::ConnectedGlobal, 5180)),
        "Network: Wi-Fi Home\n  Signal: 72%\n  Frequency: 5.18 GHz (5 GHz band, channel 36)"
    );
    assert_eq!(
        network_text(&wifi(NetworkState::ConnectedLocal, 0)),
        "Network: Wi-Fi Home\n  Signal: 72%\n  Status: No internet"
    );
    assert_eq!(
        network_text(&wifi(NetworkState::Disconnected, 0)),
        "Network: Offline"
    );
}

#[test]
fn test_network_text_wired() {
    let mut info = wifi(NetworkState::ConnectedGlobal, 0);
    info.primary = NetworkType::Wired;
    info.wired_id = "Office";
    assert_eq!(network_text(&info), "Network: Wired Office");
    info.wired_product = "RTL8111";
    assert_eq!(
        network_text(&info),
        "Network: Wired Office\n  Device: RTL8111"
    );
}

#[test]
fn test_bluetooth_text() {
    assert_eq!(bluetooth_text(false, &[]), "Bluetooth: Off");
    assert_eq!(
        bluetooth_text(true, &[device("Mouse", false, None)]),
        "Bluetooth: On, 0 connected, 1 paired"
    );
    let devices = [
        device("Keyboard", true, Some(80)),
        device("Buds", true, None),
    ];
    assert_eq!(
        bluetooth_text(true, &devices),
        "Bluetooth: On, 2 connected, 2 paired\n  Keyboard\n    Battery: 80%\n    Address: \
         AA:BB:CC:DD:EE:01\n  Buds\n    Address: AA:BB:CC:DD:EE:01"
    );
}

#[test]
fn test_audio_text() {
    assert_eq!(audio_text("Volume", 0.454, false), "Volume: 45%");
    assert_eq!(
        audio_text("Microphone", 0.6, true),
        "Microphone: Muted (60%)"
    );
}
