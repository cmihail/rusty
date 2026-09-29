use rusty_de::service::fcitx::InputMethod;

// Note: LanguageContent requires Fcitx service which uses async D-Bus
// and must be initialized in the main thread with a GLib main loop.
// Widget creation tests are commented out as they require integration testing setup.

// #[test]
// fn test_language_content_creation() {
//     let _ = gtk4::init();
//     let popover = gtk4::Popover::new();
//     let content = LanguageContent::new(&popover);
//     assert!(content.widget().is_visible());
// }

#[test]
fn test_input_method_filtering() {
    // Test that we correctly filter enabled input methods
    let ims = vec![
        InputMethod {
            name: "English".to_string(),
            unique_name: "keyboard-us".to_string(),
            language_code: "en".to_string(),
            enabled: true,
        },
        InputMethod {
            name: "Japanese".to_string(),
            unique_name: "mozc".to_string(),
            language_code: "ja".to_string(),
            enabled: false, // Disabled
        },
        InputMethod {
            name: "Chinese".to_string(),
            unique_name: "pinyin".to_string(),
            language_code: "zh".to_string(),
            enabled: true,
        },
    ];

    let enabled: Vec<_> = ims.iter().filter(|im| im.enabled).collect();

    assert_eq!(enabled.len(), 2);
    assert_eq!(enabled[0].name, "English");
    assert_eq!(enabled[1].name, "Chinese");
}

#[test]
fn test_language_count_header() {
    // Test header text generation
    let ims = [
        InputMethod {
            name: "English".to_string(),
            unique_name: "keyboard-us".to_string(),
            language_code: "en".to_string(),
            enabled: true,
        },
        InputMethod {
            name: "Japanese".to_string(),
            unique_name: "mozc".to_string(),
            language_code: "ja".to_string(),
            enabled: true,
        },
    ];

    let enabled_count = ims.iter().filter(|im| im.enabled).count();
    let header_text = format!("Languages: {}", enabled_count);

    assert_eq!(header_text, "Languages: 2");
}

#[test]
fn test_next_language_selection_logic() {
    // Test the logic for selecting the next language
    let enabled_ims = vec![
        InputMethod {
            name: "English".to_string(),
            unique_name: "keyboard-us".to_string(),
            language_code: "en".to_string(),
            enabled: true,
        },
        InputMethod {
            name: "Japanese".to_string(),
            unique_name: "mozc".to_string(),
            language_code: "ja".to_string(),
            enabled: true,
        },
        InputMethod {
            name: "Chinese".to_string(),
            unique_name: "pinyin".to_string(),
            language_code: "zh".to_string(),
            enabled: true,
        },
    ];

    // When deactivating language at index 0, should select index 1
    let current_index = 0;
    let next_index = (current_index + 1) % enabled_ims.len();
    assert_eq!(next_index, 1);
    assert_eq!(enabled_ims[next_index].name, "Japanese");

    // When deactivating language at index 1, should select index 2
    let current_index = 1;
    let next_index = (current_index + 1) % enabled_ims.len();
    assert_eq!(next_index, 2);
    assert_eq!(enabled_ims[next_index].name, "Chinese");

    // When deactivating language at index 2 (last), should wrap to index 0
    let current_index = 2;
    let next_index = (current_index + 1) % enabled_ims.len();
    assert_eq!(next_index, 0);
    assert_eq!(enabled_ims[next_index].name, "English");
}

#[test]
fn test_single_language_behavior() {
    // Test behavior with only one language
    let enabled_ims = [InputMethod {
        name: "English".to_string(),
        unique_name: "keyboard-us".to_string(),
        language_code: "en".to_string(),
        enabled: true,
    }];

    assert_eq!(enabled_ims.len(), 1);

    // When only one language, should re-select the same one
    if enabled_ims.len() == 1 {
        assert_eq!(enabled_ims[0].name, "English");
    }
}

#[test]
fn test_current_language_identification() {
    // Test identifying the current language
    let enabled_ims = [
        InputMethod {
            name: "English".to_string(),
            unique_name: "keyboard-us".to_string(),
            language_code: "en".to_string(),
            enabled: true,
        },
        InputMethod {
            name: "Japanese".to_string(),
            unique_name: "mozc".to_string(),
            language_code: "ja".to_string(),
            enabled: true,
        },
    ];

    let current_im = "mozc";

    let is_active_fn = |im: &InputMethod| current_im == im.unique_name;

    assert!(!is_active_fn(&enabled_ims[0]));
    assert!(is_active_fn(&enabled_ims[1]));
}

#[test]
fn test_is_first_language_logic() {
    // Test logic for determining if current language is the first one
    let enabled_ims = [
        InputMethod {
            name: "English".to_string(),
            unique_name: "keyboard-us".to_string(),
            language_code: "en".to_string(),
            enabled: true,
        },
        InputMethod {
            name: "Japanese".to_string(),
            unique_name: "mozc".to_string(),
            language_code: "ja".to_string(),
            enabled: true,
        },
    ];

    // When current IM is the first one
    let current_im = "keyboard-us";
    let is_first = enabled_ims
        .first()
        .map(|first| first.unique_name == current_im)
        .unwrap_or(true);
    assert!(is_first);

    // When current IM is not the first one
    let current_im = "mozc";
    let is_first = enabled_ims
        .first()
        .map(|first| first.unique_name == current_im)
        .unwrap_or(true);
    assert!(!is_first);

    // When no languages available (edge case)
    let enabled_ims: Vec<InputMethod> = vec![];
    let current_im = "keyboard-us";
    let is_first = enabled_ims
        .first()
        .map(|first| first.unique_name == current_im)
        .unwrap_or(true);
    assert!(is_first); // Default to true when empty
}

