use crate::animations::traits::Animatable;

#[derive(Clone)]
pub struct DeltaAnimation {
    current_value: f64,
    target_value: f64,
    velocity: f64,
    animating: bool,
}

impl DeltaAnimation {
    pub fn new(current_value: f64, target_value: f64) -> Self {
        Self {
            current_value,
            target_value,
            velocity: 1.,
            animating: true,
        }
    }

    pub fn with_target_value(&self, target_value: f64) -> Self {
        let prev_direction = (self.target_value - self.current_value).signum();
        let new_direction = (target_value - self.current_value).signum();

        Self {
            target_value,
            animating: true,
            velocity: if prev_direction == new_direction { self.velocity } else { 1. },
            current_value: self.current_value,
        }
    }
}

impl Animatable<f64> for DeltaAnimation {
    fn is_animating(&self) -> bool {
        self.animating
    }

    fn update(&mut self) -> f64 {
        if !self.animating {
            return self.target_value.clone();
        }

        let step = 0.1 * (self.target_value - self.current_value);
        self.velocity *= 1.01;

        if step.abs() > 0.0 {
            self.current_value += self.velocity * step;
            self.current_value
        } else {
            self.animating = false;
            self.target_value.clone()
        }
    }
}