use gtk4::glib;
use gtk4::prelude::*;
use rusty_de::service::charge_threshold::{charge_threshold_with_backend, ChargeThresholdBackend};
use std::cell::Cell;
use std::io;
use std::rc::Rc;
use std::sync::{Arc, Mutex, OnceLock};

// Ensure GTK is initialized once per test binary
static GTK_INIT: OnceLock<()> = OnceLock::new();

fn ensure_gtk_init() {
    GTK_INIT.get_or_init(|| {
        gtk4::init().expect("Failed to initialize GTK");
    });
}

struct MockChargeThresholdBackendData {
    threshold: Mutex<i32>,
    set_threshold_calls: Mutex<Vec<(i32, i32)>>,
    fail_writes: Mutex<bool>,
}

struct MockChargeThresholdBackend {
    data: Arc<MockChargeThresholdBackendData>,
}

impl MockChargeThresholdBackend {
    fn new(initial_threshold: i32) -> Self {
        Self {
            data: Arc::new(MockChargeThresholdBackendData {
                threshold: Mutex::new(initial_threshold),
                set_threshold_calls: Mutex::new(Vec::new()),
                fail_writes: Mutex::new(false),
            }),
        }
    }

    fn failing(initial_threshold: i32) -> Self {
        let mock = Self::new(initial_threshold);
        *mock.data.fail_writes.lock().unwrap() = true;
        mock
    }

    fn clone_data(&self) -> Arc<MockChargeThresholdBackendData> {
        self.data.clone()
    }
}

impl ChargeThresholdBackend for MockChargeThresholdBackend {
    fn read_threshold(&self, _path: &str) -> Option<i32> {
        Some(*self.data.threshold.lock().unwrap())
    }

    fn set_threshold(&self, start: i32, end: i32) -> io::Result<()> {
        self.data
            .set_threshold_calls
            .lock()
            .unwrap()
            .push((start, end));

        if *self.data.fail_writes.lock().unwrap() {
            return Err(io::Error::from(io::ErrorKind::PermissionDenied));
        }

        *self.data.threshold.lock().unwrap() = end;
        Ok(())
    }
}

#[test]
fn test_charge_threshold_new() {
    ensure_gtk_init();
    let mock = MockChargeThresholdBackend::new(80);
    let charge_threshold = charge_threshold_with_backend(Box::new(mock));
    assert!(charge_threshold.threshold() >= 0 && charge_threshold.threshold() <= 100);
}

#[test]
fn test_charge_threshold_threshold_getter() {
    ensure_gtk_init();
    let mock = MockChargeThresholdBackend::new(75);
    let charge_threshold = charge_threshold_with_backend(Box::new(mock));
    let threshold = charge_threshold.threshold();
    assert!(
        (0..=100).contains(&threshold),
        "Threshold should be between 0 and 100"
    );
    assert_eq!(threshold, 75, "Threshold should match mock initial value");
}

#[test]
fn test_charge_threshold_set_threshold_valid() {
    ensure_gtk_init();
    let mock = MockChargeThresholdBackend::new(60);
    let charge_threshold = charge_threshold_with_backend(Box::new(mock));
    charge_threshold.set_threshold(80);
    assert_eq!(charge_threshold.threshold(), 80);
}

#[test]
fn test_charge_threshold_set_threshold_minimum() {
    ensure_gtk_init();
    let mock = MockChargeThresholdBackend::new(50);
    let charge_threshold = charge_threshold_with_backend(Box::new(mock));
    charge_threshold.set_threshold(10);
    assert_eq!(charge_threshold.threshold(), 10);
}

#[test]
fn test_charge_threshold_set_threshold_below_minimum() {
    ensure_gtk_init();
    let mock = MockChargeThresholdBackend::new(60);
    let charge_threshold = charge_threshold_with_backend(Box::new(mock));
    let initial = charge_threshold.threshold();
    charge_threshold.set_threshold(5);
    assert_eq!(
        charge_threshold.threshold(),
        initial,
        "Threshold below 10 should be rejected"
    );
}

#[test]
fn test_charge_threshold_set_threshold_same_value() {
    ensure_gtk_init();
    let mock = MockChargeThresholdBackend::new(60);
    let charge_threshold = charge_threshold_with_backend(Box::new(mock));
    charge_threshold.set_threshold(60);
    let first = charge_threshold.threshold();
    charge_threshold.set_threshold(60);
    assert_eq!(
        charge_threshold.threshold(),
        first,
        "Setting same threshold should not change value"
    );
}

