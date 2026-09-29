use glib::subclass::prelude::*;
use glib::Object;
use gtk4::glib;
use std::cell::RefCell;
use std::sync::OnceLock;
use zbus::Connection;

glib::wrapper! {
    pub struct Fcitx(ObjectSubclass<imp::Fcitx>);
}

impl Fcitx {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn instance() -> Self {
        use std::cell::OnceCell;
        thread_local! {
            static INSTANCE: OnceCell<Fcitx> = const { OnceCell::new() };
        }

        INSTANCE.with(|cell| cell.get_or_init(Self::new).clone())
    }

    pub fn current_im(&self) -> String {
        self.imp().current_im.borrow().clone()
    }

    pub fn available_ims(&self) -> Vec<InputMethod> {
        self.imp().available_ims.borrow().clone()
    }

    pub fn set_current_im(&self, im_name: &str) {
        self.imp().set_input_method(im_name);
    }

    pub fn refresh_languages(&self) {
        self.imp().refresh_languages();
    }
}

impl Default for Fcitx {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct InputMethod {
    pub name: String,
    pub unique_name: String,
    pub language_code: String,
    pub enabled: bool,
}

mod imp {
    use super::*;
    use glib::prelude::*;
    use glib::subclass::prelude::*;
    use glib::{ParamSpec, ParamSpecBuilderExt, Value};
    use zbus::proxy;

    // Type alias to simplify complex return type
    type AvailableInputMethodsResult = Vec<(String, String, String, String, String, String, bool)>;

    pub struct Fcitx {
        pub current_im: RefCell<String>,
        pub available_ims: RefCell<Vec<InputMethod>>,
    }

    impl Default for Fcitx {
        fn default() -> Self {
            Self {
                current_im: RefCell::new(String::new()),
                available_ims: RefCell::new(Vec::new()),
            }
        }
    }

