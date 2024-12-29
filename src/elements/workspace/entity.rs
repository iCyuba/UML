use super::{item::Item, Workspace};
use crate::{
    animations::{
        animated_property::AnimatedProperty,
        standard_animation::{Easing, StandardAnimation},
        traits::Interpolate,
    },
    app::{renderer::Canvas, State},
    data::{
        entity::{Attribute, EntityType},
        Entity,
    },
    elements::primitives::{
        fancy_box::{BorderOptions, FancyBox, ShadowOptions},
        text::Text,
        traits::Draw,
    },
    geometry::{Point, Rect, Size},
    presentation::{fonts, FontResource},
};
use derive_macros::AnimatedElement;
use std::time::Duration;
use winit::event::MouseButton;

#[derive(Debug, AnimatedElement)]
pub struct EntityItemData {
    pub rect: AnimatedProperty<StandardAnimation<Rect>>,
    pub selection_outline: AnimatedProperty<StandardAnimation<f64>>,

    pub is_selected: bool,
}

impl Default for EntityItemData {
    fn default() -> Self {
        Self {
            rect: AnimatedProperty::new(StandardAnimation::initialized(
                Rect::ZERO,
                Duration::from_millis(100),
                Easing::EaseOut,
            )),
            selection_outline: AnimatedProperty::new(StandardAnimation::initialized(
                0.,
                Duration::from_millis(100),
                Easing::EaseOut,
            )),
            is_selected: false,
        }
    }
}

impl Item for Entity {
    fn update(&mut self) -> bool {
        // Compute the entity's position and size
        let mut position = Point::new(self.position.0 as f64, self.position.1 as f64) * 32.;
        let mut size = Size::ZERO;

        // Name
        let name = Text::measure(&self.name, 16., title_font(self));
        size.x = size.x.max(name.size.x);
        size.y += name.size.y + 8.; // 8px gap

        // Attributes
        for (name, attr) in self.attributes.iter() {
            let attr = Text::measure(&attr_to_string(name, attr), 12., fonts::jbmono_regular());
            size.x = size.x.max(attr.size.x);
            size.y += attr.size.y + 4.; // 4px gap
        }

        // Padding
        size += (32., 32.);

        // Add margin to the position, so it's centered in the 32px grid
        position.x += (32. - size.x) / 2.;
        position.y += (32. - size.y) / 2.;

        let rect = Rect::new(position, size);
        self.data.rect.set(rect);

        // Animate the selection outline
        self.data
            .selection_outline
            .set(if self.data.is_selected { 1. } else { 0. });

        self.data.animate()
    }

    fn render(&self, c: &mut Canvas, _: &State, ws: &Workspace) {
        let pos = ws.position();
        let zoom = ws.zoom();

        let rect = (*self.data.rect * zoom).translate(-pos);

        // Background
        FancyBox::new(
            rect,
            taffy::Rect::length(2. * zoom as f32),
            8. * zoom,
            c.colors().floating_background,
            Some(BorderOptions {
                color: Interpolate::interpolate(
                    &c.colors().border,
                    &c.colors().accent,
                    *self.data.selection_outline,
                ),
            }),
            Some(ShadowOptions {
                color: c.colors().drop_shadow,
                offset: (0., 1. * zoom).into(),
                blur_radius: 5. * zoom,
            }),
        )
        .draw(c);

        let padded: Rect = rect.inset_uniform(16. * zoom);

        // Name
        Text::new(
            &self.name,
            Rect::new(padded.origin, (padded.size.x, 16. * zoom)),
            16.0 * zoom,
            title_font(self),
            c.colors().workspace_text,
        )
        .draw(c);

        // Attributes
        let mut y = 24. * zoom; // 8px gap
        for (name, attr) in self.attributes.iter() {
            Text::new(
                &attr_to_string(name, attr),
                Rect::new(padded.origin + (0., y), (padded.size.x, 12. * zoom)),
                12.0 * zoom,
                fonts::jbmono_regular(),
                c.colors().accent,
            )
            .draw(c);

            y += 16. * zoom;
        }
    }

    fn on_mousedown(&mut self, state: &mut State, _: MouseButton) -> bool {
        state.selected_entity = Some(self.key);
        state.request_redraw();
        true
    }

    fn on_mousemove(&mut self, _: &mut State, _: Point) -> bool {
        false
    }

    fn on_mouseup(&mut self, _: &mut State, _: MouseButton) -> bool {
        false
    }

    fn on_mouseenter(&mut self, _: &mut State) -> bool {
        false
    }

    fn on_mouseleave(&mut self, _: &mut State) -> bool {
        false
    }
}

#[inline]
fn title_font(ent: &Entity) -> &FontResource {
    match ent.ty {
        EntityType::AbstractClass => fonts::jbmono_bold_italic(),
        _ => fonts::jbmono_bold(),
    }
}

fn attr_to_string(name: &str, attr: &Attribute) -> String {
    match attr {
        Attribute::Field(am, t) => format!("{} {:?} {}", am.as_char(), t, name),
        Attribute::Method(am, _, t, _) => format!("{} {:?} {}()", am.as_char(), t, name),
    }
}
