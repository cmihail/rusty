use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation, Popover, ScrolledWindow, Separator};
use std::collections::HashMap;
use std::rc::Rc;

use crate::service::hyprland::{Client, Hyprland};

pub struct ClientsContent {
    widget: ScrolledWindow,
    clients_box: Box,
    clients_popover: Rc<std::cell::RefCell<Option<Popover>>>,
    control_center_popover: Rc<std::cell::RefCell<Option<Popover>>>,
}

impl Default for ClientsContent {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientsContent {
    pub fn new() -> Self {
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

        let title_box = Box::new(Orientation::Horizontal, 8);

        let title = Label::new(Some("Hyprland Clients"));
        title.add_css_class("title-4");
        title.set_halign(gtk4::Align::Start);
        title.set_hexpand(true);
        title_box.append(&title);

        let clients_popover: Rc<std::cell::RefCell<Option<Popover>>> =
            Rc::new(std::cell::RefCell::new(None));
        let control_center_popover: Rc<std::cell::RefCell<Option<Popover>>> =
            Rc::new(std::cell::RefCell::new(None));

        let kill_label = Label::new(Some("Click to kill"));
        kill_label.set_halign(gtk4::Align::End);
        title_box.append(&kill_label);

        let kill_button = Button::from_icon_name("input-mouse-symbolic");
        kill_button.set_tooltip_text(Some("Click to kill mode"));
        kill_button.add_css_class("KillProcess");
        let clients_pop_clone = clients_popover.clone();
        let cc_pop_clone = control_center_popover.clone();
        kill_button.connect_clicked(move |_| {
            // Close both popovers
            if let Some(pop) = clients_pop_clone.borrow().as_ref() {
                pop.popdown();
            }
            if let Some(pop) = cc_pop_clone.borrow().as_ref() {
                pop.popdown();
            }

            // Invoke hyprctl kill
            use std::process::{Command, Stdio};
            let _ = Command::new("hyprctl")
                .arg("kill")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();
        });
        title_box.append(&kill_button);

        container.append(&title_box);

        let separator = Separator::new(Orientation::Horizontal);
        container.append(&separator);

        let clients_box = Box::new(Orientation::Vertical, 12);
        container.append(&clients_box);

        scrolled.set_child(Some(&container));

        Self {
            widget: scrolled,
            clients_box,
            clients_popover,
            control_center_popover,
        }
    }

    pub fn widget(&self) -> &ScrolledWindow {
        &self.widget
    }

    pub fn set_popovers(&self, clients_popover: &Popover, control_center_popover: &Popover) {
        *self.clients_popover.borrow_mut() = Some(clients_popover.clone());
        *self.control_center_popover.borrow_mut() = Some(control_center_popover.clone());
    }

    pub fn refresh(&self) {
        self.clear_clients();

        let hyprland = match Hyprland::instance() {
            Some(h) => h,
            None => {
                self.show_message("Hyprland not running");
                return;
            }
        };

        let clients = hyprland.clients();
        let monitors = hyprland.monitors();

        if clients.is_empty() {
            self.show_message("No active clients");
            return;
        }

        let clients_by_monitor = Self::group_clients_by_monitor(clients);
        let monitor_ids = Self::get_sorted_monitor_ids(&clients_by_monitor);

        self.add_monitors(&monitor_ids, &clients_by_monitor, &monitors);
    }

    fn clear_clients(&self) {
        while let Some(child) = self.clients_box.first_child() {
            self.clients_box.remove(&child);
        }
    }

    fn show_message(&self, message: &str) {
        let label = Label::new(Some(message));
        label.add_css_class("dim-label");
        self.clients_box.append(&label);
    }

    fn group_clients_by_monitor(clients: Vec<Client>) -> HashMap<i32, HashMap<i32, Vec<Client>>> {
        let mut clients_by_monitor: HashMap<i32, HashMap<i32, Vec<Client>>> = HashMap::new();
        for client in clients {
            clients_by_monitor
                .entry(client.monitor)
                .or_default()
                .entry(client.workspace.id)
                .or_default()
                .push(client);
        }
        clients_by_monitor
    }

