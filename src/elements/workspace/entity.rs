use super::{item::Item, Workspace};
use crate::{
    animations::{
        animated_property::AnimatedProperty,
        standard_animation::{Easing, StandardAnimation},
        traits::Interpolate,
    },
    app::{renderer::Canvas, State},
    data::{entity::EntityType, Entity},
    elements::{
        primitives::{
            fancy_box::{BorderOptions, FancyBox, ShadowOptions},
            text::Text,
            traits::Draw,
        },
        toolbox_item::Tool,
    },
    geometry::{Point, Rect, Size},
    presentation::{fonts, FontResource},
};
use derive_macros::AnimatedElement;
use std::time::Duration;

#[derive(Debug, AnimatedElement)]
pub struct EntityItemData {
    pub rect: AnimatedProperty<StandardAnimation<Rect>>,
    pub(super) selection_outline: AnimatedProperty<StandardAnimation<f64>>,
    pub(super) opacity: AnimatedProperty<StandardAnimation<f32>>,

    /// When moving, this will be used as the origin offset. (Only during rendering)
    ///
    /// This is to prevent a strange animation when moving the entity.
    pub move_pos: Option<Point>,
}

impl EntityItemData {
    pub fn new(pos: (i32, i32)) -> Self {
        Self {
            rect: AnimatedProperty::new(StandardAnimation::initialized(
                Rect::ZERO.translate(pos) * Workspace::GRID_SIZE,
                Duration::from_millis(100),
                Easing::EaseOut,
            )),
            ..Default::default()
        }
    }
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
            opacity: AnimatedProperty::new(StandardAnimation::initialized(
                1.,
                Duration::from_millis(100),
                Easing::EaseOut,
            )),
            move_pos: None,
        }
    }
}

impl Item for Entity {
    fn update(&mut self, state: &State, _: &Workspace) -> bool {
        // Set the opacity
        let invalid = {
            let parent_tool = state.tool == Tool::Parent;
            let impl_tool = state.tool == Tool::Implementation;

            let has_selection = state.selected_entity.is_some_and(|key| key != self.key);

            let is_interface = self.entity_type == EntityType::Interface;
            let is_sealed = self.entity_type == EntityType::SealedClass;

            // Interfaces can't have children and can't be parents
            // Sealed classes can't have children but can be parents
            // Only interfaces can be implemented
            (parent_tool && (is_interface || (is_sealed && has_selection)))
                || (impl_tool && has_selection && !is_interface)
        };

        self.data.opacity.set(if self.data.move_pos.is_some() {
            0.75
        } else if invalid {
            0.5
        } else {
            1.
        });

        // Compute the entity's position and size
        let mut position: Point = Point::from(self.position) * Workspace::GRID_SIZE;
        let mut size = Size::ZERO;

        // Name
        let name = Text::measure(&self.name, 16., title_font(self));
        size.x = size.x.max(name.x);
        size.y += name.y;

        if !self.fields.is_empty() {
            size.y += 8.; // 8px gap
        }

        // Attributes
        for field in self.fields.iter() {
            let attr = Text::measure(&field.to_string(), 12., fonts::jbmono_regular());
            size.x = size.x.max(attr.x);
            size.y += attr.y + 8.; // 4px margin
        }

        if !self.methods.is_empty() {
            size.y += 8.; // 8px gap between fields and methods
        }

        for method in self.methods.iter() {
            let attr = Text::measure(&method.to_string(), 12., fonts::jbmono_regular());
            size.x = size.x.max(attr.x);
            size.y += attr.y + 8.; // 4px margin
        }

        // Padding
        size += (Workspace::GRID_SIZE, Workspace::GRID_SIZE);

        position -= size / 2.;

        let rect = Rect::new(position, size);
        self.data.rect.set(rect);

        // Animate the selection outline
        self.data.selection_outline.set(
            if state.selected_entity == Some(self.key) || self.data.move_pos.is_some() {
                1.
            } else {
                0.
            },
        );

        self.data.animate()
    }

    fn render(&self, c: &mut Canvas, _: &State, ws: &Workspace) {
        let pos = ws.position();
        let zoom = ws.zoom();

        // Offset the position if moving
        let rect = self
            .data
            .rect
            .translate(self.data.move_pos.unwrap_or_default());

        let rect = (rect * zoom).translate(-pos);
        let opacity = *self.data.opacity;

        // Background
        FancyBox::new(
            rect,
            taffy::Rect::length(2. * zoom as f32),
            8. * zoom,
            c.colors().floating_background.multiply_alpha(opacity),
            Some(BorderOptions {
                color: Interpolate::interpolate(
                    &c.colors().border,
                    &c.colors().accent,
                    *self.data.selection_outline,
                )
                .multiply_alpha(opacity),
            }),
            Some(ShadowOptions {
                color: c.colors().drop_shadow.multiply_alpha(opacity),
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
            c.colors().text.multiply_alpha(opacity),
            false,
        )
        .draw(c);

        let mut render_property = |str: String, y| {
            Text::new(
                &str,
                Rect::new(padded.origin + (0., y), (padded.size.x, 12. * zoom)),
                12.0 * zoom,
                fonts::jbmono_regular(),
                c.colors().accent.multiply_alpha(opacity),
                false,
            )
            .draw(c);
        };

        // Attributes
        let line = (12. * 1.2 + 8.) * zoom; // 4x margin
        let mut y = (16. * 1.2 + 8.) * zoom; // 8px gap
        for field in self.fields.iter() {
            render_property(field.to_string(), y);

            y += line;
        }

        if !self.fields.is_empty() {
            y += 8. * zoom; // 8px gap
        }

        for method in self.methods.iter() {
            render_property(method.to_string(), y);

            y += line;
        }
    }
}

#[inline]
fn title_font(ent: &Entity) -> &FontResource {
    match ent.entity_type {
        EntityType::AbstractClass => fonts::jbmono_bold_italic(),
        _ => fonts::jbmono_bold(),
    }
}
