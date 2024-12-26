use crate::elements::primitives::simple_box::SimpleBox;
use crate::elements::primitives::traits::Draw;
use crate::elements::Element;
use crate::geometry::rect::Rect;
use crate::geometry::Point;
use vello::kurbo::RoundedRectRadii;
use vello::peniko::Color;
use vello::Scene;

pub struct FancyBox {
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

#[derive(Clone, Copy)]
pub struct BorderOptions {
    pub color: Color,
}

impl FancyBox {
    fn offset_radii(radii: &RoundedRectRadii, border: &taffy::Rect<f32>) -> RoundedRectRadii {
        RoundedRectRadii {
            top_left: radii.top_left + border.left.min(border.top) as f64,
            top_right: radii.top_right + border.right.min(border.top) as f64,
            bottom_left: radii.bottom_left + border.left.min(border.bottom) as f64,
            bottom_right: radii.bottom_right + border.right.min(border.bottom) as f64,
        }
    }

    pub fn new(
        element: &impl Element,
        radii: impl Into<RoundedRectRadii>,
        color: Color,
        border_options: Option<BorderOptions>,
        shadow_options: Option<ShadowOptions>,
    ) -> Self {
        let layout = element.layout();
        let hitbox = Rect::from(*layout);

        let radii = radii.into();
        let offset_radii = Self::offset_radii(&radii, &layout.border);

        let content = SimpleBox::new(hitbox.inset(layout.border), radii, color);

        let border = border_options.and_then(|opts| {
            if layout.border != taffy::Rect::zero() {
                Some(SimpleBox::new(hitbox, offset_radii, opts.color))
            } else {
                None
            }
        });

        let shadow = shadow_options
            .map(|opts| SimpleBox::new(hitbox + opts.offset, offset_radii, opts.color));

        Self {
            content,
            border,
            shadow,
            shadow_options,
        }
    }
}

impl Draw for FancyBox {
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
