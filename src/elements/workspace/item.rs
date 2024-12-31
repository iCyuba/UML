use super::Workspace;
use crate::app::{renderer::Canvas, State};

/// Something like an element, but inside the workspace.
pub trait Item {
    fn update(&mut self) -> bool; // Returns true if the item needs to be re-rendered
    fn render(&self, c: &mut Canvas, state: &State, ws: &Workspace);
}
