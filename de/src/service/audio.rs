use glib::subclass::prelude::*;
use glib::Object;
use gtk4::glib;
use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::process::Command;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeviceType {
    Speakers,
    Headphones,
    Headset,
}

enum MicDeviceType {
    Default,
    Headset,
}

#[derive(Debug, Clone)]
pub struct AudioSink {
    pub name: String,
    pub description: String,
    pub is_default: bool,
    pub is_available: bool,
}

#[derive(Debug, Clone)]
pub struct AudioSource {
    pub name: String,
    pub description: String,
    pub is_default: bool,
    pub is_available: bool,
}

glib::wrapper! {
    pub struct Audio(ObjectSubclass<imp::Audio>);
}

thread_local! {
    static AUDIO_INSTANCE: RefCell<Option<Audio>> = const { RefCell::new(None) };
}

impl Audio {
    pub fn instance() -> Audio {
        AUDIO_INSTANCE.with(|instance| {
            let mut instance_mut = instance.borrow_mut();
            if instance_mut.is_none() {
                let obj: Audio = Object::builder().build();
                *instance_mut = Some(obj);
            }
            instance_mut.as_ref().unwrap().clone()
        })
    }

    fn new() -> Self {
        Object::builder().build()
    }

    pub fn volume(&self) -> f64 {
        self.imp().volume.get()
    }

    pub fn muted(&self) -> bool {
        self.imp().muted.get()
    }

    pub fn mic_volume(&self) -> f64 {
        self.imp().mic_volume.get()
    }

    pub fn mic_muted(&self) -> bool {
        self.imp().mic_muted.get()
    }

    pub fn sink_name(&self) -> String {
        self.imp().sink_name.borrow().clone()
    }

    pub fn sinks(&self) -> Vec<AudioSink> {
        self.imp().sinks.borrow().clone()
    }

    pub fn sources(&self) -> Vec<AudioSource> {
        self.imp().sources.borrow().clone()
    }

    pub fn set_default_sink(&self, sink_name: &str) {
        let sink_name = sink_name.to_string();
        let _ = Command::new("pactl")
            .args(["set-default-sink", &sink_name])
            .output();

        // Force UI update even if the same sink is selected
        // This ensures the switch stays active when clicking on the only available device
        let imp = self.imp();
        imp.update_sinks_list();
        self.notify("sinks");
    }

    pub fn set_default_source(&self, source_name: &str) {
        let source_name = source_name.to_string();
        let _ = Command::new("pactl")
            .args(["set-default-source", &source_name])
            .output();

        // Force UI update even if the same source is selected
        // This ensures the switch stays active when clicking on the only available device
        let imp = self.imp();
        imp.update_sources_list();
        self.notify("sources");
    }

    pub fn set_volume(&self, volume: f64) {
        let volume = volume.clamp(0.0, 1.5);
        let percent = (volume * 100.0) as u32;
        let _ = Command::new("pactl")
            .args([
                "set-sink-volume",
                "@DEFAULT_SINK@",
                &format!("{}%", percent),
            ])
            .output();
    }

    pub fn set_muted(&self, muted: bool) {
        let value = if muted { "1" } else { "0" };
        let _ = Command::new("pactl")
            .args(["set-sink-mute", "@DEFAULT_SINK@", value])
            .output();
    }

    pub fn set_mic_volume(&self, volume: f64) {
        let volume = volume.clamp(0.0, 1.0);
        let percent = (volume * 100.0) as u32;
        let _ = Command::new("pactl")
            .args([
                "set-source-volume",
                "@DEFAULT_SOURCE@",
                &format!("{}%", percent),
            ])
            .output();
    }

    pub fn set_mic_muted(&self, muted: bool) {
        let value = if muted { "1" } else { "0" };
        let _ = Command::new("pactl")
            .args(["set-source-mute", "@DEFAULT_SOURCE@", value])
            .output();
    }

    pub fn source_name(&self) -> String {
        self.imp().source_name.borrow().clone()
    }

    fn device_type(&self) -> DeviceType {
        // The identifier is "sink_name|active_port" - check both parts
        let identifier = self.sink_name().to_lowercase();
        if identifier.contains("headset") {
            DeviceType::Headset
        } else if identifier.contains("headphones") || identifier.contains("headphone") {
            DeviceType::Headphones
        } else {
            DeviceType::Speakers
        }
    }

