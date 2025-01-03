use super::list::Countable;
use crate::{
    app::{context::EventContext, EventTarget, Tree},
    elements::{
        button::{Button, ButtonProps, ButtonStyle},
        node::ElementWithProps,
        primitives::icon::Symbol,
        sidebar::sidebar_entity,
        text_element::{TextElement, TextElementProps},
        Node,
    },
    presentation::fonts,
};
use taffy::{JustifyContent, Layout, LengthPercentage, NodeId, Size, Style};

pub struct SidebarImplementation(Layout);

impl SidebarImplementation {
    const STYLE: Style = Style {
        justify_content: Some(JustifyContent::Start),
        gap: Size {
            width: LengthPercentage::Length(4.),
            height: LengthPercentage::Length(0.),
        },
        ..Style::DEFAULT
    };
}

impl Countable for SidebarImplementation {
    fn count(ctx: &EventContext) -> usize {
        sidebar_entity!(ctx => get)
            .map(|ent| ent.implements.len())
            .unwrap_or(0)
    }
}

impl EventTarget for SidebarImplementation {}

impl Node for SidebarImplementation {
    fn layout(&self) -> &Layout {
        &self.0
    }

    fn layout_mut(&mut self) -> &mut Layout {
        &mut self.0
    }
}

impl ElementWithProps for SidebarImplementation {
    type Props = usize; // idx

    fn setup(tree: &mut Tree, ctx: &mut EventContext, idx: usize) -> NodeId {
        tree.add_element(
            ctx,
            Self::STYLE,
            Some(vec![
                // Label
                TextElement::create(TextElementProps {
                    getter: Box::new(move |ctx| {
                        let Some(ent) = sidebar_entity!(ctx => get) else {
                            return "".to_string();
                        };

                        let Some(conn) = ent
                            .implements
                            .get(idx)
                            .and_then(|key| ctx.project.connections.get(*key))
                        else {
                            return "".to_string();
                        };

                        let other = conn.other(ent.key);
                        let other = ctx.project.entities.get(other.entity).unwrap();

                        other.name.clone()
                    }),
                    size: 16.,
                    font: fonts::jbmono_regular(),
                }),
                // Remove button
                Button::create(ButtonProps {
                    tooltip: "Remove implementation",
                    icon: Symbol::Trash,
                    on_click: Box::new(move |ctx| {
                        if let Some(key) =
                            sidebar_entity!(ctx => get).and_then(|ent| ent.implements.get(idx))
                        {
                            ctx.project.disconnect(*key);
                            ctx.state.request_tooltip_update();
                        };
                    }),
                    style: ButtonStyle::Segmented,
                }),
            ]),
            |_, _| Self(<_>::default()),
        )
    }
}
