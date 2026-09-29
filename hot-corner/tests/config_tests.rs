use rusty_hot_corner::config::{Config, ConfigData};

#[test]
fn test_config_parse_from_toml() {
    let toml_str = r#"
top_left_corner_command = "hyprctl dispatch hyprexpo:expo toggle"
"#;

    let config: ConfigData = toml::from_str(toml_str).unwrap();
    assert_eq!(
        config.top_left_corner_command,
        Some("hyprctl dispatch hyprexpo:expo toggle".to_string())
    );
    assert_eq!(config.top_left_corner_on_all_monitors, None);
}

#[test]
fn test_config_parse_custom_command() {
    let toml_str = r#"
top_left_corner_command = "notify-send 'Hot corner triggered!'"
"#;

    let config: ConfigData = toml::from_str(toml_str).unwrap();
    assert_eq!(
        config.top_left_corner_command,
        Some("notify-send 'Hot corner triggered!'".to_string())
    );
}

#[test]
fn test_config_parse_no_command() {
    let toml_str = r#"
# No command specified
"#;

    let config: ConfigData = toml::from_str(toml_str).unwrap();
    assert_eq!(config.top_left_corner_command, None);
}

#[test]
fn test_config_path() {
    let path = Config::config_path();
    assert!(path
        .to_string_lossy()
        .ends_with(".config/rusty/hot-corner.toml"));
}

#[test]
fn test_config_singleton() {
    // Test that multiple calls return the same instance
    let config1 = Config::instance();
    let config2 = Config::instance();
    assert_eq!(config1.top_left_corner_command(), config2.top_left_corner_command());
}

#[test]
fn test_config_path_parent_exists() {
    let path = Config::config_path();
    let parent = path.parent();
    assert!(parent.is_some());
    assert!(parent.unwrap().to_string_lossy().ends_with(".config/rusty"));
}

#[test]
fn test_config_shell_command_format() {
    let toml_str = r#"
top_left_corner_command = "sh -c 'notify-send test'"
"#;
    let config: ConfigData = toml::from_str(toml_str).unwrap();
    assert_eq!(
        config.top_left_corner_command,
        Some("sh -c 'notify-send test'".to_string())
    );
}

#[test]
fn test_config_complex_command() {
    let toml_str = r#"
top_left_corner_command = "bash -c 'echo hello | grep h'"
"#;
    let config: ConfigData = toml::from_str(toml_str).unwrap();
    assert_eq!(
        config.top_left_corner_command,
        Some("bash -c 'echo hello | grep h'".to_string())
    );
}

#[test]
fn test_config_top_left_corner_on_all_monitors_true() {
    let toml_str = r#"
top_left_corner_command = "hyprctl dispatch workspace-overview"
top_left_corner_on_all_monitors = true
"#;
    let config: ConfigData = toml::from_str(toml_str).unwrap();
    assert_eq!(config.top_left_corner_on_all_monitors, Some(true));
}

#[test]
fn test_config_top_left_corner_on_all_monitors_false() {
    let toml_str = r#"
top_left_corner_command = "hyprctl dispatch workspace-overview"
top_left_corner_on_all_monitors = false
"#;
    let config: ConfigData = toml::from_str(toml_str).unwrap();
    assert_eq!(config.top_left_corner_on_all_monitors, Some(false));
}

#[test]
fn test_config_top_right_corner_command() {
    let toml_str = r#"
top_right_corner_command = "notify-send 'Top right corner!'"
"#;
    let config: ConfigData = toml::from_str(toml_str).unwrap();
    assert_eq!(
        config.top_right_corner_command,
        Some("notify-send 'Top right corner!'".to_string())
    );
}

#[test]
fn test_config_top_right_corner_on_all_monitors_true() {
    let toml_str = r#"
top_right_corner_command = "hyprctl dispatch workspace-overview"
top_right_corner_on_all_monitors = true
"#;
    let config: ConfigData = toml::from_str(toml_str).unwrap();
    assert_eq!(config.top_right_corner_on_all_monitors, Some(true));
}

#[test]
fn test_config_top_right_corner_on_all_monitors_false() {
    let toml_str = r#"
top_right_corner_command = "hyprctl dispatch workspace-overview"
top_right_corner_on_all_monitors = false
"#;
    let config: ConfigData = toml::from_str(toml_str).unwrap();
    assert_eq!(config.top_right_corner_on_all_monitors, Some(false));
}

