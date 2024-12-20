use vello::kurbo;

use super::{Point, Size, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    origin: Point,
    size: Size,
}

impl Rect {
    pub fn new(origin: Vec2, size: Size) -> Self {
        Self { origin, size }
    }
}

// Conversions
impl From<kurbo::Rect> for Rect {
    fn from(value: kurbo::Rect) -> Self {
        Self {
            origin: Point::new(value.x0, value.y0),
            size: Point::from(value.size()),
        }
    }
}

impl From<Rect> for kurbo::Rect {
    fn from(value: Rect) -> Self {
        kurbo::Rect::from_origin_size(
            kurbo::Point::from(value.origin),
            kurbo::Size::from(value.size),
        )
    }
}
