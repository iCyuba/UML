use crate::animations::animated_property::AnimatedProperty;
use crate::app::renderer::{add_text_to_scene, Renderer};
use crate::app::State;
use crate::elements::Element;
use crate::geometry::{Point, Vec2};
use crate::presentation::fonts;
use vello::kurbo::{self, Affine, Circle};
use vello::peniko::Fill;
use winit::event::MouseButton;
use winit::keyboard::NamedKey;

pub struct Workspace {
    position: AnimatedProperty<Vec2>,
    zoom: AnimatedProperty<f64>,
}

impl Workspace {
    pub fn new() -> Self {
        Self {
            position: AnimatedProperty::new(Default::default()),
            zoom: AnimatedProperty::new(1.),
        }
    }
}

impl Element for Workspace {
    fn update(&mut self, state: &mut State) {
        // TODO: automatically animate all properties
        state.redraw |= self.position.animate();
        state.redraw |= self.zoom.animate();
    }

    fn render(&self, r: &mut Renderer) {
        let window = r.window.as_ref().unwrap();
        let colors = r.colors;
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
                    r.scene.fill(
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

        r.scene.fill(
            Fill::NonZero,
            Affine::translate((-self.position.x, -self.position.y)),
            colors.workspace_dot,
            None,
            &kurbo::Rect::from_origin_size((0.0, 0.0), (64. * scale, 64. * scale)),
        );

        // Coords
        add_text_to_scene(
            &mut r.scene,
            &format!(
                "x: {:.2}, y: {:.2}, zoom: {:.1}",
                self.position.x, self.position.y, *self.zoom
            ),
            10.0 * ui_scale,
            10.0 * ui_scale,
            16.0 * ui_scale as f32,
            fonts::inter_black_italic(),
            colors.workspace_text,
        );
    }

    fn on_scroll(&mut self, state: &mut State, delta: Vec2, mouse: bool, zoom: bool, shift: bool) {
        if zoom {
            let mut zoom = *self.zoom;

            let point = (state.cursor + *self.position) / zoom;

            zoom = (zoom + zoom * delta.y / 192.).clamp(0.2, 1.5);
            self.zoom.set(zoom);

            self.position.set(point * zoom - state.cursor);
        } else {
            let (mut x, mut y) = delta.into();
            if shift {
                (x, y) = (y, x);
            }

            if mouse {
                self.position.update(-Vec2::new(x, y));
            } else {
                *self.position -= Vec2::new(x, y);
            }
        }

        state.redraw = true;
    }

    fn on_mousemove(&mut self, state: &mut State, cursor: Point) {
        let is_dragging = state.mouse_buttons.contains(&MouseButton::Middle)
            || (state.keys.contains(&NamedKey::Space.into())
                && state.mouse_buttons.contains(&MouseButton::Left));

        if is_dragging {
            *self.position -= cursor - state.cursor;
            state.redraw = true;
        }
    }
}