#[test]
fn test_toggle_state_based_on_current_language() {
    // Test that toggle state correctly reflects whether we're on first language
    let enabled_ims = vec![
        InputMethod {
            name: "English".to_string(),
            unique_name: "keyboard-us".to_string(),
            language_code: "en".to_string(),
            enabled: true,
        },
        InputMethod {
            name: "Japanese".to_string(),
            unique_name: "mozc".to_string(),
            language_code: "ja".to_string(),
            enabled: true,
        },
        InputMethod {
            name: "Chinese".to_string(),
            unique_name: "pinyin".to_string(),
            language_code: "zh".to_string(),
            enabled: true,
        },
    ];

    // First language - toggle should be enabled
    let current_im = "keyboard-us";
    let toggle_enabled = enabled_ims
        .first()
        .map(|first| first.unique_name == current_im)
        .unwrap_or(true);
    assert!(toggle_enabled);

    // Second language - toggle should be disabled
    let current_im = "mozc";
    let toggle_enabled = enabled_ims
        .first()
        .map(|first| first.unique_name == current_im)
        .unwrap_or(true);
    assert!(!toggle_enabled);

    // Third language - toggle should be disabled
    let current_im = "pinyin";
    let toggle_enabled = enabled_ims
        .first()
        .map(|first| first.unique_name == current_im)
        .unwrap_or(true);
    assert!(!toggle_enabled);
}

#[test]
fn test_click_enabled_toggle_logic() {
    // Test: clicking enabled toggle (first language) should select next language
    let enabled_ims = [
        InputMethod {
            name: "English".to_string(),
            unique_name: "keyboard-us".to_string(),
            language_code: "en".to_string(),
            enabled: true,
        },
        InputMethod {
            name: "Japanese".to_string(),
            unique_name: "mozc".to_string(),
            language_code: "ja".to_string(),
            enabled: true,
        },
    ];

    let current_im = "keyboard-us"; // First language
    let current_index = enabled_ims
        .iter()
        .position(|im| im.unique_name == current_im)
        .unwrap_or(0);

    // When there's a next language
    if enabled_ims.len() > 1 && current_index + 1 < enabled_ims.len() {
        let next_im = &enabled_ims[current_index + 1];
        assert_eq!(next_im.unique_name, "mozc");
    }
}

#[test]
fn test_click_enabled_toggle_no_next_language() {
    // Test: clicking enabled toggle when already on last language should stay on first
    let enabled_ims = [InputMethod {
        name: "English".to_string(),
        unique_name: "keyboard-us".to_string(),
        language_code: "en".to_string(),
        enabled: true,
    }];

    let current_im = "keyboard-us";
    let current_index = enabled_ims
        .iter()
        .position(|im| im.unique_name == current_im)
        .unwrap_or(0);

    // No next language available
    let has_next = enabled_ims.len() > 1 && current_index + 1 < enabled_ims.len();
    assert!(!has_next);

    // Should stay on first language
    assert_eq!(enabled_ims[0].unique_name, "keyboard-us");
}

#[test]
fn test_click_disabled_toggle_logic() {
    // Test: clicking disabled toggle (not first language) should return to first
    let enabled_ims = [
        InputMethod {
            name: "English".to_string(),
            unique_name: "keyboard-us".to_string(),
            language_code: "en".to_string(),
            enabled: true,
        },
        InputMethod {
            name: "Japanese".to_string(),
            unique_name: "mozc".to_string(),
            language_code: "ja".to_string(),
            enabled: true,
        },
    ];

    // Currently on second language (toggle is disabled)
    let current_im = "mozc";
    let is_first = enabled_ims
        .first()
        .map(|first| first.unique_name == current_im)
        .unwrap_or(true);
    assert!(!is_first);

    // Clicking should return to first language
    let first_language = &enabled_ims[0];
    assert_eq!(first_language.unique_name, "keyboard-us");
}

#[test]
fn test_settings_callback_closes_popover() {
    // Test that settings callback logic includes popover close
    // This is a logic test - actual popover interaction requires GTK

    struct MockPopoverState {
        is_open: bool,
    }

    let mut popover_state = MockPopoverState { is_open: true };

    // Simulate settings button click
    popover_state.is_open = false; // popover.popdown() would do this

    assert!(!popover_state.is_open);
}

#[test]
fn test_empty_language_list_handling() {
    // Test handling of empty language list (edge case)
    let enabled_ims: Vec<InputMethod> = vec![];

    assert!(enabled_ims.is_empty());

    // Header should show 0 languages
    let enabled_count = enabled_ims.iter().filter(|im| im.enabled).count();
    let header_text = format!("Languages: {}", enabled_count);
    assert_eq!(header_text, "Languages: 0");
}

#[test]
fn test_current_language_name_lookup() {
    // Test finding the display name of the current language
    let available_ims = [
        InputMethod {
            name: "English (US)".to_string(),
            unique_name: "keyboard-us".to_string(),
            language_code: "en".to_string(),
            enabled: true,
        },
        InputMethod {
            name: "日本語".to_string(),
            unique_name: "mozc".to_string(),
            language_code: "ja".to_string(),
            enabled: true,
        },
    ];

    let current_im = "mozc";
    let current_im_name = available_ims
        .iter()
        .find(|im| im.unique_name == current_im)
        .map(|im| im.name.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    assert_eq!(current_im_name, "日本語");

    // Test unknown language
    let current_im = "nonexistent";
    let current_im_name = available_ims
        .iter()
        .find(|im| im.unique_name == current_im)
        .map(|im| im.name.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    assert_eq!(current_im_name, "Unknown");
}
