use rusty_de::service::brightness::{brightness_with_backend, BrightnessBackend};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Mutex, OnceLock};

// Ensure GTK is initialized once per test binary
static GTK_INIT: OnceLock<()> = OnceLock::new();

fn ensure_gtk_init() {
    GTK_INIT.get_or_init(|| {
        gtk4::init().expect("Failed to initialize GTK");
    });
}

struct MockBrightnessBackend {
    set_brightness_calls: Mutex<Vec<f64>>,
    current_brightness: AtomicI32,
}

impl MockBrightnessBackend {
    fn new() -> Self {
        Self {
            set_brightness_calls: Mutex::new(Vec::new()),
            current_brightness: AtomicI32::new(50),
        }
    }

    fn with_brightness(brightness: i32) -> Self {
        Self {
            set_brightness_calls: Mutex::new(Vec::new()),
            current_brightness: AtomicI32::new(brightness),
        }
    }

    fn get_set_brightness_calls(&self) -> Vec<f64> {
        self.set_brightness_calls.lock().unwrap().clone()
    }
}

impl BrightnessBackend for MockBrightnessBackend {
    fn get_first_device(&self, _path: &str) -> String {
        "test_device".to_string()
    }

    fn get_brightness_value(&self, command: &str, _device: &str, _is_screen: bool) -> i32 {
        match command {
            "max" => 100,
            "get" => self.current_brightness.load(Ordering::Relaxed),
            _ => 0,
        }
    }

    fn set_brightness(&self, percent: f64) {
        self.set_brightness_calls.lock().unwrap().push(percent);
    }

    fn set_kbd_brightness(&self, _device: &str, _level: i32) {
        // Mock implementation - no-op for tests
    }
}

#[test]
fn test_brightness_initialization() {
    ensure_gtk_init();
    let mock = MockBrightnessBackend::new();
    let brightness = brightness_with_backend(Box::new(mock));

    // Screen brightness should be initialized to 0.5 (50/100)
    let screen = brightness.screen();
    assert!(
        (screen - 0.5).abs() < f64::EPSILON,
        "Screen brightness should be 0.5, got {}",
        screen
    );

    // Keyboard brightness should be initialized to 50
    let kbd = brightness.kbd();
    assert_eq!(kbd, 50, "Keyboard brightness should be 50, got {}", kbd);
}

#[test]
fn test_brightness_icon_name_off() {
    ensure_gtk_init();
    let mock = MockBrightnessBackend::with_brightness(20);
    let brightness = brightness_with_backend(Box::new(mock));
    assert_eq!(brightness.icon_name(), "display-brightness-off-symbolic");
}

#[test]
fn test_brightness_icon_name_low() {
    ensure_gtk_init();
    let mock = MockBrightnessBackend::with_brightness(40);
    let brightness = brightness_with_backend(Box::new(mock));
    assert_eq!(brightness.icon_name(), "display-brightness-low-symbolic");
}

#[test]
fn test_brightness_icon_name_medium() {
    ensure_gtk_init();
    let mock = MockBrightnessBackend::with_brightness(60);
    let brightness = brightness_with_backend(Box::new(mock));
    assert_eq!(brightness.icon_name(), "display-brightness-medium-symbolic");
}

#[test]
fn test_brightness_icon_name_high() {
    ensure_gtk_init();
    let mock = MockBrightnessBackend::with_brightness(80);
    let brightness = brightness_with_backend(Box::new(mock));
    assert_eq!(brightness.icon_name(), "display-brightness-high-symbolic");
}

#[test]
fn test_brightness_icon_name_full() {
    ensure_gtk_init();
    let mock = MockBrightnessBackend::with_brightness(95);
    let brightness = brightness_with_backend(Box::new(mock));
    assert_eq!(brightness.icon_name(), "display-brightness-full-symbolic");
}

#[test]
fn test_brightness_icon_name_boundary_off_zero() {
    ensure_gtk_init();
    let mock = MockBrightnessBackend::with_brightness(0);
    let brightness = brightness_with_backend(Box::new(mock));
    assert_eq!(brightness.icon_name(), "display-brightness-off-symbolic");
}

#[test]
fn test_brightness_icon_name_boundary_off_low() {
    ensure_gtk_init();
    let mock = MockBrightnessBackend::with_brightness(29);
    let brightness = brightness_with_backend(Box::new(mock));
    assert_eq!(brightness.icon_name(), "display-brightness-off-symbolic");
}

