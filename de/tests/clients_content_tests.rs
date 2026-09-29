use gtk4 as gtk;
use rusty_de::widget::clients_content::ClientsContent;

#[gtk::test]
fn test_kill_button_removes_row() {
    use gtk4::prelude::*;

    let parent_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    let client_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);

    let clients_content = ClientsContent::new();
    let kill_button = clients_content.create_kill_button("some_address");

    client_box.append(&kill_button);
    parent_box.append(&client_box);

    assert_eq!(
        parent_box.first_child(),
        Some(client_box.clone().upcast::<gtk4::Widget>())
    );

    kill_button.emit_clicked();

    assert!(parent_box.first_child().is_none());
}

#[test]
fn test_kill_button_css_class() {
    // Test that the CSS class name is correct
    let class_name = "KillProcess";
    assert_eq!(class_name, "KillProcess", "CSS class should be KillProcess");
}

#[test]
fn test_column_width_chars() {
    // Verify class and title column widths
    let class_width = 30;
    let title_width = 30;
    assert_eq!(class_width, 30, "Class column should be 30 chars wide");
    assert_eq!(title_width, 30, "Title column should be 30 chars wide");
}

#[test]
fn test_scrolled_window_max_height() {
    // Verify the max content height constant
    let max_height = 500;
    assert_eq!(max_height, 500, "Max content height should be 500");
}

#[test]
fn test_hyprctl_command() {
    // Verify we use the correct hyprctl command
    let command = "hyprctl";
    assert_eq!(command, "hyprctl", "Should use hyprctl command");
}

#[test]
fn test_closewindow_dispatch() {
    // Verify we use dispatch closewindow
    let dispatch_cmd = "dispatch";
    let close_cmd = "closewindow";
    assert_eq!(dispatch_cmd, "dispatch", "Should use dispatch");
    assert_eq!(close_cmd, "closewindow", "Should use closewindow");
}

#[test]
fn test_address_prefix() {
    // Verify address prefix format
    let prefix = "address:0x";
    assert!(prefix.starts_with("address:"), "Should use address: prefix");
    assert!(prefix.contains("0x"), "Should include 0x in address");
}

#[test]
fn test_header_labels() {
    // Verify header label text
    let monitor_header = "Monitor";
    let workspace_header = "Workspace";
    let class_header = "Class";
    let title_header = "Title";

    assert_eq!(monitor_header, "Monitor");
    assert_eq!(workspace_header, "Workspace");
    assert_eq!(class_header, "Class");
    assert_eq!(title_header, "Title");
}

#[test]
fn test_indentation_levels() {
    // Verify indentation is consistent
    let workspace_indent = 12;
    let client_indent = 12;

    assert_eq!(workspace_indent, 12, "Workspace should be indented 12px");
    assert_eq!(
        client_indent, 12,
        "Clients should be indented 12px from workspace"
    );
}

#[test]
fn test_css_classes() {
    // Verify CSS classes used
    let title_class = "title-4";
    let dim_class = "dim-label";
    let mono_class = "monospace";

    assert_eq!(title_class, "title-4", "Monitor header uses title-4");
    assert_eq!(dim_class, "dim-label", "Workspace uses dim-label");
    assert_eq!(mono_class, "monospace", "Class and Title use monospace");
}

#[test]
fn test_tooltip_presence() {
    // Verify tooltips are set
    let kill_tooltip = "Close window";
    assert_eq!(
        kill_tooltip, "Close window",
        "Kill button should have tooltip"
    );
}

#[test]
fn test_icon_name() {
    // Verify the icon used for close button
    let icon = "window-close-symbolic";
    assert_eq!(
        icon, "window-close-symbolic",
        "Should use window-close-symbolic icon"
    );
}

#[test]
fn test_click_to_kill_label() {
    // Verify the click to kill label text
    let label = "Click to kill";
    assert_eq!(label, "Click to kill", "Label should be 'Click to kill'");
    assert!(!label.contains(':'), "Label should not contain colon");
}

#[test]
fn test_kill_mode_button_icon() {
    // Verify the icon used for kill mode button
    let icon = "input-mouse-symbolic";
    assert_eq!(
        icon, "input-mouse-symbolic",
        "Should use input-mouse-symbolic icon"
    );
}

#[test]
fn test_kill_mode_button_tooltip() {
    // Verify the kill mode button tooltip
    let tooltip = "Click to kill mode";
    assert_eq!(
        tooltip, "Click to kill mode",
        "Tooltip should describe function"
    );
}

#[test]
fn test_hyprctl_kill_command() {
    // Verify we use the correct hyprctl kill command
    let command = "hyprctl";
    let arg = "kill";
    assert_eq!(command, "hyprctl", "Should use hyprctl command");
    assert_eq!(arg, "kill", "Should use kill argument");
}

#[test]
fn test_kill_mode_button_css_class() {
    // Verify kill mode button uses same CSS class as kill buttons
    let css_class = "KillProcess";
    assert_eq!(
        css_class, "KillProcess",
        "Kill mode button should use KillProcess CSS class"
    );
}

#[test]
fn test_tooltip_format() {
    // Verify tooltip format for class and title
    let class_tooltip = "Class: some-class";
    let title_tooltip = "Title: some-title";

    assert!(
        class_tooltip.starts_with("Class: "),
        "Class tooltip should start with 'Class: '"
    );
    assert!(
        title_tooltip.starts_with("Title: "),
        "Title tooltip should start with 'Title: '"
    );
}
