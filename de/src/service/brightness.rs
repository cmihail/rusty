use glib::subclass::prelude::*;
use glib::Object;
use gtk4::glib;
use std::cell::{Cell, RefCell};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub trait BrightnessBackend: Send + Sync {
    fn get_first_device(&self, path: &str) -> String;
    fn get_brightness_value(&self, command: &str, device: &str, is_screen: bool) -> i32;
    fn set_brightness(&self, percent: f64);
    fn set_kbd_brightness(&self, device: &str, level: i32);
}

pub struct SystemBrightnessBackend;

impl BrightnessBackend for SystemBrightnessBackend {
    fn get_first_device(&self, path: &str) -> String {
        let mut devices: Vec<String> = std::fs::read_dir(path)
            .map(|entries| {
                entries
                    .filter_map(|entry| entry.ok())
                    .map(|entry| entry.file_name().to_string_lossy().into_owned())
                    .collect()
            })
            .unwrap_or_default();
        devices.sort();

        // For keyboard backlight, look for devices containing "kbd" or "keyboard"
        if path.contains("/leds") {
            if let Some(device) = devices
                .iter()
                .find(|d| d.contains("kbd") || d.contains("keyboard"))
            {
                return device.clone();
            }
        }

        // Fallback to first device
        devices.into_iter().next().unwrap_or_default()
    }

    fn get_brightness_value(&self, command: &str, device: &str, is_screen: bool) -> i32 {
        let args = if is_screen {
            vec![command]
        } else {
            vec!["--device", device, command]
        };

        let output = Command::new("brightnessctl").args(&args).output();

        match output {
            Ok(output) => String::from_utf8_lossy(&output.stdout)
                .trim()
                .parse::<i32>()
                .unwrap_or(0),
            Err(_) => 0,
        }
    }

    fn set_brightness(&self, percent: f64) {
        let _ = Command::new("brightnessctl")
            .args(["set", &format!("{}%", (percent * 100.0).floor())])
            .arg("-q")
            .output();
    }

    fn set_kbd_brightness(&self, device: &str, level: i32) {
        let _ = Command::new("brightnessctl")
            .args(["--device", device, "set", &level.to_string()])
            .arg("-q")
            .output();
    }
}

glib::wrapper! {
    pub struct Brightness(ObjectSubclass<imp::Brightness>);
}

thread_local! {
    static BRIGHTNESS_INSTANCE: RefCell<Option<Brightness>> = const { RefCell::new(None) };
}

#[allow(dead_code)]
pub fn brightness_with_backend(backend: Box<dyn BrightnessBackend>) -> Brightness {
    use glib::subclass::prelude::ObjectSubclassIsExt;

    let obj: Brightness = glib::Object::new();
    *obj.imp().backend.borrow_mut() = Some(backend);

    // Manually initialize after setting the backend
    let (screen_device, kbd_device) = {
        let backend = obj.imp().backend.borrow();
        if let Some(backend) = &*backend {
            let screen_device = backend.get_first_device("/sys/class/backlight");
            let kbd_device = backend.get_first_device("/sys/class/leds");
            (screen_device, kbd_device)
        } else {
            (String::new(), String::new())
        }
    };

    if !screen_device.is_empty() || !kbd_device.is_empty() {
        obj.imp().initialize_values(&screen_device, &kbd_device);
    }

    obj
}

impl Brightness {
    pub fn instance() -> Brightness {
        BRIGHTNESS_INSTANCE.with(|instance| {
            let mut instance_mut = instance.borrow_mut();
            if instance_mut.is_none() {
                let obj: Brightness = Object::builder().build();
                *instance_mut = Some(obj);
            }
            instance_mut.as_ref().unwrap().clone()
        })
    }

    fn new() -> Self {
        Object::builder().build()
    }

    pub fn screen(&self) -> f64 {
        self.imp().screen.get()
    }

    pub fn set_screen(&self, percent: f64) {
        let percent = percent.clamp(0.0, 1.0);

        if (self.screen() - percent).abs() < f64::EPSILON {
            return;
        }

        if let Some(backend) = &*self.imp().backend.borrow() {
            backend.set_brightness(percent);
        }
    }

    pub fn kbd(&self) -> i32 {
        self.imp().kbd.get()
    }

    pub fn kbd_max(&self) -> i32 {
        self.imp().kbd_max.get()
    }

    pub fn set_kbd(&self, level: i32) {
        let kbd_max = self.kbd_max();
        if kbd_max == 0 {
            return;
        }

        let level = level.clamp(0, kbd_max);

        if self.kbd() == level {
            return;
        }

        let kbd_device = self.imp().kbd_device.borrow();
        if kbd_device.is_empty() {
            return;
        }

        if let Some(backend) = &*self.imp().backend.borrow() {
            backend.set_kbd_brightness(&kbd_device, level);
        }
    }

    pub fn icon_name(&self) -> String {
        let level = self.screen();
        if level < 0.3 {
            "display-brightness-off-symbolic".to_string()
        } else if level < 0.5 {
            "display-brightness-low-symbolic".to_string()
        } else if level < 0.7 {
            "display-brightness-medium-symbolic".to_string()
        } else if level < 0.9 {
            "display-brightness-high-symbolic".to_string()
        } else {
            "display-brightness-full-symbolic".to_string()
        }
    }

    pub fn toggle(&self) {
        let current = self.screen();

        if current > 0.0 && current < 1.0 {
            self.imp().previous_screen.set(current);
            self.set_screen(1.0);
        } else if (current - 1.0).abs() < f64::EPSILON {
            self.set_screen(0.0);
        } else {
            self.set_screen(self.imp().previous_screen.get());
        }
    }
}

