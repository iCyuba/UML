use super::{
    primitives::{
        fancy_box::{BorderOptions, FancyBox, ShadowOptions},
        text::Text,
        traits::Draw,
    },
    Element,
};
use crate::{
    animations::{
        animated_property::AnimatedProperty,
        standard_animation::{Easing, StandardAnimation},
    },
    app::{
        context::{EventContext, RenderContext},
        EventTarget, Tree,
    },
    geometry::{rect::Rect, Point, Size},
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
    pub fn setup(tree: &mut Tree, ctx: &mut EventContext) -> NodeId {
        let this = Self {
            layout: Default::default(),

            current: ctx.state.tooltip_state.clone(),

            opacity: AnimatedProperty::new(StandardAnimation::initialized(
                0.,
                Duration::from_millis(200),
                Easing::EaseInOut,
            )),
        };

        tree.new_leaf_with_context(Default::default(), Box::new(this))
            .unwrap()
    }
}

impl Element for Tooltip {
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
            let scale = c.scale();
            let font_size = 14. * scale;
            let text_size = Text::measure(text, font_size, font).size;

            let margin = font_size / 2.;
            let padding = Size::new(font_size, font_size / 2.);

            let screen_size = c.size();
            let tooltip_size = text_size + padding * 2.;

            // Anchor points
            let origin = anchor.origin - tooltip_size;
            let center = anchor.center();
            let end = anchor.end();
            let tooltip_end = end + tooltip_size;

            // Check the if the tooltip fits in the screen
            let position = match position {
                TooltipPosition::Top if origin.y < margin => TooltipPosition::Bottom,
                TooltipPosition::Bottom if tooltip_end.y > screen_size.1 as f64 - margin => {
                    TooltipPosition::Top
                }

                TooltipPosition::Left if origin.x < margin => TooltipPosition::Right,
                TooltipPosition::Right if tooltip_end.x > screen_size.0 as f64 - margin => {
                    TooltipPosition::Left
                }

                _ => *position,
            };

            // Calculate the tooltip position
            let margin = margin * *self.opacity as f64;

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

            let border = scale as f32;
            let radii = 5. * scale;

            FancyBox::new(
                1.,
                tooltip,
                taffy::Rect::new(border, border, border, border),
                radii,
                c.colors().floating_background.multiply_alpha(*self.opacity),
                Some(BorderOptions {
                    color: c.colors().border.multiply_alpha(*self.opacity),
                }),
                Some(ShadowOptions {
                    color: c.colors().drop_shadow.multiply_alpha(*self.opacity * 0.5),
                    offset: Point::new(0., 1.) * scale,
                    blur_radius: 5. * scale,
                }),
            )
            .draw(c.scene());

            Text::new(
                text,
                1.,
                tooltip + padding,
                font_size,
                font,
                c.colors().workspace_text.multiply_alpha(*self.opacity),
            )
            .draw(c.scene());
        }
    }
}
