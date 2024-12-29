use super::{
    primitives::{
        icon::{Icon, Symbol},
        traits::Draw,
    },
    toolbox_item::Tool,
    Element,
};
use crate::{
    animations::{
        animated_property::AnimatedProperty,
        standard_animation::{Easing::EaseInOut, StandardAnimation},
    },
    app::{
        context::{EventContext, RenderContext},
        EventTarget, Tree,
    },
    geometry::rect::Rect,
};
use derive_macros::AnimatedElement;
use std::time::Duration;
use taffy::{prelude::length, Layout, NodeId, Style};
use vello::peniko::Color;

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
    pub fn setup(tree: &mut Tree, _: &mut EventContext, tool_type: Tool, size: f32) -> NodeId {
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
    fn update(&mut self, ctx: &mut EventContext) {
        self.color.set(if ctx.state.tool == self.tool_type {
            ctx.c.colors().icon_active
        } else {
            ctx.c.colors().icon_inactive
        });

        if self.animate() {
            ctx.state.request_redraw();
        }
    }

    fn render(&self, RenderContext { c, .. }: &mut RenderContext) {
        let hitbox = Rect::from(self.layout);
        let icon = Icon::new(self.icon, hitbox, hitbox.size.x, *self.color);
        icon.draw(c);
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
