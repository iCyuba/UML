use crate::geometry::Vec2;
use std::ops::{Add, Sub};

pub trait Interpolate: Copy {
    fn interpolate(&self, end_value: &Self, t: f64) -> Self;
}

pub trait Animatable {
    type Value: Copy;

    fn is_animating(&self) -> bool;
    fn update(&mut self) -> Self::Value;
    fn stop_animation(&mut self);
    fn continue_animation(&mut self);
    fn set_target(&mut self, value: Self::Value);
    fn get_target(&self) -> &Self::Value;
    fn current_value(&self) -> &Self::Value;
}

pub trait Magnitude {
    fn magnitude(&self) -> f64;
}

pub trait ScalarMul {
    fn scalar_mul(self, rhs: f64) -> Self;
}

pub trait DotMul {
    fn dot_mul(self, rhs: Self) -> f64;
}

pub trait Numeric:
    Copy + Sub<Output = Self> + Add<Output = Self> + Magnitude + DotMul + ScalarMul
{
}

macro_rules! numeric_impl {
    ($t:ty) => {
        impl Magnitude for $t {
            #[inline]
            fn magnitude(&self) -> f64 {
                (*self as f64).abs()
            }
        }
        impl ScalarMul for $t {
            #[inline]
            fn scalar_mul(self, rhs: f64) -> Self {
                (self as f64 * rhs) as $t
            }
        }
        impl DotMul for $t {
            #[inline]
            fn dot_mul(self, rhs: Self) -> f64 {
                self as f64 * rhs as f64
            }
        }
        impl Numeric for $t {}
    };
}

numeric_impl!(usize);
numeric_impl!(u8);
numeric_impl!(u16);
numeric_impl!(u32);
numeric_impl!(u64);
numeric_impl!(u128);

numeric_impl!(isize);
numeric_impl!(i8);
numeric_impl!(i16);
numeric_impl!(i32);
numeric_impl!(i64);
numeric_impl!(i128);

numeric_impl!(f32);
numeric_impl!(f64);

impl Magnitude for Vec2 {
    fn magnitude(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
}

impl DotMul for Vec2 {
    fn dot_mul(self, rhs: Self) -> f64 {
        self.x * rhs.x + self.y * rhs.y
    }
}

impl ScalarMul for Vec2 {
    fn scalar_mul(self, rhs: f64) -> Self {
        Vec2 {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Numeric for Vec2 {}