    #[proxy(
        interface = "org.fcitx.Fcitx.Controller1",
        default_service = "org.fcitx.Fcitx5",
        default_path = "/controller"
    )]
    trait FcitxController {
        fn current_input_method(&self) -> zbus::Result<String>;
        fn current_input_method_group(&self) -> zbus::Result<String>;
        #[zbus(name = "SetCurrentIM")]
        fn set_current_im(&self, im_name: &str) -> zbus::Result<()>;
        fn input_method_group_info(
            &self,
            group_name: &str,
        ) -> zbus::Result<(String, Vec<(String, String)>)>;
        fn available_input_methods(&self) -> zbus::Result<AvailableInputMethodsResult>;

        #[zbus(signal)]
        fn input_method_groups_changed(&self) -> zbus::Result<()>;
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Fcitx {
        const NAME: &'static str = "Fcitx";
        type Type = super::Fcitx;
    }

    impl ObjectImpl for Fcitx {
        fn constructed(&self) {
            self.parent_constructed();

            // Initialize current state
            self.update_state();

            // Setup event listener
            self.setup_event_listener();
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: OnceLock<Vec<ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![
                    glib::ParamSpecString::builder("current-im")
                        .read_only()
                        .build(),
                    glib::ParamSpecUInt::builder("available-ims-count")
                        .read_only()
                        .build(),
                ]
            })
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "current-im" => self.current_im.borrow().to_value(),
                "available-ims-count" => (self.available_ims.borrow().len() as u32).to_value(),
                _ => unimplemented!(),
            }
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: OnceLock<Vec<glib::subclass::Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![glib::subclass::Signal::builder("input-method-changed")
                    .param_types([String::static_type()])
                    .build()]
            })
        }
    }

    impl Fcitx {
        fn setup_event_listener(&self) {
            let obj = self.obj().clone();

            // Listen to InputMethodGroupsChanged signal asynchronously
            glib::MainContext::default().spawn_local(async move {
                let connection = match Connection::session().await {
                    Ok(conn) => conn,
                    Err(e) => {
                        let err_msg = format!("Failed to connect to D-Bus: {}", e);
                        crate::service::idle_add_safe(move || {
                            crate::service::notifications::Notifications::instance()
                                .send_notification("rusty-de", "Fcitx Error", &err_msg);
                        });
                        return;
                    }
                };

                let proxy = match FcitxControllerProxy::new(&connection).await {
                    Ok(p) => p,
                    Err(e) => {
                        let err_msg = format!("Failed to create proxy: {}", e);
                        crate::service::idle_add_safe(move || {
                            crate::service::notifications::Notifications::instance()
                                .send_notification("rusty-de", "Fcitx Error", &err_msg);
                        });
                        return;
                    }
                };

                // Listen to InputMethodGroupsChanged signal
                let mut stream = proxy.receive_input_method_groups_changed().await.unwrap();

                loop {
                    use futures_util::StreamExt;

                    if stream.next().await.is_some() {
                        // Check if current IM changed
                        if let Ok(new_im) = proxy.current_input_method().await {
                            let old_im = obj.imp().current_im.borrow().clone();
                            if new_im != old_im {
                                obj.imp().current_im.replace(new_im.clone());
                                obj.notify("current-im");
                                obj.emit_by_name::<()>("input-method-changed", &[&new_im]);
                            }
                        }
                    } else {
                        break;
                    }
                }
            });
        }

        fn update_state(&self) {
            let runtime =
                match tokio::runtime::Runtime::new() {
                    Ok(rt) => rt,
                    Err(e) => {
                        let err_msg = format!("Failed to create async runtime: {}", e);
                        crate::service::idle_add_safe(move || {
                            crate::service::notifications::Notifications::instance()
                                .send_notification("rusty-de", "Fcitx Error", &err_msg);
                        });
                        return;
                    }
                };

            runtime.block_on(async {
                let connection = match Connection::session().await {
                    Ok(conn) => conn,
                    Err(e) => {
                        let err_msg = format!("Failed to connect to D-Bus: {}", e);
                        crate::service::idle_add_safe(move || {
                            crate::service::notifications::Notifications::instance()
                                .send_notification("rusty-de", "Fcitx Error", &err_msg);
                        });
                        return;
                    }
                };

                let proxy = match FcitxControllerProxy::new(&connection).await {
                    Ok(p) => p,
                    Err(e) => {
                        let err_msg = format!("Failed to create proxy: {}", e);
                        crate::service::idle_add_safe(move || {
                            crate::service::notifications::Notifications::instance()
                                .send_notification("rusty-de", "Fcitx Error", &err_msg);
                        });
                        return;
                    }
                };

                // Get current input method
                if let Ok(current) = proxy.current_input_method().await {
                    self.current_im.replace(current.clone());
                }

                // Get current input method group
                let group_name = match proxy.current_input_method_group().await {
                    Ok(g) => g,
                    Err(_) => "Default".to_string(),
                };

                self.populate_input_methods(&proxy, &group_name).await;
            });
        }

        async fn populate_input_methods(&self, proxy: &FcitxControllerProxy<'_>, group_name: &str) {
            let (_, ims) =
                match proxy.input_method_group_info(group_name).await {
                    Ok(info) => info,
                    Err(e) => {
                        let err_msg = format!("Failed to get input method group info: {}", e);
                        crate::service::idle_add_safe(move || {
                            crate::service::notifications::Notifications::instance()
                                .send_notification("rusty-de", "Fcitx Error", &err_msg);
                        });
                        return;
                    }
                };

            let all_ims =
                match proxy.available_input_methods().await {
                    Ok(ims) => ims,
                    Err(e) => {
                        let err_msg = format!("Failed to get available input methods: {}", e);
                        crate::service::idle_add_safe(move || {
                            crate::service::notifications::Notifications::instance()
                                .send_notification("rusty-de", "Fcitx Error", &err_msg);
                        });
                        return;
                    }
                };

            let methods: Vec<InputMethod> = ims
                .into_iter()
                .filter_map(|(unique_name, _display_name)| {
                    all_ims
                        .iter()
                        .find(|(id, _, _, _, _, _, _)| id == &unique_name)
                        .map(|(_, name, _, _, _, lang_code, _)| InputMethod {
                            name: name.clone(),
                            unique_name: unique_name.clone(),
                            language_code: lang_code.clone(),
                            enabled: true,
                        })
                })
                .collect();

            self.available_ims.replace(methods);
        }

        pub fn refresh_languages(&self) {
            let obj = self.obj().clone();

            glib::MainContext::default().spawn_local(async move {
                let connection = match Connection::session().await {
                    Ok(conn) => conn,
                    Err(_) => return,
                };

                let proxy = match FcitxControllerProxy::new(&connection).await {
                    Ok(p) => p,
                    Err(_) => return,
                };

                let old_count = obj.imp().available_ims.borrow().len();

                // Get current input method group and refresh available IMs
                if let Ok(group_name) = proxy.current_input_method_group().await {
                    obj.imp().populate_input_methods(&proxy, &group_name).await;

                    let new_count = obj.imp().available_ims.borrow().len();

                    // Always notify to update UI (even if count unchanged, language names may have changed)
                    if new_count != old_count || new_count > 0 {
                        obj.notify("available-ims-count");
                    }
                }

                // Also refresh current IM
                if let Ok(new_im) = proxy.current_input_method().await {
                    let old_im = obj.imp().current_im.borrow().clone();
                    if new_im != old_im {
                        obj.imp().current_im.replace(new_im.clone());
                        obj.notify("current-im");
                    }
                }
            });
        }

        pub fn set_input_method(&self, im_name: &str) {
            let im_name = im_name.to_string();
            let obj = self.obj().clone();

            glib::MainContext::default().spawn_local(async move {
                let connection = match Connection::session().await {
                    Ok(conn) => conn,
                    Err(e) => {
                        let err_msg = format!("Failed to connect to D-Bus: {}", e);
                        crate::service::idle_add_safe(move || {
                            crate::service::notifications::Notifications::instance()
                                .send_notification("rusty-de", "Fcitx Error", &err_msg);
                        });
                        return;
                    }
                };

                let proxy = match FcitxControllerProxy::new(&connection).await {
                    Ok(p) => p,
                    Err(e) => {
                        let err_msg = format!("Failed to create proxy: {}", e);
                        crate::service::idle_add_safe(move || {
                            crate::service::notifications::Notifications::instance()
                                .send_notification("rusty-de", "Fcitx Error", &err_msg);
                        });
                        return;
                    }
                };

                match proxy.set_current_im(&im_name).await {
                    Ok(_) => {
                        // Update our internal state and emit signal
                        obj.imp().current_im.replace(im_name.clone());
                        obj.notify("current-im");
                        obj.emit_by_name::<()>("input-method-changed", &[&im_name]);
                    }
                    Err(e) => {
                        let err_msg = format!("Failed to set input method: {}", e);
                        crate::service::idle_add_safe(move || {
                            crate::service::notifications::Notifications::instance()
                                .send_notification("rusty-de", "Fcitx Error", &err_msg);
                        });
                    }
                }
            });
        }
    }
}
