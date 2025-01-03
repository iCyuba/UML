use super::list::Countable;
use crate::{
    app::{context::EventContext, EventTarget, Tree},
    data::connection::{Multiplicity, RelationType},
    elements::{
        button::{Button, ButtonProps, ButtonStyle},
        node::ElementWithProps,
        primitives::icon::Symbol,
        segmented_control::{SegmentedControl, SegmentedControlProps},
        sidebar::sidebar_entity,
        text_element::{TextElement, TextElementProps},
        Node,
    },
    presentation::fonts,
};
use taffy::{
    prelude::{auto, length, percent},
    JustifyContent, Layout, NodeId, Size, Style,
};

pub struct SidebarConnection(Layout);

impl Countable for SidebarConnection {
    fn count(ctx: &EventContext) -> usize {
        sidebar_entity!(ctx => get)
            .map(|ent| ent.connections.len())
            .unwrap_or(0)
    }
}

macro_rules! get_connection {
    ($ctx: expr, $idx: expr => $get: ident) => {
        {
            if let Some(ent) = sidebar_entity!($ctx => get) {
                if let Some(conn) = ent.connections.get_index($idx).and_then(|idx| $ctx.project.connections.$get(*idx)) {
                    Some((ent, conn))
                } else {
                    None
                }
            } else {
                None
            }
        }
    };
}

impl EventTarget for SidebarConnection {}

impl Node for SidebarConnection {
    fn layout(&self) -> &Layout {
        &self.0
    }

    fn layout_mut(&mut self) -> &mut Layout {
        &mut self.0
    }
}

impl ElementWithProps for SidebarConnection {
    type Props = usize; // idx

    fn setup(tree: &mut Tree, ctx: &mut EventContext, idx: usize) -> NodeId {
        tree.add_element(
            ctx,
            Style {
                max_size: Size {
                    width: percent(1.),
                    height: auto(),
                },
                justify_content: Some(JustifyContent::Start),
                gap: length(4.),
                ..<_>::default()
            },
            Some(vec![
                // Type
                SegmentedControl::create(SegmentedControlProps {
                    items: vec![
                        (Symbol::Association, "Association"),
                        (Symbol::Aggregation, "Aggregation"),
                        (Symbol::Composition, "Composition"),
                    ],
                    getter: Box::new(move |ctx| {
                        get_connection!(ctx, idx => get)
                            .map(|(_, conn)| match conn.relation {
                                RelationType::Association => 0,
                                RelationType::Aggregation => 1,
                                RelationType::Composition => 2,
                                _ => 0,
                            })
                            .unwrap_or(0)
                    }),
                    setter: Box::new(move |ctx, index| {
                        if let Some((_, conn)) = get_connection!(ctx, idx => get_mut) {
                            conn.relation = match index {
                                0 => RelationType::Association,
                                1 => RelationType::Aggregation,
                                2 => RelationType::Composition,
                                _ => return,
                            };
                        }
                    }),
                }),
                // Multiplicity
                SegmentedControl::create(SegmentedControlProps {
                    items: vec![(Symbol::One, "One"), (Symbol::Many, "Many")],
                    getter: Box::new(move |ctx| {
                        let Some((ent, conn)) = get_connection!(ctx, idx => get) else {
                            return 0;
                        };

                        let other = conn.other(ent.key);

                        match other.multiplicity {
                            Multiplicity::One => 0,
                            Multiplicity::Many => 1,
                        }
                    }),
                    setter: Box::new(move |ctx, index| {
                        let Some((ent, conn)) = get_connection!(ctx, idx => get_mut) else {
                            return;
                        };

                        let other = conn.other_mut(ent.key);

                        match index {
                            0 => other.multiplicity = Multiplicity::One,
                            1 => other.multiplicity = Multiplicity::Many,
                            _ => unreachable!(),
                        }
                    }),
                }),
                // Label
                TextElement::create(TextElementProps {
                    getter: Box::new(move |ctx| {
                        let Some((ent, conn)) = get_connection!(ctx, idx => get) else {
                            return "".to_string();
                        };

                        let other = conn.other(ent.key);
                        let other = ctx.project.entities.get(other.entity).unwrap();

                        other.name.clone()
                    }),
                    size: 16.,
                    font: fonts::jbmono_regular(),
                }),
                // Swap button
                Button::create(ButtonProps {
                    tooltip: "Swap sides",
                    icon: Symbol::Swap,
                    on_click: Box::new(move |ctx| {
                        let Some((_, conn)) = get_connection!(ctx, idx => get_mut) else {
                            return;
                        };

                        conn.swap();
                        ctx.state.request_redraw();
                    }),
                    style: ButtonStyle::Segmented,
                }),
                // Delete button
                Button::create(ButtonProps {
                    tooltip: "Delete relation",
                    icon: Symbol::Trash,
                    on_click: Box::new(move |ctx| {
                        if let Some(key) = sidebar_entity!(ctx => get)
                            .and_then(|ent| ent.connections.get_index(idx))
                        {
                            ctx.project.disconnect(*key);
                            ctx.state.request_redraw();
                        };
                    }),
                    style: ButtonStyle::Segmented,
                }),
            ]),
            |_, _| Self(<_>::default()),
        )
    }
}
