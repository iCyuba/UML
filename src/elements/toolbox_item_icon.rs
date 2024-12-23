use crate::app::{Renderer, State};
use crate::elements::primitives::icon::Icon;
use crate::elements::primitives::traits::Draw;
use crate::elements::toolbox_item::Tool;
use crate::elements::Element;
use taffy::prelude::length;
use taffy::{Layout, NodeId, Style, TaffyTree};

fn get_icon(tool_type: Tool) -> char {
    match tool_type {
        Tool::Select => 'A',
        Tool::Entity => 'A',
        Tool::Relation => 'A',
    }
}

pub struct ToolboxItemIcon {
    layout: Layout,
    node_id: NodeId,

    icon: char,
    tool_type: Tool,
}

impl ToolboxItemIcon {
    pub fn new(flex_tree: &mut TaffyTree, tool_type: Tool, size: f32) -> Self {
        Self {
            layout: Default::default(),
            node_id: flex_tree
                .new_leaf(Style {
                    size: length(size),
                    ..Default::default()
                })
                .unwrap(),
            icon: get_icon(tool_type),
            tool_type,
        }
    }
}

impl Element for ToolboxItemIcon {
    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn get_layout(&self) -> &Layout {
        &self.layout
    }

    fn set_layout(&mut self, layout: Layout) {
        self.layout = layout;
    }

    fn render(&self, r: &mut Renderer, state: &State) {
        let hitbox = self.get_hitbox();
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
