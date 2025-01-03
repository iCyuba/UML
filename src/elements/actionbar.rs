use super::{
    button::{Button, ButtonProps, ButtonStyle},
    node::{Element, ElementWithProps},
    primitives::{
        fancy_box::{BorderOptions, FancyBox, ShadowOptions},
        icon::Symbol,
        traits::Draw,
    },
    Node,
};
use crate::app::{
    context::{EventContext, RenderContext},
    EventTarget, Tree,
};
use taffy::{prelude::length, Layout, NodeId, Position, Style};

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
                #[cfg(not(target_arch = "wasm32"))]
                Button::create(ButtonProps {
                    tooltip: "Screenshot",
                    icon: Symbol::Screenshot,
                    on_click: Box::new(|ctx| ctx.state.screenshot()),
                    style: ButtonStyle::Default,
                }),
            ]),
            |_, _| Actionbar {
                layout: <_>::default(),
            },
        )
    }
}
