use glib::subclass::prelude::*;
use glib::Object;
use gtk4::glib;
use std::cell::{Cell, RefCell};
use std::sync::OnceLock;
use zbus::Connection;

glib::wrapper! {
    pub struct Battery(ObjectSubclass<imp::Battery>);
}

thread_local! {
    static BATTERY_INSTANCE: RefCell<Option<Battery>> = const { RefCell::new(None) };
}

#[derive(Debug, Clone, PartialEq)]
pub struct KeyboardBattery {
    pub name: String,
    pub address: String,
    pub central_percentage: Option<u8>,
    pub peripheral_percentage: Option<u8>,
}

#[derive(Debug, Clone)]
pub(crate) struct BatteryCharacteristicPath {
    keyboard_address: String,
    characteristic_path: String,
    is_peripheral: bool,
}

impl Battery {
    pub fn instance() -> Battery {
        BATTERY_INSTANCE.with(|instance| {
            let mut instance_mut = instance.borrow_mut();
            if instance_mut.is_none() {
                let obj: Battery = Object::builder().build();
                *instance_mut = Some(obj);
            }
            instance_mut.as_ref().unwrap().clone()
        })
    }

    fn new() -> Self {
        Object::builder().build()
    }

    pub fn percentage(&self) -> f64 {
        self.imp().percentage.get()
    }

    pub fn state(&self) -> BatteryState {
        BatteryState::from_u32(self.imp().state.get())
    }

    pub fn time_to_full(&self) -> i64 {
        self.imp().time_to_full.get()
    }

    pub fn time_to_empty(&self) -> i64 {
        self.imp().time_to_empty.get()
    }

    pub fn keyboard_batteries(&self) -> Vec<KeyboardBattery> {
        self.imp().keyboard_batteries.borrow().clone()
    }

    pub fn icon_name(&self) -> String {
        let percentage = self.percentage();
        let state = self.state();

        match state {
            BatteryState::Charging => {
                if percentage >= 0.8 {
                    "battery-level-90-charging-symbolic".to_string()
                } else if percentage >= 0.7 {
                    "battery-level-80-charging-symbolic".to_string()
                } else if percentage >= 0.6 {
                    "battery-level-70-charging-symbolic".to_string()
                } else if percentage >= 0.5 {
                    "battery-level-60-charging-symbolic".to_string()
                } else if percentage >= 0.4 {
                    "battery-level-50-charging-symbolic".to_string()
                } else if percentage >= 0.3 {
                    "battery-level-40-charging-symbolic".to_string()
                } else if percentage >= 0.2 {
                    "battery-level-30-charging-symbolic".to_string()
                } else if percentage >= 0.1 {
                    "battery-level-20-charging-symbolic".to_string()
                } else if percentage >= 0.05 {
                    "battery-level-10-charging-symbolic".to_string()
                } else {
                    "battery-level-0-charging-symbolic".to_string()
                }
            }
            BatteryState::FullyCharged => "battery-level-100-charged-symbolic".to_string(),
            _ => {
                if percentage >= 0.9 {
                    "battery-level-100-symbolic".to_string()
                } else if percentage >= 0.8 {
                    "battery-level-90-symbolic".to_string()
                } else if percentage >= 0.7 {
                    "battery-level-80-symbolic".to_string()
                } else if percentage >= 0.6 {
                    "battery-level-70-symbolic".to_string()
                } else if percentage >= 0.5 {
                    "battery-level-60-symbolic".to_string()
                } else if percentage >= 0.4 {
                    "battery-level-50-symbolic".to_string()
                } else if percentage >= 0.3 {
                    "battery-level-40-symbolic".to_string()
                } else if percentage >= 0.2 {
                    "battery-level-30-symbolic".to_string()
                } else if percentage >= 0.1 {
                    "battery-level-20-symbolic".to_string()
                } else if percentage >= 0.05 {
                    "battery-level-10-symbolic".to_string()
                } else {
                    "battery-level-0-symbolic".to_string()
                }
            }
        }
    }
}

