use super::item::Item;
use crate::animations::animated_property::AnimatedProperty;
use crate::animations::delta_animation::DeltaAnimation;
use crate::app::renderer::Renderer;
use crate::app::{EventTarget, State, Tree};
use crate::data::project::EntityKey;
use crate::data::{Entity, Project};
use crate::elements::primitives::text::Text;
use crate::elements::primitives::traits::Draw;
use crate::elements::toolbox_item::Tool;
use crate::elements::Element;
use crate::geometry::rect::Rect;
use crate::geometry::{Point, Vec2};
use crate::presentation::fonts;
use derive_macros::AnimatedElement;
use taffy::{Layout, NodeId, Position, Style};
use vello::kurbo::{Affine, Circle};
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

    previous_tool: Option<Tool>,
    hovered: Option<EntityKey>,
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

            previous_tool: None,
            hovered: None,
        };

        tree.set_node_context(node_id, Some(Box::new(this)))
            .unwrap();

        state.key_listeners.insert(node_id);

        node_id
    }

    #[inline]
    pub fn position(&self) -> Vec2 {
        *self.position
    }

    #[inline]
    pub fn zoom(&self) -> f64 {
        *self.zoom
    }

    #[inline]
    fn is_dragging(&self, state: &State) -> bool {
        state.focused == Some(self.node_id)
    }

    fn select_hand(&mut self, state: &mut State) {
        if state.tool == Tool::Hand {
            return;
        }

        self.previous_tool = Some(state.tool);
        state.tool = Tool::Hand;

        state.request_redraw();
        state.request_cursor_update();
    }

    fn deselect_hand(&mut self, state: &mut State) {
        if self.is_dragging(state) {
            return;
        }

        if let Some(tool) = self.previous_tool.take() {
            state.tool = tool;
            state.request_redraw();
            state.request_cursor_update();
        }
    }

    fn entity_mut(
        &mut self,
        project: &mut Project,
        entity: Option<EntityKey>,
        mut f: impl FnMut(&mut Entity) -> bool,
    ) -> bool {
        if let Some(entity) = entity.and_then(|key| project.entities.get_mut(key)) {
            return f(entity);
        }

        false
    }

    pub fn entity_at_point(&self, project: &Project, point: Point) -> Option<EntityKey> {
        project
            .entities
            .iter()
            .find(|(_, e)| (e.rect * *self.zoom).contains(point))
            .map(|(k, _)| k)
    }
}

impl EventTarget for Workspace {
    fn update(&mut self, _: &Renderer, state: &mut State, project: &mut Project) {
        if self.animate() {
            state.request_redraw();
        }

        // Entities
        for (_, entity) in project.entities.iter_mut() {
            entity.update();
        }
    }

    fn render(&self, r: &mut Renderer, state: &State, project: &Project) {
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

        // Render workspace items

        // Entities
        for (_, entity) in project.entities.iter() {
            entity.render(r, state, self);
        }

        // Coords
        Text::new(
            &format!(
                "x: {:.2}, y: {:.2}, zoom: {:.1}",
                self.position.x, self.position.y, *self.zoom
            ),
            ui_scale,
            Rect::new((10., 10.), (r.size().width as f64 - 20., 16.)),
            16.0,
            fonts::inter_black_italic(),
            colors.workspace_text,
        )
        .draw(&mut r.scene);
    }

    fn cursor(&self, state: &State) -> Option<CursorIcon> {
        if self.is_dragging(state) {
            Some(CursorIcon::Grabbing)
        } else if state.tool == Tool::Hand {
            Some(CursorIcon::Grab)
        } else {
            None
        }
    }

    fn on_keydown(&mut self, state: &mut State, _: &mut Project, key: &Key) -> bool {
        if matches!(key, Key::Named(NamedKey::Space)) {
            self.select_hand(state);
            return true;
        }

        false
    }

    fn on_keyup(&mut self, state: &mut State, _: &mut Project, key: &Key) -> bool {
        if matches!(key, Key::Named(NamedKey::Space)) {
            self.deselect_hand(state);
            return true;
        }

        false
    }

    fn on_mousedown(
        &mut self,
        state: &mut State,
        project: &mut Project,
        button: MouseButton,
    ) -> bool {
        let middle = button == MouseButton::Middle;
        let left = button == MouseButton::Left;

        if middle {
            self.select_hand(state);
        }

        if (left || middle) && state.tool == Tool::Hand {
            state.focused = Some(self.node_id);
            state.request_cursor_update();

            return true;
        }

        if !self.entity_mut(project, self.hovered, |entity| {
            entity.on_mousedown(state, button)
        }) {
            state.selected_entity = None;
            state.request_redraw();
        }

        true
    }

    fn on_mousemove(&mut self, state: &mut State, project: &mut Project, cursor: Point) -> bool {
        if self.is_dragging(state) {
            let pos: Vec2 = *self.position - (cursor - state.cursor);
            self.position.reset(pos);
            state.request_redraw();

            return true;
        }

        let entity = self.entity_at_point(project, cursor + *self.position);
        if entity != self.hovered {
            self.entity_mut(project, self.hovered, |entity| entity.on_mouseleave(state));

            self.entity_mut(project, entity, |entity| entity.on_mouseenter(state));

            state.request_cursor_update();

            self.hovered = entity;
        }

        self.entity_mut(project, self.hovered, |entity| {
            entity.on_mousemove(state, cursor)
        })
    }

    fn on_mouseup(&mut self, state: &mut State, project: &mut Project, mb: MouseButton) -> bool {
        let left = state.mouse_buttons.contains(&MouseButton::Left);
        let middle = state.mouse_buttons.contains(&MouseButton::Middle);
        let space = state.keys.contains(&NamedKey::Space.into());

        if self.is_dragging(state) && !left && !middle {
            state.focused = None;
            state.request_cursor_update();

            if !space {
                self.deselect_hand(state);
            }

            return true;
        }

        self.entity_mut(project, self.hovered, |entity| entity.on_mouseup(state, mb))
    }

    fn on_wheel(
        &mut self,
        state: &mut State,
        _: &mut Project,
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
