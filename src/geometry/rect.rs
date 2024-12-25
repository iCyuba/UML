use super::{Point, Size};
use std::ops::Add;
use taffy::Layout;
use vello::kurbo;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    pub origin: Point,
    pub size: Size,
}

impl Rect {
    pub fn new(origin: impl Into<Point>, size: impl Into<Size>) -> Self {
        Self {
            origin: origin.into(),
            size: size.into(),
        }
    }

    pub fn inset(&self, insets: impl Into<Rect>) -> Self {
        let insets = insets.into();
        Self {
            origin: self.origin + insets.origin,
            size: self.size - insets.size,
        }
    }

    pub fn contains(&self, point: impl Into<Point>) -> bool {
        let point = point.into();
        point.x >= self.origin.x
            && point.x <= self.origin.x + self.size.x
            && point.y >= self.origin.y
            && point.y <= self.origin.y + self.size.y
    }
}

impl Add<Point> for Rect {
    type Output = Rect;

    fn add(self, rhs: Point) -> Self::Output {
        Self::Output {
            origin: self.origin + rhs,
            size: self.size,
        }
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

impl From<Layout> for Rect {
    fn from(value: Layout) -> Self {
        Self {
            origin: value.location.into(),
            size: value.size.into(),
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

impl From<taffy::Rect<f32>> for Rect {
    fn from(value: taffy::Rect<f32>) -> Self {
        Self {
            origin: (value.left as f64, value.top as f64).into(),
            size: (
                (value.left + value.right) as f64,
                (value.top + value.bottom) as f64,
            )
                .into(),
        }
    }
}

impl From<&taffy::Rect<f32>> for Rect {
    fn from(value: &taffy::Rect<f32>) -> Self {
        Self::from(*value)
    }
}
