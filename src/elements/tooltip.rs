use super::{
    primitives::{
        fancy_box::{BorderOptions, FancyBox, ShadowOptions},
        text::Text,
        traits::Draw,
    },
    Node,
};
use crate::elements::node::Element;
use crate::{
    animations::{
        animated_property::AnimatedProperty,
        standard_animation::{Easing, StandardAnimation},
    },
    app::{
        context::{EventContext, RenderContext},
        EventTarget, Tree,
    },
    geometry::{Point, Rect, Size},
    presentation::fonts,
};
use derive_macros::AnimatedElement;
use std::time::Duration;
use taffy::{Layout, NodeId};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TooltipPosition {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TooltipState {
    pub text: String,
    pub anchor: Rect,
    pub position: TooltipPosition,
}

#[derive(AnimatedElement)]
pub struct Tooltip {
    layout: Layout,

    current: Option<TooltipState>,

    opacity: AnimatedProperty<StandardAnimation<f32>>,
}

impl Tooltip {
    const FONT_SIZE: f64 = 14.;
}

impl Node for Tooltip {
    fn layout(&self) -> &Layout {
        &self.layout
    }

    fn layout_mut(&mut self) -> &mut Layout {
        &mut self.layout
    }
}

impl EventTarget for Tooltip {
    fn update(&mut self, ctx: &mut EventContext) {
        if self.animate() {
            ctx.state.request_redraw();
        } else if self.current.is_none() {
            if let Some(new) = &ctx.state.tooltip_state {
                self.opacity.set(1.);
                self.current = Some(new.clone());
                ctx.state.request_redraw();
            }
        } else if self.current.is_some() {
            if *self.opacity == 0. {
                self.current = None;

                // Re-run the update to check if there's a new tooltip to show
                self.update(ctx);
            } else if self.current != ctx.state.tooltip_state {
                self.opacity.set(0.);
                ctx.state.request_redraw();
            }
        }
    }

    fn render(&self, RenderContext { c, .. }: &mut RenderContext) {
        if let Some(TooltipState {
            text,
            anchor,
            position,
        }) = &self.current
        {
            let font = fonts::inter_regular();

            let text_size = Text::measure(text, Tooltip::FONT_SIZE, font);

            const MARGIN: f64 = Tooltip::FONT_SIZE / 2.;
            const PADDING: Size = Size::new(Tooltip::FONT_SIZE, Tooltip::FONT_SIZE / 2.);

            let screen_size = c.size();
            let screen_size = Size::new(screen_size.0 as f64, screen_size.1 as f64) / c.scale();

            let tooltip_size = text_size + PADDING * 2.;

            // Anchor points
            let origin = anchor.origin - tooltip_size;
            let center = anchor.center();
            let end = anchor.end();
            let tooltip_end = end + tooltip_size;

            // Check the if the tooltip fits in the screen
            let position = match position {
                TooltipPosition::Top if origin.y < MARGIN => TooltipPosition::Bottom,
                TooltipPosition::Bottom if tooltip_end.y > screen_size.y - MARGIN => {
                    TooltipPosition::Top
                }

                TooltipPosition::Left if origin.x < MARGIN => TooltipPosition::Right,
                TooltipPosition::Right if tooltip_end.x > screen_size.x - MARGIN => {
                    TooltipPosition::Left
                }

                _ => *position,
            };

            // Calculate the tooltip position
            let margin = MARGIN * *self.opacity as f64;

            let tooltip_origin_x = match position {
                TooltipPosition::Top | TooltipPosition::Bottom => center.x - (tooltip_size.x / 2.),
                TooltipPosition::Left => origin.x - margin,
                TooltipPosition::Right => end.x + margin,
            };

            let tooltip_origin_y = match position {
                TooltipPosition::Top => origin.y - margin,
                TooltipPosition::Bottom => end.y + margin,
                TooltipPosition::Left | TooltipPosition::Right => center.y - (tooltip_size.y / 2.),
            };

            let tooltip = Rect::new((tooltip_origin_x, tooltip_origin_y), tooltip_size);

            FancyBox::new(
                tooltip,
                taffy::Rect::length(1.),
                5.,
                c.colors().floating_background.multiply_alpha(*self.opacity),
                Some(BorderOptions {
                    color: c.colors().border.multiply_alpha(*self.opacity),
                }),
                Some(ShadowOptions {
                    color: c.colors().drop_shadow.multiply_alpha(*self.opacity * 0.5),
                    offset: Point::new(0., 1.),
                    blur_radius: 5.,
                }),
            )
            .draw(c);

            Text::new(
                text,
                tooltip.translate(PADDING),
                Tooltip::FONT_SIZE,
                font,
                c.colors().workspace_text.multiply_alpha(*self.opacity),
            )
            .draw(c);
        }
    }
}

impl Element for Tooltip {
    fn setup(tree: &mut Tree, ctx: &mut EventContext) -> NodeId {
        tree.add_element(ctx, Default::default(), None, |_, ctx| Self {
            layout: Default::default(),

            current: ctx.state.tooltip_state.clone(),

            opacity: AnimatedProperty::new(StandardAnimation::initialized(
                0.,
                Duration::from_millis(200),
                Easing::EaseInOut,
            )),
        })
    }
}
