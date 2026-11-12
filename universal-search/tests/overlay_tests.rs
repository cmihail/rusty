use rusty_universal_search::overlay::is_outside;

const CARD: (f64, f64, f64, f64) = (560.0, 250.0, 800.0, 700.0);

#[test]
fn click_inside_the_card_is_not_outside() {
    assert!(!is_outside(CARD, 960.0, 600.0));
}

#[test]
fn click_on_the_card_edge_is_not_outside() {
    assert!(!is_outside(CARD, 560.0, 250.0));
    assert!(!is_outside(CARD, 1360.0, 950.0));
}

#[test]
fn click_on_the_backdrop_is_outside() {
    assert!(is_outside(CARD, 100.0, 600.0));
    assert!(is_outside(CARD, 1500.0, 600.0));
    assert!(is_outside(CARD, 960.0, 100.0));
    assert!(is_outside(CARD, 960.0, 1000.0));
}
