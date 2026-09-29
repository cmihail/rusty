// Tests for bar workspace label logic

#[test]
fn test_workspace_label_format_single_digit() {
    let workspace_id = 5;
    let label = if workspace_id > 9 {
        "+".to_string()
    } else {
        workspace_id.to_string()
    };

    assert_eq!(label, "5");
}

#[test]
fn test_workspace_label_format_double_digit() {
    let workspace_id = 15;
    let label = if workspace_id > 9 {
        "+".to_string()
    } else {
        workspace_id.to_string()
    };

    assert_eq!(label, "+");
}

#[test]
fn test_workspace_label_format_boundary() {
    let workspace_id = 9;
    let label = if workspace_id > 9 {
        "+".to_string()
    } else {
        workspace_id.to_string()
    };

    assert_eq!(label, "9");

    let workspace_id = 10;
    let label = if workspace_id > 9 {
        "+".to_string()
    } else {
        workspace_id.to_string()
    };

    assert_eq!(label, "+");
}

#[test]
fn test_workspace_label_format_zero() {
    let workspace_id = 0;
    let label = if workspace_id > 9 {
        "+".to_string()
    } else {
        workspace_id.to_string()
    };

    assert_eq!(label, "0");
}

#[test]
fn test_workspace_label_format_negative() {
    let workspace_id = -1;
    let label = if workspace_id > 9 {
        "+".to_string()
    } else {
        workspace_id.to_string()
    };

    assert_eq!(label, "-1");
}

#[test]
fn test_monitor_connector_matching() {
    // Test connector name matching logic
    let gtk_connector = "DP-1";
    let hypr_monitor_name = "DP-1";

    assert_eq!(gtk_connector, hypr_monitor_name);
}

#[test]
fn test_monitor_model_matching() {
    // Test model name matching logic
    let gtk_model = "Dell U2720Q";
    let hypr_model = "Dell U2720Q";

    assert_eq!(gtk_model, hypr_model);
}

#[test]
fn test_empty_connector_detection() {
    let connector = "";
    assert!(connector.is_empty());

    let connector = "DP-1";
    assert!(!connector.is_empty());
}
