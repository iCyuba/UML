pub trait Interpolate {
    fn interpolate(&self, end_value: &Self, t: f64) -> Self;
}

pub trait Animatable<T> {
    fn is_animating(&self) -> bool;
    fn update(&mut self) -> T;
}