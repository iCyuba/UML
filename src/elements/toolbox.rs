use crate::app::{Renderer, State};
use crate::elements::element_style::ElementStyle;
use crate::elements::primitives::fancy_box::{BorderOptions, FancyBox, ShadowOptions};
use crate::elements::primitives::traits::Draw;
use crate::elements::toolbox_item::{Tool, ToolboxItem};
use crate::elements::Element;
use std::collections::HashMap;
use taffy::prelude::length;
use taffy::Display::Flex;
use taffy::FlexDirection::Column;
use taffy::{NodeId, Style, TaffyTree};

pub struct Toolbox {
    element_style: ElementStyle,
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
            padding: length(8.),
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
            element_style: Default::default(),
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

    fn get_style(&self) -> &ElementStyle {
        &self.element_style
    }

    fn get_mut_style(&mut self) -> &mut ElementStyle {
        &mut self.element_style
    }

    fn children(&self) -> Box<dyn Iterator<Item = &dyn Element> + '_> {
        Box::new(self.tools.values().map(|item| item as &dyn Element))
    }

    fn children_mut(&mut self) -> Box<dyn Iterator<Item = &mut dyn Element> + '_> {
        Box::new(self.tools.values_mut().map(|item| item as &mut dyn Element))
    }

    fn render(&self, r: &mut Renderer, state: &State) {
        FancyBox::new(
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

        self.render_children(r, state);
    }
}
