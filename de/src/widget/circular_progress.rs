use gtk4::cairo;
use gtk4::gdk::RGBA;
use gtk4::prelude::*;
use gtk4::DrawingArea;
use std::cell::RefCell;
use std::f64::consts::PI;
use std::rc::Rc;

#[derive(Clone)]
pub struct CircularProgress {
    drawing_area: DrawingArea,
    percentage: Rc<RefCell<f64>>,
    color: Rc<RefCell<RGBA>>,
}

impl CircularProgress {
    pub fn new(size: i32, line_width: f64, show_background: bool) -> Self {
        let drawing_area = DrawingArea::new();
        drawing_area.set_size_request(size, size);

        let percentage = Rc::new(RefCell::new(0.0));
        let default_color =
            RGBA::parse("rgba(255, 255, 255, 0.9)").unwrap_or(RGBA::new(1.0, 1.0, 1.0, 0.9));
        let color = Rc::new(RefCell::new(default_color));

        let percentage_clone = percentage.clone();
        let color_clone = color.clone();

        drawing_area.set_draw_func(move |_area, cr, width, height| {
            let center_x = width as f64 / 2.0;
            let center_y = height as f64 / 2.0;
            let radius = (width.min(height) as f64) / 2.0 - line_width / 2.0;
            let start_angle = -PI / 2.0; // Start from top (12 o'clock position)
            let full_circle = 2.0 * PI;
            let end_angle = start_angle + *percentage_clone.borrow() * full_circle;

            // Clear the surface
            cr.save().unwrap();
            cr.set_operator(cairo::Operator::Clear);
            cr.paint().unwrap();
            cr.restore().unwrap();

            // Background circle
            if show_background {
                cr.save().unwrap();
                cr.set_line_width(line_width);
                let c = color_clone.borrow();
                cr.set_source_rgba(
                    c.red() as f64,
                    c.green() as f64,
                    c.blue() as f64,
                    c.alpha() as f64 / 3.0,
                );
                cr.translate(center_x, center_y);
                cr.arc(0.0, 0.0, radius, 0.0, full_circle);
                cr.stroke().unwrap();
                cr.restore().unwrap();
            }

            // Progress arc
            if *percentage_clone.borrow() > 0.001 {
                cr.save().unwrap();
                cr.set_line_width(line_width);
                let c = color_clone.borrow();
                cr.set_source_rgba(
                    c.red() as f64,
                    c.green() as f64,
                    c.blue() as f64,
                    c.alpha() as f64,
                );
                cr.translate(center_x, center_y);
                cr.arc(0.0, 0.0, radius, start_angle, end_angle);
                cr.stroke().unwrap();
                cr.restore().unwrap();
            }
        });

        Self {
            drawing_area,
            percentage,
            color,
        }
    }

    pub fn set_percentage(&self, percentage: f64) {
        *self.percentage.borrow_mut() = percentage.clamp(0.0, 1.0);
        self.drawing_area.queue_draw();
    }

    pub fn set_color(&self, color: RGBA) {
        *self.color.borrow_mut() = color;
        self.drawing_area.queue_draw();
    }

    pub fn widget(&self) -> &DrawingArea {
        &self.drawing_area
    }
}
