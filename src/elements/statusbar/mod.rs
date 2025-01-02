use super::{
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
use button::{StatusbarButton, StatusbarButtonProps};
use taffy::{prelude::length, Layout, NodeId, Position, Style};

mod button;

pub struct Statusbar {
    layout: Layout,
}

impl EventTarget for Statusbar {
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

impl Node for Statusbar {
    fn layout(&self) -> &Layout {
        &self.layout
    }

    fn layout_mut(&mut self) -> &mut Layout {
        &mut self.layout
    }
}

impl Element for Statusbar {
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
                StatusbarButton::create(StatusbarButtonProps {
                    tooltip: "Save",
                    icon: Symbol::Save,
                    on_click: |ctx| ctx.state.save(),
                }),
                StatusbarButton::create(StatusbarButtonProps {
                    tooltip: "Load",
                    icon: Symbol::Load,
                    on_click: |ctx| ctx.state.load(),
                }),
                #[cfg(not(target_arch = "wasm32"))]
                StatusbarButton::create(StatusbarButtonProps {
                    tooltip: "Screenshot",
                    icon: Symbol::Screenshot,
                    on_click: |ctx| ctx.state.screenshot(),
                }),
            ]),
            |_, _| Statusbar {
                layout: <_>::default(),
            },
        )
    }
}
