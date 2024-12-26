use super::primitives::traits::Draw;
use crate::animations::animated_property::AnimatedProperty;
use crate::animations::standard_animation::Easing::EaseInOut;
use crate::animations::standard_animation::StandardAnimation;
use crate::app::{EventTarget, Renderer, State, Tree};
use crate::elements::primitives::icon::Icon;
use crate::elements::toolbox_item::Tool;
use crate::elements::Element;
use crate::geometry::rect::Rect;
use derive_macros::AnimatedElement;
use std::time::Duration;
use taffy::prelude::length;
use taffy::{Layout, NodeId, Style};
use vello::peniko::Color;

fn get_icon(tool_type: Tool) -> char {
    match tool_type {
        Tool::Select => 'A',
        Tool::Entity => 'B',
        Tool::Relation => 'C',
    }
}

#[derive(AnimatedElement)]
pub struct ToolboxItemIcon {
    layout: Layout,

    icon: char,
    color: AnimatedProperty<StandardAnimation<Color>>,
    tool_type: Tool,

    active: bool,
    initialized: bool,
}

impl ToolboxItemIcon {
    pub fn setup(tree: &mut Tree, tool_type: Tool, size: f32) -> NodeId {
        let this = Self {
            layout: Default::default(),

            icon: get_icon(tool_type),
            color: AnimatedProperty::new(StandardAnimation::new(
                Default::default(),
                Duration::from_millis(50),
                EaseInOut,
            )),
            tool_type,
            active: false,
            initialized: false,
        };

        tree.new_leaf_with_context(
            Style {
                size: length(size),
                ..Default::default()
            },
            Box::new(this),
        )
        .unwrap()
    }
}

impl EventTarget for ToolboxItemIcon {
    fn update(&mut self, r: &Renderer, state: &mut State) {
        self.active = state.tool == self.tool_type;
        let color = if self.active {
            r.colors.icon_active
        } else {
            r.colors.icon_inactive
        };

        if self.initialized {
            self.color.set(color);
        } else {
            self.color.reset(color);
            self.initialized = true;
        }

        if self.animate() {
            state.request_redraw();
        }
    }

    fn render(&self, r: &mut Renderer, _: &State) {
        let hitbox = Rect::from(self.layout);
        let icon = Icon::new(self.icon, hitbox.origin, hitbox.size.x as f32, *self.color);
        icon.draw(&mut r.scene);
    }
}

impl Element for ToolboxItemIcon {
    fn layout(&self) -> &Layout {
        &self.layout
    }

    fn layout_mut(&mut self) -> &mut Layout {
        &mut self.layout
    }
}
