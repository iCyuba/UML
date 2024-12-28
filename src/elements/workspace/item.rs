use super::Workspace;
use crate::app::{Renderer, State};
use crate::geometry::Point;
use winit::event::MouseButton;

/// Something like an element, but inside the workspace.
pub trait Item {
    fn update(&mut self);
    fn render(&self, r: &mut Renderer, state: &State, ws: &Workspace);
    fn on_mousedown(&mut self, state: &mut State, button: MouseButton) -> bool;
    fn on_mousemove(&mut self, state: &mut State, cursor: Point) -> bool;
    fn on_mouseup(&mut self, state: &mut State, _: MouseButton) -> bool;
    fn on_mouseenter(&mut self, state: &mut State) -> bool;
    fn on_mouseleave(&mut self, state: &mut State) -> bool;
}
