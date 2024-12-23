use crate::elements::primitives::traits::Draw;
use crate::geometry::rect::Rect;
use taffy::Layout;
use vello::kurbo::{Affine, RoundedRect, RoundedRectRadii};
use vello::peniko::{Color, Fill};
use vello::Scene;

pub struct SimpleBox {
    rect: RoundedRect,
    color: Color,
}

impl SimpleBox {
    fn adjust_size(
        size: &Rect,
        border: &taffy::Rect<f32>,
        is_border_box: bool,
    ) -> Rect {
        if is_border_box {
            *size
        } else {
            size.inset(border)
        }
    }

    fn adjust_radii(
        radii: &RoundedRectRadii,
        border: &taffy::Rect<f32>,
        is_border_box: bool,
    ) -> RoundedRectRadii {
        if is_border_box {
            RoundedRectRadii {
                top_left: radii.top_left + border.left.min(border.top) as f64,
                top_right: radii.top_right + border.right.min(border.top) as f64,
                bottom_left: radii.bottom_left + border.left.min(border.bottom) as f64,
                bottom_right: radii.bottom_right + border.right.min(border.bottom) as f64,
            }
        } else {
            *radii
        }
    }

    pub fn new(
        hitbox: &Rect,
        layout: &Layout,
        radii: &RoundedRectRadii,
        color: Color,
        is_border_box: bool,
    ) -> Self {
        let size = Self::adjust_size(hitbox, &layout.border, is_border_box);
        let adjusted_radii = Self::adjust_radii(radii, &layout.border, is_border_box);

        Self {
            rect: RoundedRect::from_rect(size.into(), adjusted_radii),
            color,
        }
    }
}

impl SimpleBox {
    pub fn draw_blurred(&self, scene: &mut Scene, std_dev: f64) {
        scene.draw_blurred_rounded_rect(
            Affine::IDENTITY,
            self.rect.rect(),
            self.color,
            self.rect.radii().as_single_radius().unwrap(),
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
