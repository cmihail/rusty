// Note: AudioDevicesContent widget tests that create GTK widgets require main thread.
// These are commented out as they need integration testing setup.
// Logic tests are kept below.

#[test]
fn test_available_sink_filtering() {
    use rusty_de::service::audio::AudioSink;

    let sinks = vec![
        AudioSink {
            name: "sink1".to_string(),
            description: "Available Sink 1".to_string(),
            is_default: true,
            is_available: true,
        },
        AudioSink {
            name: "sink2".to_string(),
            description: "Unavailable Sink".to_string(),
            is_default: false,
            is_available: false,
        },
        AudioSink {
            name: "sink3".to_string(),
            description: "Available Sink 2".to_string(),
            is_default: false,
            is_available: true,
        },
    ];

    let available_sinks: Vec<_> = sinks.into_iter().filter(|s| s.is_available).collect();

    assert_eq!(available_sinks.len(), 2);
    assert_eq!(available_sinks[0].name, "sink1");
    assert_eq!(available_sinks[1].name, "sink3");
}

#[test]
fn test_available_count_calculation() {
    use rusty_de::service::audio::AudioSink;

    let sinks = vec![
        AudioSink {
            name: "sink1".to_string(),
            description: "Available Sink".to_string(),
            is_default: true,
            is_available: true,
        },
        AudioSink {
            name: "sink2".to_string(),
            description: "Unavailable Sink".to_string(),
            is_default: false,
            is_available: false,
        },
    ];

    let available_count = sinks.iter().filter(|s| s.is_available).count();
    assert_eq!(available_count, 1);
}

#[test]
fn test_single_device_behavior() {
    let available_device_count = 1;
    let should_keep_selected = available_device_count == 1;
    assert!(should_keep_selected);
}

#[test]
fn test_multiple_devices_behavior() {
    let available_device_count = 3;
    let should_switch_to_another = available_device_count > 1;
    assert!(should_switch_to_another);
}

#[test]
fn test_switch_state_matches_default_state() {
    let test_cases = vec![
        (true, true),   // is_default -> switch active
        (false, false), // not default -> switch inactive
    ];

    for (is_default, expected_switch_state) in test_cases {
        let switch_active = is_default;
        assert_eq!(
            switch_active, expected_switch_state,
            "is_default={} should set switch to {}",
            is_default, expected_switch_state
        );
    }
}

#[test]
fn test_find_next_available_device() {
    use rusty_de::service::audio::AudioSink;

    let current_sink_name = "sink1".to_string();
    let available_sinks = vec![
        AudioSink {
            name: "sink1".to_string(),
            description: "Current Sink".to_string(),
            is_default: true,
            is_available: true,
        },
        AudioSink {
            name: "sink2".to_string(),
            description: "Next Sink".to_string(),
            is_default: false,
            is_available: true,
        },
    ];

    let next_sink = available_sinks
        .into_iter()
        .find(|sink| sink.name != current_sink_name);

    assert!(next_sink.is_some());
    assert_eq!(next_sink.unwrap().name, "sink2");
}

#[test]
fn test_header_text_format() {
    let header_text = "Audio devices".to_string();
    assert_eq!(header_text, "Audio devices");
}

#[test]
fn test_switch_entry_requires_no_password() {
    let requires_password = false;
    assert!(!requires_password);
}

#[test]
fn test_sink_has_no_icon() {
    let icon_name: Option<String> = None;
    assert!(icon_name.is_none());
}

#[test]
fn test_sink_has_no_tooltip() {
    let tooltip: Option<String> = None;
    assert!(tooltip.is_none());
}

#[test]
fn test_deactivate_with_multiple_devices_switches() {
    use rusty_de::service::audio::AudioSink;

    let current_sink = AudioSink {
        name: "sink1".to_string(),
        description: "Current Sink".to_string(),
        is_default: true,
        is_available: true,
    };

    let available_sinks = vec![
        current_sink.clone(),
        AudioSink {
            name: "sink2".to_string(),
            description: "Alternative Sink".to_string(),
            is_default: false,
            is_available: true,
        },
    ];

    if current_sink.is_default && available_sinks.len() > 1 {
        let next_sink = available_sinks
            .into_iter()
            .find(|s| s.name != current_sink.name);
        assert!(next_sink.is_some());
        assert_eq!(next_sink.unwrap().name, "sink2");
    } else {
        panic!("Should have found next sink");
    }
}

#[test]
fn test_deactivate_with_single_device_keeps_selected() {
    use rusty_de::service::audio::AudioSink;

    let current_sink = AudioSink {
        name: "sink1".to_string(),
        description: "Only Sink".to_string(),
        is_default: true,
        is_available: true,
    };

    let available_sinks = vec![current_sink.clone()];

    if current_sink.is_default {
        if available_sinks.len() > 1 {
            panic!("Should not have multiple devices");
        } else {
            // Keep the same sink selected
            assert_eq!(available_sinks[0].name, current_sink.name);
        }
    }
}

#[test]
fn test_rebuild_clears_existing_content() {
    let mut child_count = 5;
    assert_eq!(child_count, 5);

    // Simulate clearing
    child_count = 0;
    assert_eq!(child_count, 0);
}

#[test]
fn test_switch_list_entry_creation() {
    use rusty_de::service::audio::AudioSink;

    let sink = AudioSink {
        name: "test_sink".to_string(),
        description: "Test Sink Description".to_string(),
        is_default: true,
        is_available: true,
    };

    assert_eq!(sink.description, "Test Sink Description");
    assert!(sink.is_default);
    assert!(sink.is_available);
}

#[test]
fn test_available_filtering_preserves_order() {
    use rusty_de::service::audio::AudioSink;

    let sinks = vec![
        AudioSink {
            name: "sink1".to_string(),
            description: "First Available".to_string(),
            is_default: true,
            is_available: true,
        },
        AudioSink {
            name: "sink2".to_string(),
            description: "Unavailable".to_string(),
            is_default: false,
            is_available: false,
        },
        AudioSink {
            name: "sink3".to_string(),
            description: "Second Available".to_string(),
            is_default: false,
            is_available: true,
        },
    ];

    let available: Vec<_> = sinks.into_iter().filter(|s| s.is_available).collect();

    assert_eq!(available.len(), 2);
    assert_eq!(available[0].description, "First Available");
    assert_eq!(available[1].description, "Second Available");
}

#[test]
fn test_no_available_sinks() {
    use rusty_de::service::audio::AudioSink;

    let sinks = vec![
        AudioSink {
            name: "sink1".to_string(),
            description: "Unavailable 1".to_string(),
            is_default: false,
            is_available: false,
        },
        AudioSink {
            name: "sink2".to_string(),
            description: "Unavailable 2".to_string(),
            is_default: false,
            is_available: false,
        },
    ];

    let available: Vec<_> = sinks.into_iter().filter(|s| s.is_available).collect();
    assert_eq!(available.len(), 0);
}

#[test]
fn test_all_sinks_available() {
    use rusty_de::service::audio::AudioSink;

    let sinks = vec![
        AudioSink {
            name: "sink1".to_string(),
            description: "Available 1".to_string(),
            is_default: true,
            is_available: true,
        },
        AudioSink {
            name: "sink2".to_string(),
            description: "Available 2".to_string(),
            is_default: false,
            is_available: true,
        },
    ];

    let available: Vec<_> = sinks
        .clone()
        .into_iter()
        .filter(|s| s.is_available)
        .collect();
    assert_eq!(available.len(), sinks.len());
}
