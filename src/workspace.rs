use crate::fonts;
use crate::renderer::{add_text_to_scene, WindowRenderer};
use vello::kurbo::{Affine, Circle};
use vello::peniko::{Color, Fill};
use winit::dpi::PhysicalPosition;

pub struct Workspace {
    x: f64,
    y: f64,
    zoom: f64,
    cursor: PhysicalPosition<f64>,
}

impl Workspace {
    pub fn new() -> Self {
        Self {
            x: 0.,
            y: 0.,
            cursor: PhysicalPosition::default(),
            zoom: 1.,
        }
    }

    pub fn render(&self, renderer: &mut WindowRenderer) {
        let mut scene = &mut renderer.scene;
        let window = renderer.window.as_ref().unwrap();
        let size = window.inner_size();
        let ui_scale = window.scale_factor();
        let scale = ui_scale * self.zoom;

        // Draw dots
        let gap = 32.0 * scale;

        let mut x = (gap - self.x) % gap;
        let start_y = (gap - self.y) % gap;
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
            &format!("x: {:.2}, y: {:.2}, zoom: {:.1}", self.x, self.y, self.zoom),
            10.0 * ui_scale,
            10.0 * ui_scale,
            16.0 * ui_scale as f32,
            fonts::inter_black_italic(),
        );
    }

    pub fn update_position(&mut self, x: f64, y: f64) {
        self.x -= x;
        self.y -= y;
    }

    pub fn update_cursor(&mut self, cursor: PhysicalPosition<f64>) {
        self.cursor = cursor;
    }

    pub fn update_zoom(&mut self, delta: f64) {
        let point_x = (self.cursor.x + self.x) / self.zoom;
        let point_y = (self.cursor.y + self.y) / self.zoom;

        self.zoom = (self.zoom + delta).clamp(0.3, 1.5);

        self.x = point_x * self.zoom - self.cursor.x;
        self.y = point_y * self.zoom - self.cursor.y;
    }
}
