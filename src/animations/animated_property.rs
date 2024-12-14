use crate::animations::delta_animation::DeltaAnimation;
use crate::animations::traits::Animatable;

pub struct AnimatedProperty {
    animation: Option<DeltaAnimation>,
    value: f64,
}

impl AnimatedProperty {
    pub fn new(value: f64) -> Self {
        Self {
            animation: None,
            value,
        }
    }

    pub fn get(&self) -> f64 {
        self.value
    }

    pub fn set(&mut self, value: f64) {
        self.animation = if let Some(animation) = &self.animation {
            Some(animation.with_target_value(value))
        } else {
            Some(DeltaAnimation::new(self.value, value))
        }
    }

    pub fn update(&mut self, difference: f64) {
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

impl From<f64> for AnimatedProperty {
    fn from(value: f64) -> Self {
        Self::new(value)
    }
}

impl Default for AnimatedProperty {
    fn default() -> Self {
        Self::new(0.)
    }
}