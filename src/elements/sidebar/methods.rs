use super::list::Countable;
use crate::{
    app::{context::EventContext, EventTarget, Tree},
    data::entity::AccessModifier,
    elements::{
        button::{Button, ButtonProps, ButtonStyle},
        node::ElementWithProps,
        primitives::icon::Symbol,
        segmented_control::{SegmentedControl, SegmentedControlProps},
        sidebar::sidebar_entity,
        text_input::{TextInput, TextInputProps},
        Node,
    },
    presentation::fonts,
};
use taffy::{
    prelude::{auto, length, percent},
    Layout, NodeId, Size, Style,
};

pub struct SidebarMethod(Layout);

impl Countable for SidebarMethod {
    fn count(ctx: &EventContext) -> usize {
        sidebar_entity!(ctx => get)
            .map(|ent| ent.methods.len())
            .unwrap_or(0)
    }
}

impl EventTarget for SidebarMethod {}

impl Node for SidebarMethod {
    fn layout(&self) -> &Layout {
        &self.0
    }

    fn layout_mut(&mut self) -> &mut Layout {
        &mut self.0
    }
}

impl ElementWithProps for SidebarMethod {
    type Props = usize; // idx

    fn setup(tree: &mut Tree, ctx: &mut EventContext, idx: usize) -> NodeId {
        tree.add_element(
            ctx,
            Style {
                max_size: Size {
                    width: percent(1.),
                    height: auto(),
                },
                gap: length(4.),
                ..<_>::default()
            },
            Some(vec![
                // Access modifier
                SegmentedControl::create(SegmentedControlProps {
                    items: vec![
                        (Symbol::Plus, "Public"),
                        (Symbol::Minus, "Private"),
                        (Symbol::Hashtag, "Protected"),
                    ],
                    getter: Box::new(move |ctx| {
                        if let Some(method) =
                            sidebar_entity!(ctx => get).and_then(|e| e.methods.get(idx))
                        {
                            method.modifier as usize
                        } else {
                            0
                        }
                    }),
                    setter: Box::new(move |ctx, index| {
                        if let Some(method) =
                            sidebar_entity!(ctx => get_mut).and_then(|e| e.methods.get_mut(idx))
                        {
                            method.modifier = match index {
                                0 => AccessModifier::Public,
                                1 => AccessModifier::Private,
                                2 => AccessModifier::Protected,
                                _ => return,
                            }
                        }
                    }),
                }),
                // Name
                TextInput::create(TextInputProps {
                    getter: Box::new(move |ctx| {
                        if let Some(method) =
                            sidebar_entity!(ctx => get).and_then(|e| e.methods.get(idx))
                        {
                            format!(
                                "{name}({args}): {ret}",
                                name = method.name,
                                args = method.arguments.join(", "),
                                ret = method.return_type
                            )
                        } else {
                            "".to_string()
                        }
                    }),
                    setter: Box::new(move |ctx, str| {
                        if let Some(method) =
                            sidebar_entity!(ctx => get_mut).and_then(|e| e.methods.get_mut(idx))
                        {
                            let Some((name, args_ret)) = str.split_once('(') else {
                                return;
                            };

                            let Some((args, ret)) = args_ret.split_once("):") else {
                                return;
                            };

                            method.name = name.trim().to_string();
                            method.arguments =
                                args.split(',').map(|s| s.trim().to_string()).collect();
                            method.return_type = ret.trim().to_string();
                        }
                    }),
                    size: 16.,
                    font: fonts::jbmono_regular(),
                    placeholder: None,
                }),
                // Delete button
                Button::create(ButtonProps {
                    tooltip: "Delete method",
                    icon: Symbol::Trash,
                    on_click: Box::new(move |ctx| {
                        if let Some(entity) = sidebar_entity!(ctx => get_mut) {
                            entity.methods.remove(idx);
                            ctx.state.request_redraw();
                        }
                    }),
                    style: ButtonStyle::Segmented,
                }),
            ]),
            |_, _| Self(<_>::default()),
        )
    }
}
