use glib::subclass::prelude::*;
use glib::Object;
use gtk4::glib;
use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::fs;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: String,
    pub cpu: String,
    pub mem: String,
    pub command: String,
    pub full_command: String,
}

glib::wrapper! {
    pub struct SystemInfo(ObjectSubclass<imp::SystemInfo>);
}

thread_local! {
    static SYSTEM_INFO_INSTANCE: RefCell<Option<SystemInfo>> = const { RefCell::new(None) };
}

impl SystemInfo {
    pub fn instance() -> SystemInfo {
        SYSTEM_INFO_INSTANCE.with(|instance| {
            let mut instance_mut = instance.borrow_mut();
            if instance_mut.is_none() {
                let obj: SystemInfo = Object::builder().build();
                *instance_mut = Some(obj);
            }
            instance_mut.as_ref().unwrap().clone()
        })
    }

    pub fn cpu_usage(&self) -> f64 {
        self.imp().cpu_usage.get()
    }

    pub fn memory_usage(&self) -> String {
        self.imp().memory_usage.borrow().clone()
    }

    pub fn swap_usage(&self) -> String {
        self.imp().swap_usage.borrow().clone()
    }

    pub fn disk_usage(&self) -> String {
        self.imp().disk_usage.borrow().clone()
    }

    pub fn top_processes(&self) -> Vec<ProcessInfo> {
        self.imp().top_processes.borrow().clone()
    }

    pub fn refresh(&self) {
        self.imp().update_cpu_usage();
        self.imp().update_memory_info();
        self.imp().update_disk_usage();
        self.imp().update_top_processes();

        self.notify("cpu-usage");
        self.notify("memory-usage");
        self.notify("swap-usage");
        self.notify("disk-usage");
        self.notify("top-processes");
    }
}

impl Default for SystemInfo {
    fn default() -> Self {
        Self::instance()
    }
}

mod imp {
    use super::*;
    use glib::subclass::prelude::*;
    use glib::{ParamSpec, ParamSpecBuilderExt, Value};
    use std::sync::OnceLock;

    pub struct SystemInfo {
        pub cpu_usage: Cell<f64>,
        pub memory_usage: RefCell<String>,
        pub swap_usage: RefCell<String>,
        pub disk_usage: RefCell<String>,
        pub top_processes: RefCell<Vec<ProcessInfo>>,
        prev_total: Cell<u64>,
        prev_idle: Cell<u64>,
    }

    impl Default for SystemInfo {
        fn default() -> Self {
            Self {
                cpu_usage: Cell::new(0.0),
                memory_usage: RefCell::new(String::new()),
                swap_usage: RefCell::new(String::new()),
                disk_usage: RefCell::new(String::new()),
                top_processes: RefCell::new(Vec::new()),
                prev_total: Cell::new(0),
                prev_idle: Cell::new(0),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SystemInfo {
        const NAME: &'static str = "SystemInfo";
        type Type = super::SystemInfo;
    }

    impl ObjectImpl for SystemInfo {
        fn constructed(&self) {
            self.parent_constructed();
            self.update_cpu_usage();
            self.update_memory_info();
            self.update_disk_usage();
            self.update_top_processes();
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: OnceLock<Vec<ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![
                    glib::ParamSpecDouble::builder("cpu-usage")
                        .read_only()
                        .build(),
                    glib::ParamSpecString::builder("memory-usage")
                        .read_only()
                        .build(),
                    glib::ParamSpecString::builder("swap-usage")
                        .read_only()
                        .build(),
                    glib::ParamSpecString::builder("disk-usage")
                        .read_only()
                        .build(),
                    glib::ParamSpecBoxed::builder::<glib::StrV>("top-processes")
                        .read_only()
                        .build(),
                ]
            })
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "cpu-usage" => self.cpu_usage.get().to_value(),
                "memory-usage" => self.memory_usage.borrow().to_value(),
                "swap-usage" => self.swap_usage.borrow().to_value(),
                "disk-usage" => self.disk_usage.borrow().to_value(),
                "top-processes" => {
                    let processes = self.top_processes.borrow();
                    let strings: Vec<String> = processes
                        .iter()
                        .map(|p| format!("{}\t{}\t{}\t{}", p.pid, p.cpu, p.mem, p.command))
                        .collect();
                    glib::StrV::from(strings).to_value()
                }
                _ => unimplemented!(),
            }
        }
    }

