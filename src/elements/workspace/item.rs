use super::Workspace;
use crate::app::{Renderer, State};

/// Something like an element, but inside the workspace.
pub trait Item {
    fn update(&mut self);
    fn render(&self, r: &mut Renderer, state: &State, ws: &Workspace);
}
