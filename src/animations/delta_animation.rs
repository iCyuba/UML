use crate::animations::traits::{Animatable, Numeric};
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

#[derive(Clone)]
pub struct DeltaAnimation<T: Numeric> {
    current_value: T,
    target_value: T,
    animating: bool,
    frame_time: Instant,
    multiplier: f64,
}

impl<T: Numeric> DeltaAnimation<T> {
    pub fn new(current_value: T, target_value: T) -> Self {
        Self {
            current_value,
            target_value,
            animating: true,
            frame_time: Instant::now(),
            multiplier: 35.,
        }
    }

    pub fn with_target_value(&self, target_value: T) -> Self {
        Self {
            target_value,
            animating: true,
            ..*self
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
            return self.target_value.clone();
        }

        let elapsed = self.frame_time.elapsed().as_secs_f64();
        self.frame_time = Instant::now();

        let step = (self.target_value - self.current_value).scalar_mul(elapsed * self.multiplier);

        if step.is_zero() {
            self.animating = false;
            self.target_value.clone()
        } else {
            self.current_value = self.current_value + step;
            self.current_value
        }
    }
}