    fn mic_device_type(&self) -> MicDeviceType {
        // Check the source name to determine microphone type
        let identifier = self.source_name().to_lowercase();
        if identifier.contains("headset") {
            MicDeviceType::Headset
        } else {
            MicDeviceType::Default
        }
    }

    pub fn icon_name(&self) -> String {
        let device_type = self.device_type();
        let vol = self.volume();
        let muted = self.muted();

        // Use device-specific icons for headphones/headset with mute variants
        // Use volume-based icons for speakers (including muted icon)
        match device_type {
            DeviceType::Headset => {
                if muted {
                    "audio-headset-muted-symbolic".to_string()
                } else {
                    "audio-headset-symbolic".to_string()
                }
            }
            DeviceType::Headphones => {
                if muted {
                    "audio-headphones-muted-symbolic".to_string()
                } else {
                    "audio-headphones-symbolic".to_string()
                }
            }
            DeviceType::Speakers => {
                if muted || vol == 0.0 {
                    "audio-volume-muted-symbolic".to_string()
                } else if vol > 1.0 {
                    "audio-volume-overamplified-symbolic".to_string()
                } else if vol < 0.33 {
                    "audio-volume-low-symbolic".to_string()
                } else if vol < 0.67 {
                    "audio-volume-medium-symbolic".to_string()
                } else {
                    "audio-volume-high-symbolic".to_string()
                }
            }
        }
    }

    pub fn mic_icon_name(&self) -> String {
        let mic_type = self.mic_device_type();
        let muted = self.mic_muted();

        match mic_type {
            MicDeviceType::Headset => {
                if muted {
                    "audio-headset-mic-muted-symbolic".to_string()
                } else {
                    "audio-headset-mic-symbolic".to_string()
                }
            }
            MicDeviceType::Default => {
                if muted {
                    "microphone-sensitivity-muted-symbolic".to_string()
                } else {
                    let vol = self.mic_volume();
                    if vol == 0.0 {
                        "microphone-sensitivity-muted-symbolic".to_string()
                    } else if vol < 0.33 {
                        "microphone-sensitivity-low-symbolic".to_string()
                    } else if vol < 0.67 {
                        "microphone-sensitivity-medium-symbolic".to_string()
                    } else {
                        "microphone-sensitivity-high-symbolic".to_string()
                    }
                }
            }
        }
    }
}

impl Default for Audio {
    fn default() -> Self {
        Self::new()
    }
}

mod imp {
    use super::*;

    use glib::subclass::prelude::*;
    use glib::{ParamSpec, ParamSpecBuilderExt, Value};

    pub struct Audio {
        pub volume: Cell<f64>,
        pub muted: Cell<bool>,
        pub mic_volume: Cell<f64>,
        pub mic_muted: Cell<bool>,
        pub sink_name: RefCell<String>,
        pub source_name: RefCell<String>,
        pub sinks: RefCell<Vec<super::AudioSink>>,
        pub sources: RefCell<Vec<super::AudioSource>>,
    }