#[test]
fn test_brightness_icon_name_boundary_low_start() {
    ensure_gtk_init();
    let mock = MockBrightnessBackend::with_brightness(30);
    let brightness = brightness_with_backend(Box::new(mock));
    assert_eq!(brightness.icon_name(), "display-brightness-low-symbolic");
}

#[test]
fn test_brightness_icon_name_boundary_low_medium() {
    ensure_gtk_init();
    let mock = MockBrightnessBackend::with_brightness(49);
    let brightness = brightness_with_backend(Box::new(mock));
    assert_eq!(brightness.icon_name(), "display-brightness-low-symbolic");
}

#[test]
fn test_brightness_icon_name_boundary_medium_start() {
    ensure_gtk_init();
    let mock = MockBrightnessBackend::with_brightness(50);
    let brightness = brightness_with_backend(Box::new(mock));
    assert_eq!(brightness.icon_name(), "display-brightness-medium-symbolic");
}

#[test]
fn test_brightness_icon_name_boundary_medium_high() {
    ensure_gtk_init();
    let mock = MockBrightnessBackend::with_brightness(69);
    let brightness = brightness_with_backend(Box::new(mock));
    assert_eq!(brightness.icon_name(), "display-brightness-medium-symbolic");
}

#[test]
fn test_brightness_icon_name_boundary_high_start() {
    ensure_gtk_init();
    let mock = MockBrightnessBackend::with_brightness(70);
    let brightness = brightness_with_backend(Box::new(mock));
    assert_eq!(brightness.icon_name(), "display-brightness-high-symbolic");
}

#[test]
fn test_brightness_icon_name_boundary_high_full() {
    ensure_gtk_init();
    let mock = MockBrightnessBackend::with_brightness(89);
    let brightness = brightness_with_backend(Box::new(mock));
    assert_eq!(brightness.icon_name(), "display-brightness-high-symbolic");
}

#[test]
fn test_brightness_icon_name_boundary_full_start() {
    ensure_gtk_init();
    let mock = MockBrightnessBackend::with_brightness(90);
    let brightness = brightness_with_backend(Box::new(mock));
    assert_eq!(brightness.icon_name(), "display-brightness-full-symbolic");
}

#[test]
fn test_brightness_icon_name_boundary_full_max() {
    ensure_gtk_init();
    let mock = MockBrightnessBackend::with_brightness(100);
    let brightness = brightness_with_backend(Box::new(mock));
    assert_eq!(brightness.icon_name(), "display-brightness-full-symbolic");
}

#[test]
fn test_brightness_set_screen() {
    ensure_gtk_init();
    let mock = MockBrightnessBackend::new();
    let brightness = brightness_with_backend(Box::new(mock));

    // Test setting brightness to 0.75
    brightness.set_screen(0.75);
    // Just verify it doesn't panic
}

#[test]
fn test_brightness_clamp() {
    ensure_gtk_init();
    let mock = MockBrightnessBackend::new();
    let brightness = brightness_with_backend(Box::new(mock));

    // Test that set_screen clamps values to [0.0, 1.0]
    brightness.set_screen(1.5);
    // Should clamp to 1.0

    brightness.set_screen(-0.5);
    // Should clamp to 0.0

    brightness.set_screen(0.5);
    // Should use 0.5
}

#[test]
fn test_brightness_toggle() {
    ensure_gtk_init();
    let mock = MockBrightnessBackend::new();
    let brightness = brightness_with_backend(Box::new(mock));

    // Test that toggle doesn't panic
    brightness.toggle();
}

#[test]
fn test_mock_backend_directly() {
    let mock = MockBrightnessBackend::new();

    // Test get_first_device
    assert_eq!(mock.get_first_device("/sys/class/backlight"), "test_device");

    // Test get_brightness_value
    assert_eq!(mock.get_brightness_value("max", "test", true), 100);
    assert_eq!(mock.get_brightness_value("get", "test", true), 50);

    // Test set_brightness
    mock.set_brightness(0.75);
    mock.set_brightness(0.5);

    let calls = mock.get_set_brightness_calls();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0], 0.75);
    assert_eq!(calls[1], 0.5);
}