#[test]
fn test_config_both_corners() {
    let toml_str = r#"
top_left_corner_command = "hyprctl dispatch workspace-overview"
top_left_corner_on_all_monitors = true
top_right_corner_command = "notify-send 'Right corner!'"
top_right_corner_on_all_monitors = false
"#;
    let config: ConfigData = toml::from_str(toml_str).unwrap();
    assert_eq!(
        config.top_left_corner_command,
        Some("hyprctl dispatch workspace-overview".to_string())
    );
    assert_eq!(config.top_left_corner_on_all_monitors, Some(true));
    assert_eq!(
        config.top_right_corner_command,
        Some("notify-send 'Right corner!'".to_string())
    );
    assert_eq!(config.top_right_corner_on_all_monitors, Some(false));
}

#[test]
fn test_config_trigger_delay_ms() {
    let toml_str = r#"
trigger_delay_ms = 500
"#;
    let config: ConfigData = toml::from_str(toml_str).unwrap();
    assert_eq!(config.trigger_delay_ms, Some(500));
}

#[test]
fn test_config_trigger_delay_ms_default() {
    let toml_str = r#"
top_left_corner_command = "hyprctl dispatch workspace-overview"
"#;
    let config: ConfigData = toml::from_str(toml_str).unwrap();
    assert_eq!(config.trigger_delay_ms, None);
}

#[test]
fn test_config_trigger_delay_ms_zero() {
    let toml_str = r#"
trigger_delay_ms = 0
"#;
    let config: ConfigData = toml::from_str(toml_str).unwrap();
    assert_eq!(config.trigger_delay_ms, Some(0));
}

#[test]
fn test_config_trigger_delay_ms_large_value() {
    let toml_str = r#"
trigger_delay_ms = 5000
"#;
    let config: ConfigData = toml::from_str(toml_str).unwrap();
    assert_eq!(config.trigger_delay_ms, Some(5000));
}

#[test]
fn test_config_with_all_fields() {
    let toml_str = r#"
top_left_corner_command = "hyprctl dispatch workspace-overview"
top_left_corner_on_all_monitors = true
top_right_corner_command = "notify-send 'Right corner!'"
top_right_corner_on_all_monitors = false
trigger_delay_ms = 750
"#;
    let config: ConfigData = toml::from_str(toml_str).unwrap();
    assert_eq!(
        config.top_left_corner_command,
        Some("hyprctl dispatch workspace-overview".to_string())
    );
    assert_eq!(config.top_left_corner_on_all_monitors, Some(true));
    assert_eq!(
        config.top_right_corner_command,
        Some("notify-send 'Right corner!'".to_string())
    );
    assert_eq!(config.top_right_corner_on_all_monitors, Some(false));
    assert_eq!(config.trigger_delay_ms, Some(750));
}

#[test]
fn test_config_bottom_left_corner_command() {
    let toml_str = r#"
bottom_left_corner_command = "notify-send 'Bottom left corner!'"
"#;
    let config: ConfigData = toml::from_str(toml_str).unwrap();
    assert_eq!(
        config.bottom_left_corner_command,
        Some("notify-send 'Bottom left corner!'".to_string())
    );
}

#[test]
fn test_config_bottom_right_corner_command() {
    let toml_str = r#"
bottom_right_corner_command = "notify-send 'Bottom right corner!'"
"#;
    let config: ConfigData = toml::from_str(toml_str).unwrap();
    assert_eq!(
        config.bottom_right_corner_command,
        Some("notify-send 'Bottom right corner!'".to_string())
    );
}

#[test]
fn test_config_bottom_left_corner_on_all_monitors_true() {
    let toml_str = r#"
bottom_left_corner_command = "hyprctl dispatch workspace-overview"
bottom_left_corner_on_all_monitors = true
"#;
    let config: ConfigData = toml::from_str(toml_str).unwrap();
    assert_eq!(config.bottom_left_corner_on_all_monitors, Some(true));
}

#[test]
fn test_config_bottom_left_corner_on_all_monitors_false() {
    let toml_str = r#"
bottom_left_corner_command = "hyprctl dispatch workspace-overview"
bottom_left_corner_on_all_monitors = false
"#;
    let config: ConfigData = toml::from_str(toml_str).unwrap();
    assert_eq!(config.bottom_left_corner_on_all_monitors, Some(false));
}

