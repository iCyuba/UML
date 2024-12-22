use crate::app::{Renderer, State};
use crate::elements::primitives::icon::Icon;
use crate::elements::primitives::traits::Draw;
use crate::elements::toolbox_item::Tool;
use crate::elements::Element;
use crate::geometry::rect::Rect;
use crate::geometry::Point;
use taffy::prelude::length;
use taffy::{NodeId, Style, TaffyTree};

pub struct ToolboxItemIcon {
    icon: char,

    tool_type: Tool,
    node_id: NodeId,
}

impl ToolboxItemIcon {
    pub fn new(flex_tree: &mut TaffyTree, tool_type: Tool, size: f32, icon: char) -> Self {
        Self {
            node_id: flex_tree
                .new_leaf(Style {
                    size: length(size),
                    ..Default::default()
                })
                .unwrap(),
            icon,
            tool_type,
        }
    }
}

impl Element for ToolboxItemIcon {
    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn render(&self, r: &mut Renderer, state: &State, pos: Point) {
        let Rect { size, origin } = self.hitbox(state, pos);
        let icon = Icon::new(
            self.icon,
            origin,
            size.x as f32,
            if state.tool == self.tool_type {
                r.colors.icon_active
            } else {
                r.colors.icon_inactive
            },
        );
        icon.draw(&mut r.scene);
    }
}
