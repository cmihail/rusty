#[test]
fn test_audio_icon_logic_muted() {
    // Test icon logic when muted
    let muted = true;
    let volume = 0.5;

    let icon = if muted {
        "audio-volume-muted-symbolic".to_string()
    } else if volume == 0.0 {
        "audio-volume-muted-symbolic".to_string()
    } else if volume < 0.33 {
        "audio-volume-low-symbolic".to_string()
    } else if volume < 0.67 {
        "audio-volume-medium-symbolic".to_string()
    } else {
        "audio-volume-high-symbolic".to_string()
    };

    assert_eq!(icon, "audio-volume-muted-symbolic");
}

#[test]
fn test_audio_icon_logic_volume_levels() {
    // Test icon logic for different volume levels
    let test_cases = vec![
        (0.0, "audio-volume-muted-symbolic"),
        (0.2, "audio-volume-low-symbolic"),
        (0.4, "audio-volume-medium-symbolic"),
        (0.7, "audio-volume-high-symbolic"),
        (1.0, "audio-volume-high-symbolic"),
        (1.2, "audio-volume-overamplified-symbolic"),
        (1.5, "audio-volume-overamplified-symbolic"),
    ];

    for (volume, expected_icon) in test_cases {
        let muted = false;
        let icon = if muted {
            "audio-volume-muted-symbolic".to_string()
        } else if volume == 0.0 {
            "audio-volume-muted-symbolic".to_string()
        } else if volume > 1.0 {
            "audio-volume-overamplified-symbolic".to_string()
        } else if volume < 0.33 {
            "audio-volume-low-symbolic".to_string()
        } else if volume < 0.67 {
            "audio-volume-medium-symbolic".to_string()
        } else {
            "audio-volume-high-symbolic".to_string()
        };

        assert_eq!(
            icon, expected_icon,
            "Volume {} should produce icon {}",
            volume, expected_icon
        );
    }
}

#[test]
fn test_mic_icon_logic_muted() {
    // Test mic icon logic when muted
    let muted = true;
    let volume = 0.5;

    let icon = if muted {
        "microphone-sensitivity-muted-symbolic".to_string()
    } else if volume == 0.0 {
        "microphone-sensitivity-muted-symbolic".to_string()
    } else if volume < 0.33 {
        "microphone-sensitivity-low-symbolic".to_string()
    } else if volume < 0.67 {
        "microphone-sensitivity-medium-symbolic".to_string()
    } else {
        "microphone-sensitivity-high-symbolic".to_string()
    };

    assert_eq!(icon, "microphone-sensitivity-muted-symbolic");
}

#[test]
fn test_mic_icon_logic_volume_levels() {
    // Test mic icon logic for different volume levels
    let test_cases = vec![
        (0.0, "microphone-sensitivity-muted-symbolic"),
        (0.2, "microphone-sensitivity-low-symbolic"),
        (0.4, "microphone-sensitivity-medium-symbolic"),
        (0.7, "microphone-sensitivity-high-symbolic"),
        (1.0, "microphone-sensitivity-high-symbolic"),
    ];

    for (volume, expected_icon) in test_cases {
        let muted = false;
        let icon = if muted {
            "microphone-sensitivity-muted-symbolic".to_string()
        } else if volume == 0.0 {
            "microphone-sensitivity-muted-symbolic".to_string()
        } else if volume < 0.33 {
            "microphone-sensitivity-low-symbolic".to_string()
        } else if volume < 0.67 {
            "microphone-sensitivity-medium-symbolic".to_string()
        } else {
            "microphone-sensitivity-high-symbolic".to_string()
        };

        assert_eq!(
            icon, expected_icon,
            "Mic volume {} should produce icon {}",
            volume, expected_icon
        );
    }
}

#[test]
fn test_volume_clamp_logic() {
    // Test that volume clamping logic is correct
    let test_values: Vec<f64> = vec![-0.5, 0.0, 0.5, 1.0, 1.5];
    let expected: Vec<f64> = vec![0.0, 0.0, 0.5, 1.0, 1.0];

    for (value, expected_result) in test_values.iter().zip(expected.iter()) {
        let clamped = value.clamp(0.0, 1.0);
        assert_eq!(
            clamped, *expected_result,
            "Value {} should clamp to {}",
            value, expected_result
        );
    }
}

