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
    JustifyContent, Layout, NodeId, Size, Style,
};

pub struct SidebarField(Layout);

impl Countable for SidebarField {
    fn count(ctx: &EventContext) -> usize {
        sidebar_entity!(ctx => get)
            .map(|ent| ent.fields.len())
            .unwrap_or(0)
    }
}

impl EventTarget for SidebarField {}

impl Node for SidebarField {
    fn layout(&self) -> &Layout {
        &self.0
    }

    fn layout_mut(&mut self) -> &mut Layout {
        &mut self.0
    }
}

impl ElementWithProps for SidebarField {
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
                flex_wrap: taffy::FlexWrap::Wrap,
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
                        if let Some(field) =
                            sidebar_entity!(ctx => get).and_then(|e| e.fields.get(idx))
                        {
                            field.modifier as usize
                        } else {
                            0
                        }
                    }),
                    setter: Box::new(move |ctx, index| {
                        if let Some(field) =
                            sidebar_entity!(ctx => get_mut).and_then(|e| e.fields.get_mut(idx))
                        {
                            field.modifier = match index {
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
                        if let Some(field) =
                            sidebar_entity!(ctx => get).and_then(|e| e.fields.get(idx))
                        {
                            format!("{}: {}", field.name, field.r#type)
                        } else {
                            "".to_string()
                        }
                    }),
                    setter: Box::new(move |ctx, str| {
                        if let Some(field) =
                            sidebar_entity!(ctx => get_mut).and_then(|e| e.fields.get_mut(idx))
                        {
                            let Some((name, ty)) = str.split_once(':') else {
                                return;
                            };

                            field.name = name.trim().to_string();
                            field.r#type = ty.trim().to_string();
                        }
                    }),
                    size: 16.,
                    font: fonts::jbmono_regular(),
                    placeholder: None,
                }),
                // Delete button
                Button::create(ButtonProps {
                    tooltip: "Delete field",
                    icon: Symbol::Trash,
                    on_click: Box::new(move |ctx| {
                        if let Some(entity) = sidebar_entity!(ctx => get_mut) {
                            entity.fields.remove(idx);
                            ctx.state.request_tooltip_update();
                        }
                    }),
                    style: ButtonStyle::Segmented,
                }),
            ]),
            |_, _| Self(<_>::default()),
        )
    }
}
