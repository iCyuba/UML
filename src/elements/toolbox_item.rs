use crate::app::{Renderer, State};
use crate::elements::primitives::box_element::BoxElement;
use crate::elements::primitives::traits::Draw;
use crate::elements::toolbox_item_icon::ToolboxItemIcon;
use crate::elements::Element;
use crate::geometry::Point;
use std::iter;
use taffy::prelude::length;
use taffy::{NodeId, Style, TaffyTree};
use vello::kurbo::RoundedRectRadii;

#[derive(Eq, PartialEq, Hash, Debug, Default, Copy, Clone)]
pub enum Tool {
    #[default]
    Select,
    Entity,
    Relation,
}

pub struct ToolboxItem {
    node_id: NodeId,

    tool_type: Tool,
    icon: ToolboxItemIcon,
}

impl ToolboxItem {
    pub fn new(flex_tree: &mut TaffyTree, tool_type: Tool) -> Self {
        let icon = match tool_type {
            Tool::Select => 'A',
            Tool::Entity => 'A',
            Tool::Relation => 'A',
        };

        let icon = ToolboxItemIcon::new(flex_tree, tool_type, 20., icon);

        Self {
            node_id: flex_tree
                .new_with_children(
                    Style {
                        padding: length(6.),
                        ..Default::default()
                    },
                    &[icon.node_id()],
                )
                .unwrap(),
            tool_type,
            icon,
        }
    }
}

impl Element for ToolboxItem {
    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn children(&self) -> Box<dyn Iterator<Item = &dyn Element> + '_> {
        Box::new(iter::once(&self.icon as &dyn Element))
    }

    fn children_mut(&mut self) -> Box<dyn Iterator<Item = &mut dyn Element> + '_> {
        Box::new(iter::once(&mut self.icon as &mut dyn Element))
    }

    fn render(&self, r: &mut Renderer, state: &State, pos: Point) {
        BoxElement::new(
            &self.hitbox(state, pos),
            self.get_layout(state),
            self.get_style(state),
            RoundedRectRadii::from(5.),
            if state.tool == self.tool_type {
                r.colors.accent
            } else {
                r.colors.toolbox_background
            },
            None,
            None,
        )
        .draw(&mut r.scene);

        self.render_children(r, state, pos);
    }
}
