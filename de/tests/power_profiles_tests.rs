#[test]
fn test_icon_name_format() {
    assert_eq!(
        format!("power-profile-{}-symbolic", "balanced"),
        "power-profile-balanced-symbolic"
    );
    assert_eq!(
        format!("power-profile-{}-symbolic", "performance"),
        "power-profile-performance-symbolic"
    );
    assert_eq!(
        format!("power-profile-{}-symbolic", "power-saver"),
        "power-profile-power-saver-symbolic"
    );
}
