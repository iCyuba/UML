use super::context::{EventContext, GetterContext, RenderContext};
use crate::{
    elements::tooltip::TooltipState,
    geometry::{Point, Vec2},
};
use winit::{
    event::{KeyEvent, MouseButton},
    window::CursorIcon,
};

macro_rules! noop {
    ($($arg:tt)*) => {
        _ = ($($arg)*);

        return Default::default();
    };
}

pub(crate) use noop;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WheelEvent {
    pub delta: Vec2,
    pub mouse: bool,
    pub zoom: bool,
    pub reverse: bool,
}

pub trait EventTarget {
    // Lifecycle

    fn update(&mut self, ctx: &mut EventContext) {
        noop!(ctx);
    }

    fn render(&self, ctx: &mut RenderContext) {
        noop!(ctx);
    }

    // Getters

    /// The cursor icon to display when hovering over the element.
    fn cursor(&self, ctx: &GetterContext) -> Option<CursorIcon> {
        noop!(ctx);
    }

    /// The tooltip to display when hovering over the element.
    fn tooltip(&self, ctx: &GetterContext) -> Option<TooltipState> {
        noop!(ctx);
    }

    // Events

    /// `mousedown` + `mouseup` via the primary mouse button
    fn on_click(&mut self, ctx: &mut EventContext) -> bool {
        noop!(ctx);
    }

    /// Fired on a key listener when a key is pressed down.
    ///
    /// The element must be either focused, or in the `key_listeners` set.
    ///
    /// Does not bubble.
    fn on_keydown(&mut self, ctx: &mut EventContext, event: KeyEvent) -> bool {
        noop!(ctx, event);
    }

    /// Fired on a key listener when a key is released.
    ///
    /// The element must be either focused, or in the `key_listeners` set.
    ///
    /// Does not bubble.
    fn on_keyup(&mut self, ctx: &mut EventContext, event: KeyEvent) -> bool {
        noop!(ctx, event);
    }

    /// Fired on the hovered element when the mouse is pressed down.
    fn on_mousedown(&mut self, ctx: &mut EventContext, button: MouseButton) -> bool {
        noop!(ctx, button);
    }

    /// Fired when the cursor enters the hovered element.
    fn on_mouseenter(&mut self, ctx: &mut EventContext) -> bool {
        noop!(ctx);
    }

    /// Fired when the cursor leaves the hovered element.
    fn on_mouseleave(&mut self, ctx: &mut EventContext) -> bool {
        noop!(ctx);
    }

    /// Fired when the cursor moves on the currently hovered or focused element.
    fn on_mousemove(&mut self, ctx: &mut EventContext, cursor: Point) -> bool {
        noop!(ctx, cursor);
    }

    /// Fired when the mouse is released.
    fn on_mouseup(&mut self, ctx: &mut EventContext, button: MouseButton) -> bool {
        noop!(ctx, button);
    }

    /// Fired when the mouse wheel is scrolled.
    fn on_wheel(&mut self, ctx: &mut EventContext, event: WheelEvent) -> bool {
        noop!(ctx, event);
    }
}