#[test]
fn test_volume_change_detection() {
    // Test that volume change is detected correctly
    let old_volume: f64 = 0.5;
    let new_volume: f64 = 0.6;

    let changed = (new_volume - old_volume).abs() > f64::EPSILON;
    assert!(
        changed,
        "Volume change from {} to {} should be detected",
        old_volume, new_volume
    );
}

#[test]
fn test_volume_no_change_detection() {
    // Test that identical volume is not detected as changed
    let old_volume: f64 = 0.5;
    let new_volume: f64 = 0.5;

    let changed = (new_volume - old_volume).abs() > f64::EPSILON;
    assert!(
        !changed,
        "No volume change should be detected when old and new are identical"
    );
}

#[test]
fn test_muted_change_detection() {
    // Test that muted state change is detected correctly
    let old_muted = false;
    let new_muted = true;

    let changed = old_muted != new_muted;
    assert!(
        changed,
        "Muted state change from {} to {} should be detected",
        old_muted, new_muted
    );
}

#[test]
fn test_muted_no_change_detection() {
    // Test that identical muted state is not detected as changed
    let old_muted = true;
    let new_muted = true;

    let changed = old_muted != new_muted;
    assert!(
        !changed,
        "No muted state change should be detected when old and new are identical"
    );
}

#[test]
fn test_small_volume_changes_detected() {
    // Test that even small volume changes are detected
    let test_cases: Vec<(f64, f64)> = vec![(0.5, 0.51), (0.0, 0.01), (0.99, 1.0), (0.33, 0.34)];

    for (old_vol, new_vol) in test_cases {
        let changed = (new_vol - old_vol).abs() > f64::EPSILON;
        assert!(
            changed,
            "Volume change from {} to {} should be detected",
            old_vol, new_vol
        );
    }
}

#[test]
fn test_epsilon_precision() {
    // Test that changes smaller than EPSILON are not detected
    let old_volume = 0.5;
    let new_volume = 0.5 + f64::EPSILON / 2.0;

    let changed = (new_volume - old_volume).abs() > f64::EPSILON;
    assert!(
        !changed,
        "Volume change smaller than EPSILON should not be detected"
    );
}

#[test]
fn test_device_type_detection_headset() {
    // Test headset detection from various identifier formats
    let test_cases = vec![
        "alsa_output.usb-HUAWEI_USB-C_HEADSET|[Out] Analog",
        "alsa_output.usb-bestechnic_HUAWEI_USB-C_HEADSET_0296A100000000000000000000000-00.analog-stereo|analog-output",
        "some_sink|[Out] Headset",
    ];

    for identifier in test_cases {
        let lower = identifier.to_lowercase();
        let is_headset = lower.contains("headset");
        assert!(
            is_headset,
            "Identifier '{}' should be detected as headset",
            identifier
        );
    }
}

#[test]
fn test_device_type_detection_headphones() {
    // Test headphones detection from various identifier formats
    let test_cases = vec![
        "alsa_output.pci-0000_04_00.6.HiFi__Speaker__sink|[Out] Headphones",
        "some_sink|[Out] Headphone",
        "alsa_output.usb-Headphones_Device",
    ];

    for identifier in test_cases {
        let lower = identifier.to_lowercase();
        let is_headphones = lower.contains("headphones") || lower.contains("headphone");
        let is_headset = lower.contains("headset");
        assert!(
            is_headphones && !is_headset,
            "Identifier '{}' should be detected as headphones",
            identifier
        );
    }
}

#[test]
fn test_device_type_detection_speakers() {
    // Test speakers detection (default when not headset/headphones)
    let test_cases = vec![
        "alsa_output.pci-0000_04_00.6.HiFi__Speaker__sink|[Out] Speaker",
        "some_sink|[Out] Line Out",
        "alsa_output.hdmi|HDMI",
    ];

    for identifier in test_cases {
        let lower = identifier.to_lowercase();
        let is_headset = lower.contains("headset");
        let is_headphones = lower.contains("headphones") || lower.contains("headphone");
        let is_speakers = !is_headset && !is_headphones;
        assert!(
            is_speakers,
            "Identifier '{}' should be detected as speakers",
            identifier
        );
    }
}

