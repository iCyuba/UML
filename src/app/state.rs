use crate::geometry::Point;
use std::collections::HashSet;
use winit::event::MouseButton;
use winit::keyboard::{Key, ModifiersState};

#[derive(Debug, Default)]
pub struct State {
    #[cfg(target_arch = "wasm32")]
    pub use_super: bool,

    // Internal state
    pub redraw: bool,

    // User state
    pub cursor: Point,
    pub modifiers: ModifiersState,
    pub keys: HashSet<Key>,
    pub mouse_buttons: HashSet<MouseButton>,
}

impl State {
    #[inline]
    pub fn main_modifier(&self) -> bool {
        #[cfg(target_os = "macos")]
        return self.modifiers.super_key();

        #[cfg(target_arch = "wasm32")]
        return if self.use_super {
            self.modifiers.super_key()
        } else {
            self.modifiers.control_key()
        };

        #[cfg(all(not(target_os = "macos"), not(target_arch = "wasm32")))]
        return self.modifiers.control_key();
    }
}
