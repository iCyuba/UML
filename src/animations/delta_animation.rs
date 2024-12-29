#![allow(dead_code)]

use crate::animations::traits::{Animatable, Numeric};
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

#[derive(Debug)]
pub struct DeltaAnimation<T: Numeric> {
    start_value: T,
    current_value: T,
    end_value: T,

    frame_time: Instant,
    multiplier: f64,

    animating: bool,
    initialized: bool,
}

impl<T: Numeric> DeltaAnimation<T> {
    pub fn new(multiplier: f64) -> Self {
        Self {
            initialized: false,
            ..Self::initialized(Default::default(), multiplier)
        }
    }

    pub fn initialized(initial_value: T, multiplier: f64) -> Self {
        Self {
            start_value: initial_value,
            current_value: initial_value,
            end_value: initial_value,
            animating: false,
            initialized: true,
            frame_time: Instant::now(),
            multiplier,
        }
    }
}

impl<T: Numeric> Animatable for DeltaAnimation<T> {
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
            let elapsed = self.frame_time.elapsed().as_secs_f64();
            self.frame_time = Instant::now();

            let direction = self.end_value - self.start_value;
            let step = self.end_value - self.current_value;

            // Stop the animation if the value surpasses the target or the step is too small
            if direction.dot_mul(step).is_sign_negative() || step.magnitude() < 1e-9 {
                self.stop_animation();
            } else {
                let step = step.scalar_mul(elapsed * self.multiplier);
                self.current_value = self.current_value + step;
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
        // Don't reset the frame time if an animation is already running
        if !self.animating {
            self.frame_time = Instant::now();
        }

        self.animating = true;
    }

    fn set_target(&mut self, value: T) {
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
