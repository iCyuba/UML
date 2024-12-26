use super::primitives::traits::Draw;
use crate::app::{EventTarget, Renderer, State, Tree};
use crate::elements::primitives::fancy_box::{BorderOptions, FancyBox, ShadowOptions};
use crate::elements::toolbox_item::{Tool, ToolboxItem};
use crate::elements::Element;
use taffy::prelude::length;
use taffy::Display::Flex;
use taffy::FlexDirection::Column;
use taffy::{Layout, NodeId, Style};

pub struct Toolbox(Layout);

impl Toolbox {
    pub fn setup(tree: &mut Tree) -> NodeId {
        let style = Style {
            display: Flex,
            flex_direction: Column,
            border: length(1.),
            margin: length(12.),
            padding: length(8.),
            gap: length(8.),
            align_self: Some(taffy::AlignSelf::Center),
            ..Default::default()
        };

        let selection_tool = ToolboxItem::setup(tree, Tool::Select);
        let entity_tool = ToolboxItem::setup(tree, Tool::Entity);
        let relation_tool = ToolboxItem::setup(tree, Tool::Relation);

        let node = tree
            .new_with_children(style, &[selection_tool, entity_tool, relation_tool])
            .unwrap();

        tree.set_node_context(node, Some(Box::new(Self(Layout::new()))))
            .unwrap();

        node
    }
}

impl EventTarget for Toolbox {
    fn render(&self, r: &mut Renderer, _: &State) {
        FancyBox::new(
            r.scale(),
            self,
            13.,
            r.colors.toolbox_background,
            Some(BorderOptions {
                color: r.colors.toolbox_border,
            }),
            Some(ShadowOptions {
                color: r.colors.drop_shadow,
                offset: (0., 1.).into(),
                blur_radius: 5.,
            }),
        )
        .draw(&mut r.scene);
    }
}

impl Element for Toolbox {
    fn layout(&self) -> &Layout {
        &self.0
    }

    fn layout_mut(&mut self) -> &mut Layout {
        &mut self.0
    }
}
