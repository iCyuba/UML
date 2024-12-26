use super::{Renderer, State};
use crate::geometry::{Point, Vec2};
use winit::event::MouseButton;

pub trait EventTarget {
    // Lifecycle
    fn update(&mut self, state: &mut State) {
        let _ = state;
    }

    fn render(&self, r: &mut Renderer, state: &State);

    // Events

    /// `mousedown` + `mouseup` via the primary mouse button
    fn on_click(&mut self, state: &mut State) -> bool {
        let _ = state;

        false
    }

    /// Fired on the hovered element when the mouse is pressed down.
    fn on_mousedown(&mut self, state: &mut State, button: MouseButton) -> bool {
        let _ = state;
        let _ = button;

        false
    }

    /// Fired when the cursor moves on the currently hovered or focused element.
    fn on_mousemove(&mut self, state: &mut State, cursor: Point) -> bool {
        let _ = state;
        let _ = cursor;

        false
    }

    /// Fired when the mouse is released.
    fn on_mouseup(&mut self, state: &mut State, button: MouseButton) -> bool {
        let _ = state;
        let _ = button;

        false
    }

    /// Fired when the mouse wheel is scrolled.
    fn on_wheel(
        &mut self,
        state: &mut State,
        delta: Vec2,
        mouse: bool,
        zoom: bool,
        reverse: bool,
    ) -> bool {
        let _ = state;
        let _ = delta;
        let _ = mouse;
        let _ = zoom;
        let _ = reverse;

        false
    }
}
