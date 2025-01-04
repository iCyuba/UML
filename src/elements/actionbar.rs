use super::{
    button::{Button, ButtonProps, ButtonStyle},
    node::{Element, ElementWithProps},
    primitives::{
        fancy_box::{BorderOptions, FancyBox, ShadowOptions},
        icon::Symbol,
        traits::Draw,
    },
    text_input::{TextInput, TextInputProps},
    Node,
};
use crate::{
    app::{
        context::{EventContext, RenderContext},
        EventTarget, Tree,
    },
    presentation::fonts,
};
use taffy::{prelude::length, AlignItems, Layout, NodeId, Position, Style};

pub struct Actionbar {
    layout: Layout,
}

impl EventTarget for Actionbar {
    fn render(&self, ctx: &mut RenderContext) {
        FancyBox::from_node(
            self,
            13.,
            ctx.c.colors().floating_background,
            Some(BorderOptions {
                color: ctx.c.colors().border,
            }),
            Some(ShadowOptions {
                color: ctx.c.colors().drop_shadow,
                offset: (0., 1.).into(),
                blur_radius: 5.,
            }),
        )
        .draw(ctx.c);
    }
}

impl Node for Actionbar {
    fn layout(&self) -> &Layout {
        &self.layout
    }

    fn layout_mut(&mut self) -> &mut Layout {
        &mut self.layout
    }
}

impl Element for Actionbar {
    fn setup(tree: &mut Tree, ctx: &mut EventContext) -> NodeId {
        tree.add_element(
            ctx,
            Style {
                position: Position::Absolute,
                border: length(1.),
                margin: length(12.),
                padding: length(8.),
                gap: length(8.),
                align_items: Some(AlignItems::Center),
                ..<_>::default()
            },
            Some(vec![
                Button::create(ButtonProps {
                    tooltip: "Save",
                    icon: Symbol::Save,
                    on_click: Box::new(|ctx| ctx.state.save()),
                    style: ButtonStyle::Default,
                }),
                Button::create(ButtonProps {
                    tooltip: "Load",
                    icon: Symbol::Load,
                    on_click: Box::new(|ctx| ctx.state.load()),
                    style: ButtonStyle::Default,
                }),
                Button::create(ButtonProps {
                    tooltip: "Screenshot",
                    icon: Symbol::Screenshot,
                    on_click: Box::new(|ctx| ctx.state.screenshot()),
                    style: ButtonStyle::Default,
                }),
                Button::create(ButtonProps {
                    tooltip: "Export",
                    icon: Symbol::Export,
                    on_click: Box::new(|ctx| ctx.state.export()),
                    style: ButtonStyle::Default,
                }),
                TextInput::create(TextInputProps {
                    size: 20.,
                    font: fonts::jbmono_bold(),
                    placeholder: Some("Project name".to_string()),
                    getter: Box::new(|ctx| ctx.project.name.clone()),
                    setter: Box::new(|ctx, value| ctx.project.name = value.to_string()),
                }),
            ]),
            |_, _| Actionbar {
                layout: <_>::default(),
            },
        )
    }
}
