#![allow(dead_code)]

use std::time::{Duration, Instant};
use crate::animations::traits::{Animatable, Interpolate};

pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    EaseOutCubic,
}

pub struct StandardAnimation<T: Interpolate + Clone> {
    start_value: T,
    end_value: T,
    duration: Duration,
    start_time: Instant,
    animating: bool,
    easing: Easing,
}

impl<T: Interpolate + Clone> StandardAnimation<T> {
    pub fn new(start_value: T, end_value: T, duration: Duration, easing: Easing) -> Self {
        Self {
            start_value,
            end_value,
            duration,
            start_time: Instant::now(),
            animating: true,
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
            Easing::EaseOutCubic => 1.0 - (1.0 - t).powf(3.0)
        }
    }
}

impl<T: Interpolate + Clone> Animatable for StandardAnimation<T> {
    type Value = T;

    fn is_animating(&self) -> bool {
        self.animating
    }

    fn update(&mut self) -> T {
        if !self.animating {
            return self.end_value.clone();
        }

        let elapsed = self.start_time.elapsed();
        if elapsed >= self.duration {
            self.animating = false;
            return self.end_value.clone();
        }

        let t = self.calculate_time(elapsed.as_secs_f64() / self.duration.as_secs_f64());
        self.start_value.interpolate(&self.end_value, t)
    }
}

impl Interpolate for f64 {
    fn interpolate(&self, end_value: &Self, t: f64) -> Self {
        self + (end_value - self) * t
    }
}
