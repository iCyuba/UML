use crate::geometry::Point;
use std::collections::HashSet;
use winit::event::MouseButton;
use winit::keyboard::{Key, NamedKey};

pub struct State {
    #[cfg(target_arch = "wasm32")]
    pub main_modifier: NamedKey,

    // User state
    pub cursor: Point,
    pub keys: HashSet<Key>,
    pub mouse_buttons: HashSet<MouseButton>,
}

impl State {
    pub fn new() -> Self {
        Self {
            #[cfg(target_arch = "wasm32")]
            main_modifier: NamedKey::Control,

            cursor: Default::default(),

            keys: Default::default(),
            mouse_buttons: Default::default(),
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn main_modifier(&self) -> NamedKey {
        self.main_modifier
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[inline]
    pub fn main_modifier(&self) -> NamedKey {
        #[cfg(target_os = "macos")]
        return NamedKey::Super;

        #[cfg(not(target_os = "macos"))]
        return NamedKey::Control;
    }
}