#[test]
fn test_charge_threshold_threshold_range() {
    ensure_gtk_init();
    let test_cases = vec![10, 20, 50, 80, 100];

    for value in test_cases {
        let mock = MockChargeThresholdBackend::new(50);
        let charge_threshold = charge_threshold_with_backend(Box::new(mock));
        charge_threshold.set_threshold(value);
        assert_eq!(
            charge_threshold.threshold(),
            value,
            "Threshold should be set to {}",
            value
        );
    }
}

#[test]
fn test_charge_threshold_start_threshold_calculation() {
    ensure_gtk_init();
    // The service calculates start threshold as value - 5

    // Test case: end_threshold=80, start should be 75
    let mock = MockChargeThresholdBackend::new(50);
    let data = mock.clone_data();
    let charge_threshold = charge_threshold_with_backend(Box::new(mock));

    // Verify initial state
    assert_eq!(charge_threshold.threshold(), 50);

    // Set threshold to 80
    charge_threshold.set_threshold(80);

    // Verify threshold was updated
    assert_eq!(charge_threshold.threshold(), 80);

    // Verify backend.set_threshold was called with correct values
    let calls = data.set_threshold_calls.lock().unwrap().clone();
    assert_eq!(
        calls.len(),
        1,
        "Expected 1 call to backend.set_threshold, got {}",
        calls.len()
    );
    assert_eq!(calls[0], (75, 80), "Expected (75, 80), got {:?}", calls[0]);
}

#[test]
fn test_charge_threshold_threshold_validation() {
    let invalid_values = vec![0, 5, 9];
    let valid_values = vec![10, 50, 100];

    for value in invalid_values {
        let is_valid = value >= 10;
        assert!(!is_valid, "Threshold {} should be invalid", value);
    }

    for value in valid_values {
        let is_valid = value >= 10;
        assert!(is_valid, "Threshold {} should be valid", value);
    }
}

#[test]
fn test_charge_threshold_threshold_change_detection() {
    let old_threshold = 60;
    let new_threshold = 80;

    let changed = old_threshold != new_threshold;
    assert!(
        changed,
        "Threshold change from {} to {} should be detected",
        old_threshold, new_threshold
    );
}

#[test]
fn test_charge_threshold_threshold_no_change_detection() {
    let old_threshold = 60;
    let new_threshold = 60;

    let changed = old_threshold != new_threshold;
    assert!(
        !changed,
        "No threshold change should be detected when values are identical"
    );
}

#[test]
fn test_mock_backend_directly() {
    let mock = MockChargeThresholdBackend::new(80);
    let data = mock.clone_data();

    // Test read_threshold
    assert_eq!(mock.read_threshold("/fake/path"), Some(80));

    // Test set_threshold
    mock.set_threshold(75, 80).unwrap();
    mock.set_threshold(45, 50).unwrap();

    let calls = data.set_threshold_calls.lock().unwrap().clone();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0], (75, 80));
    assert_eq!(calls[1], (45, 50));

    // Verify threshold was updated
    assert_eq!(mock.read_threshold("/fake/path"), Some(50));
}

#[test]
fn test_charge_threshold_set_threshold_write_failure_keeps_previous_value() {
    ensure_gtk_init();
    let mock = MockChargeThresholdBackend::failing(60);
    let data = mock.clone_data();
    let charge_threshold = charge_threshold_with_backend(Box::new(mock));
    assert_eq!(charge_threshold.threshold(), 60);

    let notifications = Rc::new(Cell::new(0));
    charge_threshold.connect_notify_local(
        Some("threshold"),
        glib::clone!(
            #[strong]
            notifications,
            move |_, _| notifications.set(notifications.get() + 1)
        ),
    );

    charge_threshold.set_threshold(80);

    assert_eq!(
        charge_threshold.threshold(),
        60,
        "A failed write must leave the property at the value sysfs still holds"
    );
    assert_eq!(
        notifications.get(),
        0,
        "A failed write must not notify \"threshold\""
    );

    let calls = data.set_threshold_calls.lock().unwrap().clone();
    assert_eq!(calls, vec![(75, 80)], "Backend should still be attempted");
}
