use super::{Renderer, State};
use crate::geometry::{Point, Vec2};

pub trait EventTarget {
    // Lifecycle
    fn update(&mut self, state: &mut State) {
        let _ = state;
    }

    fn render(&self, r: &mut Renderer, state: &State);

    // Events
    fn on_scroll(
        &mut self,
        state: &mut State,
        delta: Vec2,
        mouse: bool,
        zoom: bool,
        reverse: bool,
    ) {
        let _ = state;
        let _ = delta;
        let _ = mouse;
        let _ = zoom;
        let _ = reverse;
    }

    fn on_mousemove(&mut self, state: &mut State, cursor: Point) {
        let _ = state;
        let _ = cursor;
    }
}
