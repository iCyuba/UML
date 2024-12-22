use crate::app::renderer::add_text_to_scene;
use crate::elements::primitives::traits::Draw;
use crate::geometry::Point;
use crate::presentation::fonts;
use vello::peniko::Color;

pub struct Icon {
    content: char,
    color: Color,
    size: f32,
    pos: Point,
}

impl Icon {
    pub fn new(content: char, pos: Point, size: f32, color: Color) -> Self {
        Self {
            content,
            pos,
            size,
            color,
        }
    }
}

impl Draw for Icon {
    fn draw(&self, scene: &mut vello::Scene) {
        add_text_to_scene(
            scene,
            &self.content.to_string(),
            self.pos.x,
            self.pos.y,
            self.size,
            fonts::icons(),
            self.color,
        );
    }
}
