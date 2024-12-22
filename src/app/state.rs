use crate::geometry::{Point, Size};
use std::collections::HashSet;
use taffy::TaffyTree;
use winit::event::MouseButton;
use winit::keyboard::{Key, ModifiersState};
use crate::elements::toolbox_item::Tool;

#[derive(Debug, Default)]
pub struct State {
    #[cfg(target_arch = "wasm32")]
    pub use_super: bool,

    // Shared
    pub flex_tree: TaffyTree<()>,

    // Internal state
    pub redraw: bool,
    pub size: Size,

    // User state
    pub cursor: Point,
    pub modifiers: ModifiersState,
    pub keys: HashSet<Key>,
    pub mouse_buttons: HashSet<MouseButton>,
    pub tool: Tool,
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
