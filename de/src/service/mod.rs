pub mod audio;
pub mod battery;
pub mod bluetooth;
pub mod brightness;
pub mod charge_threshold;
pub mod desktop_entry;
pub mod ethernet;
pub mod fcitx;
pub mod hyprland;
pub mod network;
pub mod notifications;
pub mod power_profiles;
pub mod system_info;
pub mod vpn;
pub mod wifi;

/// Safe wrapper for idle_add that checks display availability
pub fn idle_add_safe<F>(func: F)
where
    F: FnOnce() + Send + 'static,
{
    glib::idle_add_once(move || {
        if gdk4::Display::default().is_some() {
            func();
        }
    });
}
