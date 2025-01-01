#![allow(dead_code)]

use crate::animations::traits::Animatable;
use std::ops::Deref;

#[derive(Debug)]
pub struct AnimatedProperty<A: Animatable> {
    animation: A,
}

impl<A: Animatable> AnimatedProperty<A> {
    pub fn new(animatable: A) -> Self {
        Self {
            animation: animatable,
        }
    }

    pub fn get(&self) -> &A::Value {
        self.animation.current_value()
    }

    pub fn get_target(&self) -> &A::Value {
        self.animation.get_target()
    }

    // Returns true if the property was initialized for the first time -> thus needs to be redrawn
    fn initialize(&mut self, value: A::Value) -> bool {
        let initialized = self.animation.is_initialized();
        if !initialized {
            self.animation.initialize(value);
        }

        !initialized
    }

    // Returns true if the value was changed
    pub fn set(&mut self, value: A::Value) -> bool {
        if !self.initialize(value) && value != *self.animation.get_target() {
            self.animation.continue_animation();
            self.animation.set_target(value);
            true
        } else {
            false
        }
    }

    // Returns true if the value was changed
    pub fn reset(&mut self, value: A::Value) -> bool {
        if self.initialize(value) {
            return true;
        }

        self.animation.stop_animation();

        if value != *self.animation.get_target() {
            self.animation.set_target(value);
            true
        } else {
            false
        }
    }

    // Returns true if the property is currently animating
    pub fn animate(&mut self) -> bool {
        self.animation.update();
        self.animation.is_animating()
    }
}

impl<A: Animatable> Deref for AnimatedProperty<A> {
    type Target = A::Value;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}