impl Default for Brightness {
    fn default() -> Self {
        Self::new()
    }
}

mod imp {
    use super::*;
    use gio::prelude::*;
    use gio::FileMonitor;
    use glib::prelude::*;
    use glib::subclass::prelude::*;
    use glib::{ParamSpec, ParamSpecBuilderExt, Value};
    use gtk4::gio;
    use std::cell::RefCell;
    use std::sync::OnceLock;

    pub struct Brightness {
        pub kbd: Cell<i32>,
        pub screen: Cell<f64>,
        pub previous_screen: Cell<f64>,
        pub kbd_max: Cell<i32>,
        screen_max: Cell<i32>,
        pub kbd_device: RefCell<String>,
        _screen_monitor: OnceLock<FileMonitor>,
        _kbd_monitor: OnceLock<FileMonitor>,
        pub backend: RefCell<Option<Box<dyn super::BrightnessBackend>>>,
    }

    impl Default for Brightness {
        fn default() -> Self {
            Self {
                kbd: Cell::new(0),
                screen: Cell::new(0.0),
                previous_screen: Cell::new(0.0),
                kbd_max: Cell::new(0),
                screen_max: Cell::new(0),
                kbd_device: RefCell::new(String::new()),
                _screen_monitor: OnceLock::new(),
                _kbd_monitor: OnceLock::new(),
                backend: RefCell::new(Some(Box::new(super::SystemBrightnessBackend))),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Brightness {
        const NAME: &'static str = "Brightness";
        type Type = super::Brightness;
    }

    impl ObjectImpl for Brightness {
        fn constructed(&self) {
            self.parent_constructed();

            let (screen_device, kbd_device) = {
                let backend = self.backend.borrow();
                if let Some(backend) = &*backend {
                    let screen_device = backend.get_first_device("/sys/class/backlight");
                    let kbd_device = backend.get_first_device("/sys/class/leds");
                    (screen_device, kbd_device)
                } else {
                    (String::new(), String::new())
                }
            };

            if !screen_device.is_empty() || !kbd_device.is_empty() {
                self.initialize_values(&screen_device, &kbd_device);
                self.setup_screen_monitor(&screen_device);
                self.setup_kbd_monitor(&kbd_device);
            }
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: OnceLock<Vec<ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![
                    glib::ParamSpecInt::builder("kbd").read_only().build(),
                    glib::ParamSpecDouble::builder("screen").read_only().build(),
                ]
            })
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "kbd" => self.kbd.get().to_value(),
                "screen" => self.screen.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl Brightness {
        pub(crate) fn initialize_values(&self, screen_device: &str, kbd_device: &str) {
            // Store kbd_device for later use
            *self.kbd_device.borrow_mut() = kbd_device.to_string();

            let backend = self.backend.borrow();
            if let Some(backend) = &*backend {
                let screen_max = backend.get_brightness_value("max", screen_device, true);
                let kbd_max = backend.get_brightness_value("max", kbd_device, false);

                self.screen_max.set(screen_max);
                self.kbd_max.set(kbd_max);

                let screen_current = backend.get_brightness_value("get", screen_device, true);
                let kbd_current = backend.get_brightness_value("get", kbd_device, false);

                let screen_percent = if screen_max > 0 {
                    screen_current as f64 / screen_max as f64
                } else {
                    0.0
                };

                self.screen.set(screen_percent);
                self.kbd.set(kbd_current);
                self.previous_screen.set(screen_percent);
            }
        }

        fn setup_screen_monitor(&self, screen_device: &str) {
            let path_str = format!("/sys/class/backlight/{}/brightness", screen_device);
            let screen_path = PathBuf::from(&path_str);
            let file = gio::File::for_path(&screen_path)
                .monitor_file(gio::FileMonitorFlags::NONE, gio::Cancellable::NONE);

            if let Ok(file) = file {
                let obj = self.obj().clone();
                let screen_max = self.screen_max.get();
                file.connect_changed(move |_, file, _, event| {
                    handle_screen_change(&obj, file, event, screen_max);
                });

                self._screen_monitor.set(file).ok();
            }
        }

        fn setup_kbd_monitor(&self, kbd_device: &str) {
            let path_str = format!("/sys/class/leds/{}/brightness", kbd_device);
            let kbd_path = PathBuf::from(&path_str);
            let file = gio::File::for_path(&kbd_path)
                .monitor_file(gio::FileMonitorFlags::NONE, gio::Cancellable::NONE);

            if let Ok(file) = file {
                let obj = self.obj().clone();
                file.connect_changed(move |_, file, _, event| {
                    handle_kbd_change(&obj, file, event);
                });

                self._kbd_monitor.set(file).ok();
            }
        }
    }

    fn handle_screen_change(
        obj: &super::Brightness,
        file: &gio::File,
        _event: gio::FileMonitorEvent,
        screen_max: i32,
    ) {
        if let Some(path) = file.path() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(value) = content.trim().parse::<i32>() {
                    let percent = if screen_max > 0 {
                        value as f64 / screen_max as f64
                    } else {
                        0.0
                    };
                    obj.imp().screen.set(percent);
                    obj.notify("screen");
                }
            }
        }
    }

    fn handle_kbd_change(obj: &super::Brightness, file: &gio::File, _event: gio::FileMonitorEvent) {
        if let Some(path) = file.path() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(value) = content.trim().parse::<i32>() {
                    obj.imp().kbd.set(value);
                    obj.notify("kbd");
                }
            }
        }
    }
}