    impl SystemInfo {
        pub(crate) fn update_cpu_usage(&self) {
            if let Ok(stat) = fs::read_to_string("/proc/stat") {
                if let Some(cpu_line) = stat.lines().next() {
                    let parts: Vec<&str> = cpu_line.split_whitespace().collect();
                    if parts.len() > 4 && parts[0] == "cpu" {
                        let user: u64 = parts[1].parse().unwrap_or(0);
                        let nice: u64 = parts[2].parse().unwrap_or(0);
                        let system: u64 = parts[3].parse().unwrap_or(0);
                        let idle: u64 = parts[4].parse().unwrap_or(0);

                        let total = user + nice + system + idle;
                        let prev_total = self.prev_total.get();
                        let prev_idle = self.prev_idle.get();

                        if prev_total > 0 {
                            let total_diff = total.saturating_sub(prev_total);
                            let idle_diff = idle.saturating_sub(prev_idle);

                            if total_diff > 0 {
                                let usage =
                                    ((total_diff - idle_diff) as f64 / total_diff as f64) * 100.0;
                                self.cpu_usage.set(usage);
                            }
                        }

                        self.prev_total.set(total);
                        self.prev_idle.set(idle);
                    }
                }
            }
        }

        pub(crate) fn update_memory_info(&self) {
            if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
                let mut mem_total = 0u64;
                let mut mem_available = 0u64;
                let mut swap_total = 0u64;
                let mut swap_free = 0u64;

                for line in meminfo.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() < 2 {
                        continue;
                    }

                    match parts[0] {
                        "MemTotal:" => mem_total = parts[1].parse().unwrap_or(0),
                        "MemAvailable:" => mem_available = parts[1].parse().unwrap_or(0),
                        "SwapTotal:" => swap_total = parts[1].parse().unwrap_or(0),
                        "SwapFree:" => swap_free = parts[1].parse().unwrap_or(0),
                        _ => {}
                    }
                }

                let mem_used = mem_total.saturating_sub(mem_available);
                let mem_used_gb = mem_used as f64 / 1024.0 / 1024.0;
                let mem_total_gb = mem_total as f64 / 1024.0 / 1024.0;
                let mem_percent = if mem_total > 0 {
                    (mem_used as f64 / mem_total as f64) * 100.0
                } else {
                    0.0
                };

                *self.memory_usage.borrow_mut() = format!(
                    "{:.1} GB / {:.1} GB ({:.0}%)",
                    mem_used_gb, mem_total_gb, mem_percent
                );

                let swap_used = swap_total.saturating_sub(swap_free);
                let swap_used_gb = swap_used as f64 / 1024.0 / 1024.0;
                let swap_total_gb = swap_total as f64 / 1024.0 / 1024.0;
                let swap_percent = if swap_total > 0 {
                    (swap_used as f64 / swap_total as f64) * 100.0
                } else {
                    0.0
                };

                *self.swap_usage.borrow_mut() = if swap_total > 0 {
                    format!(
                        "{:.1} GB / {:.1} GB ({:.0}%)",
                        swap_used_gb, swap_total_gb, swap_percent
                    )
                } else {
                    "Not available".to_string()
                };
            }
        }

        pub(crate) fn update_disk_usage(&self) {
            let output = Command::new("df").args(["-h", "/"]).output();

            if let Ok(output) = output {
                if let Ok(stdout) = String::from_utf8(output.stdout) {
                    for line in stdout.lines().skip(1) {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 5 {
                            let total = parts[1];
                            let used = parts[2];
                            let percent = parts[4];

                            *self.disk_usage.borrow_mut() =
                                format!("{} / {} ({})", used, total, percent);
                            return;
                        }
                    }
                }
            }

            *self.disk_usage.borrow_mut() = "Not available".to_string();
        }

        pub(crate) fn update_top_processes(&self) {
            let output = Command::new("ps").args(["aux", "--sort=-%cpu"]).output();

            if let Ok(output) = output {
                if let Ok(stdout) = String::from_utf8(output.stdout) {
                    let mut processes = Vec::new();

                    for (i, line) in stdout.lines().skip(1).enumerate() {
                        if i >= 10 {
                            break;
                        }

                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 11 {
                            let pid = parts[1].to_string();
                            let cpu = parts[2].to_string();
                            let mem = parts[3].to_string();
                            let full_command = parts[10..].join(" ");

                            let command = if full_command.len() > 40 {
                                format!("{}...", &full_command[..37])
                            } else {
                                full_command.clone()
                            };

                            processes.push(ProcessInfo {
                                pid,
                                cpu,
                                mem,
                                command,
                                full_command,
                            });
                        }
                    }

                    *self.top_processes.borrow_mut() = processes;
                }
            }
        }
    }
}
