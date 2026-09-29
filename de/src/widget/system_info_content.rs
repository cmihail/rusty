use glib::clone;
use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation, ScrolledWindow, Separator};

use crate::service::system_info::SystemInfo;

pub struct SystemInfoContent {
    widget: ScrolledWindow,
}

impl Default for SystemInfoContent {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemInfoContent {
    pub fn new() -> Self {
        let system_info = SystemInfo::instance();

        let scrolled = ScrolledWindow::new();
        scrolled.set_hscrollbar_policy(gtk4::PolicyType::Never);
        scrolled.set_vscrollbar_policy(gtk4::PolicyType::Automatic);
        scrolled.set_propagate_natural_height(true);
        scrolled.set_max_content_height(500);

        let container = Box::new(Orientation::Vertical, 12);
        container.set_margin_top(12);
        container.set_margin_bottom(12);
        container.set_margin_start(12);
        container.set_margin_end(12);

        let title = Label::new(Some("System Information"));
        title.add_css_class("title-4");
        title.set_halign(gtk4::Align::Start);
        container.append(&title);

        let separator1 = Separator::new(Orientation::Horizontal);
        container.append(&separator1);

        let cpu_label = Label::new(None);
        cpu_label.set_halign(gtk4::Align::Start);
        cpu_label.set_use_markup(true);
        Self::update_cpu_label(&cpu_label, system_info.cpu_usage());
        container.append(&cpu_label);

        let memory_label = Label::new(None);
        memory_label.set_halign(gtk4::Align::Start);
        memory_label.set_use_markup(true);
        Self::update_memory_label(&memory_label, &system_info.memory_usage());
        container.append(&memory_label);

        let swap_label = Label::new(None);
        swap_label.set_halign(gtk4::Align::Start);
        swap_label.set_use_markup(true);
        Self::update_swap_label(&swap_label, &system_info.swap_usage());
        container.append(&swap_label);

        let disk_label = Label::new(None);
        disk_label.set_halign(gtk4::Align::Start);
        disk_label.set_use_markup(true);
        Self::update_disk_label(&disk_label, &system_info.disk_usage());
        container.append(&disk_label);

        let separator2 = Separator::new(Orientation::Horizontal);
        container.append(&separator2);

        let processes_title = Label::new(Some("Top 10 Processes (by CPU)"));
        processes_title.add_css_class("title-4");
        processes_title.set_halign(gtk4::Align::Start);
        container.append(&processes_title);

        let processes_box = Box::new(Orientation::Vertical, 4);
        Self::update_processes(&processes_box, &system_info.top_processes());
        container.append(&processes_box);

        scrolled.set_child(Some(&container));

        system_info.connect_notify_local(
            Some("cpu-usage"),
            clone!(
                #[weak]
                cpu_label,
                move |si, _| {
                    Self::update_cpu_label(&cpu_label, si.cpu_usage());
                }
            ),
        );

        system_info.connect_notify_local(
            Some("memory-usage"),
            clone!(
                #[weak]
                memory_label,
                move |si, _| {
                    Self::update_memory_label(&memory_label, &si.memory_usage());
                }
            ),
        );

        system_info.connect_notify_local(
            Some("swap-usage"),
            clone!(
                #[weak]
                swap_label,
                move |si, _| {
                    Self::update_swap_label(&swap_label, &si.swap_usage());
                }
            ),
        );

        system_info.connect_notify_local(
            Some("disk-usage"),
            clone!(
                #[weak]
                disk_label,
                move |si, _| {
                    Self::update_disk_label(&disk_label, &si.disk_usage());
                }
            ),
        );

        system_info.connect_notify_local(
            Some("top-processes"),
            clone!(
                #[weak]
                processes_box,
                move |si, _| {
                    Self::update_processes(&processes_box, &si.top_processes());
                }
            ),
        );

        Self { widget: scrolled }
    }

    pub fn widget(&self) -> &ScrolledWindow {
        &self.widget
    }

    fn update_cpu_label(label: &Label, cpu_usage: f64) {
        label.set_markup(&format!("<b>CPU Usage:</b> {:.1}%", cpu_usage));
    }

    fn update_memory_label(label: &Label, memory_usage: &str) {
        label.set_markup(&format!("<b>Memory:</b> {}", memory_usage));
    }

    fn update_swap_label(label: &Label, swap_usage: &str) {
        label.set_markup(&format!("<b>Swap:</b> {}", swap_usage));
    }

    fn update_disk_label(label: &Label, disk_usage: &str) {
        label.set_markup(&format!("<b>Disk Usage (/):</b> {}", disk_usage));
    }

    fn update_processes(
        processes_box: &Box,
        processes: &[crate::service::system_info::ProcessInfo],
    ) {
        while let Some(child) = processes_box.first_child() {
            processes_box.remove(&child);
        }

        let header_box = Box::new(Orientation::Horizontal, 8);
        header_box.set_homogeneous(false);

        let pid_label = Label::new(Some("PID"));
        pid_label.add_css_class("dim-label");
        pid_label.set_width_chars(8);
        pid_label.set_xalign(0.0);
        header_box.append(&pid_label);

        let cpu_label = Label::new(Some("CPU%"));
        cpu_label.add_css_class("dim-label");
        cpu_label.set_width_chars(6);
        cpu_label.set_xalign(0.0);
        header_box.append(&cpu_label);

        let mem_label = Label::new(Some("MEM%"));
        mem_label.add_css_class("dim-label");
        mem_label.set_width_chars(6);
        mem_label.set_xalign(0.0);
        header_box.append(&mem_label);

        let cmd_label = Label::new(Some("Command"));
        cmd_label.add_css_class("dim-label");
        cmd_label.set_hexpand(true);
        cmd_label.set_width_chars(40);
        cmd_label.set_xalign(0.0);
        header_box.append(&cmd_label);

        // Empty space for kill button column
        let spacer_label = Label::new(None);
        spacer_label.set_width_chars(3);
        header_box.append(&spacer_label);

        processes_box.append(&header_box);

        for process in processes {
            let process_box = Box::new(Orientation::Horizontal, 8);
            process_box.set_homogeneous(false);
            process_box.set_tooltip_text(Some(&process.full_command));

            let pid_label = Label::new(Some(&process.pid));
            pid_label.set_width_chars(8);
            pid_label.set_xalign(0.0);
            pid_label.add_css_class("monospace");
            process_box.append(&pid_label);

            let cpu_label = Label::new(Some(&process.cpu));
            cpu_label.set_width_chars(6);
            cpu_label.set_xalign(0.0);
            cpu_label.add_css_class("monospace");
            process_box.append(&cpu_label);

            let mem_label = Label::new(Some(&process.mem));
            mem_label.set_width_chars(6);
            mem_label.set_xalign(0.0);
            mem_label.add_css_class("monospace");
            process_box.append(&mem_label);

            let cmd_label = Label::new(Some(&process.command));
            cmd_label.set_hexpand(true);
            cmd_label.set_xalign(0.0);
            cmd_label.set_width_chars(40);
            cmd_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
            cmd_label.add_css_class("monospace");
            process_box.append(&cmd_label);

            // Kill button
            let kill_button = Button::from_icon_name("window-close-symbolic");
            kill_button.add_css_class("KillProcess");
            kill_button.set_tooltip_text(Some("Kill process"));
            let pid = process.pid.clone();
            kill_button.connect_clicked(move |_| {
                use std::process::{Command, Stdio};
                let _ = Command::new("kill")
                    .arg("-9")
                    .arg(&pid)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn();
            });
            process_box.append(&kill_button);

            processes_box.append(&process_box);
        }
    }
}
