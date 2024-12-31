use crate::{
    animations::{
        animated_property::AnimatedProperty,
        standard_animation::{Easing, StandardAnimation},
    },
    app::{
        context::{EventContext, GetterContext, RenderContext},
        EventTarget, Tree,
    },
    elements::{
        node::ElementWithProps,
        primitives::{
            icon::{Icon, Symbol},
            simple_box::SimpleBox,
            traits::Draw,
        },
        tooltip::{TooltipPosition, TooltipState},
        Node,
    },
    geometry::Rect,
};
use derive_macros::AnimatedElement;
use std::time::Duration;
use taffy::{prelude::length, Layout, NodeId, Style};
use winit::window::CursorIcon;

pub struct StatusbarButtonProps {
    pub tooltip: &'static str,
    pub icon: Symbol,
    pub on_click: fn(&mut EventContext),
}

#[derive(AnimatedElement)]
pub struct StatusbarButton {
    layout: Layout,
    props: StatusbarButtonProps,

    hover_opacity: AnimatedProperty<StandardAnimation<f32>>,
}

impl EventTarget for StatusbarButton {
    fn update(&mut self, ctx: &mut EventContext) {
        if self.animate() {
            ctx.state.request_redraw();
        }
    }

    fn cursor(&self, _: &GetterContext) -> Option<CursorIcon> {
        Some(CursorIcon::Pointer)
    }

    fn tooltip(&self, _: &GetterContext) -> Option<TooltipState> {
        Some(TooltipState {
            text: self.props.tooltip.to_string(),
            anchor: self.layout.into(),
            position: TooltipPosition::Top,
        })
    }

    fn render(&self, ctx: &mut RenderContext) {
        let rect: Rect = self.layout.into();
        let hover = ctx.c.colors().hover.multiply_alpha(*self.hover_opacity);

        SimpleBox::new(rect, 5., ctx.c.colors().floating_background).draw(ctx.c);
        SimpleBox::new(rect, 5., hover).draw(ctx.c);

        // Icon
        Icon::new(
            self.props.icon,
            rect.inset_uniform(6.),
            20.,
            ctx.c.colors().icon_inactive,
        )
        .draw(ctx.c);
    }

    fn on_click(&mut self, ctx: &mut EventContext) -> bool {
        (self.props.on_click)(ctx);

        true
    }

    fn on_mouseenter(&mut self, ctx: &mut EventContext) -> bool {
        self.hover_opacity.set(0.1);
        ctx.state.request_tooltip_update();
        ctx.state.request_redraw();

        true
    }

    fn on_mouseleave(&mut self, ctx: &mut EventContext) -> bool {
        self.hover_opacity.set(0.);
        ctx.state.request_tooltip_update();
        ctx.state.request_redraw();

        true
    }
}

impl Node for StatusbarButton {
    fn layout(&self) -> &Layout {
        &self.layout
    }

    fn layout_mut(&mut self) -> &mut Layout {
        &mut self.layout
    }
}

impl ElementWithProps for StatusbarButton {
    type Props = StatusbarButtonProps;

    fn setup(tree: &mut Tree, ctx: &mut EventContext, props: Self::Props) -> NodeId {
        tree.add_element(
            ctx,
            Style {
                size: length(32.),
                ..<_>::default()
            },
            None,
            |_, _| StatusbarButton {
                layout: <_>::default(),
                props,

                hover_opacity: AnimatedProperty::new(StandardAnimation::initialized(
                    0.,
                    Duration::from_millis(100),
                    Easing::EaseInOut,
                )),
            },
        )
    }
}
