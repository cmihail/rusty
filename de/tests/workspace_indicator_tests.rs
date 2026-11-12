use rusty_de::window::workspace_indicator::indicator_text;

#[test]
fn test_indicator_text_shows_the_workspace_name() {
    assert_eq!(indicator_text("3,3").as_deref(), Some("3"));
    assert_eq!(indicator_text("12,code").as_deref(), Some("code"));
}

#[test]
fn test_indicator_text_skips_special_and_malformed_data() {
    assert_eq!(indicator_text("-99,special:scratch"), None);
    assert_eq!(indicator_text("3"), None);
    assert_eq!(indicator_text("3,"), None);
}
