use crate::app::{Renderer, State};
use crate::elements::element_style::ElementStyle;
use crate::elements::primitives::fancy_box::FancyBox;
use crate::elements::primitives::traits::Draw;
use crate::elements::toolbox_item_icon::ToolboxItemIcon;
use crate::elements::Element;
use std::iter;
use taffy::prelude::length;
use taffy::{AlignContent, AlignItems, NodeId, Style, TaffyTree};

#[derive(Eq, PartialEq, Hash, Debug, Default, Copy, Clone)]
pub enum Tool {
    #[default]
    Select,
    Entity,
    Relation,
}

pub struct ToolboxItem {
    element_style: ElementStyle,
    node_id: NodeId,

    tool_type: Tool,
    icon: ToolboxItemIcon,
}

impl ToolboxItem {
    pub fn new(flex_tree: &mut TaffyTree, tool_type: Tool) -> Self {
        let icon = ToolboxItemIcon::new(flex_tree, tool_type, 20.);

        Self {
            element_style: Default::default(),
            node_id: flex_tree
                .new_with_children(
                    Style {
                        size: length(32.),
                        justify_content: Some(AlignContent::Center),
                        align_items: Some(AlignItems::Center),
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

    fn get_style(&self) -> &ElementStyle {
        &self.element_style
    }

    fn get_mut_style(&mut self) -> &mut ElementStyle {
        &mut self.element_style
    }

    fn children(&self) -> Box<dyn Iterator<Item = &dyn Element> + '_> {
        Box::new(iter::once(&self.icon as &dyn Element))
    }

    fn children_mut(&mut self) -> Box<dyn Iterator<Item = &mut dyn Element> + '_> {
        Box::new(iter::once(&mut self.icon as &mut dyn Element))
    }

    fn render(&self, r: &mut Renderer, state: &State) {
        FancyBox::new(
            self,
            5.,
            if state.tool == self.tool_type {
                r.colors.accent
            } else {
                r.colors.toolbox_background
            },
            None,
            None,
        )
        .draw(&mut r.scene);

        self.render_children(r, state);
    }
}
