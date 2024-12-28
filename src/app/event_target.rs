use super::{Renderer, State};
use crate::data::Project;
use crate::geometry::{Point, Vec2};
use winit::{event::MouseButton, keyboard::Key, window::CursorIcon};
use crate::elements::tooltip::TooltipState;

pub trait EventTarget {
    // Lifecycle

    fn update(&mut self, r: &Renderer, state: &mut State, project: &mut Project) {
        _ = (r, state, project);
    }

    fn render(&self, r: &mut Renderer, state: &State, project: &Project);

    // Getters

    /// The cursor icon to display when hovering over the element.
    fn cursor(&self, state: &State) -> Option<CursorIcon> {
        _ = state;

        None
    }

    /// The tooltip to display when hovering over the element.
    fn tooltip(&self, state: &State) -> Option<TooltipState> {
        _ = state;

        None
    }

    // Events

    /// `mousedown` + `mouseup` via the primary mouse button
    fn on_click(&mut self, state: &mut State, project: &mut Project) -> bool {
        _ = (state, project);

        false
    }

    /// Fired on a key listener when a key is pressed down.
    ///
    /// The element must be either focused, or in the `key_listeners` set.
    ///
    /// Does not bubble.
    fn on_keydown(&mut self, state: &mut State, project: &mut Project, key: &Key) -> bool {
        _ = (state, key, project);

        false
    }

    /// Fired on a key listener when a key is released.
    ///
    /// The element must be either focused, or in the `key_listeners` set.
    ///
    /// Does not bubble.
    fn on_keyup(&mut self, state: &mut State, project: &mut Project, key: &Key) -> bool {
        _ = (state, key, project);

        false
    }

    /// Fired on the hovered element when the mouse is pressed down.
    fn on_mousedown(
        &mut self,
        state: &mut State,
        project: &mut Project,
        button: MouseButton,
    ) -> bool {
        _ = (state, button, project);

        false
    }

    /// Fired when the cursor enters the hovered element.
    fn on_mouseenter(&mut self, state: &mut State, project: &mut Project) -> bool {
        _ = (state, project);

        false
    }

    /// Fired when the cursor leaves the hovered element.
    fn on_mouseleave(&mut self, state: &mut State, project: &mut Project) -> bool {
        _ = (state, project);

        false
    }

    /// Fired when the cursor moves on the currently hovered or focused element.
    fn on_mousemove(&mut self, state: &mut State, project: &mut Project, cursor: Point) -> bool {
        _ = (state, cursor, project);

        false
    }

    /// Fired when the mouse is released.
    fn on_mouseup(
        &mut self,
        state: &mut State,
        project: &mut Project,
        button: MouseButton,
    ) -> bool {
        _ = (state, button, project);

        false
    }

    /// Fired when the mouse wheel is scrolled.
    fn on_wheel(
        &mut self,
        state: &mut State,
        project: &mut Project,
        delta: Vec2,
        mouse: bool,
        zoom: bool,
        reverse: bool,
    ) -> bool {
        _ = (state, project, delta, mouse, zoom, reverse);

        false
    }
}
