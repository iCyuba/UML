#![allow(dead_code)]

use crate::animations::traits::Animatable;
use std::ops::Deref;

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

    pub fn set(&mut self, value: A::Value) {
        self.animation.continue_animation();
        self.animation.set_target(value);
    }

    pub fn reset(&mut self, value: A::Value) {
        self.animation.stop_animation();
        self.animation.set_target(value);
    }

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
