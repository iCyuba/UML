use std::collections::HashSet;
use crate::fonts;
use crate::renderer::{add_text_to_scene, WindowRenderer};
use vello::kurbo::{Affine, Circle};
use vello::peniko::Fill;
use winit::dpi::PhysicalPosition;

use winit::event::MouseButton;
use winit::keyboard::{Key, NamedKey};

pub struct Workspace {
    x: f64,
    y: f64,
    zoom: f64,
    cursor: PhysicalPosition<f64>,
    mouse_buttons: HashSet<MouseButton>,
    keys: HashSet<Key>,
}

impl Workspace {
    pub fn new() -> Self {
        Self {
            x: 0.,
            y: 0.,
            cursor: PhysicalPosition::default(),
            zoom: 1.,
            mouse_buttons: HashSet::new(),
            keys: HashSet::new(),
        }
    }

    pub fn render(&self, renderer: &mut WindowRenderer) {
        let mut scene = &mut renderer.scene;
        let window = renderer.window.as_ref().unwrap();
        let colors = renderer.colors;
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
                    colors.workspace_dot,
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
            colors.workspace_text,
        );
    }

    pub fn handle_scroll(&mut self, x: f32, y: f32) {
        if self.keys.contains(&Key::Named(NamedKey::Control)) {
            self.update_zoom(y as f64 * 0.1);
        } else {
            self.update_position(x as f64 * 32., y as f64 * 32.);
        }
    }

    pub fn handle_mouse_move(&mut self, cursor: PhysicalPosition<f64>) -> bool {
        let is_dragging = self.mouse_buttons.contains(&MouseButton::Middle);

        if is_dragging {
            self.update_position(cursor.x - self.cursor.x, cursor.y - self.cursor.y);
        }

        self.update_cursor(cursor);
        is_dragging
    }

    pub fn update_position(&mut self, x: f64, y: f64) {
        self.x -= x;
        self.y -= y;
    }

    pub fn update_cursor(&mut self, cursor: PhysicalPosition<f64>) {
        self.cursor = cursor;
    }

    pub fn update_mouse_buttons(&mut self, button: MouseButton, pressed: bool) {
        if pressed {
            self.mouse_buttons.insert(button);
        } else {
            self.mouse_buttons.remove(&button);
        }
    }

    pub fn update_keys(&mut self, key: Key, pressed: bool) {
        if pressed {
            self.keys.insert(key);
        } else {
            self.keys.remove(&key);
        }
    }

    pub fn update_zoom(&mut self, delta: f64) {
        let point_x = (self.cursor.x + self.x) / self.zoom;
        let point_y = (self.cursor.y + self.y) / self.zoom;

        self.zoom = (self.zoom + delta).clamp(0.3, 1.5);

        self.x = point_x * self.zoom - self.cursor.x;
        self.y = point_y * self.zoom - self.cursor.y;
    }
}