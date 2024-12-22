use crate::elements::primitives::simple_box::SimpleBox;
use crate::elements::primitives::traits::Draw;
use crate::geometry::rect::Rect;
use crate::geometry::Point;
use taffy::{Layout, Style};
use vello::kurbo::RoundedRectRadii;
use vello::peniko::Color;
use vello::Scene;

pub struct BoxElement {
    content: SimpleBox,
    border: Option<SimpleBox>,
    shadow: Option<SimpleBox>,

    shadow_options: Option<ShadowOptions>,
}

#[derive(Clone, Copy)]
pub struct ShadowOptions {
    pub color: Color,
    pub blur_radius: f64,
    pub offset: Point,
}

impl BoxElement {
    pub fn new(
        hitbox: &Rect,
        layout: &Layout,
        style: &Style,
        radii: RoundedRectRadii,
        color: Color,
        border_color: Option<Color>,
        shadow_options: Option<ShadowOptions>,
    ) -> Self {
        let content = SimpleBox::new(hitbox, layout, &radii, color, false);

        let border = border_color.and_then(|border_color| {
            if style.border != taffy::Rect::zero() {
                Some(SimpleBox::new(hitbox, layout, &radii, border_color, true))
            } else {
                None
            }
        });

        let shadow = shadow_options
            .map(|opts| SimpleBox::new(&(*hitbox + opts.offset), layout, &radii, opts.color, true));

        Self {
            content,
            border,
            shadow,
            shadow_options,
        }
    }
}

impl Draw for BoxElement {
    fn draw(&self, scene: &mut Scene) {
        if let Some(opts) = &self.shadow_options {
            let shadow = self.shadow.as_ref().unwrap();
            shadow.draw_blurred(scene, opts.blur_radius);
        }

        if let Some(border) = &self.border {
            border.draw(scene);
        }

        self.content.draw(scene);
    }
}
