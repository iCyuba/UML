use crate::fonts;
use crate::renderer::{add_text_to_scene, WindowRenderer};
use vello::kurbo::{Affine, Circle};
use vello::peniko::{Color, Fill};

pub struct Workspace {
    x: f64,
    y: f64,
}

impl Workspace {
    pub fn new() -> Self {
        Self {
            x: 0.,
            y: 0.,
        }
    }

    pub fn render(&self, renderer: &mut WindowRenderer) {
        let mut scene = &mut renderer.scene;
        let window = renderer.window.as_ref().unwrap();
        let size = window.inner_size();
        let scale = window.scale_factor();

        // Draw dots
        let gap = 32.0 * scale;

        let mut x = self.x % gap;
        let start_y = self.y % gap;
        let mut y = start_y;

        while x < size.width as f64 {
            while y < size.height as f64 {
                scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    Color::rgb8(203, 213, 225),
                    None,
                    &Circle::new((x, y), 2.0 * scale),
                );

                y += gap;
            }

            x += gap;
            y = start_y;
        }

        // Coords
        add_text_to_scene(
            &mut scene,
            &format!("x: {:.2}, y: {:.2}", self.x / scale, self.y / scale),
            10.0 * scale,
            10.0 * scale,
            16.0 * scale as f32,
            fonts::inter_extra_light()
        );
    }

    pub fn update_position(&mut self, x: f64, y: f64) {
        self.x += x;
        self.y += y;
    }
}