impl Default for Battery {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatteryState {
    Unknown = 0,
    Charging = 1,
    Discharging = 2,
    Empty = 3,
    FullyCharged = 4,
    PendingCharge = 5,
    PendingDischarge = 6,
}

impl BatteryState {
    pub fn from_u32(value: u32) -> Self {
        match value {
            1 => BatteryState::Charging,
            2 => BatteryState::Discharging,
            3 => BatteryState::Empty,
            4 => BatteryState::FullyCharged,
            5 => BatteryState::PendingCharge,
            6 => BatteryState::PendingDischarge,
            _ => BatteryState::Unknown,
        }
    }
}

mod imp {
    use super::*;
    use glib::clone;
    use glib::prelude::*;
    use glib::subclass::prelude::*;
    use glib::{ParamSpec, ParamSpecBuilderExt, Value};
    use zbus::proxy;

    pub struct Battery {
        pub percentage: Cell<f64>,
        pub state: Cell<u32>,
        pub time_to_full: Cell<i64>,
        pub time_to_empty: Cell<i64>,
        pub keyboard_batteries: RefCell<Vec<KeyboardBattery>>,
        battery_characteristic_paths: RefCell<Vec<BatteryCharacteristicPath>>,
    }

    impl Default for Battery {
        fn default() -> Self {
            Self {
                percentage: Cell::new(0.0),
                state: Cell::new(0),
                time_to_full: Cell::new(0),
                time_to_empty: Cell::new(0),
                keyboard_batteries: RefCell::new(Vec::new()),
                battery_characteristic_paths: RefCell::new(Vec::new()),
            }
        }
    }

