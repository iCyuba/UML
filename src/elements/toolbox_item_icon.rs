use super::primitives::traits::Draw;
use crate::app::{EventTarget, Renderer, State, Tree};
use crate::elements::primitives::icon::Icon;
use crate::elements::toolbox_item::Tool;
use crate::elements::Element;
use crate::geometry::rect::Rect;
use taffy::prelude::length;
use taffy::{Layout, NodeId, Style};

fn get_icon(tool_type: Tool) -> char {
    match tool_type {
        Tool::Select => 'A',
        Tool::Entity => 'B',
        Tool::Relation => 'C',
    }
}

pub struct ToolboxItemIcon {
    layout: Layout,

    icon: char,
    tool_type: Tool,
}

impl ToolboxItemIcon {
    pub fn setup(tree: &mut Tree, tool_type: Tool, size: f32) -> NodeId {
        let this = Self {
            layout: Default::default(),

            icon: get_icon(tool_type),
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
    fn render(&self, r: &mut Renderer, state: &State) {
        let hitbox = Rect::from(self.layout);
        let icon = Icon::new(
            self.icon,
            hitbox.origin,
            hitbox.size.x as f32,
            if state.tool == self.tool_type {
                r.colors.icon_active
            } else {
                r.colors.icon_inactive
            },
        );
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
