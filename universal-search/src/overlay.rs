/// Whether a click at (`x`, `y`) lands outside the `(x, y, width, height)` rectangle.
pub fn is_outside(area: (f64, f64, f64, f64), x: f64, y: f64) -> bool {
    let (left, top, width, height) = area;
    x < left || x > left + width || y < top || y > top + height
}
