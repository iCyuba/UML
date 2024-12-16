use crate::fonts;
use crate::renderer::{add_text_to_scene, WindowRenderer};
use std::collections::HashSet;
use vello::kurbo::{Affine, Circle, Rect, Vec2};
use vello::peniko::Fill;
use winit::dpi::PhysicalPosition;

use crate::animations::animated_property::AnimatedProperty;
use winit::event::{MouseButton, MouseScrollDelta};
use winit::event_loop::ControlFlow;
use winit::keyboard::{Key, NamedKey};

pub struct Workspace {
    position: AnimatedProperty<Vec2>,
    zoom: AnimatedProperty<f64>,
    cursor: PhysicalPosition<f64>,
    mouse_buttons: HashSet<MouseButton>,
    keys: HashSet<Key>,
}

impl Workspace {
    pub fn new() -> Self {
        Self {
            position: AnimatedProperty::new(Vec2::ZERO),
            cursor: PhysicalPosition::default(),
            zoom: AnimatedProperty::new(1.),
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
        let scale = *self.zoom * ui_scale;

        // Draw dots
        if *self.zoom > 0.3 {
            let gap = 32.0 * scale;

            let mut x = (gap - self.position.x) % gap;
            let start_y = (gap - self.position.y) % gap;
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
        }

        scene.fill(
            Fill::NonZero,
            Affine::translate((-self.position.x, -self.position.y)),
            colors.workspace_dot,
            None,
            &Rect::from_origin_size((0.0, 0.0), (64. * scale, 64. * scale)),
        );

        // Coords
        add_text_to_scene(
            &mut scene,
            &format!(
                "x: {:.2}, y: {:.2}, zoom: {:.1}",
                self.position.x,
                self.position.y,
                *self.zoom
            ),
            10.0 * ui_scale,
            10.0 * ui_scale,
            16.0 * ui_scale as f32,
            fonts::inter_black_italic(),
            colors.workspace_text,
        );
    }

    // TODO: Don't pass main_modifier as a parameter
    pub fn handle_scroll(&mut self, delta: MouseScrollDelta, main_modifier: NamedKey) {
        if self.keys.contains(&main_modifier.into()) {
            let zoom = match delta {
                MouseScrollDelta::LineDelta(_, y) => y as f64 * 0.2,
                MouseScrollDelta::PixelDelta(PhysicalPosition { y, .. }) => y / 128.,
            } * self.zoom.get();

            self.update_zoom(zoom);
        } else {
            let (mut x, mut y) = match delta {
                MouseScrollDelta::LineDelta(x, y) => (x as f64 * 64., y as f64 * 64.),
                MouseScrollDelta::PixelDelta(PhysicalPosition { x, y }) => (x, y),
            };

            let swap_direction = self.keys.contains(&NamedKey::Shift.into());
            if swap_direction {
                (x, y) = (y, x);
            }

            match delta {
                MouseScrollDelta::LineDelta(_, _) => {
                    self.position.update(-Vec2::new(x, y));
                }
                MouseScrollDelta::PixelDelta(_) => {
                    *self.position -= Vec2::new(x, y);
                }
            }
        }
    }

    pub fn handle_mouse_move(&mut self, cursor: PhysicalPosition<f64>) -> bool {
        let is_dragging = self.mouse_buttons.contains(&MouseButton::Middle)
            || (self.keys.contains(&NamedKey::Space.into())
            && self.mouse_buttons.contains(&MouseButton::Left));

        if is_dragging {
            *self.position -= Vec2::new(cursor.x - self.cursor.x, cursor.y - self.cursor.y);
        }

        self.update_cursor(cursor);
        is_dragging
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
        let mut zoom = *self.zoom;

        let point_x = (self.cursor.x + self.position.x) / zoom;
        let point_y = (self.cursor.y + self.position.y) / zoom;

        zoom = (zoom + delta).clamp(0.2, 1.5);
        self.zoom.set(zoom);

        let pos = Vec2::new(
            point_x * zoom - self.cursor.x,
            point_y * zoom - self.cursor.y,
        );

        self.position.set(pos);
    }

    pub fn animate(&mut self, control_flow: &mut ControlFlow, renderer: &WindowRenderer) {
        // TODO: automatically animate all properties
        let redraw_pos = self.position.animate();
        let redraw_zoom = self.zoom.animate();

        if redraw_pos || redraw_zoom {
            *control_flow = ControlFlow::Poll;
            renderer.request_redraw();
        }
    }
}
