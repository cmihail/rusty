use glib::prelude::ObjectExt;
use glib::subclass::prelude::*;
use glib::Object;
use gtk4::glib;
use serde::Deserialize;
use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct ConfigData {
    pub top_left_corner_command: Option<String>,
    pub top_left_corner_on_all_monitors: Option<bool>,
    pub top_right_corner_command: Option<String>,
    pub top_right_corner_on_all_monitors: Option<bool>,
    pub bottom_left_corner_command: Option<String>,
    pub bottom_left_corner_on_all_monitors: Option<bool>,
    pub bottom_right_corner_command: Option<String>,
    pub bottom_right_corner_on_all_monitors: Option<bool>,
    pub trigger_delay_ms: Option<u32>,
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

    pub fn top_left_corner_command(&self) -> Option<String> {
        self.imp().top_left_corner_command.borrow().clone()
    }

    pub fn top_left_corner_on_all_monitors(&self) -> bool {
        self.imp()
            .top_left_corner_on_all_monitors
            .borrow()
            .unwrap_or(false)
    }

    pub fn top_right_corner_command(&self) -> Option<String> {
        self.imp().top_right_corner_command.borrow().clone()
    }

    pub fn top_right_corner_on_all_monitors(&self) -> bool {
        self.imp()
            .top_right_corner_on_all_monitors
            .borrow()
            .unwrap_or(false)
    }

    pub fn bottom_left_corner_command(&self) -> Option<String> {
        self.imp().bottom_left_corner_command.borrow().clone()
    }

    pub fn bottom_left_corner_on_all_monitors(&self) -> bool {
        self.imp()
            .bottom_left_corner_on_all_monitors
            .borrow()
            .unwrap_or(false)
    }

    pub fn bottom_right_corner_command(&self) -> Option<String> {
        self.imp().bottom_right_corner_command.borrow().clone()
    }

    pub fn bottom_right_corner_on_all_monitors(&self) -> bool {
        self.imp()
            .bottom_right_corner_on_all_monitors
            .borrow()
            .unwrap_or(false)
    }

    pub fn trigger_delay_ms(&self) -> u32 {
        self.imp().trigger_delay_ms.borrow().unwrap_or(1000)
    }

    pub fn reload(&self) {
        let config_path = Self::config_path();

        let (
            new_top_left_cmd_val,
            new_top_left_on_all,
            new_top_right_cmd,
            new_top_right_on_all,
            new_bottom_left_cmd,
            new_bottom_left_on_all,
            new_bottom_right_cmd,
            new_bottom_right_on_all,
            new_trigger_delay,
        ) = if let Ok(contents) = fs::read_to_string(&config_path) {
            if let Ok(config) = toml::from_str::<ConfigData>(&contents) {
                (
                    config.top_left_corner_command,
                    config.top_left_corner_on_all_monitors,
                    config.top_right_corner_command,
                    config.top_right_corner_on_all_monitors,
                    config.bottom_left_corner_command,
                    config.bottom_left_corner_on_all_monitors,
                    config.bottom_right_corner_command,
                    config.bottom_right_corner_on_all_monitors,
                    config.trigger_delay_ms,
                )
            } else {
                (None, None, None, None, None, None, None, None, None)
            }
        } else {
            (None, None, None, None, None, None, None, None, None)
        };

        let old_top_left_cmd = self.imp().top_left_corner_command.borrow().clone();
        if old_top_left_cmd != new_top_left_cmd_val {
            *self.imp().top_left_corner_command.borrow_mut() = new_top_left_cmd_val;
            self.notify("top-left-corner-command");
        }

        let old_top_left_on_all = self.top_left_corner_on_all_monitors();
        let new_top_left_on_all_value = new_top_left_on_all.unwrap_or(false);
        if old_top_left_on_all != new_top_left_on_all_value {
            *self.imp().top_left_corner_on_all_monitors.borrow_mut() = new_top_left_on_all;
            self.notify("top-left-corner-on-all-monitors");
        }

        let old_top_right_cmd = self.top_right_corner_command();
        if old_top_right_cmd != new_top_right_cmd {
            *self.imp().top_right_corner_command.borrow_mut() = new_top_right_cmd;
            self.notify("top-right-corner-command");
        }

        let old_top_right_on_all = self.top_right_corner_on_all_monitors();
        let new_top_right_on_all_value = new_top_right_on_all.unwrap_or(false);
        if old_top_right_on_all != new_top_right_on_all_value {
            *self.imp().top_right_corner_on_all_monitors.borrow_mut() = new_top_right_on_all;
            self.notify("top-right-corner-on-all-monitors");
        }

        let old_bottom_left_cmd = self.bottom_left_corner_command();
        if old_bottom_left_cmd != new_bottom_left_cmd {
            *self.imp().bottom_left_corner_command.borrow_mut() = new_bottom_left_cmd;
            self.notify("bottom-left-corner-command");
        }

        let old_bottom_left_on_all = self.bottom_left_corner_on_all_monitors();
        let new_bottom_left_on_all_value = new_bottom_left_on_all.unwrap_or(false);
        if old_bottom_left_on_all != new_bottom_left_on_all_value {
            *self.imp().bottom_left_corner_on_all_monitors.borrow_mut() = new_bottom_left_on_all;
            self.notify("bottom-left-corner-on-all-monitors");
        }

        let old_bottom_right_cmd = self.bottom_right_corner_command();
        if old_bottom_right_cmd != new_bottom_right_cmd {
            *self.imp().bottom_right_corner_command.borrow_mut() = new_bottom_right_cmd;
            self.notify("bottom-right-corner-command");
        }

        let old_bottom_right_on_all = self.bottom_right_corner_on_all_monitors();
        let new_bottom_right_on_all_value = new_bottom_right_on_all.unwrap_or(false);
        if old_bottom_right_on_all != new_bottom_right_on_all_value {
            *self.imp().bottom_right_corner_on_all_monitors.borrow_mut() =
                new_bottom_right_on_all;
            self.notify("bottom-right-corner-on-all-monitors");
        }

        let old_trigger_delay = self.trigger_delay_ms();
        let new_trigger_delay_value = new_trigger_delay.unwrap_or(1000);
        if old_trigger_delay != new_trigger_delay_value {
            *self.imp().trigger_delay_ms.borrow_mut() = new_trigger_delay;
            self.notify("trigger-delay-ms");
        }
    }

    pub fn config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".config/rusty/hot-corner.toml")
    }
}

