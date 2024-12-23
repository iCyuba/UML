use crate::app::{Renderer, State};
use crate::elements::element_style::ElementStyle;
use crate::elements::primitives::icon::Icon;
use crate::elements::primitives::traits::Draw;
use crate::elements::toolbox_item::Tool;
use crate::elements::Element;
use crate::geometry::rect::Rect;
use taffy::prelude::length;
use taffy::{NodeId, Style, TaffyTree};

fn get_icon(tool_type: Tool) -> char {
    match tool_type {
        Tool::Select => 'A',
        Tool::Entity => 'A',
        Tool::Relation => 'A',
    }
}

pub struct ToolboxItemIcon {
    element_style: ElementStyle,
    node_id: NodeId,

    icon: char,
    tool_type: Tool,
}

impl ToolboxItemIcon {
    pub fn new(flex_tree: &mut TaffyTree, tool_type: Tool, size: f32) -> Self {
        Self {
            element_style: Default::default(),
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

    fn get_style(&self) -> &ElementStyle {
        &self.element_style
    }

    fn get_mut_style(&mut self) -> &mut ElementStyle {
        &mut self.element_style
    }

    fn render(&self, r: &mut Renderer, state: &State) {
        let Rect { size, origin } = self.get_hitbox();
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