    fn get_sorted_monitor_ids(
        clients_by_monitor: &HashMap<i32, HashMap<i32, Vec<Client>>>,
    ) -> Vec<i32> {
        let mut monitor_ids: Vec<i32> = clients_by_monitor.keys().copied().collect();
        monitor_ids.sort();
        monitor_ids
    }

    fn add_monitors(
        &self,
        monitor_ids: &[i32],
        clients_by_monitor: &HashMap<i32, HashMap<i32, Vec<Client>>>,
        monitors: &[crate::service::hyprland::Monitor],
    ) {
        for monitor_id in monitor_ids {
            self.add_monitor(
                *monitor_id,
                clients_by_monitor.get(monitor_id).unwrap(),
                monitors,
            );

            let separator = Separator::new(Orientation::Horizontal);
            self.clients_box.append(&separator);
        }
    }

    fn add_monitor(
        &self,
        monitor_id: i32,
        workspaces: &HashMap<i32, Vec<Client>>,
        monitors: &[crate::service::hyprland::Monitor],
    ) {
        let monitor_name = monitors
            .iter()
            .find(|m| m.id == monitor_id)
            .map(|m| format!("{} ({})", m.name, m.model))
            .unwrap_or_else(|| format!("Monitor {}", monitor_id));

        let monitor_label = Label::new(Some(&monitor_name));
        monitor_label.add_css_class("title-4");
        monitor_label.set_halign(gtk4::Align::Start);
        self.clients_box.append(&monitor_label);

        self.add_workspaces(workspaces);
    }

    fn add_workspaces(&self, workspaces: &HashMap<i32, Vec<Client>>) {
        let mut workspace_ids: Vec<i32> = workspaces.keys().copied().collect();
        workspace_ids.sort();

        for workspace_id in workspace_ids {
            let workspace_clients = workspaces.get(&workspace_id).unwrap();
            self.add_workspace(workspace_id, workspace_clients);
        }
    }

    fn add_workspace(&self, workspace_id: i32, workspace_clients: &[Client]) {
        let workspace_name = workspace_clients
            .first()
            .map(|c| c.workspace.name.clone())
            .unwrap_or_else(|| format!("Workspace {}", workspace_id));

        let workspace_box = Box::new(Orientation::Vertical, 4);
        workspace_box.set_margin_start(12);

        let workspace_label = Label::new(Some(&format!("Workspace: {}", workspace_name)));
        workspace_label.add_css_class("dim-label");
        workspace_label.set_halign(gtk4::Align::Start);
        workspace_box.append(&workspace_label);

        for client in workspace_clients {
            let client_box = self.create_client_row(client);
            workspace_box.append(&client_box);
        }

        self.clients_box.append(&workspace_box);
    }

    fn create_client_row(&self, client: &Client) -> Box {
        let client_box = Box::new(Orientation::Horizontal, 8);
        client_box.set_margin_start(12);

        let class_label = Self::create_client_label(&client.class, "Class");
        client_box.append(&class_label);

        let title_label = Self::create_client_label(&client.title, "Title");
        client_box.append(&title_label);

        let kill_button = self.create_kill_button(&client.address);
        client_box.append(&kill_button);

        client_box
    }

    fn create_client_label(text: &str, label_type: &str) -> Label {
        let label = Label::new(Some(text));
        label.set_width_chars(30);
        label.set_xalign(0.0);
        label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        label.add_css_class("monospace");
        label.set_tooltip_text(Some(&format!("{}: {}", label_type, text)));
        label
    }

    pub fn create_kill_button(&self, address: &str) -> Button {
        let kill_button = Button::from_icon_name("window-close-symbolic");
        kill_button.add_css_class("KillProcess");
        kill_button.set_tooltip_text(Some("Close window"));
        let address = address.to_string();

        kill_button.connect_clicked(move |button| {
            use std::process::{Command, Stdio};
            let _ = Command::new("hyprctl")
                .args(["dispatch", "closewindow", &format!("address:0x{}", address)])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();

            if let Some(client_box) = button.parent() {
                if let Some(parent) = client_box.parent() {
                    if let Some(parent_box) = parent.downcast_ref::<Box>() {
                        parent_box.remove(&client_box);
                    }
                }
            }
        });

        kill_button
    }
}
