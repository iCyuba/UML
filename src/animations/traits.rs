use crate::geometry::Vec2;
use std::ops::{Add, Sub};

pub trait Interpolate: Copy {
    fn interpolate(&self, end_value: &Self, t: f64) -> Self;
}

pub trait Animatable {
    type Value;

    fn is_animating(&self) -> bool;
    fn update(&mut self) -> Self::Value;
}

pub trait Zero: Sized {
    fn is_zero(&self) -> bool;
}

pub trait ScalarMul {
    fn scalar_mul(self, rhs: f64) -> Self;
}

pub trait Numeric: Copy + Sub<Output = Self> + Add<Output = Self> + ScalarMul + Zero {}

macro_rules! numeric_impl {
    ($t:ty, $v:expr) => {
        impl Zero for $t {
            #[inline]
            fn is_zero(&self) -> bool {
                *self == $v
            }
        }
        impl ScalarMul for $t {
            #[inline]
            fn scalar_mul(self, rhs: f64) -> Self {
                (self as f64 * rhs) as $t
            }
        }
        impl Numeric for $t {}
    };
}

numeric_impl!(usize, 0);
numeric_impl!(u8, 0);
numeric_impl!(u16, 0);
numeric_impl!(u32, 0);
numeric_impl!(u64, 0);
numeric_impl!(u128, 0);

numeric_impl!(isize, 0);
numeric_impl!(i8, 0);
numeric_impl!(i16, 0);
numeric_impl!(i32, 0);
numeric_impl!(i64, 0);
numeric_impl!(i128, 0);

numeric_impl!(f32, 0.0);
numeric_impl!(f64, 0.0);

impl ScalarMul for Vec2 {
    fn scalar_mul(self, rhs: f64) -> Self {
        self * rhs
    }
}

impl Zero for Vec2 {
    fn is_zero(&self) -> bool {
        self.x == 0.0 && self.y == 0.0
    }
}

impl Numeric for Vec2 {}
