use std::ops::{Deref, DerefMut};
use crate::animations::delta_animation::DeltaAnimation;
use crate::animations::traits::{Animatable, Numeric};

pub struct AnimatedProperty<T: Numeric> {
    animation: Option<DeltaAnimation<T>>,
    value: T,
}

impl<T: Numeric> AnimatedProperty<T> {
    pub fn new(value: T) -> Self {
        Self {
            animation: None,
            value,
        }
    }

    pub fn get(&self) -> T {
        self.value
    }

    pub fn set(&mut self, value: T) {
        self.animation = if let Some(animation) = &self.animation {
            Some(animation.with_target_value(value))
        } else {
            Some(DeltaAnimation::new(self.value, value))
        }
    }

    pub fn update(&mut self, difference: T) {
        self.set(self.value + difference);
    }

    pub fn animate(&mut self) -> bool {
        if let Some(animation) = &mut self.animation {
            self.value = animation.update();

            if !animation.is_animating() {
                self.animation = None;
            }
        }

        self.animation.is_some()
    }
}

impl<T: Numeric> From<T> for AnimatedProperty<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T: Numeric> Deref for AnimatedProperty<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: Numeric> DerefMut for AnimatedProperty<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.animation = None;
        &mut self.value
    }
}