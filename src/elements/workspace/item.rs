use super::Workspace;
use crate::{
    app::{event_target::noop, renderer::Canvas, State},
    geometry::Point,
};
use winit::event::MouseButton;

/// Something like an element, but inside the workspace.
pub trait Item {
    fn update(&mut self) -> bool; // Returns true if the item needs to be re-rendered
    fn render(&self, c: &mut Canvas, state: &State, ws: &Workspace);

    fn on_mousedown(&mut self, state: &mut State, button: MouseButton) -> bool {
        noop!(state, button);
    }

    fn on_mousemove(&mut self, state: &mut State, cursor: Point) -> bool {
        noop!(state, cursor);
    }

    fn on_mouseup(&mut self, state: &mut State, button: MouseButton) -> bool {
        noop!(state, button);
    }

    fn on_mouseenter(&mut self, state: &mut State) -> bool {
        noop!(state);
    }

    fn on_mouseleave(&mut self, state: &mut State) -> bool {
        noop!(state);
    }
}
