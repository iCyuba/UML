use crate::animations::animated_property::AnimatedProperty;
use crate::animations::delta_animation::DeltaAnimation;
use crate::app::renderer::{add_text_to_scene, Renderer};
use crate::app::{EventTarget, State, Tree};
use crate::elements::Element;
use crate::geometry::{Point, Vec2};
use crate::presentation::fonts;
use derive_macros::AnimatedElement;
use taffy::{Layout, NodeId, Position, Style};
use vello::kurbo::{self, Affine, Circle};
use vello::peniko::Fill;
use winit::event::MouseButton;
use winit::keyboard::{Key, NamedKey};
use winit::window::CursorIcon;

#[derive(AnimatedElement)]
pub struct Workspace {
    layout: Layout,
    node_id: NodeId,

    position: AnimatedProperty<DeltaAnimation<Vec2>>,
    zoom: AnimatedProperty<DeltaAnimation<f64>>,
}

impl Workspace {
    const ZOOM_MIN: f64 = 0.2;
    const ZOOM_MAX: f64 = 1.5;

    const STYLE: Style = {
        let mut style = Style::DEFAULT;
        style.position = Position::Absolute;
        style.inset = taffy::Rect::zero();

        style
    };

    pub fn setup(tree: &mut Tree, state: &mut State) -> NodeId {
        let node_id = tree.new_leaf(Self::STYLE).unwrap();
        let this = Self {
            layout: Default::default(),
            node_id,

            position: AnimatedProperty::new(DeltaAnimation::initialized(Default::default(), 30.)),
            zoom: AnimatedProperty::new(DeltaAnimation::initialized(1., 30.)),
        };

        tree.set_node_context(node_id, Some(Box::new(this)))
            .unwrap();

        state.key_listeners.insert(node_id);

        node_id
    }

    fn is_dragging(&self, state: &State) -> bool {
        state.focused == Some(self.node_id)
    }
}

impl EventTarget for Workspace {
    fn update(&mut self, _: &Renderer, state: &mut State) {
        if self.animate() {
            state.request_redraw();
        }
    }

    fn render(&self, r: &mut Renderer, _: &State) {
        let colors = r.colors;

        let zoom = *self.zoom;
        let ui_scale = r.scale();
        let scale = zoom * ui_scale;

        // Draw dots
        if *self.zoom > 0.3 {
            let gap = 32.0 * scale;

            let mut x = (gap - self.position.x) % gap;
            let start_y = (gap - self.position.y) % gap;
            let mut y = start_y;

            while x < self.layout.size.width as f64 {
                while y < self.layout.size.height as f64 {
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

    fn cursor(&self, state: &State) -> Option<CursorIcon> {
        if self.is_dragging(state) {
            Some(CursorIcon::Grabbing)
        } else if state.keys.contains(&NamedKey::Space.into()) {
            Some(CursorIcon::Grab)
        } else {
            None
        }
    }

    fn on_keydown(&mut self, state: &mut State, key: &Key) -> bool {
        if matches!(key, Key::Named(NamedKey::Space)) {
            state.request_cursor_update();
            return true;
        }

        false
    }

    fn on_keyup(&mut self, state: &mut State, key: &Key) -> bool {
        if matches!(key, Key::Named(NamedKey::Space)) {
            state.request_cursor_update();
            return true;
        }

        false
    }

    fn on_mousedown(&mut self, state: &mut State, button: MouseButton) -> bool {
        let left = button == MouseButton::Left && state.keys.contains(&NamedKey::Space.into());
        let middle = button == MouseButton::Middle;

        if left || middle {
            state.focused = Some(self.node_id);
            state.request_cursor_update();

            return true;
        }

        false
    }

    fn on_mousemove(&mut self, state: &mut State, cursor: Point) -> bool {
        if self.is_dragging(state) {
            let pos: Vec2 = *self.position - (cursor - state.cursor);
            self.position.reset(pos);
            state.request_redraw();

            return true;
        }

        false
    }

    fn on_mouseup(&mut self, state: &mut State, _: MouseButton) -> bool {
        let left = state.mouse_buttons.contains(&MouseButton::Left);
        let middle = state.mouse_buttons.contains(&MouseButton::Middle);

        if self.is_dragging(state) && !left && !middle {
            state.focused = None;
            state.request_cursor_update();

            return true;
        }

        false
    }

    fn on_wheel(
        &mut self,
        state: &mut State,
        delta: Vec2,
        mouse: bool,
        zoom: bool,
        reverse: bool,
    ) -> bool {
        if zoom {
            let zoom = *self.zoom;
            let point = (state.cursor + *self.position) / zoom;

            let zoom = (self.zoom.get_target() + zoom * delta.y / 256.)
                .clamp(Self::ZOOM_MIN, Self::ZOOM_MAX);

            self.zoom.set(zoom);
            self.position.set(point * zoom - state.cursor);
        } else {
            let (mut x, mut y) = delta.into();
            if reverse {
                (x, y) = (y, x);
            }

            let target = *self.position - Vec2::new(x, y);

            if mouse {
                self.position.set(target);
            } else {
                self.position.reset(target);
            }
        }

        state.request_redraw();

        true
    }
}

impl Element for Workspace {
    fn layout(&self) -> &Layout {
        &self.layout
    }

    fn layout_mut(&mut self) -> &mut Layout {
        &mut self.layout
    }
}
