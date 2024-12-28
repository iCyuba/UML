use super::{Point, Size, Vec2};
use serde::{Deserialize, Serialize};
use std::ops::{Add, Mul};
use taffy::Layout;
use vello::kurbo;

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Rect {
    pub origin: Point,
    pub size: Size,
}

impl Rect {
    pub const ZERO: Self = Self {
        origin: Point::ZERO,
        size: Size::ZERO,
    };

    pub fn new(origin: impl Into<Point>, size: impl Into<Size>) -> Self {
        Self {
            origin: origin.into(),
            size: size.into(),
        }
    }

    pub fn center(self) -> Point {
        self.origin + self.size / 2.0
    }

    pub fn end(self) -> Point {
        self.origin + self.size
    }

    pub fn inset(self, insets: impl Into<Rect>) -> Self {
        let insets = insets.into();
        Self {
            origin: self.origin + insets.origin,
            size: self.size - insets.size,
        }
    }

    pub fn inset_uniform(self, inset: f64) -> Self {
        let inset = Vec2::new(inset, inset);
        self.inset(Rect::new(inset, inset))
    }

    pub fn translate(self, offset: impl Into<Vec2>) -> Self {
        Self {
            origin: self.origin + offset.into(),
            size: self.size,
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

impl Mul<f64> for Rect {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self::Output {
            origin: self.origin * rhs,
            size: self.size * rhs,
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
