use crate::app::renderer::Canvas;
use crate::elements::primitives::traits::Draw;
use crate::geometry::rect::Rect;
use vello::kurbo::{self, Affine, RoundedRect, RoundedRectRadii};
use vello::peniko::{Color, Fill};

pub struct SimpleBox {
    rect: Rect,
    radii: RoundedRectRadii,
    color: Color,
}

impl SimpleBox {
    fn scale_radii(radii: RoundedRectRadii, scale: f64) -> RoundedRectRadii {
        RoundedRectRadii {
            top_left: radii.top_left * scale,
            top_right: radii.top_right * scale,
            bottom_left: radii.bottom_left * scale,
            bottom_right: radii.bottom_right * scale,
        }
    }

    pub fn new(rect: impl Into<Rect>, radii: impl Into<RoundedRectRadii>, color: Color) -> Self {
        Self {
            rect: rect.into(),
            radii: radii.into(),
            color,
        }
    }
}

impl SimpleBox {
    pub fn draw_blurred(&self, c: &mut Canvas, std_dev: f64) {
        let scale = c.scale();
        let radii = Self::scale_radii(self.radii, scale);
        let std_dev = std_dev * scale;

        c.scene().draw_blurred_rounded_rect(
            Affine::IDENTITY,
            kurbo::Rect::from(self.rect * scale),
            self.color,
            (radii.top_left + radii.top_right + radii.bottom_left + radii.bottom_right) / 4.,
            std_dev,
        );
    }
}

impl Draw for SimpleBox {
    fn draw(&self, c: &mut Canvas) {
        let scale = c.scale();
        let rect = self.rect * scale;
        let rounded_rect = RoundedRect::from_origin_size(
            rect.origin,
            rect.size,
            Self::scale_radii(self.radii, scale),
        );

        c.scene().fill(
            Fill::NonZero,
            Affine::IDENTITY,
            self.color,
            None,
            &rounded_rect,
        );
    }
}
