#![allow(deprecated)]

use glib::prelude::ObjectExt;
use glib::subclass::prelude::*;
use glib::Object;
use gtk4::glib;
use std::cell::{Cell, RefCell};
use std::sync::OnceLock;
use zbus::dbus_proxy;

glib::wrapper! {
    pub struct PowerProfiles(ObjectSubclass<imp::PowerProfiles>);
}

thread_local! {
    static POWER_PROFILES_INSTANCE: RefCell<Option<PowerProfiles>> = const { RefCell::new(None) };
}

impl PowerProfiles {
    pub fn instance() -> PowerProfiles {
        POWER_PROFILES_INSTANCE.with(|instance| {
            let mut instance_mut = instance.borrow_mut();
            if instance_mut.is_none() {
                let obj: PowerProfiles = Object::builder().build();
                *instance_mut = Some(obj);
            }
            instance_mut.as_ref().unwrap().clone()
        })
    }

    fn new() -> Self {
        Object::builder().build()
    }

    pub fn active_profile(&self) -> String {
        self.imp().active_profile.borrow().clone()
    }

    pub fn set_active_profile(&self, profile: &str) {
        let proxy = self.imp().proxy.borrow().clone();
        let profile = profile.to_string();
        match proxy {
            Some(proxy) => {
                glib::spawn_future_local(async move {
                    if let Err(e) = proxy.set_active_profile(&profile).await {
                        log::error!("PowerProfiles: failed to set profile {profile}: {e}");
                    }
                });
            }
            None => {
                log::error!("PowerProfiles: cannot set profile {profile}: daemon unavailable");
            }
        }
    }

    pub fn available(&self) -> bool {
        self.imp().available.get()
    }

    pub fn icon_name(&self) -> String {
        let profile = self.active_profile();
        if profile.is_empty() {
            "power-profile-balanced-symbolic".to_string()
        } else {
            format!("power-profile-{}-symbolic", profile)
        }
    }
}

impl Default for PowerProfiles {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(deprecated)]
#[dbus_proxy(
    interface = "org.freedesktop.UPower.PowerProfiles",
    default_service = "org.freedesktop.UPower.PowerProfiles",
    default_path = "/org/freedesktop/UPower/PowerProfiles"
)]
trait PowerProfilesProxy {
    #[dbus_proxy(property)]
    fn active_profile(&self) -> zbus::Result<String>;

    #[dbus_proxy(property)]
    fn set_active_profile(&self, profile: &str) -> zbus::Result<()>;
}

async fn initialize_power_profiles(obj: PowerProfiles) {
    match zbus::Connection::system().await {
        Ok(connection) => match PowerProfilesProxyProxy::new(&connection).await {
            Ok(proxy) => {
                initialize_profile_and_listener(obj, proxy).await;
            }
            Err(e) => {
                log::error!("PowerProfiles: failed to create D-Bus proxy: {e}");
            }
        },
        Err(e) => {
            log::error!("PowerProfiles: failed to connect to system D-Bus: {e}");
        }
    }
}

async fn initialize_profile_and_listener(
    obj: PowerProfiles,
    proxy: PowerProfilesProxyProxy<'static>,
) {
    if let Ok(profile) = proxy.active_profile().await {
        *obj.imp().active_profile.borrow_mut() = profile.clone();
        obj.notify("active-profile");
    }

    *obj.imp().proxy.borrow_mut() = Some(proxy.clone());
    obj.imp().available.set(true);
    obj.notify("available");

    // Listen for property changes via D-Bus signals
    glib::spawn_future_local(async move {
        listen_for_profile_changes(obj, proxy).await;
    });
}

async fn listen_for_profile_changes(obj: PowerProfiles, proxy: PowerProfilesProxyProxy<'static>) {
    use futures_util::StreamExt;
    use glib::prelude::*;
    let mut stream = proxy.receive_active_profile_changed().await;
    while let Some(change) = stream.next().await {
        if let Ok(profile) = change.get().await {
            let current = obj.imp().active_profile.borrow().clone();
            if current != profile {
                *obj.imp().active_profile.borrow_mut() = profile;
                obj.notify("active-profile");
            }
        }
    }
}

mod imp {
    use super::*;
    use glib::prelude::*;
    use glib::subclass::prelude::*;
    use glib::{ParamSpec, ParamSpecBuilderExt, Value};

    #[derive(Default)]
    pub struct PowerProfiles {
        pub active_profile: RefCell<String>,
        pub available: Cell<bool>,
        pub proxy: RefCell<Option<super::PowerProfilesProxyProxy<'static>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PowerProfiles {
        const NAME: &'static str = "PowerProfiles";
        type Type = super::PowerProfiles;
    }

    impl ObjectImpl for PowerProfiles {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj().clone();
            glib::spawn_future_local(async move {
                initialize_power_profiles(obj).await;
            });
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: OnceLock<Vec<ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![
                    glib::ParamSpecString::builder("active-profile")
                        .read_only()
                        .build(),
                    glib::ParamSpecBoolean::builder("available")
                        .read_only()
                        .build(),
                ]
            })
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "active-profile" => self.active_profile.borrow().to_value(),
                "available" => self.available.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}