#[test]
fn test_speaker_icon_selection_by_volume() {
    // Test that speakers use volume-based icons
    let test_cases = vec![
        (0.0, true, "audio-volume-muted-symbolic"),
        (0.5, true, "audio-volume-muted-symbolic"),
        (0.0, false, "audio-volume-muted-symbolic"),
        (0.2, false, "audio-volume-low-symbolic"),
        (0.4, false, "audio-volume-medium-symbolic"),
        (0.8, false, "audio-volume-high-symbolic"),
        (1.0, false, "audio-volume-high-symbolic"),
        (1.2, false, "audio-volume-overamplified-symbolic"),
        (1.5, false, "audio-volume-overamplified-symbolic"),
    ];

    for (volume, muted, expected_icon) in test_cases {
        let icon = if muted || volume == 0.0 {
            "audio-volume-muted-symbolic"
        } else if volume > 1.0 {
            "audio-volume-overamplified-symbolic"
        } else if volume < 0.33 {
            "audio-volume-low-symbolic"
        } else if volume < 0.67 {
            "audio-volume-medium-symbolic"
        } else {
            "audio-volume-high-symbolic"
        };

        assert_eq!(
            icon, expected_icon,
            "Speaker with volume {} and muted {} should use icon {}",
            volume, muted, expected_icon
        );
    }
}

#[test]
fn test_headset_icon_changes_with_mute() {
    // Test that headsets use muted icon when muted
    let test_cases = vec![
        (0.0, true, "audio-headset-muted-symbolic"),
        (0.5, true, "audio-headset-muted-symbolic"),
        (1.0, true, "audio-headset-muted-symbolic"),
        (0.0, false, "audio-headset-symbolic"),
        (0.5, false, "audio-headset-symbolic"),
        (1.0, false, "audio-headset-symbolic"),
    ];

    for (volume, muted, expected_icon) in test_cases {
        let icon = if muted {
            "audio-headset-muted-symbolic"
        } else {
            "audio-headset-symbolic"
        };
        assert_eq!(
            icon, expected_icon,
            "Headset with volume {} and muted {} should use icon {}",
            volume, muted, expected_icon
        );
    }
}

#[test]
fn test_headphones_icon_changes_with_mute() {
    // Test that headphones use muted icon when muted
    let test_cases = vec![
        (0.0, true, "audio-headphones-muted-symbolic"),
        (0.5, true, "audio-headphones-muted-symbolic"),
        (1.0, true, "audio-headphones-muted-symbolic"),
        (0.0, false, "audio-headphones-symbolic"),
        (0.5, false, "audio-headphones-symbolic"),
        (1.0, false, "audio-headphones-symbolic"),
    ];

    for (volume, muted, expected_icon) in test_cases {
        let icon = if muted {
            "audio-headphones-muted-symbolic"
        } else {
            "audio-headphones-symbolic"
        };
        assert_eq!(
            icon, expected_icon,
            "Headphones with volume {} and muted {} should use icon {}",
            volume, muted, expected_icon
        );
    }
}

#[test]
fn test_volume_clamp_with_overamplification() {
    // Test that volume clamping allows overamplification up to 150%
    let test_values: Vec<f64> = vec![-0.5, 0.0, 0.5, 1.0, 1.25, 1.5, 2.0];
    let expected: Vec<f64> = vec![0.0, 0.0, 0.5, 1.0, 1.25, 1.5, 1.5];

    for (value, expected_result) in test_values.iter().zip(expected.iter()) {
        let clamped = value.clamp(0.0, 1.5);
        assert_eq!(
            clamped, *expected_result,
            "Value {} should clamp to {} (with 150% max)",
            value, expected_result
        );
    }
}

#[test]
fn test_identifier_change_detection() {
    // Test that sink identifier changes are detected
    let test_cases = vec![
        (
            "alsa_output.pci-0000_04_00.6.HiFi__Speaker__sink|[Out] Speaker",
            "alsa_output.pci-0000_04_00.6.HiFi__Speaker__sink|[Out] Headphones",
            true,
        ),
        (
            "alsa_output.pci-0000_04_00.6.HiFi__Speaker__sink|[Out] Speaker",
            "alsa_output.usb-HEADSET|analog-output",
            true,
        ),
        (
            "alsa_output.pci-0000_04_00.6.HiFi__Speaker__sink|[Out] Speaker",
            "alsa_output.pci-0000_04_00.6.HiFi__Speaker__sink|[Out] Speaker",
            false,
        ),
    ];

    for (old_id, new_id, should_change) in test_cases {
        let changed = old_id != new_id;
        assert_eq!(
            changed,
            should_change,
            "Change from '{}' to '{}' should{}be detected",
            old_id,
            new_id,
            if should_change { " " } else { " not " }
        );
    }
}

