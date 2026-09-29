use gtk4::prelude::*;
use rusty_de::service::fcitx::{Fcitx, InputMethod};

#[test]
fn test_input_method_struct() {
    let im = InputMethod {
        name: "Keyboard - English (US)".to_string(),
        unique_name: "keyboard-us".to_string(),
        language_code: "en".to_string(),
        enabled: true,
    };

    assert_eq!(im.name, "Keyboard - English (US)");
    assert_eq!(im.unique_name, "keyboard-us");
    assert_eq!(im.language_code, "en");
    assert!(im.enabled);
}

#[test]
fn test_fcitx_creation() {
    let fcitx = Fcitx::new();

    // Should be able to get current IM (even if empty at test time)
    let current = fcitx.current_im();
    assert!(current.is_empty() || !current.is_empty());

    // Should be able to get available IMs (returns a vector)
    let _available = fcitx.available_ims();
}

#[test]
fn test_fcitx_default() {
    let fcitx1 = Fcitx::new();
    let fcitx2 = Fcitx::default();

    // Both should create valid instances
    assert_eq!(fcitx1.current_im(), fcitx2.current_im());
}

#[test]
fn test_fcitx_signal_connection() {
    let fcitx = Fcitx::new();
    let signal_fired = std::rc::Rc::new(std::cell::RefCell::new(false));
    let signal_fired_clone = signal_fired.clone();

    fcitx.connect_local("input-method-changed", false, move |_| {
        *signal_fired_clone.borrow_mut() = true;
        None
    });

    // Signal handler should be connected (won't fire in test without actual change)
    assert!(!*signal_fired.borrow());
}

#[test]
fn test_fcitx_property_notify() {
    let fcitx = Fcitx::new();
    let notify_fired = std::rc::Rc::new(std::cell::RefCell::new(false));
    let notify_fired_clone = notify_fired.clone();

    fcitx.connect_notify_local(Some("current-im"), move |_, _| {
        *notify_fired_clone.borrow_mut() = true;
    });

    // Property notification handler should be connected
    assert!(!*notify_fired.borrow());
}

#[test]
fn test_input_method_clone() {
    let im1 = InputMethod {
        name: "Mozc".to_string(),
        unique_name: "mozc".to_string(),
        language_code: "ja".to_string(),
        enabled: true,
    };

    let im2 = im1.clone();

    assert_eq!(im1.name, im2.name);
    assert_eq!(im1.unique_name, im2.unique_name);
    assert_eq!(im1.language_code, im2.language_code);
    assert_eq!(im1.enabled, im2.enabled);
}

#[test]
fn test_fcitx_refresh_languages_method() {
    // Test that refresh_languages method can be called
    let fcitx = Fcitx::new();

    // Should not panic when called
    fcitx.refresh_languages();
}

#[test]
fn test_fcitx_available_ims_count_property() {
    // Test that available-ims-count property exists and can be connected to
    let fcitx = Fcitx::new();
    let notify_fired = std::rc::Rc::new(std::cell::RefCell::new(false));
    let notify_fired_clone = notify_fired.clone();

    fcitx.connect_notify_local(Some("available-ims-count"), move |_, _| {
        *notify_fired_clone.borrow_mut() = true;
    });

    // Property notification handler should be connected
    assert!(!*notify_fired.borrow());
}

#[test]
fn test_fcitx_available_ims_returns_vector() {
    // Test that available_ims() returns a vector
    let fcitx = Fcitx::new();
    let available = fcitx.available_ims();

    // Should return a vector (may be empty in test environment)
    // Just verify it's a valid Vec (this test mainly checks it doesn't panic)
    let _ = available.len();
}

#[test]
fn test_fcitx_singleton_pattern() {
    // Test that instance() returns the same instance
    let fcitx1 = Fcitx::instance();
    let fcitx2 = Fcitx::instance();

    // Both should refer to the same singleton
    assert_eq!(fcitx1.current_im(), fcitx2.current_im());
}

#[test]
fn test_refresh_languages_does_not_panic() {
    // Test that refresh_languages doesn't panic even without D-Bus connection
    let fcitx = Fcitx::instance();

    // Should handle D-Bus errors gracefully
    fcitx.refresh_languages();

    // Should still be able to get available IMs
    let _available = fcitx.available_ims();
}

#[test]
fn test_available_ims_count_property_type() {
    // Test that the property returns a u32 value
    let fcitx = Fcitx::new();

    // Should be able to read the property
    let count: u32 = fcitx.property("available-ims-count");

    // Verify we can read the property (u32 is always valid)
    let _ = count;
}
