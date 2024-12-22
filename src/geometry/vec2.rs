use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use taffy::AvailableSpace;
use vello::kurbo;
use winit::dpi::{PhysicalPosition, PhysicalSize};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

pub type Point = Vec2;
pub type Size = Vec2;

impl Vec2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

// Operations
impl Mul<f64> for Vec2 {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self::Output {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Div<f64> for Vec2 {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        Self::Output {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl Add<Vec2> for Vec2 {
    type Output = Self;

    fn add(self, rhs: Vec2) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub<Vec2> for Vec2 {
    type Output = Self;

    fn sub(self, rhs: Vec2) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Neg for Vec2 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::Output {
            x: -self.x,
            y: -self.y,
        }
    }
}

// Operations with assignment
impl MulAssign<f64> for Vec2 {
    fn mul_assign(&mut self, rhs: f64) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl DivAssign<f64> for Vec2 {
    fn div_assign(&mut self, rhs: f64) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl AddAssign<Vec2> for Vec2 {
    fn add_assign(&mut self, rhs: Vec2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl SubAssign<Vec2> for Vec2 {
    fn sub_assign(&mut self, rhs: Vec2) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

// Conversions
impl From<(f64, f64)> for Vec2 {
    fn from(value: (f64, f64)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}

impl From<(f32, f32)> for Vec2 {
    fn from(value: (f32, f32)) -> Self {
        Self {
            x: value.0 as f64,
            y: value.1 as f64,
        }
    }
}

impl From<Vec2> for (f64, f64) {
    fn from(point: Vec2) -> Self {
        (point.x, point.y)
    }
}

impl From<PhysicalPosition<f64>> for Vec2 {
    fn from(value: PhysicalPosition<f64>) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl From<PhysicalSize<f64>> for Vec2 {
    fn from(value: PhysicalSize<f64>) -> Self {
        Self {
            x: value.width,
            y: value.height,
        }
    }
}

impl From<kurbo::Point> for Vec2 {
    fn from(value: kurbo::Point) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl From<kurbo::Size> for Vec2 {
    fn from(value: kurbo::Size) -> Self {
        Self {
            x: value.width,
            y: value.height,
        }
    }
}

impl From<taffy::Point<f32>> for Vec2 {
    fn from(value: taffy::Point<f32>) -> Self {
        Self {
            x: value.x as f64,
            y: value.y as f64,
        }
    }
}

impl From<taffy::Size<f32>> for Vec2 {
    fn from(value: taffy::Size<f32>) -> Self {
        Self {
            x: value.width as f64,
            y: value.height as f64,
        }
    }
}

impl From<Vec2> for PhysicalPosition<f64> {
    fn from(point: Vec2) -> Self {
        PhysicalPosition::<f64> {
            x: point.x,
            y: point.y,
        }
    }
}

impl From<Vec2> for PhysicalSize<f64> {
    fn from(point: Vec2) -> Self {
        PhysicalSize::<f64> {
            width: point.x,
            height: point.y,
        }
    }
}

impl From<Vec2> for kurbo::Point {
    fn from(point: Vec2) -> Self {
        kurbo::Point {
            x: point.x,
            y: point.y,
        }
    }
}

impl From<Vec2> for kurbo::Size {
    fn from(point: Vec2) -> Self {
        kurbo::Size {
            width: point.x,
            height: point.y,
        }
    }
}

impl From<Vec2> for taffy::Point<f32> {
    fn from(point: Vec2) -> Self {
        taffy::Point::<f32> {
            x: point.x as f32,
            y: point.y as f32,
        }
    }
}

impl From<Vec2> for taffy::Size<f32> {
    fn from(point: Vec2) -> Self {
        taffy::Size::<f32> {
            width: point.x as f32,
            height: point.y as f32,
        }
    }
}

impl From<Vec2> for taffy::Size<AvailableSpace> {
    fn from(value: Vec2) -> Self {
        taffy::Size::<AvailableSpace> {
            width: AvailableSpace::Definite(value.x as f32),
            height: AvailableSpace::Definite(value.y as f32),
        }
    }
}
