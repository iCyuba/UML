use super::primitives::traits::Draw;
use crate::app::context::{EventContext, RenderContext};
use crate::app::{EventTarget, Tree};
use crate::elements::primitives::fancy_box::{BorderOptions, FancyBox, ShadowOptions};
use crate::elements::toolbox_item::{Tool, ToolboxItem};
use crate::elements::Element;
use taffy::prelude::length;
use taffy::Display::Flex;
use taffy::FlexDirection::Column;
use taffy::{Layout, NodeId, Style};

pub struct Toolbox(Layout);

impl Toolbox {
    pub fn setup(tree: &mut Tree, ctx: &mut EventContext) -> NodeId {
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

        let selection_tool = ToolboxItem::setup(tree, ctx, Tool::Select);
        let hand_tool = ToolboxItem::setup(tree, ctx, Tool::Hand);
        let entity_tool = ToolboxItem::setup(tree, ctx, Tool::Entity);
        let relation_tool = ToolboxItem::setup(tree, ctx, Tool::Relation);

        let node = tree
            .new_with_children(
                style,
                &[selection_tool, hand_tool, entity_tool, relation_tool],
            )
            .unwrap();

        tree.set_node_context(node, Some(Box::new(Self(Layout::new()))))
            .unwrap();

        node
    }
}

impl EventTarget for Toolbox {
    fn render(&self, RenderContext { c, .. }: &mut RenderContext) {
        FancyBox::from_element(
            self,
            c.scale(),
            13.,
            c.colors().floating_background,
            Some(BorderOptions {
                color: c.colors().border,
            }),
            Some(ShadowOptions {
                color: c.colors().drop_shadow,
                offset: (0., 1.).into(),
                blur_radius: 5.,
            }),
        )
        .draw(c.scene());
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
