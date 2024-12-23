use crate::animations::animated_property::AnimatedProperty;
use crate::animations::delta_animation::DeltaAnimation;
use crate::app::renderer::{add_text_to_scene, Renderer};
use crate::app::State;
use crate::elements::Element;
use crate::geometry::{Point, Vec2};
use crate::presentation::fonts;
use derive_macros::AnimatedElement;
use taffy::prelude::length;
use taffy::Position::Absolute;
use taffy::{Layout, NodeId, Style, TaffyTree};
use vello::kurbo::{self, Affine, Circle};
use vello::peniko::Fill;
use winit::event::MouseButton;
use winit::keyboard::NamedKey;

#[derive(AnimatedElement)]
pub struct Workspace {
    layout: Layout,
    node_id: NodeId,

    position: AnimatedProperty<DeltaAnimation<Vec2>>,
    zoom: AnimatedProperty<DeltaAnimation<f64>>,
}

impl Workspace {
    pub fn new(flex_tree: &mut TaffyTree) -> Self {
        Self {
            layout: Default::default(),
            node_id: flex_tree
                .new_leaf(Style {
                    position: Absolute,
                    inset: length(0.),
                    ..Default::default()
                })
                .unwrap(),

            position: AnimatedProperty::new(DeltaAnimation::new(Default::default(), 30.)),
            zoom: AnimatedProperty::new(DeltaAnimation::new(1., 30.)),
        }
    }
}

impl Element for Workspace {
    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn get_layout(&self) -> &Layout {
        &self.layout
    }

    fn set_layout(&mut self, layout: Layout) {
        self.layout = layout;
    }

    fn update(&mut self, state: &mut State, _: Point) {
        state.redraw |= self.animate();
    }

    fn render(&self, r: &mut Renderer, _: &State) {
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
            let zoom = *self.zoom;
            let point = (state.cursor + *self.position) / zoom;

            let zoom = (self.zoom.get_target() + zoom * delta.y / 256.).clamp(0.2, 1.5);

            self.zoom.set(zoom);
            self.position.set(point * zoom - state.cursor);
        } else {
            let (mut x, mut y) = delta.into();
            if shift {
                (x, y) = (y, x);
            }

            let target = *self.position - Vec2::new(x, y);

            if mouse {
                self.position.set(target);
            } else {
                self.position.reset(target);
            }
        }

        state.redraw = true;
    }

    fn on_mousemove(&mut self, state: &mut State, cursor: Point) {
        let is_dragging = state.mouse_buttons.contains(&MouseButton::Middle)
            || (state.keys.contains(&NamedKey::Space.into())
                && state.mouse_buttons.contains(&MouseButton::Left));

        if is_dragging {
            let pos = *self.position - (cursor - state.cursor);
            self.position.reset(pos);
            state.redraw = true;
        }
    }
}
