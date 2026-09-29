use glib::subclass::prelude::*;
use glib::Object;
use gtk4::glib;
use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::OnceLock;

pub const START_THRESHOLD_PATH: &str =
    "/sys/class/power_supply/BAT0/charge_control_start_threshold";
pub const END_THRESHOLD_PATH: &str = "/sys/class/power_supply/BAT0/charge_control_end_threshold";

pub trait ChargeThresholdBackend: Send + Sync {
    fn read_threshold(&self, path: &str) -> Option<i32>;
    fn set_threshold(&self, start: i32, end: i32) -> io::Result<()>;
}

pub struct SysfsChargeThresholdBackend;

impl ChargeThresholdBackend for SysfsChargeThresholdBackend {
    fn read_threshold(&self, path: &str) -> Option<i32> {
        fs::read_to_string(path)
            .ok()
            .and_then(|content| content.trim().parse::<i32>().ok())
    }

    fn set_threshold(&self, start: i32, end: i32) -> io::Result<()> {
        // thinkpad_acpi rejects a start threshold above the current end threshold, so the
        // attribute being raised has to be written first.
        let raising = self
            .read_threshold(END_THRESHOLD_PATH)
            .is_none_or(|current_end| end >= current_end);

        if raising {
            fs::write(END_THRESHOLD_PATH, end.to_string())?;
            fs::write(START_THRESHOLD_PATH, start.to_string())
        } else {
            fs::write(START_THRESHOLD_PATH, start.to_string())?;
            fs::write(END_THRESHOLD_PATH, end.to_string())
        }
    }
}

glib::wrapper! {
    pub struct ChargeThreshold(ObjectSubclass<imp::ChargeThreshold>);
}

thread_local! {
    static CHARGE_THRESHOLD_INSTANCE: RefCell<Option<ChargeThreshold>> =
        const { RefCell::new(None) };
}

#[allow(dead_code)]
pub fn charge_threshold_with_backend(backend: Box<dyn ChargeThresholdBackend>) -> ChargeThreshold {
    use glib::subclass::prelude::ObjectSubclassIsExt;

    let obj: ChargeThreshold = glib::Object::new();
    *obj.imp().backend.borrow_mut() = Some(backend);

    // Manually initialize after setting the backend
    obj.imp().initialize_threshold();

    obj
}

impl ChargeThreshold {
    pub fn instance() -> ChargeThreshold {
        CHARGE_THRESHOLD_INSTANCE.with(|instance| {
            let mut instance_mut = instance.borrow_mut();
            if instance_mut.is_none() {
                let obj: ChargeThreshold = Object::builder().build();
                *instance_mut = Some(obj);
            }
            instance_mut.as_ref().unwrap().clone()
        })
    }

    fn new() -> Self {
        Object::builder().build()
    }

    pub fn threshold(&self) -> i32 {
        self.imp().threshold.get()
    }

    pub fn set_threshold(&self, value: i32) {
        if value < 10 {
            return;
        }

        if self.threshold() == value {
            return;
        }

        // Only mirror the new value once the EC has accepted it, so a failed write leaves the
        // property showing what sysfs actually holds.
        if let Some(backend) = &*self.imp().backend.borrow() {
            let start = value - 5;
            if let Err(e) = backend.set_threshold(start, value) {
                log::error!("ChargeThreshold: failed to set threshold {start}-{value}: {e}");
                return;
            }
        }

        self.imp().threshold.set(value);
        self.notify("threshold");
    }
}

impl Default for ChargeThreshold {
    fn default() -> Self {
        Self::new()
    }
}

mod imp {
    use super::*;
    use gio::FileMonitor;
    use glib::subclass::prelude::*;
    use glib::{ParamSpec, ParamSpecBuilderExt, Value};
    use gtk4::gio;
    use std::cell::RefCell;

    pub struct ChargeThreshold {
        pub threshold: Cell<i32>,
        _monitor: OnceLock<FileMonitor>,
        pub backend: RefCell<Option<Box<dyn super::ChargeThresholdBackend>>>,
    }

    impl Default for ChargeThreshold {
        fn default() -> Self {
            Self {
                threshold: Cell::new(0),
                _monitor: OnceLock::new(),
                backend: RefCell::new(Some(Box::new(super::SysfsChargeThresholdBackend))),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ChargeThreshold {
        const NAME: &'static str = "ChargeThreshold";
        type Type = super::ChargeThreshold;
    }

    impl ObjectImpl for ChargeThreshold {
        fn constructed(&self) {
            self.parent_constructed();

            // Only initialize if the file exists (i.e., we're not in a test)
            if PathBuf::from(END_THRESHOLD_PATH).exists() {
                self.initialize_threshold();
                self.setup_monitor();
            }
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: OnceLock<Vec<ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![glib::ParamSpecInt::builder("threshold")
                    .minimum(0)
                    .maximum(100)
                    .read_only()
                    .build()]
            })
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "threshold" => self.threshold.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl ChargeThreshold {
        pub(crate) fn initialize_threshold(&self) {
            let backend = self.backend.borrow();
            if let Some(backend) = &*backend {
                if let Some(value) = backend.read_threshold(END_THRESHOLD_PATH) {
                    self.threshold.set(value);
                }
            }
        }

        fn setup_monitor(&self) {
            let path = PathBuf::from(END_THRESHOLD_PATH);
            let file = gio::File::for_path(&path)
                .monitor_file(gio::FileMonitorFlags::NONE, gio::Cancellable::NONE);

            if let Ok(file) = file {
                let obj = self.obj().clone();
                file.connect_changed(move |_, _, _, _event| {
                    let value = obj
                        .imp()
                        .backend
                        .borrow()
                        .as_ref()
                        .and_then(|backend| backend.read_threshold(END_THRESHOLD_PATH));

                    if let Some(value) = value {
                        if value != obj.imp().threshold.get() {
                            obj.imp().threshold.set(value);
                            obj.notify("threshold");
                        }
                    }
                });

                self._monitor.set(file).ok();
            }
        }
    }
}
