use super::primitives::icon::Symbol;
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
use crate::data::Project;

fn get_icon(tool_type: Tool) -> Symbol {
    match tool_type {
        Tool::Select => Symbol::Cursor,
        Tool::Hand => Symbol::Hand,
        Tool::Entity => Symbol::PlusSquare,
        Tool::Relation => Symbol::Workflow,
    }
}

#[derive(AnimatedElement)]
pub struct ToolboxItemIcon {
    layout: Layout,

    icon: Symbol,
    color: AnimatedProperty<StandardAnimation<Color>>,
    tool_type: Tool,
}

impl ToolboxItemIcon {
    pub fn setup(tree: &mut Tree, _: &mut State, tool_type: Tool, size: f32) -> NodeId {
        let this = Self {
            layout: Default::default(),

            icon: get_icon(tool_type),
            color: AnimatedProperty::new(StandardAnimation::new(
                Duration::from_millis(50),
                EaseInOut,
            )),
            tool_type,
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
    fn update(&mut self, r: &Renderer, state: &mut State, _: &mut Project) {
        self.color.set(if state.tool == self.tool_type {
            r.colors.icon_active
        } else {
            r.colors.icon_inactive
        });

        if self.animate() {
            state.request_redraw();
        }
    }

    fn render(&self, r: &mut Renderer, _: &State, _: &Project) {
        let hitbox = Rect::from(self.layout);
        let icon = Icon::new(self.icon, hitbox, hitbox.size.x, *self.color);
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
