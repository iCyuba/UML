use super::primitives::traits::Draw;
use crate::app::{EventTarget, Renderer, State, Tree};
use crate::elements::primitives::fancy_box::FancyBox;
use crate::elements::toolbox_item_icon::ToolboxItemIcon;
use crate::elements::Element;
use taffy::prelude::length;
use taffy::{AlignContent, AlignItems, Layout, NodeId, Style};

#[derive(Eq, PartialEq, Hash, Debug, Default, Copy, Clone)]
pub enum Tool {
    #[default]
    Select,
    Entity,
    Relation,
}

pub struct ToolboxItem {
    layout: Layout,

    tool_type: Tool,
}

impl ToolboxItem {
    pub fn setup(tree: &mut Tree, tool_type: Tool) -> NodeId {
        let icon = ToolboxItemIcon::setup(tree, tool_type, 20.);
        let this = Self {
            layout: Default::default(),
            tool_type,
        };

        let node = tree
            .new_with_children(
                Style {
                    size: length(32.),
                    justify_content: Some(AlignContent::Center),
                    align_items: Some(AlignItems::Center),
                    ..Default::default()
                },
                &[icon],
            )
            .unwrap();

        tree.set_node_context(node, Some(Box::new(this))).unwrap();

        node
    }
}

impl EventTarget for ToolboxItem {
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
    }
}

impl Element for ToolboxItem {
    fn layout(&self) -> &Layout {
        &self.layout
    }

    fn layout_mut(&mut self) -> &mut Layout {
        &mut self.layout
    }
}
