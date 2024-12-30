use super::{
    node::ElementWithProps,
    primitives::{
        fancy_box::{BorderOptions, FancyBox, ShadowOptions},
        traits::Draw,
    },
    text_element::{TextElement, TextElementProps},
    Node,
};
use crate::{
    app::{
        context::{EventContext, RenderContext},
        EventTarget, Tree,
    },
    elements::node::Element,
    presentation::fonts,
};
use taffy::{
    prelude::{auto, length},
    Display::Flex,
    FlexDirection::Column,
    Layout, NodeId, Size, Style,
};

pub struct Sidebar {
    layout: Layout,
}

impl EventTarget for Sidebar {
    fn render(&self, RenderContext { c, state, .. }: &mut RenderContext) {
        if state.selected_entity.is_none() {
            return;
        }

        FancyBox::from_node(
            self,
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
        .draw(c);
    }
}

impl Node for Sidebar {
    fn layout(&self) -> &Layout {
        &self.layout
    }

    fn layout_mut(&mut self) -> &mut Layout {
        &mut self.layout
    }
}

impl Element for Sidebar {
    fn setup(tree: &mut Tree, ctx: &mut EventContext) -> NodeId {
        tree.add_element(
            ctx,
            Style {
                display: Flex,
                flex_direction: Column,
                border: length(1.),
                margin: length(12.),
                padding: length(16.),
                gap: length(8.),
                size: Size {
                    width: length(300.),
                    height: auto(),
                },
                ..Default::default()
            },
            Some(vec![TextElement::create(TextElementProps {
                text: Box::new(|ctx| {
                    if let Some(selected_entity) = ctx.state.selected_entity {
                        let entity = ctx.project.entities.get(selected_entity).unwrap();
                        entity.name.clone()
                    } else {
                        "".to_string()
                    }
                }),
                size: 24.,
                font: fonts::jbmono_bold(),
            })]),
            |_, _| Self {
                layout: Default::default(),
            },
        )
    }
}
