use super::{AppUserEvent, Tree};
use crate::data::project::EntityKey;
use crate::elements::sidebar::SidebarState;
use crate::elements::toolbox_item::Tool;
use crate::elements::tooltip::TooltipState;
use crate::geometry::Point;
use std::collections::HashSet;
use taffy::NodeId;
use winit::event::MouseButton;
use winit::event_loop::EventLoopProxy;
use winit::keyboard::{Key, ModifiersState};

pub struct State {
    #[cfg(target_arch = "wasm32")]
    pub use_super: bool,

    /// Event loop proxy for sending custom events
    pub event_loop: EventLoopProxy<AppUserEvent>,

    // User state
    pub cursor: Point,
    pub modifiers: ModifiersState,
    pub keys: HashSet<Key>,
    pub mouse_buttons: HashSet<MouseButton>,

    // Elements
    pub hovered: Option<NodeId>,
    pub focused: Option<NodeId>,
    pub key_listeners: HashSet<NodeId>,

    // App state
    pub tool: Tool,
    pub selected_entity: Option<EntityKey>,
    pub tooltip_state: Option<TooltipState>,

    // Individual elements' state
    pub sidebar: SidebarState,
}

impl State {
    pub fn new(event_loop: EventLoopProxy<AppUserEvent>) -> Self {
        Self {
            #[cfg(target_arch = "wasm32")]
            use_super: false,

            event_loop,

            cursor: Point::default(),
            modifiers: ModifiersState::default(),
            keys: HashSet::new(),
            mouse_buttons: HashSet::new(),

            hovered: None,
            focused: None,
            key_listeners: HashSet::new(),

            tool: Tool::Select,
            tooltip_state: None,
            selected_entity: None,

            sidebar: <_>::default(),
        }
    }

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

    #[inline]
    pub fn send_event(&self, event: AppUserEvent) {
        self.event_loop.send_event(event).unwrap();
    }

    #[inline]
    pub fn request_redraw(&self) {
        self.send_event(AppUserEvent::RequestRedraw);
    }

    #[inline]
    pub fn request_cursor_update(&self) {
        self.send_event(AppUserEvent::RequestCursorUpdate)
    }

    #[inline]
    pub fn request_tooltip_update(&self) {
        self.send_event(AppUserEvent::RequestTooltipUpdate)
    }

    #[inline]
    pub fn modify_tree(&self, f: impl FnOnce(&mut Tree) + 'static) {
        self.send_event(AppUserEvent::ModifyTree(Box::new(f)));
    }

    #[inline]
    pub fn screenshot(&self) {
        self.send_event(AppUserEvent::Screenshot);
    }
}

impl AsRef<State> for State {
    fn as_ref(&self) -> &State {
        self
    }
}
