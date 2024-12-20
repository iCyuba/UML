#![allow(dead_code)]

use crate::animations::traits::{Animatable, Numeric};
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

pub struct DeltaAnimation<T: Numeric> {
    current_value: T,
    target_value: T,

    frame_time: Instant,
    multiplier: f64,

    animating: bool,
}

impl<T: Numeric> DeltaAnimation<T> {
    pub fn new(initial_value: T, multiplier: f64) -> Self {
        Self {
            current_value: initial_value,
            target_value: initial_value,
            animating: false,
            frame_time: Instant::now(),
            multiplier,
        }
    }
}

impl<T: Numeric> Animatable for DeltaAnimation<T> {
    type Value = T;

    fn is_animating(&self) -> bool {
        self.animating
    }

    fn update(&mut self) -> T {
        if !self.animating {
            return self.target_value;
        }

        let elapsed = self.frame_time.elapsed().as_secs_f64();
        self.frame_time = Instant::now();

        let step = (self.target_value - self.current_value).scalar_mul(elapsed * self.multiplier);

        if step.is_zero() {
            self.current_value = self.target_value;
            self.animating = false;
            self.target_value
        } else {
            self.current_value = self.current_value + step;
            self.current_value
        }
    }

    fn stop_animation(&mut self) {
        self.animating = false;
        self.current_value = self.target_value;
    }

    fn continue_animation(&mut self) {
        self.animating = true;
        self.frame_time = Instant::now();
    }

    fn set_target(&mut self, value: T) {
        self.target_value = value;
    }

    fn get_target(&self) -> Self::Value {
        self.target_value
    }

    fn current_value(&self) -> Self::Value {
        self.current_value
    }
}
