#![allow(dead_code)]

use crate::animations::traits::{Animatable, Interpolate};
use crate::geometry::Vec2;
#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};
use vello::peniko::Color;
#[cfg(target_arch = "wasm32")]
use web_time::{Duration, Instant};

pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    EaseOutCubic,
}

pub struct StandardAnimation<T: Interpolate> {
    start_value: T,
    current_value: T,
    end_value: T,

    duration: Duration,
    easing: Easing,
    start_time: Instant,

    animating: bool,
    initialized: bool,
}

impl<T: Interpolate> StandardAnimation<T> {
    pub fn new(duration: Duration, easing: Easing) -> Self {
        Self {
            initialized: false,
            ..Self::initialized(Default::default(), duration, easing)
        }
    }

    pub fn initialized(initial_value: T, duration: Duration, easing: Easing) -> Self {
        Self {
            start_value: initial_value,
            current_value: initial_value,
            end_value: initial_value,
            duration,
            start_time: Instant::now(),
            animating: false,
            initialized: true,
            easing,
        }
    }

    fn calculate_time(&self, t: f64) -> f64 {
        match self.easing {
            Easing::Linear => t,
            Easing::EaseIn => t * t,
            Easing::EaseOut => t * (2.0 - t),
            Easing::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    -1.0 + (4.0 - 2.0 * t) * t
                }
            }
            Easing::EaseOutCubic => 1.0 - (1.0 - t).powf(3.0),
        }
    }
}

impl<T: Interpolate> Animatable for StandardAnimation<T> {
    type Value = T;

    fn is_animating(&self) -> bool {
        self.initialized && self.animating
    }

    fn is_initialized(&self) -> bool {
        self.initialized
    }

    fn initialize(&mut self, value: Self::Value) {
        self.set_target(value);
        self.initialized = true;
    }

    fn update(&mut self) -> T {
        if self.animating {
            let progress = self.start_time.elapsed().as_secs_f64() / self.duration.as_secs_f64();

            if progress >= 1.0 {
                self.animating = false;

                self.current_value = self.end_value;
                self.start_value = self.end_value;
            } else {
                let time = self.calculate_time(progress);
                self.current_value = self.start_value.interpolate(&self.end_value, time);
            }
        }

        self.current_value
    }

    fn stop_animation(&mut self) {
        self.animating = false;
        self.current_value = self.end_value;
        self.start_value = self.end_value;
    }

    fn continue_animation(&mut self) {
        if !self.animating {
            self.start_time = Instant::now();
        }

        self.animating = true;
    }

    fn set_target(&mut self, value: Self::Value) {
        self.start_time = Instant::now();

        if !self.animating {
            self.current_value = value;
        }

        self.start_value = self.current_value;
        self.end_value = value;
    }

    fn get_target(&self) -> &Self::Value {
        &self.end_value
    }

    fn current_value(&self) -> &Self::Value {
        &self.current_value
    }
}

macro_rules! interpolate_impl {
    ($($t:ty)*) => {
        $(
            impl Interpolate for $t {
                fn interpolate(&self, end_value: &Self, t: f64) -> Self {
                    (*self as f64 + (*end_value as f64 - *self as f64) * t) as Self
                }
            }
        )*
    }
}

interpolate_impl!(usize u8 u16 u32 u64 u128 isize i8 i16 i32 i64 i128 f32 f64);

impl Interpolate for Vec2 {
    fn interpolate(&self, end_value: &Self, t: f64) -> Self {
        Vec2 {
            x: self.x.interpolate(&end_value.x, t),
            y: self.y.interpolate(&end_value.y, t),
        }
    }
}

impl Interpolate for Color {
    fn interpolate(&self, end_value: &Self, t: f64) -> Self {
        Color {
            r: self.r.interpolate(&end_value.r, t),
            g: self.g.interpolate(&end_value.g, t),
            b: self.b.interpolate(&end_value.b, t),
            a: self.a.interpolate(&end_value.a, t),
        }
    }
}
