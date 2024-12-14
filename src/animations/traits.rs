pub trait Interpolate {
    fn interpolate(&self, end_value: &Self, t: f64) -> Self;
}

pub trait Animatable {
    type Value;

    fn is_animating(&self) -> bool;
    fn update(&mut self) -> Self::Value;
}