    #[proxy(
        interface = "org.freedesktop.UPower.Device",
        default_service = "org.freedesktop.UPower",
        default_path = "/org/freedesktop/UPower/devices/DisplayDevice"
    )]
    trait UPowerDevice {
        #[zbus(property)]
        fn percentage(&self) -> zbus::Result<f64>;

        #[zbus(property)]
        fn state(&self) -> zbus::Result<u32>;

        #[zbus(property)]
        fn time_to_full(&self) -> zbus::Result<i64>;

        #[zbus(property)]
        fn time_to_empty(&self) -> zbus::Result<i64>;
    }

    #[proxy(
        interface = "org.bluez.GattCharacteristic1",
        default_service = "org.bluez"
    )]
    trait GattCharacteristic {
        fn read_value(
            &self,
            options: std::collections::HashMap<String, zbus::zvariant::Value<'_>>,
        ) -> zbus::Result<Vec<u8>>;

        #[zbus(property)]
        fn value(&self) -> zbus::Result<Vec<u8>>;
    }

    #[proxy(interface = "org.bluez.Device1", default_service = "org.bluez")]
    trait BluezDevice {
        #[zbus(property)]
        fn name(&self) -> zbus::Result<String>;

        #[zbus(property)]
        fn address(&self) -> zbus::Result<String>;

        #[zbus(property)]
        fn connected(&self) -> zbus::Result<bool>;

        #[zbus(property)]
        fn appearance(&self) -> zbus::Result<u16>;
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Battery {
        const NAME: &'static str = "Battery";
        type Type = super::Battery;
    }

    impl ObjectImpl for Battery {
        fn constructed(&self) {
            self.parent_constructed();

            // Initialize current state
            self.update_battery_state();

            // Setup event listener
            self.setup_event_listener();

            // Initialize keyboard batteries and set up periodic updates
            self.setup_keyboard_battery_updates();
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: OnceLock<Vec<ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![
                    glib::ParamSpecDouble::builder("percentage")
                        .minimum(0.0)
                        .maximum(1.0)
                        .read_only()
                        .build(),
                    glib::ParamSpecUInt::builder("state").read_only().build(),
                    glib::ParamSpecInt64::builder("time-to-full")
                        .read_only()
                        .build(),
                    glib::ParamSpecInt64::builder("time-to-empty")
                        .read_only()
                        .build(),
                ]
            })
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "percentage" => self.percentage.get().to_value(),
                "state" => self.state.get().to_value(),
                "time-to-full" => self.time_to_full.get().to_value(),
                "time-to-empty" => self.time_to_empty.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl Battery {
        fn setup_event_listener(&self) {
            let obj = self.obj().clone();

            // Spawn async D-Bus listener on glib main context - fully event-driven!
            glib::MainContext::default().spawn_local(async move {
                let connection = match Connection::system().await {
                    Ok(conn) => conn,
                    Err(e) => {
                        let err_msg = format!("Failed to connect to D-Bus: {}", e);
                        crate::service::idle_add_safe(move || {
                            crate::service::notifications::Notifications::instance()
                                .send_notification("rusty-de", "Battery Error", &err_msg);
                        });
                        return;
                    }
                };

                let proxy = match UPowerDeviceProxy::new(&connection).await {
                    Ok(p) => p,
                    Err(e) => {
                        let err_msg = format!("Failed to create UPower proxy: {}", e);
                        crate::service::idle_add_safe(move || {
                            crate::service::notifications::Notifications::instance()
                                .send_notification("rusty-de", "Battery Error", &err_msg);
                        });
                        return;
                    }
                };

                use futures_util::stream::{select_all, StreamExt};

                let percentage_stream = proxy.receive_percentage_changed().await.map(|_| ());
                let state_stream = proxy.receive_state_changed().await.map(|_| ());

                let mut combined_stream = select_all(vec![
                    Box::pin(percentage_stream)
                        as std::pin::Pin<Box<dyn futures_util::Stream<Item = ()> + Send>>,
                    Box::pin(state_stream)
                        as std::pin::Pin<Box<dyn futures_util::Stream<Item = ()> + Send>>,
                ]);

                loop {
                    if combined_stream.next().await.is_some() {
                        let changes = obj.imp().update_battery_state();
                        if changes.percentage_changed {
                            obj.notify("percentage");
                        }
                        if changes.state_changed {
                            obj.notify("state");
                        }
                        if changes.time_to_full_changed {
                            obj.notify("time-to-full");
                        }
                        if changes.time_to_empty_changed {
                            obj.notify("time-to-empty");
                        }
                    } else {
                        break;
                    }
                }
            });
        }

        fn update_battery_state(&self) -> BatteryChanges {
            let mut changes = BatteryChanges::default();

            let runtime =
                match tokio::runtime::Runtime::new() {
                    Ok(rt) => rt,
                    Err(e) => {
                        let err_msg = format!("Failed to create async runtime: {}", e);
                        crate::service::idle_add_safe(move || {
                            crate::service::notifications::Notifications::instance()
                                .send_notification("rusty-de", "Battery Error", &err_msg);
                        });
                        return changes;
                    }
                };

            runtime.block_on(async {
                let connection = match Connection::system().await {
                    Ok(conn) => conn,
                    Err(e) => {
                        let err_msg = format!("Failed to connect to D-Bus: {}", e);
                        crate::service::idle_add_safe(move || {
                            crate::service::notifications::Notifications::instance()
                                .send_notification("rusty-de", "Battery Error", &err_msg);
                        });
                        return;
                    }
                };

                let proxy = match UPowerDeviceProxy::new(&connection).await {
                    Ok(p) => p,
                    Err(e) => {
                        let err_msg = format!("Failed to create UPower proxy: {}", e);
                        crate::service::idle_add_safe(move || {
                            crate::service::notifications::Notifications::instance()
                                .send_notification("rusty-de", "Battery Error", &err_msg);
                        });
                        return;
                    }
                };

                // Get percentage
                if let Ok(percentage) = proxy.percentage().await {
                    let percentage = percentage / 100.0;
                    let old_percentage = self.percentage.get();
                    if (percentage - old_percentage).abs() > 0.001 {
                        self.percentage.set(percentage);
                        changes.percentage_changed = true;
                    }
                }

                // Get state
                if let Ok(state) = proxy.state().await {
                    let old_state = self.state.get();
                    if state != old_state {
                        self.state.set(state);
                        changes.state_changed = true;
                    }
                }

                // Get time to full
                if let Ok(time_to_full) = proxy.time_to_full().await {
                    let old_time_to_full = self.time_to_full.get();
                    if time_to_full != old_time_to_full {
                        self.time_to_full.set(time_to_full);
                        changes.time_to_full_changed = true;
                    }
                }

                // Get time to empty
                if let Ok(time_to_empty) = proxy.time_to_empty().await {
                    let old_time_to_empty = self.time_to_empty.get();
                    if time_to_empty != old_time_to_empty {
                        self.time_to_empty.set(time_to_empty);
                        changes.time_to_empty_changed = true;
                    }
                }
            });

            changes
        }

        fn setup_keyboard_battery_updates(&self) {
            let obj = self.obj().clone();

            // Initial update and setup signal monitoring for real-time updates
            glib::MainContext::default().spawn_local(clone!(
                #[weak]
                obj,
                async move {
                    if let Ok(connection) = Connection::system().await {
                        let (keyboards, paths) = Self::update_keyboard_batteries(&connection).await;
                        let had_keyboards = !keyboards.is_empty();
                        obj.imp().keyboard_batteries.replace(keyboards);
                        obj.imp()
                            .battery_characteristic_paths
                            .replace(paths.clone());

                        // Trigger Bluetooth device list refresh if we found keyboards
                        if had_keyboards {
                            use crate::service::bluetooth::Bluetooth;
                            // Trigger a devices property notification to refresh the UI
                            Bluetooth::instance().notify("devices");
                        }

                        // Set up signal monitoring for real-time updates (replaces polling)
                        if !paths.is_empty() {
                            Self::setup_battery_signal_monitoring(obj.clone(), connection, paths)
                                .await;
                        }
                    }
                }
            ));
        }

        async fn update_keyboard_batteries(
            connection: &Connection,
        ) -> (Vec<KeyboardBattery>, Vec<BatteryCharacteristicPath>) {
            let mut keyboards = Vec::new();
            let mut characteristic_paths = Vec::new();

            // List all BlueZ devices
            let object_manager = match zbus::fdo::ObjectManagerProxy::builder(connection)
                .destination("org.bluez")
                .and_then(|b| b.path("/"))
                .ok()
            {
                Some(builder) => match builder.build().await {
                    Ok(om) => om,
                    Err(_e) => {
                        return (keyboards, characteristic_paths);
                    }
                },
                None => {
                    return (keyboards, characteristic_paths);
                }
            };

            let managed_objects = match object_manager.get_managed_objects().await {
                Ok(objs) => objs,
                Err(_e) => {
                    return (keyboards, characteristic_paths);
                }
            };

            for (path, interfaces) in &managed_objects {
                // Check if this is a device and if it's a keyboard (appearance 0x03c1 = 961)
                let device_props = interfaces
                    .iter()
                    .find(|(name, _)| name.as_str() == "org.bluez.Device1")
                    .map(|(_, props)| props);
                if let Some(device_props) = device_props {
                    if let Some(appearance_value) = device_props.get("Appearance") {
                        if let Ok(appearance) = <u16>::try_from(appearance_value) {
                            if appearance == 961 {
                                // Keyboard appearance
                                // Check if connected
                                if let Some(connected_value) = device_props.get("Connected") {
                                    if let Ok(true) = <bool>::try_from(connected_value) {
                                        // Get name and address
                                        if let (Some(name_value), Some(address_value)) =
                                            (device_props.get("Name"), device_props.get("Address"))
                                        {
                                            if let (Ok(name), Ok(address)) = (
                                                <&str>::try_from(name_value),
                                                <&str>::try_from(address_value),
                                            ) {
                                                // Found a connected keyboard, now look for battery services
                                                let path_str = path.as_str();
                                                let device_path_prefix =
                                                    format!("{}/service", path_str);

                                                // Look for battery service characteristics
                                                let mut central_battery = None;
                                                let mut peripheral_battery = None;

                                                for (service_path, service_interfaces) in
                                                    &managed_objects
                                                {
                                                    let service_path_str = service_path.as_str();
                                                    if service_path_str
                                                        .starts_with(&device_path_prefix)
                                                    {
                                                        // Check for battery service (0000180f-0000-1000-8000-00805f9b34fb)
                                                        let service_props = service_interfaces
                                                            .iter()
                                                            .find(|(name, _)| {
                                                                name.as_str()
                                                                    == "org.bluez.GattService1"
                                                            })
                                                            .map(|(_, props)| props);
                                                        if let Some(service_props) = service_props {
                                                            if let Some(uuid_value) =
                                                                service_props.get("UUID")
                                                            {
                                                                if let Ok(uuid) =
                                                                    <&str>::try_from(uuid_value)
                                                                {
                                                                    if uuid == "0000180f-0000-1000-8000-00805f9b34fb" {
                                                                                                                // Found a battery service, now find the characteristic
                                                        for (char_path, char_interfaces) in &managed_objects {
                                                            let char_path_str = char_path.as_str();
                                                            if char_path_str.starts_with(service_path_str) && char_path_str.contains("/char") {
                                                                let char_props = char_interfaces.iter().find(|(name, _)| name.as_str() == "org.bluez.GattCharacteristic1").map(|(_, props)| props);
                                                                if let Some(char_props) = char_props {
                                                                    if let Some(char_uuid_value) = char_props.get("UUID") {
                                                                        if let Ok(char_uuid) = <&str>::try_from(char_uuid_value) {
                                                                                                                                                        // Battery Level characteristic
                                                                            if char_uuid == "00002a19-0000-1000-8000-00805f9b34fb" {
                                                                                                                                                                // Read the battery level
                                                                                if let Ok(level) = Self::read_battery_characteristic(connection, char_path).await {
                                                                                                                                                                                // Check descriptor to determine if central or peripheral
                                                                                    let is_peripheral = Self::is_peripheral_battery(connection, char_path, &managed_objects).await;

                                                                                    // Store characteristic path for monitoring
                                                                                    characteristic_paths.push(BatteryCharacteristicPath {
                                                                                        keyboard_address: address.to_string(),
                                                                                        characteristic_path: char_path.as_str().to_string(),
                                                                                        is_peripheral,
                                                                                    });

                                                                                    if is_peripheral {
                                                                                        peripheral_battery = Some(level);
                                                                                    } else if central_battery.is_none() {
                                                                                        central_battery = Some(level);
                                                                                    } else if peripheral_battery.is_none() {
                                                                                        // If we already have a central battery and this isn't marked as peripheral,
                                                                                        // assume it's the peripheral battery (for split keyboards with 2 batteries)
                                                                                                                                                                                    peripheral_battery = Some(level);
                                                                                        // Update the last added characteristic path to mark it as peripheral
                                                                                        if let Some(last_path) = characteristic_paths.last_mut() {
                                                                                            if last_path.keyboard_address == address {
                                                                                                last_path.is_peripheral = true;
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }

                                                // Only add if we found at least one battery

                                                if central_battery.is_some()
                                                    || peripheral_battery.is_some()
                                                {
                                                    let kb = KeyboardBattery {
                                                        name: name.to_string(),
                                                        address: address.to_string(),
                                                        central_percentage: central_battery,
                                                        peripheral_percentage: peripheral_battery,
                                                    };
                                                    keyboards.push(kb);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            for _kb in &keyboards {}
            for _path in &characteristic_paths {}
            (keyboards, characteristic_paths)
        }

        async fn setup_battery_signal_monitoring(
            obj: super::Battery,
            connection: Connection,
            paths: Vec<BatteryCharacteristicPath>,
        ) {
            let _paths_count = paths.len();

            use futures_util::stream::StreamExt;
            use glib::clone;

            // Spawn a monitoring task for each characteristic
            for path_info in paths {
                let connection = connection.clone();
                let char_path = path_info.characteristic_path.clone();
                let keyboard_address = path_info.keyboard_address.clone();
                let is_peripheral = path_info.is_peripheral;

                // Spawn task in glib context (handles non-Send types like Battery)
                glib::MainContext::default().spawn_local(clone!(
                    #[strong]
                    obj,
                    async move {
                        // Create proxy for this characteristic
                        let proxy_result = GattCharacteristicProxy::builder(&connection)
                            .path(char_path.clone())
                            .ok();

                        if let Some(builder) = proxy_result {
                            let proxy = match builder.build().await {
                                Ok(p) => p,
                                Err(_e) => {
                                    return;
                                }
                            };
                            // Subscribe to Value property changes
                            let mut value_stream = proxy.receive_value_changed().await;

                            while let Some(change) = value_stream.next().await {
                                if let Ok(new_value) = change.get().await {
                                    if !new_value.is_empty() {
                                        let battery_level = new_value[0];

                                        // Update the keyboard_batteries data
                                        Self::update_battery_value(
                                            &obj,
                                            &keyboard_address,
                                            is_peripheral,
                                            battery_level,
                                        );

                                        // Trigger UI refresh
                                        use crate::service::bluetooth::Bluetooth;
                                        Bluetooth::instance().notify("devices");
                                    }
                                }
                            }
                        }
                    }
                ));
            }
        }

        fn update_battery_value(
            obj: &super::Battery,
            keyboard_address: &str,
            is_peripheral: bool,
            battery_level: u8,
        ) {
            let mut keyboards = obj.imp().keyboard_batteries.borrow_mut();

            // Find the keyboard by address
            if let Some(keyboard) = keyboards
                .iter_mut()
                .find(|kb| kb.address == keyboard_address)
            {
                if is_peripheral {
                    keyboard.peripheral_percentage = Some(battery_level);
                } else {
                    keyboard.central_percentage = Some(battery_level);
                }
            }
        }

        async fn read_battery_characteristic(
            connection: &Connection,
            char_path: &zbus::zvariant::OwnedObjectPath,
        ) -> Result<u8, ()> {
            let builder = GattCharacteristicProxy::builder(connection)
                .path(char_path)
                .ok();

            if let Some(builder) = builder {
                if let Ok(proxy) = builder.build().await {
                    let options = std::collections::HashMap::new();
                    if let Ok(value) = proxy.read_value(options).await {
                        if !value.is_empty() {
                            return Ok(value[0]);
                        }
                    }
                }
            }
            Err(())
        }

        async fn is_peripheral_battery(
            connection: &Connection,
            char_path: &zbus::zvariant::OwnedObjectPath,
            managed_objects: &std::collections::HashMap<
                zbus::zvariant::OwnedObjectPath,
                std::collections::HashMap<
                    zbus::names::OwnedInterfaceName,
                    std::collections::HashMap<String, zbus::zvariant::OwnedValue>,
                >,
            >,
        ) -> bool {
            // Look for a descriptor with "Peripheral" in the user description
            let char_path_str = char_path.as_str();

            for (desc_path, desc_interfaces) in managed_objects {
                let desc_path_str = desc_path.as_str();
                if desc_path_str.starts_with(char_path_str) && desc_path_str.contains("/desc") {
                    // Find GattDescriptor1 interface
                    let desc_props = desc_interfaces
                        .iter()
                        .find(|(name, _)| name.as_str() == "org.bluez.GattDescriptor1")
                        .map(|(_, props)| props);
                    if let Some(desc_props) = desc_props {
                        if let Some(desc_uuid_value) = desc_props.get("UUID") {
                            if let Ok(desc_uuid) = <&str>::try_from(desc_uuid_value) {
                                // Characteristic User Description UUID
                                if desc_uuid == "00002901-0000-1000-8000-00805f9b34fb" {
                                    // Read the descriptor value
                                    let builder = GattCharacteristicProxy::builder(connection)
                                        .path(desc_path)
                                        .ok();

                                    if let Some(builder) = builder {
                                        if let Ok(proxy) = builder.build().await {
                                            let options = std::collections::HashMap::new();
                                            if let Ok(value) = proxy.read_value(options).await {
                                                if let Ok(text) = String::from_utf8(value.clone()) {
                                                    if text.contains("Peripheral") {
                                                        return true;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            false
        }
    }

    #[derive(Default)]
    struct BatteryChanges {
        percentage_changed: bool,
        state_changed: bool,
        time_to_full_changed: bool,
        time_to_empty_changed: bool,
    }
}