    impl Default for Audio {
        fn default() -> Self {
            Self {
                volume: Cell::new(0.0),
                muted: Cell::new(false),
                mic_volume: Cell::new(0.0),
                mic_muted: Cell::new(false),
                sink_name: RefCell::new(String::new()),
                source_name: RefCell::new(String::new()),
                sinks: RefCell::new(Vec::new()),
                sources: RefCell::new(Vec::new()),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Audio {
        const NAME: &'static str = "Audio";
        type Type = super::Audio;
    }

    impl ObjectImpl for Audio {
        fn constructed(&self) {
            self.parent_constructed();

            // Initialize current state
            self.update_sink_state();
            self.update_source_state();
            self.update_sinks_list();
            self.update_sources_list();

            // Setup event listener
            self.setup_event_listener();
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: OnceLock<Vec<ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![
                    glib::ParamSpecDouble::builder("volume")
                        .minimum(0.0)
                        .maximum(1.0)
                        .read_only()
                        .build(),
                    glib::ParamSpecBoolean::builder("muted").read_only().build(),
                    glib::ParamSpecDouble::builder("mic-volume")
                        .minimum(0.0)
                        .maximum(1.0)
                        .read_only()
                        .build(),
                    glib::ParamSpecBoolean::builder("mic-muted")
                        .read_only()
                        .build(),
                    glib::ParamSpecString::builder("sink-name")
                        .read_only()
                        .build(),
                    glib::ParamSpecString::builder("source-name")
                        .read_only()
                        .build(),
                    glib::ParamSpecUInt::builder("sinks").read_only().build(),
                    glib::ParamSpecUInt::builder("sources").read_only().build(),
                ]
            })
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "volume" => self.volume.get().to_value(),
                "muted" => self.muted.get().to_value(),
                "mic-volume" => self.mic_volume.get().to_value(),
                "mic-muted" => self.mic_muted.get().to_value(),
                "sink-name" => self.sink_name.borrow().to_value(),
                "source-name" => self.source_name.borrow().to_value(),
                "sinks" => (self.sinks.borrow().len() as u32).to_value(),
                "sources" => (self.sources.borrow().len() as u32).to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl Audio {
        fn setup_event_listener(&self) {
            let obj = self.obj().clone();

            // Spawn async pactl listener on glib main context - fully event-driven!
            glib::MainContext::default().spawn_local(async move {
                let mut child = match async_process::Command::new("pactl")
                    .arg("subscribe")
                    .stdout(async_process::Stdio::piped())
                    .spawn()
                {
                    Ok(child) => child,
                    Err(e) => {
                        let err_msg = format!("Failed to spawn pactl subscribe: {}", e);
                        crate::service::idle_add_safe(move || {
                            crate::service::notifications::Notifications::instance()
                                .send_notification("rusty-de", "Audio Error", &err_msg);
                        });
                        return;
                    }
                };

                let stdout = match child.stdout.take() {
                    Some(stdout) => stdout,
                    None => {
                        crate::service::idle_add_safe(move || {
                            crate::service::notifications::Notifications::instance()
                                .send_notification(
                                    "rusty-de",
                                    "Audio Error",
                                    "Failed to capture stdout",
                                );
                        });
                        return;
                    }
                };

                use futures_util::io::BufReader;
                use futures_util::AsyncBufReadExt;
                use futures_util::StreamExt;

                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();

                while let Some(Ok(line)) = lines.next().await {
                    let is_sink_event = line.contains("sink") && !line.contains("sink-input");
                    let is_source_event =
                        line.contains("source") && !line.contains("source-output");
                    let is_server = line.contains("server");

                    if line.contains("'change'") || line.contains("'new'") {
                        // Handle sink and server changes (both update sink state)
                        if is_sink_event || is_server {
                            let (volume_changed, muted_changed, name_changed) =
                                obj.imp().update_sink_state();
                            if volume_changed {
                                obj.notify("volume");
                            }
                            if muted_changed {
                                obj.notify("muted");
                            }
                            if name_changed {
                                obj.notify("sink-name");
                            }
                        }
                        // Handle source changes (microphone)
                        else if is_source_event {
                            let (volume_changed, muted_changed) = obj.imp().update_source_state();
                            if volume_changed {
                                obj.notify("mic-volume");
                            }
                            if muted_changed {
                                obj.notify("mic-muted");
                            }
                        }
                    }

                    // Update device lists on sink/source/server change, new, or remove events
                    // Server events occur when default sink/source changes
                    if (is_sink_event || is_server)
                        && (line.contains("'change'")
                            || line.contains("'new'")
                            || line.contains("'remove'"))
                        && obj.imp().update_sinks_list()
                    {
                        obj.notify("sinks");
                    }

                    if (is_source_event || is_server)
                        && (line.contains("'change'")
                            || line.contains("'new'")
                            || line.contains("'remove'"))
                        && obj.imp().update_sources_list()
                    {
                        obj.notify("sources");
                    }
                }

                // Clean up child process
                let _ = child.kill();
                let _ = child.status().await;
            });
        }

        fn update_sink_state(&self) -> (bool, bool, bool) {
            let mut volume_changed = false;
            let mut muted_changed = false;
            let mut name_changed = false;

            // Strategy: Track both sink name AND active port to detect device changes
            // For USB headsets: sink name changes (e.g., speaker sink -> USB headset sink)
            // For built-in jacks: sink stays same but active port changes
            if let Ok(output) = Command::new("pactl").args(["list", "sinks"]).output() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let mut in_default_sink = false;
                let mut default_sink_name = String::new();

                // First get the default sink name
                if let Ok(info_output) = Command::new("pactl").args(["info"]).output() {
                    let info_str = String::from_utf8_lossy(&info_output.stdout);
                    for line in info_str.lines() {
                        if line.starts_with("Default Sink: ") {
                            default_sink_name =
                                line.trim_start_matches("Default Sink: ").trim().to_string();
                            break;
                        }
                    }
                }

                // Build identifier: "sink_name|active_port" to detect changes in either
                let mut current_identifier = default_sink_name.clone();

                // Now parse the sink list to find active port and description
                for line in output_str.lines() {
                    let trimmed = line.trim();

                    if trimmed.starts_with("Name: ") {
                        let sink_name = trimmed.trim_start_matches("Name: ");
                        in_default_sink = sink_name == default_sink_name;
                    }

                    if in_default_sink && trimmed.starts_with("Active Port: ") {
                        let port_info = trimmed.trim_start_matches("Active Port: ");
                        current_identifier.push('|');
                        current_identifier.push_str(port_info);
                        break;
                    }
                }

                let old_identifier = self.sink_name.borrow().clone();
                if current_identifier != old_identifier {
                    *self.sink_name.borrow_mut() = current_identifier;
                    name_changed = true;
                }
            }

            // Get volume
            if let Ok(output) = Command::new("pactl")
                .args(["get-sink-volume", "@DEFAULT_SINK@"])
                .output()
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if let Some(volume) = parse_volume(&output_str) {
                    let old_volume = self.volume.get();
                    if (volume - old_volume).abs() > f64::EPSILON {
                        self.volume.set(volume);
                        volume_changed = true;
                    }
                }
            }

            // Get mute state
            if let Ok(output) = Command::new("pactl")
                .args(["get-sink-mute", "@DEFAULT_SINK@"])
                .output()
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let muted = output_str.trim().ends_with("yes");
                let old_muted = self.muted.get();
                if muted != old_muted {
                    self.muted.set(muted);
                    muted_changed = true;
                }
            }

            (volume_changed, muted_changed, name_changed)
        }

