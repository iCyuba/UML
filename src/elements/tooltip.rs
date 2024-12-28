use crate::animations::animated_property::AnimatedProperty;
use crate::animations::standard_animation::{Easing, StandardAnimation};
use crate::app::{EventTarget, Renderer, State, Tree};
use crate::elements::primitives::fancy_box::{BorderOptions, FancyBox, ShadowOptions};
use crate::elements::primitives::text::Text;
use crate::elements::primitives::traits::Draw;
use crate::elements::Element;
use crate::geometry::rect::Rect;
use crate::geometry::{Point, Size};
use crate::presentation::fonts;
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
    pub fn setup(tree: &mut Tree, state: &mut State) -> NodeId {
        let this = Self {
            layout: Default::default(),

            current: state.tooltip_state.clone(),

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
    fn update(&mut self, _r: &Renderer, state: &mut State) {
        if self.animate() {
            state.request_redraw();
        } else if self.current.is_none() {
            if let Some(new) = &state.tooltip_state {
                self.opacity.set(1.);
                self.current = Some(new.clone());
                state.request_redraw();
            }
        } else if self.current.is_some() && self.current != state.tooltip_state {
            if *self.opacity == 0. {
                self.current = None;

                // Re-run the update to check if there's a new tooltip to show
                self.update(_r, state);
            } else {
                self.opacity.set(0.);
                state.request_redraw();
            }
        }
    }

    fn render(&self, r: &mut Renderer, _: &State) {
        if let Some(TooltipState {
            text,
            anchor,
            position,
        }) = &self.current
        {
            let font = fonts::inter_regular();
            let scale = r.scale();
            let font_size = 14. * scale;
            let text_size = Text::measure(text, font_size, font).size;

            let margin = font_size / 2.;
            let padding = Size::new(font_size, font_size / 2.);

            let screen_size = r.size();
            let tooltip_size = text_size + padding * 2.;

            // Anchor points
            let origin = anchor.origin - tooltip_size;
            let center = anchor.center();
            let end = anchor.end();
            let tooltip_end = end + tooltip_size;

            // Check the if the tooltip fits in the screen
            let position = match position {
                TooltipPosition::Top if origin.y < margin => TooltipPosition::Bottom,
                TooltipPosition::Bottom if tooltip_end.y > screen_size.height as f64 - margin => {
                    TooltipPosition::Top
                }

                TooltipPosition::Left if origin.x < margin => TooltipPosition::Right,
                TooltipPosition::Right if tooltip_end.x > screen_size.width as f64 - margin => {
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
                r.colors.floating_background.multiply_alpha(*self.opacity),
                Some(BorderOptions {
                    color: r.colors.border.multiply_alpha(*self.opacity),
                }),
                Some(ShadowOptions {
                    color: r.colors.drop_shadow.multiply_alpha(*self.opacity * 0.5),
                    offset: Point::new(0., 1.) * scale,
                    blur_radius: 5. * scale,
                }),
            )
            .draw(&mut r.scene);

            Text::new(
                text,
                1.,
                tooltip + padding,
                font_size,
                font,
                r.colors.workspace_text.multiply_alpha(*self.opacity),
            )
            .draw(&mut r.scene);
        }
    }
}