mod imp {
    use super::*;
    use glib::prelude::ToValue;
    use glib::subclass::prelude::*;
    use glib::{ParamSpec, ParamSpecBuilderExt, Value};
    use std::cell::RefCell;
    use std::sync::OnceLock;

    pub struct Config {
        pub top_left_corner_command: RefCell<Option<String>>,
        pub top_left_corner_on_all_monitors: RefCell<Option<bool>>,
        pub top_right_corner_command: RefCell<Option<String>>,
        pub top_right_corner_on_all_monitors: RefCell<Option<bool>>,
        pub bottom_left_corner_command: RefCell<Option<String>>,
        pub bottom_left_corner_on_all_monitors: RefCell<Option<bool>>,
        pub bottom_right_corner_command: RefCell<Option<String>>,
        pub bottom_right_corner_on_all_monitors: RefCell<Option<bool>>,
        pub trigger_delay_ms: RefCell<Option<u32>>,
    }

    impl Default for Config {
        fn default() -> Self {
            Self {
                top_left_corner_command: RefCell::new(None),
                top_left_corner_on_all_monitors: RefCell::new(None),
                top_right_corner_command: RefCell::new(None),
                top_right_corner_on_all_monitors: RefCell::new(None),
                bottom_left_corner_command: RefCell::new(None),
                bottom_left_corner_on_all_monitors: RefCell::new(None),
                bottom_right_corner_command: RefCell::new(None),
                bottom_right_corner_on_all_monitors: RefCell::new(None),
                trigger_delay_ms: RefCell::new(None),
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

            // Load initial config
            let config_path = super::Config::config_path();
            if let Ok(contents) = std::fs::read_to_string(&config_path) {
                if let Ok(config) = toml::from_str::<ConfigData>(&contents) {
                    *self.top_left_corner_command.borrow_mut() = config.top_left_corner_command;
                    *self.top_left_corner_on_all_monitors.borrow_mut() =
                        config.top_left_corner_on_all_monitors;
                    *self.top_right_corner_command.borrow_mut() = config.top_right_corner_command;
                    *self.top_right_corner_on_all_monitors.borrow_mut() =
                        config.top_right_corner_on_all_monitors;
                    *self.bottom_left_corner_command.borrow_mut() =
                        config.bottom_left_corner_command;
                    *self.bottom_left_corner_on_all_monitors.borrow_mut() =
                        config.bottom_left_corner_on_all_monitors;
                    *self.bottom_right_corner_command.borrow_mut() =
                        config.bottom_right_corner_command;
                    *self.bottom_right_corner_on_all_monitors.borrow_mut() =
                        config.bottom_right_corner_on_all_monitors;
                    *self.trigger_delay_ms.borrow_mut() = config.trigger_delay_ms;
                }
            }
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: OnceLock<Vec<ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![
                    glib::ParamSpecString::builder("top-left-corner-command")
                        .read_only()
                        .build(),
                    glib::ParamSpecBoolean::builder("top-left-corner-on-all-monitors")
                        .read_only()
                        .build(),
                    glib::ParamSpecString::builder("top-right-corner-command")
                        .read_only()
                        .build(),
                    glib::ParamSpecBoolean::builder("top-right-corner-on-all-monitors")
                        .read_only()
                        .build(),
                    glib::ParamSpecString::builder("bottom-left-corner-command")
                        .read_only()
                        .build(),
                    glib::ParamSpecBoolean::builder("bottom-left-corner-on-all-monitors")
                        .read_only()
                        .build(),
                    glib::ParamSpecString::builder("bottom-right-corner-command")
                        .read_only()
                        .build(),
                    glib::ParamSpecBoolean::builder("bottom-right-corner-on-all-monitors")
                        .read_only()
                        .build(),
                    glib::ParamSpecUInt::builder("trigger-delay-ms")
                        .read_only()
                        .build(),
                ]
            })
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "top-left-corner-command" => self.top_left_corner_command.borrow().to_value(),
                "top-left-corner-on-all-monitors" => self
                    .top_left_corner_on_all_monitors
                    .borrow()
                    .unwrap_or(false)
                    .to_value(),
                "top-right-corner-command" => self.top_right_corner_command.borrow().to_value(),
                "top-right-corner-on-all-monitors" => self
                    .top_right_corner_on_all_monitors
                    .borrow()
                    .unwrap_or(false)
                    .to_value(),
                "bottom-left-corner-command" => self.bottom_left_corner_command.borrow().to_value(),
                "bottom-left-corner-on-all-monitors" => self
                    .bottom_left_corner_on_all_monitors
                    .borrow()
                    .unwrap_or(false)
                    .to_value(),
                "bottom-right-corner-command" => {
                    self.bottom_right_corner_command.borrow().to_value()
                }
                "bottom-right-corner-on-all-monitors" => self
                    .bottom_right_corner_on_all_monitors
                    .borrow()
                    .unwrap_or(false)
                    .to_value(),
                "trigger-delay-ms" => self.trigger_delay_ms.borrow().unwrap_or(1000).to_value(),
                _ => unimplemented!(),
            }
        }
    }
}
