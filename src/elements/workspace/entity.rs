use super::{item::Item, Workspace};
use crate::{
    app::{Renderer, State},
    data::{
        entity::{Attribute, EntityType},
        Entity,
    },
    elements::primitives::{
        fancy_box::{BorderOptions, FancyBox, ShadowOptions},
        text::Text,
        traits::Draw,
    },
    geometry::{rect::Rect, Size},
    presentation::{fonts, FontResource},
};

impl Item for Entity {
    fn update(&mut self) {
        // Compute the size of the entity
        let mut size = Size::ZERO;

        // Name
        let name = Text::calculate_dimensions(&self.name, 16., title_font(self));
        size.x = size.x.max(name.size.x);
        size.y += name.size.y + 8.; // 8px gap

        // Attributes
        for (name, attr) in self.attributes.iter() {
            let attr = Text::calculate_dimensions(
                &attr_to_string(name, attr),
                12.,
                fonts::jbmono_regular(),
            );
            size.x = size.x.max(attr.size.x);
            size.y += attr.size.y + 4.; // 4px gap
        }

        // Set the size
        self.rect.size = size + (32., 32.); // Padding
    }

    fn render(&self, r: &mut Renderer, _: &State, ws: &Workspace) {
        let pos = ws.position();
        let ui_scale = r.scale();
        let zoom = ws.zoom();
        let scale = ui_scale * zoom;

        let rect = (self.rect * scale).translate(-pos);

        // Background
        FancyBox::new(
            1.,
            rect,
            taffy::Rect::length(2. * scale as f32),
            8. * scale,
            r.colors.toolbox_background,
            Some(BorderOptions {
                color: r.colors.toolbox_border,
            }),
            Some(ShadowOptions {
                color: r.colors.drop_shadow,
                offset: (0., 1. * scale).into(),
                blur_radius: 5. * scale,
            }),
        )
        .draw(&mut r.scene);

        let padded: Rect = rect.inset_uniform(16. * scale);

        // Name
        Text::new(
            &self.name,
            1.,
            Rect::new(padded.origin, (padded.size.x, 16. * scale)),
            16.0 * scale,
            title_font(self),
            r.colors.workspace_text,
        )
        .draw(&mut r.scene);

        // Attributes
        let mut y = 24. * scale; // 8px gap
        for (name, attr) in self.attributes.iter() {
            Text::new(
                &attr_to_string(name, attr),
                1.,
                Rect::new(padded.origin + (0., y), (padded.size.x, 12. * scale)),
                12.0 * scale,
                fonts::jbmono_regular(),
                r.colors.accent,
            )
            .draw(&mut r.scene);

            y += 16. * scale;
        }
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
