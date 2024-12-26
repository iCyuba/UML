use crate::elements::primitives::traits::Draw;
use vello::kurbo::{Affine, Rect, RoundedRect, RoundedRectRadii};
use vello::peniko::{Color, Fill};
use vello::Scene;

pub struct SimpleBox {
    rect: RoundedRect,
    color: Color,
}

impl SimpleBox {
    fn scale_radii(radii: &RoundedRectRadii, scale: f64) -> RoundedRectRadii {
        RoundedRectRadii {
            top_left: radii.top_left * scale,
            top_right: radii.top_right * scale,
            bottom_left: radii.bottom_left * scale,
            bottom_right: radii.bottom_right * scale,
        }
    }
    
    pub fn new(
        scale: f64,
        rect: impl Into<Rect>,
        radii: impl Into<RoundedRectRadii>,
        color: Color,
    ) -> Self {
        Self {
            rect: RoundedRect::from_rect(rect.into(), Self::scale_radii(&radii.into(), scale)),
            color,
        }
    }
}

impl SimpleBox {
    pub fn draw_blurred(&self, scene: &mut Scene, std_dev: f64) {
        let radii = self.rect.radii();
        scene.draw_blurred_rounded_rect(
            Affine::IDENTITY,
            self.rect.rect(),
            self.color,
            (radii.top_left + radii.top_right + radii.bottom_left + radii.bottom_right) / 4.,
            std_dev,
        );
    }
}

impl Draw for SimpleBox {
    fn draw(&self, scene: &mut Scene) {
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            self.color,
            None,
            &self.rect,
        );
    }
}