#[test]
fn test_config_bottom_right_corner_on_all_monitors_true() {
    let toml_str = r#"
bottom_right_corner_command = "hyprctl dispatch workspace-overview"
bottom_right_corner_on_all_monitors = true
"#;
    let config: ConfigData = toml::from_str(toml_str).unwrap();
    assert_eq!(config.bottom_right_corner_on_all_monitors, Some(true));
}

#[test]
fn test_config_bottom_right_corner_on_all_monitors_false() {
    let toml_str = r#"
bottom_right_corner_command = "hyprctl dispatch workspace-overview"
bottom_right_corner_on_all_monitors = false
"#;
    let config: ConfigData = toml::from_str(toml_str).unwrap();
    assert_eq!(config.bottom_right_corner_on_all_monitors, Some(false));
}

#[test]
fn test_config_all_four_corners() {
    let toml_str = r#"
top_left_corner_command = "hyprctl dispatch workspace-overview"
top_left_corner_on_all_monitors = true
top_right_corner_command = "notify-send 'Top right corner!'"
top_right_corner_on_all_monitors = false
bottom_left_corner_command = "notify-send 'Bottom left corner!'"
bottom_left_corner_on_all_monitors = false
bottom_right_corner_command = "notify-send 'Bottom right corner!'"
bottom_right_corner_on_all_monitors = true
"#;
    let config: ConfigData = toml::from_str(toml_str).unwrap();
    assert_eq!(
        config.top_left_corner_command,
        Some("hyprctl dispatch workspace-overview".to_string())
    );
    assert_eq!(config.top_left_corner_on_all_monitors, Some(true));
    assert_eq!(
        config.top_right_corner_command,
        Some("notify-send 'Top right corner!'".to_string())
    );
    assert_eq!(config.top_right_corner_on_all_monitors, Some(false));
    assert_eq!(
        config.bottom_left_corner_command,
        Some("notify-send 'Bottom left corner!'".to_string())
    );
    assert_eq!(config.bottom_left_corner_on_all_monitors, Some(false));
    assert_eq!(
        config.bottom_right_corner_command,
        Some("notify-send 'Bottom right corner!'".to_string())
    );
    assert_eq!(config.bottom_right_corner_on_all_monitors, Some(true));
}

#[test]
fn test_config_with_all_fields_including_bottom_corners() {
    let toml_str = r#"
top_left_corner_command = "hyprctl dispatch workspace-overview"
top_left_corner_on_all_monitors = true
top_right_corner_command = "notify-send 'Top right corner!'"
top_right_corner_on_all_monitors = false
bottom_left_corner_command = "notify-send 'Bottom left corner!'"
bottom_left_corner_on_all_monitors = false
bottom_right_corner_command = "notify-send 'Bottom right corner!'"
bottom_right_corner_on_all_monitors = true
trigger_delay_ms = 750
"#;
    let config: ConfigData = toml::from_str(toml_str).unwrap();
    assert_eq!(
        config.top_left_corner_command,
        Some("hyprctl dispatch workspace-overview".to_string())
    );
    assert_eq!(config.top_left_corner_on_all_monitors, Some(true));
    assert_eq!(
        config.top_right_corner_command,
        Some("notify-send 'Top right corner!'".to_string())
    );
    assert_eq!(config.top_right_corner_on_all_monitors, Some(false));
    assert_eq!(
        config.bottom_left_corner_command,
        Some("notify-send 'Bottom left corner!'".to_string())
    );
    assert_eq!(config.bottom_left_corner_on_all_monitors, Some(false));
    assert_eq!(
        config.bottom_right_corner_command,
        Some("notify-send 'Bottom right corner!'".to_string())
    );
    assert_eq!(config.bottom_right_corner_on_all_monitors, Some(true));
    assert_eq!(config.trigger_delay_ms, Some(750));
}

#[test]
fn test_config_only_bottom_corners() {
    let toml_str = r#"
bottom_left_corner_command = "notify-send 'Bottom left!'"
bottom_right_corner_command = "notify-send 'Bottom right!'"
"#;
    let config: ConfigData = toml::from_str(toml_str).unwrap();
    assert_eq!(config.top_left_corner_command, None);
    assert_eq!(config.top_right_corner_command, None);
    assert_eq!(
        config.bottom_left_corner_command,
        Some("notify-send 'Bottom left!'".to_string())
    );
    assert_eq!(
        config.bottom_right_corner_command,
        Some("notify-send 'Bottom right!'".to_string())
    );
}
