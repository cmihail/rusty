use glib::prelude::ObjectExt;
use glib::subclass::prelude::*;
use glib::Object;
use gtk4::glib;
use serde::Deserialize;
use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ConfigData {
    #[serde(default)]
    pub bluetooth: BluetoothConfig,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct BluetoothConfig {
    #[serde(default)]
    pub expected_devices: Vec<ExpectedDevice>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ExpectedDevice {
    pub address: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub name: Option<String>,
}

glib::wrapper! {
    pub struct Config(ObjectSubclass<imp::Config>);
}

thread_local! {
    static CONFIG_INSTANCE: RefCell<Option<Config>> = const { RefCell::new(None) };
}

impl Config {
    pub fn instance() -> Config {
        CONFIG_INSTANCE.with(|instance| {
            let mut instance_mut = instance.borrow_mut();
            if instance_mut.is_none() {
                let obj: Config = Object::builder().build();
                *instance_mut = Some(obj);
            }
            instance_mut.as_ref().unwrap().clone()
        })
    }

    pub fn bluetooth_expected_devices(&self) -> Vec<ExpectedDevice> {
        self.imp().bluetooth_expected_devices.borrow().clone()
    }

    pub fn reload(&self) {
        let config_path = Self::config_path();

        let new_bluetooth_expected_devices = if let Ok(contents) = fs::read_to_string(&config_path)
        {
            if let Ok(config) = toml::from_str::<ConfigData>(&contents) {
                config.bluetooth.expected_devices
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        let old_devices = self.bluetooth_expected_devices();
        if old_devices != new_bluetooth_expected_devices {
            *self.imp().bluetooth_expected_devices.borrow_mut() = new_bluetooth_expected_devices;
            self.notify("bluetooth-expected-devices");
        }
    }

    pub fn config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home).join(".config/rusty/de.toml")
    }
}

mod imp {
    use super::*;
    use glib::prelude::ToValue;
    use glib::subclass::prelude::*;
    use glib::{ParamSpec, ParamSpecBuilderExt};
    use std::cell::RefCell;
    use std::sync::OnceLock;

    pub struct Config {
        pub bluetooth_expected_devices: RefCell<Vec<ExpectedDevice>>,
    }

    impl Default for Config {
        fn default() -> Self {
            Self {
                bluetooth_expected_devices: RefCell::new(Vec::new()),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Config {
        const NAME: &'static str = "Config";
        type Type = super::Config;
    }

    impl ObjectImpl for Config {
        fn constructed(&self) {
            self.parent_constructed();

            let config_path = super::Config::config_path();
            if let Ok(contents) = std::fs::read_to_string(&config_path) {
                if let Ok(config) = toml::from_str::<ConfigData>(&contents) {
                    *self.bluetooth_expected_devices.borrow_mut() =
                        config.bluetooth.expected_devices;
                }
            }
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: OnceLock<Vec<ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![glib::ParamSpecString::builder("bluetooth-expected-devices")
                    .read_only()
                    .build()]
            })
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "bluetooth-expected-devices" => {
                    format!("{:?}", self.bluetooth_expected_devices.borrow()).to_value()
                }
                _ => unimplemented!(),
            }
        }
    }
}
