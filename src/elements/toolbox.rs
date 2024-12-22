use crate::app::{Renderer, State};
use crate::elements::primitives::box_element::{BoxElement, ShadowOptions};
use crate::elements::primitives::traits::Draw;
use crate::elements::toolbox_item::{Tool, ToolboxItem};
use crate::elements::Element;
use crate::geometry::Point;
use std::collections::HashMap;
use taffy::prelude::length;
use taffy::Display::Flex;
use taffy::FlexDirection::Column;
use taffy::{NodeId, Style, TaffyTree};
use vello::kurbo::RoundedRectRadii;

pub struct Toolbox {
    node_id: NodeId,

    tools: HashMap<Tool, ToolboxItem>,
}

impl Toolbox {
    pub fn new(flex_tree: &mut TaffyTree) -> Self {
        let style = Style {
            display: Flex,
            flex_direction: Column,
            border: length(1.),
            margin: length(12.),
            padding: length(6.),
            gap: length(8.),
            align_self: Some(taffy::AlignSelf::Center),
            ..Default::default()
        };

        let selection_tool = ToolboxItem::new(flex_tree, Tool::Select);
        let entity_tool = ToolboxItem::new(flex_tree, Tool::Entity);
        let relation_tool = ToolboxItem::new(flex_tree, Tool::Relation);

        let node_id = flex_tree
            .new_with_children(
                style,
                &[
                    selection_tool.node_id(),
                    entity_tool.node_id(),
                    relation_tool.node_id(),
                ],
            )
            .unwrap();

        Self {
            node_id,
            tools: [
                (Tool::Select, selection_tool),
                (Tool::Entity, entity_tool),
                (Tool::Relation, relation_tool),
            ]
            .into_iter()
            .collect(),
        }
    }
}

impl Element for Toolbox {
    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn children(&self) -> Box<dyn Iterator<Item = &dyn Element> + '_> {
        Box::new(self.tools.values().map(|item| item as &dyn Element))
    }

    fn children_mut(&mut self) -> Box<dyn Iterator<Item = &mut dyn Element> + '_> {
        Box::new(self.tools.values_mut().map(|item| item as &mut dyn Element))
    }

    fn render(&self, r: &mut Renderer, state: &State, pos: Point) {
        BoxElement::new(
            &self.hitbox(state, pos),
            self.get_layout(state),
            self.get_style(state),
            RoundedRectRadii::from(11.),
            r.colors.toolbox_background,
            Some(r.colors.toolbox_border),
            Some(ShadowOptions {
                color: r.colors.drop_shadow,
                offset: (0., 1.).into(),
                blur_radius: 5.,
            }),
        )
        .draw(&mut r.scene);

        self.render_children(r, state, pos);
    }
}