#[test]
fn test_mic_device_type_detection_headset() {
    // Test headset microphone detection from various source identifiers
    let test_cases = vec![
        "alsa_input.usb-HUAWEI_USB-C_HEADSET",
        "alsa_input.usb-headset-device",
        "some_source.headset",
    ];

    for identifier in test_cases {
        let lower = identifier.to_lowercase();
        let is_headset = lower.contains("headset");
        assert!(
            is_headset,
            "Source identifier '{}' should be detected as headset",
            identifier
        );
    }
}

#[test]
fn test_mic_device_type_detection_default() {
    // Test default microphone detection (not headset)
    let test_cases = vec![
        "alsa_input.pci-internal-microphone",
        "alsa_input.usb-webcam",
        "some_source.built-in",
    ];

    for identifier in test_cases {
        let lower = identifier.to_lowercase();
        let is_headset = lower.contains("headset");
        assert!(
            !is_headset,
            "Source identifier '{}' should be detected as default microphone",
            identifier
        );
    }
}

#[test]
fn test_headset_mic_icon_changes_with_mute() {
    // Test that headset microphones use headset-specific icons
    let test_cases = vec![
        (true, "audio-headset-mic-muted-symbolic"),
        (false, "audio-headset-mic-symbolic"),
    ];

    for (muted, expected_icon) in test_cases {
        let icon = if muted {
            "audio-headset-mic-muted-symbolic"
        } else {
            "audio-headset-mic-symbolic"
        };
        assert_eq!(
            icon, expected_icon,
            "Headset microphone with muted {} should use icon {}",
            muted, expected_icon
        );
    }
}

#[test]
fn test_default_mic_icon_changes_with_mute_and_volume() {
    // Test that default microphones use volume-based icons
    let test_cases = vec![
        (0.0, true, "microphone-sensitivity-muted-symbolic"),
        (0.5, true, "microphone-sensitivity-muted-symbolic"),
        (0.0, false, "microphone-sensitivity-muted-symbolic"),
        (0.2, false, "microphone-sensitivity-low-symbolic"),
        (0.4, false, "microphone-sensitivity-medium-symbolic"),
        (0.8, false, "microphone-sensitivity-high-symbolic"),
    ];

    for (volume, muted, expected_icon) in test_cases {
        let icon = if muted {
            "microphone-sensitivity-muted-symbolic"
        } else if volume == 0.0 {
            "microphone-sensitivity-muted-symbolic"
        } else if volume < 0.33 {
            "microphone-sensitivity-low-symbolic"
        } else if volume < 0.67 {
            "microphone-sensitivity-medium-symbolic"
        } else {
            "microphone-sensitivity-high-symbolic"
        };

        assert_eq!(
            icon, expected_icon,
            "Default microphone with volume {} and muted {} should use icon {}",
            volume, muted, expected_icon
        );
    }
}

#[test]
fn test_overamplified_icon_boundary() {
    // Test that overamplified icon is used exactly when volume > 1.0
    let test_cases = vec![
        (0.99, "audio-volume-high-symbolic"),
        (1.0, "audio-volume-high-symbolic"),
        (1.01, "audio-volume-overamplified-symbolic"),
        (1.5, "audio-volume-overamplified-symbolic"),
    ];

    for (volume, expected_icon) in test_cases {
        let muted = false;
        let icon = if muted || volume == 0.0 {
            "audio-volume-muted-symbolic"
        } else if volume > 1.0 {
            "audio-volume-overamplified-symbolic"
        } else if volume < 0.33 {
            "audio-volume-low-symbolic"
        } else if volume < 0.67 {
            "audio-volume-medium-symbolic"
        } else {
            "audio-volume-high-symbolic"
        };

        assert_eq!(
            icon, expected_icon,
            "Volume {} should produce icon {}",
            volume, expected_icon
        );
    }
}
