use crate::elements::primitives::text::Text;
use crate::elements::primitives::traits::Draw;
use crate::geometry::rect::Rect;
use crate::presentation::fonts;
use vello::peniko::Color;

pub struct Icon {
    content: char,
    color: Color,
    size: f64,
    rect: Rect,
}

impl Icon {
    pub fn new(content: char, rect: Rect, size: f64, color: Color) -> Self {
        Self {
            content,
            rect,
            size,
            color,
        }
    }
}

impl Draw for Icon {
    fn draw(&self, scene: &mut vello::Scene) {
        Text::new(
            &self.content.to_string(),
            1.,
            self.rect,
            self.size,
            fonts::icons(),
            self.color,
        )
        .draw(scene);
    }
}
