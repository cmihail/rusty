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
    #[serde(default)]
    pub dock: DockConfig,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct DockConfig {
    /// Desktop entry ids, without `.desktop`, in dock order.
    #[serde(default)]
    pub pinned: Vec<String>,
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

    pub fn dock_pinned(&self) -> Vec<String> {
        self.imp().dock_pinned.borrow().clone()
    }

    /// Writes the pinned apps to the config file, keeping the rest of it as it is.
    pub fn set_dock_pinned(&self, pinned: Vec<String>) -> Result<(), String> {
        let path = Self::config_path();
        let contents = match fs::read_to_string(&path) {
            Ok(contents) => contents,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(e) => return Err(format!("failed to read {}: {}", path.display(), e)),
        };
        let updated = with_dock_pinned(&contents, &pinned)?;
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        fs::write(&path, updated)
            .map_err(|e| format!("failed to write {}: {}", path.display(), e))?;
        self.apply(ConfigData {
            dock: DockConfig { pinned },
            bluetooth: BluetoothConfig {
                expected_devices: self.bluetooth_expected_devices(),
            },
        });
        Ok(())
    }

    pub fn reload(&self) {
        self.apply(Self::read());
    }

    fn read() -> ConfigData {
        fs::read_to_string(Self::config_path())
            .ok()
            .and_then(|contents| toml::from_str::<ConfigData>(&contents).ok())
            .unwrap_or_default()
    }

    fn apply(&self, config: ConfigData) {
        let devices = config.bluetooth.expected_devices;
        if self.bluetooth_expected_devices() != devices {
            *self.imp().bluetooth_expected_devices.borrow_mut() = devices;
            self.notify("bluetooth-expected-devices");
        }
        if self.dock_pinned() != config.dock.pinned {
            *self.imp().dock_pinned.borrow_mut() = config.dock.pinned;
            self.notify("dock-pinned");
        }
    }

    pub fn config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home).join(".config/rusty/de.toml")
    }
}

/// `contents` with `[dock] pinned` set to `pinned`, keeping its comments and layout.
pub fn with_dock_pinned(contents: &str, pinned: &[String]) -> Result<String, String> {
    let mut document = contents
        .parse::<toml_edit::DocumentMut>()
        .map_err(|e| format!("invalid config: {}", e))?;
    let dock = document
        .entry("dock")
        .or_insert_with(toml_edit::table)
        .as_table_like_mut()
        .ok_or("dock in the config is not a table")?;
    let ids: toml_edit::Array = pinned.iter().map(String::as_str).collect();
    match dock.get_mut("pinned") {
        Some(item) => *item = toml_edit::value(ids),
        None => {
            dock.insert("pinned", toml_edit::value(ids));
        }
    }
    Ok(document.to_string())
}

/// `pinned` with `id` removed when it is there, otherwise added at the end.
pub fn toggled_pin(mut pinned: Vec<String>, id: &str) -> Vec<String> {
    match pinned.iter().position(|pin| pin == id) {
        Some(index) => {
            pinned.remove(index);
        }
        None => pinned.push(id.to_string()),
    }
    pinned
}

/// Moves pinned `id` next to pinned `target`, after it when `after` is set.
pub fn moved_pin(mut pinned: Vec<String>, id: &str, target: &str, after: bool) -> Vec<String> {
    if id == target || !pinned.iter().any(|pin| pin == target) {
        return pinned;
    }
    let Some(from) = pinned.iter().position(|pin| pin == id) else {
        return pinned;
    };
    let moved = pinned.remove(from);
    let to = pinned
        .iter()
        .position(|pin| pin == target)
        .unwrap_or(pinned.len());
    pinned.insert(to + usize::from(after), moved);
    pinned
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
        pub dock_pinned: RefCell<Vec<String>>,
    }

    impl Default for Config {
        fn default() -> Self {
            Self {
                bluetooth_expected_devices: RefCell::new(Vec::new()),
                dock_pinned: RefCell::new(Vec::new()),
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

            let config = super::Config::read();
            *self.bluetooth_expected_devices.borrow_mut() = config.bluetooth.expected_devices;
            *self.dock_pinned.borrow_mut() = config.dock.pinned;
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: OnceLock<Vec<ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![
                    glib::ParamSpecString::builder("bluetooth-expected-devices")
                        .read_only()
                        .build(),
                    glib::ParamSpecString::builder("dock-pinned")
                        .read_only()
                        .build(),
                ]
            })
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "bluetooth-expected-devices" => {
                    format!("{:?}", self.bluetooth_expected_devices.borrow()).to_value()
                }
                "dock-pinned" => format!("{:?}", self.dock_pinned.borrow()).to_value(),
                _ => unimplemented!(),
            }
        }
    }
}