        fn update_source_state(&self) -> (bool, bool) {
            let mut volume_changed = false;
            let mut muted_changed = false;

            // Get volume
            if let Ok(output) = Command::new("pactl")
                .args(["get-source-volume", "@DEFAULT_SOURCE@"])
                .output()
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if let Some(volume) = parse_volume(&output_str) {
                    let old_volume = self.mic_volume.get();
                    if (volume - old_volume).abs() > f64::EPSILON {
                        self.mic_volume.set(volume);
                        volume_changed = true;
                    }
                }
            }

            // Get mute state
            if let Ok(output) = Command::new("pactl")
                .args(["get-source-mute", "@DEFAULT_SOURCE@"])
                .output()
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let muted = output_str.trim().ends_with("yes");
                let old_muted = self.mic_muted.get();
                if muted != old_muted {
                    self.mic_muted.set(muted);
                    muted_changed = true;
                }
            }

            (volume_changed, muted_changed)
        }

        pub fn update_sinks_list(&self) -> bool {
            let mut new_sinks = Vec::new();
            let mut default_sink_name = String::new();

            // Get the default sink name
            if let Ok(info_output) = Command::new("pactl").args(["info"]).output() {
                let info_str = String::from_utf8_lossy(&info_output.stdout);
                for line in info_str.lines() {
                    if line.starts_with("Default Sink: ") {
                        default_sink_name =
                            line.trim_start_matches("Default Sink: ").trim().to_string();
                        break;
                    }
                }
            }

            // Parse all sinks
            if let Ok(output) = Command::new("pactl")
                .args(["list", "sinks", "short"])
                .output()
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let sink_name = parts[1].to_string();
                        let description = if parts.len() >= 3 {
                            parts[2..].join(" ")
                        } else {
                            sink_name.clone()
                        };

                        // Get better description and availability from detailed list
                        let mut better_description = description.clone();
                        let mut is_available = true; // Default to available
                        if let Ok(detail_output) =
                            Command::new("pactl").args(["list", "sinks"]).output()
                        {
                            let detail_str = String::from_utf8_lossy(&detail_output.stdout);
                            let mut in_target_sink = false;
                            let mut found_description = false;
                            for detail_line in detail_str.lines() {
                                let trimmed = detail_line.trim();
                                if trimmed.starts_with("Name: ") {
                                    in_target_sink =
                                        trimmed.trim_start_matches("Name: ") == sink_name;
                                    found_description = false;
                                }
                                if in_target_sink {
                                    if trimmed.starts_with("Description: ") {
                                        better_description =
                                            trimmed.trim_start_matches("Description: ").to_string();
                                        found_description = true;
                                    }
                                    // Check port availability
                                    if trimmed.contains("not available") {
                                        is_available = false;
                                    }
                                    // Stop after we found description and checked ports
                                    if found_description && trimmed.starts_with("Active Port:") {
                                        break;
                                    }
                                }
                            }
                        }

                        new_sinks.push(super::AudioSink {
                            name: sink_name.clone(),
                            description: better_description,
                            is_default: sink_name == default_sink_name,
                            is_available,
                        });
                    }
                }
            }

            let old_sinks = self.sinks.borrow().clone();
            let changed = new_sinks.len() != old_sinks.len()
                || new_sinks
                    .iter()
                    .zip(old_sinks.iter())
                    .any(|(new, old)| new.name != old.name || new.is_default != old.is_default);

            if changed {
                *self.sinks.borrow_mut() = new_sinks;
            }

            changed
        }

        pub fn update_sources_list(&self) -> bool {
            let mut new_sources = Vec::new();
            let mut default_source_name = String::new();

            // Get the default source name
            if let Ok(info_output) = Command::new("pactl").args(["info"]).output() {
                let info_str = String::from_utf8_lossy(&info_output.stdout);
                for line in info_str.lines() {
                    if line.starts_with("Default Source: ") {
                        default_source_name = line
                            .trim_start_matches("Default Source: ")
                            .trim()
                            .to_string();
                        break;
                    }
                }
            }

            // Parse all sources, excluding monitors
            if let Ok(output) = Command::new("pactl")
                .args(["list", "sources", "short"])
                .output()
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let source_name = parts[1].to_string();

                        // Skip monitor sources (they're outputs, not inputs)
                        if source_name.contains(".monitor") {
                            continue;
                        }

                        let description = if parts.len() >= 3 {
                            parts[2..].join(" ")
                        } else {
                            source_name.clone()
                        };

                        // Get better description and availability from detailed list
                        let mut better_description = description.clone();
                        let mut is_available = true; // Default to available
                        if let Ok(detail_output) =
                            Command::new("pactl").args(["list", "sources"]).output()
                        {
                            let detail_str = String::from_utf8_lossy(&detail_output.stdout);
                            let mut in_target_source = false;
                            let mut found_description = false;
                            for detail_line in detail_str.lines() {
                                let trimmed = detail_line.trim();
                                if trimmed.starts_with("Name: ") {
                                    in_target_source =
                                        trimmed.trim_start_matches("Name: ") == source_name;
                                    found_description = false;
                                }
                                if in_target_source {
                                    if trimmed.starts_with("Description: ") {
                                        better_description =
                                            trimmed.trim_start_matches("Description: ").to_string();
                                        found_description = true;
                                    }
                                    // Check port availability
                                    if trimmed.contains("not available") {
                                        is_available = false;
                                    }
                                    // Stop after we found description and checked ports
                                    if found_description && trimmed.starts_with("Active Port:") {
                                        break;
                                    }
                                }
                            }
                        }

                        new_sources.push(super::AudioSource {
                            name: source_name.clone(),
                            description: better_description,
                            is_default: source_name == default_source_name,
                            is_available,
                        });
                    }
                }
            }

            let old_sources = self.sources.borrow().clone();
            let changed = new_sources.len() != old_sources.len()
                || new_sources
                    .iter()
                    .zip(old_sources.iter())
                    .any(|(new, old)| new.name != old.name || new.is_default != old.is_default);

            if changed {
                *self.sources.borrow_mut() = new_sources;
            }

            // Update source name if changed
            let old_source_name = self.source_name.borrow().clone();
            if default_source_name != old_source_name {
                *self.source_name.borrow_mut() = default_source_name;
                self.obj().notify("source-name");
            }

            changed
        }
    }

    fn parse_volume(output: &str) -> Option<f64> {
        // Parse output like "Volume: front-left: 65536 / 100% / 0.00 dB, front-right: 65536 / 100%"
        for part in output.split_whitespace() {
            if part.ends_with('%') {
                if let Ok(percent) = part.trim_end_matches('%').parse::<f64>() {
                    return Some(percent / 100.0);
                }
            }
        }
        None
    }
}
