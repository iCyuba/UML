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

pub enum ButtonStyle {
    Default,
    Segmented,
}

pub struct ButtonProps {
    pub tooltip: &'static str,
    pub icon: Symbol,
    pub on_click: Box<dyn Fn(&mut EventContext)>,
    pub style: ButtonStyle,
}

#[derive(AnimatedElement)]
pub struct Button {
    layout: Layout,
    props: ButtonProps,

    hover_opacity: AnimatedProperty<StandardAnimation<f32>>,
}

impl EventTarget for Button {
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

        let bg = match self.props.style {
            ButtonStyle::Default => ctx.c.colors().floating_background,
            ButtonStyle::Segmented => ctx.c.colors().border,
        };

        SimpleBox::new(rect, 5., bg).draw(ctx.c);
        SimpleBox::new(rect, 5., hover).draw(ctx.c);

        // Icon
        let (size, inset) = match self.props.style {
            ButtonStyle::Default => (20., 6.),
            ButtonStyle::Segmented => (16., 4.),
        };

        Icon::new(
            self.props.icon,
            rect.inset_uniform(inset),
            size,
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

impl Node for Button {
    fn layout(&self) -> &Layout {
        &self.layout
    }

    fn layout_mut(&mut self) -> &mut Layout {
        &mut self.layout
    }
}

impl ElementWithProps for Button {
    type Props = ButtonProps;

    fn setup(tree: &mut Tree, ctx: &mut EventContext, props: Self::Props) -> NodeId {
        let size = match props.style {
            ButtonStyle::Default => 32.,
            ButtonStyle::Segmented => 24.,
        };

        tree.add_element(
            ctx,
            Style {
                size: length(size),
                flex_shrink: 0.,
                ..<_>::default()
            },
            None,
            |_, _| Button {
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
