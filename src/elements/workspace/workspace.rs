use super::item::Item;
use crate::data::connection::{Relation, RelationType};
use crate::data::entity::EntityType;
use crate::data::project::ConnectionKey;
use crate::data::Connection;
use crate::elements::node::Element;
use crate::{
    animations::{animated_property::AnimatedProperty, delta_animation::DeltaAnimation},
    app::{
        context::{EventContext, GetterContext, RenderContext},
        event_target::WheelEvent,
        EventTarget, State, Tree,
    },
    data::{project::EntityKey, Entity, Project},
    elements::{
        primitives::{text::Text, traits::Draw},
        toolbox_item::Tool,
        Node,
    },
    geometry::{Point, Rect, Vec2},
    presentation::fonts,
};
use derive_macros::AnimatedElement;
use taffy::{Layout, NodeId, Position, Style};
use vello::{
    kurbo::{Affine, Circle},
    peniko::Fill,
};
use winit::{
    event::{KeyEvent, MouseButton},
    keyboard::{Key, NamedKey},
    window::CursorIcon,
};

#[derive(AnimatedElement)]
pub struct Workspace {
    layout: Layout,
    node_id: NodeId,

    position: AnimatedProperty<DeltaAnimation<Vec2>>,
    zoom: AnimatedProperty<DeltaAnimation<f64>>,

    pub previous_tool: Option<Tool>,
    pub hovered_entity: Option<EntityKey>,
    pub hovered_connection: Option<ConnectionKey>,

    /// The point at which the mouse was pressed down
    ///
    /// Used for moving entities
    move_start_point: Option<Point>,
}

impl Workspace {
    const ZOOM_MIN: f64 = 0.2;
    const ZOOM_MAX: f64 = 1.5;
    pub const GRID_SIZE: f64 = 32.;

    const STYLE: Style = {
        let mut style = Style::DEFAULT;
        style.position = Position::Absolute;
        style.inset = taffy::Rect::zero();

        style
    };

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
        state.capturing == Some(self.node_id)
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

    pub fn cursor_to_point(&self, cursor: Point) -> Point {
        (cursor + *self.position) / *self.zoom
    }

    /// Finds the highest (z-order) entity located at the given point
    pub fn entity_at_point(&self, project: &Project, point: Point) -> Option<EntityKey> {
        project
            .ordered_entities
            .iter()
            .rev()
            .find(|&key| (*project.entities[*key].data.rect * *self.zoom).contains(point))
            .copied()
    }
}

impl EventTarget for Workspace {
    fn update(&mut self, ctx: &mut EventContext) {
        if self.animate() {
            ctx.state.request_redraw();
        }

        // Entities
        let mut redraw = false;
        for (_, entity) in ctx.project.entities.iter_mut() {
            if entity.update(ctx.state, self) {
                // Update connections in case the entity was resized
                for conn in entity.connections.iter() {
                    ctx.project.connections[*conn].update_origin(
                        entity.key,
                        entity.get_rect(),
                        true,
                    );
                }

                redraw = true;
            }
        }

        for (_, conn) in ctx.project.connections.iter_mut() {
            redraw |= conn.update(ctx.state, self);
        }

        if redraw {
            ctx.state.request_redraw();
        }
    }

    fn render(&self, RenderContext { c, project, state }: &mut RenderContext) {
        let colors = c.colors();

        // Draw dots
        if *self.zoom > 0.3 {
            let scale = c.scale();
            let (width, height) = c.size();

            let scale_zoom = scale * *self.zoom;

            let gap = Self::GRID_SIZE * scale_zoom;
            let position = *self.position * scale;

            let mut x = -gap;
            let mut y = -gap;

            // Grid is offset by half a grid cell to simplify rendering items in the workspace
            let offset = Point::new(gap, gap) / 2.;
            let transform = Affine::translate(-position % gap + offset);

            while x < width as f64 {
                while y < (height + gap as u32) as f64 {
                    c.scene().fill(
                        Fill::NonZero,
                        transform,
                        colors.workspace_dot,
                        None,
                        &Circle::new((x, y), 2.0 * scale_zoom),
                    );

                    y += gap;
                }

                x += gap;
                y = -gap;
            }
        }

        // Render workspace items

        // Connections
        for (_, conn) in project.connections.iter() {
            conn.render(c, state, self)
        }

        // Entities
        for entity in project.ordered_entities.iter() {
            project.entities[*entity].render(c, state, self);
        }

        // Coords
        Text::new(
            &format!(
                "x: {:.2}, y: {:.2}, zoom: {:.1}",
                self.position.x, self.position.y, *self.zoom
            ),
            Rect::new((10., 10.), (c.size().0 as f64 - 20., 16.)),
            16.0,
            fonts::inter_black_italic(),
            colors.text,
            true,
        )
        .draw(c);
    }

    fn cursor(&self, ctx: &GetterContext) -> Option<CursorIcon> {
        if self.is_dragging(ctx.state) {
            Some(CursorIcon::Grabbing)
        } else if ctx.state.tool == Tool::Hand {
            Some(CursorIcon::Grab)
        } else {
            None
        }
    }

    fn on_keydown(&mut self, ctx: &mut EventContext, event: KeyEvent) -> bool {
        let key = event.logical_key;
        if matches!(key, Key::Named(NamedKey::Space)) {
            self.select_hand(ctx.state);
            return true;
        }

        false
    }

    fn on_keyup(&mut self, ctx: &mut EventContext, event: KeyEvent) -> bool {
        let key = event.logical_key;
        if matches!(key, Key::Named(NamedKey::Space)) {
            self.deselect_hand(ctx.state);
            return true;
        }

        false
    }

    fn on_mousedown(&mut self, ctx: &mut EventContext, button: MouseButton) -> bool {
        let middle = button == MouseButton::Middle;
        let left = button == MouseButton::Left;
        let right = button == MouseButton::Right;
        let point = self.cursor_to_point(ctx.state.cursor);

        if middle {
            self.select_hand(ctx.state);
        }

        if (left || middle) && ctx.state.tool == Tool::Hand {
            ctx.state.capturing = Some(self.node_id);
            ctx.state.request_cursor_update();

            return true;
        }

        if ctx.state.tool == Tool::Relation {
            // Connect entities
            if match (self.hovered_entity, ctx.state.selected_entity) {
                (Some(old), Some(new)) => {
                    if old == new {
                        return false;
                    }

                    let from_rect = ctx.project.entities[old].get_rect();
                    let to_rect = ctx.project.entities[new].get_rect();

                    ctx.project.connect(Connection::new(
                        RelationType::Association,
                        Relation::new(old),
                        Relation::new(new),
                        Vec::new(),
                        from_rect,
                        to_rect,
                    ));

                    true
                }
                _ => false,
            } {
                self.hovered_entity = None;
                ctx.state.selected_entity = None;
                ctx.state.request_redraw();
                return true;
            }

            // Disconnect entities
            if let Some(connection) = self.hovered_connection {
                if right {
                    ctx.project.disconnect(connection);
                    ctx.state.request_redraw();
                    return true;
                }
            }
        }

        if ctx.state.tool == Tool::Pen {
            // Set hovered point
            if let Some(key) = self.hovered_connection {
                let connection = &mut ctx.project.connections[key];
                let cursor = self.cursor_to_point(ctx.state.cursor) / Workspace::GRID_SIZE;
                let zoom_adjustment = ctx.c.scale() * self.zoom();

                if let Some(point) =
                    connection.get_hovered_path_point(&cursor, 0.5 / zoom_adjustment)
                {
                    if left {
                        // Select point
                        ctx.state.selected_point = Some((key, point));
                        return true;
                    } else if right {
                        // Remove point
                        connection.remove_point(point);
                        ctx.state.request_redraw();
                        return true;
                    }
                } else if let Some((index, point)) =
                    connection.get_hovered_line(&cursor, 0.5 / zoom_adjustment)
                {
                    if left {
                        // Add point
                        connection.add_point(index, point.round());
                        ctx.state.selected_point = Some((key, index));
                        ctx.state.request_redraw();
                        return true;
                    }
                }
            }
        }

        // Move entity
        if !ctx.project.entity_mut(self.hovered_entity, |entity| {
            if left && ctx.state.tool == Tool::Select {
                self.move_start_point = Some(point);
                entity.data.move_pos = Some(Vec2::ZERO);
            }

            false
        }) {
            ctx.state.selected_entity = None;
            ctx.state.request_redraw();
        }

        // Create new entity
        if left && ctx.state.tool == Tool::Entity {
            let entity = Entity::new(
                "Empty".to_string(),
                EntityType::Class,
                (point / Workspace::GRID_SIZE).into(),
            );
            let key = ctx.project.add_entity(entity);

            ctx.state.selected_entity = Some(key);
            ctx.state.request_redraw();
        }

        // Move the selected entity to the top
        if let Some(entity) = self.hovered_entity {
            ctx.project.ordered_entities.retain(|&k| k != entity);
            ctx.project.ordered_entities.push(entity);
        }

        true
    }

    fn on_mousemove(&mut self, ctx: &mut EventContext, cursor: Point) -> bool {
        if self.is_dragging(ctx.state) {
            let pos: Vec2 = *self.position - (cursor - ctx.state.cursor);
            self.position.reset(pos);
            ctx.state.request_redraw();

            return true;
        }

        if let Some(old) = self.move_start_point {
            let cursor = self.cursor_to_point(ctx.state.cursor);
            let diff = cursor - old;

            // Move the selected entity
            if ctx.project.entity_mut(self.hovered_entity, |entity| {
                entity.data.move_pos = Some(diff);
                ctx.state.request_redraw();

                true
            }) {
                let key = self.hovered_entity.unwrap();
                let rect = ctx.project.entities[key].get_rect();

                for conn in ctx.project.get_entity_connections(key) {
                    ctx.project.connections[conn].update_origin(key, rect, true);
                }

                return true;
            }
        }

        // Hovered entity
        let entity = self.entity_at_point(ctx.project, cursor + *self.position);
        if entity != self.hovered_entity {
            ctx.state.request_cursor_update();
            ctx.state.request_redraw();
            self.hovered_entity = entity;
            return true;
        }

        // Hovered connection point
        let zoom_adjustment = ctx.c.scale() * self.zoom();
        let point = self.cursor_to_point(ctx.state.cursor) / Workspace::GRID_SIZE;
        let mut connection = None;

        if ctx.state.selected_point.is_none() {
            for (key, conn) in ctx.project.connections.iter_mut() {
                if let Some((_, point)) = conn.get_hovered_line(&point, 0.5 / zoom_adjustment) {
                    conn.data.ghost_point = Some(point.round());
                    connection = Some(key);
                } else {
                    conn.data.ghost_point = None;
                }
            }
        }

        if connection != self.hovered_connection {
            ctx.state.request_cursor_update();
            ctx.state.request_redraw();
            self.hovered_connection = connection;
            return true;
        }

        // Force update when the connection line is hovered
        if self.hovered_connection.is_some() {
            ctx.state.request_redraw();
            return true;
        }

        // Connection point dragging
        if ctx.state.tool == Tool::Pen {
            if let Some((connection, index)) = ctx.state.selected_point {
                let connection = &mut ctx.project.connections[connection];
                let cursor =
                    (self.cursor_to_point(ctx.state.cursor) / Workspace::GRID_SIZE).round();

                connection.update_point(index, cursor);
                ctx.state.request_redraw();
                return true;
            }
        }

        false
    }

    fn on_mouseup(&mut self, ctx: &mut EventContext, _: MouseButton) -> bool {
        self.move_start_point = None;

        let left = ctx.state.mouse_buttons.contains(&MouseButton::Left);
        let middle = ctx.state.mouse_buttons.contains(&MouseButton::Middle);
        let space = ctx.state.keys.contains(&NamedKey::Space.into());

        if self.is_dragging(ctx.state) && !left && !middle {
            ctx.state.capturing = None;
            ctx.state.request_cursor_update();

            if !space {
                self.deselect_hand(ctx.state);
            }

            return true;
        }

        // Release connection point
        if let Some((connection, index)) = ctx.state.selected_point {
            let connection = &mut ctx.project.connections[connection];
            let cursor = (self.cursor_to_point(ctx.state.cursor) / Workspace::GRID_SIZE).round();

            connection.update_point(index, cursor);

            ctx.state.selected_point = None;
            ctx.state.request_redraw();
            return true;
        }

        if ctx.project.entity_mut(self.hovered_entity, |entity| {
            // Snap the entity to the grid
            let pos = entity.data.move_pos.take();
            if let Some(pos) = pos {
                // Set the rect origin for a smooth transition
                let rect = entity.data.rect.translate(pos);
                entity.data.rect.reset(rect);

                entity.position = (rect.center() / Workspace::GRID_SIZE).into();
                if entity.update(ctx.state, self) {
                    ctx.state.request_redraw();
                }
            }

            ctx.state.selected_entity = Some(entity.key);
            ctx.state.request_redraw();

            true
        }) {
            let key = self.hovered_entity.unwrap();
            let rect = ctx.project.entities[key].get_rect();
            let rect = Rect::new(rect.center().round() - rect.size / 2., rect.size);

            for conn in ctx.project.get_entity_connections(key) {
                let connection = &mut ctx.project.connections[conn];
                connection.update_origin(key, rect, false);
            }

            return true;
        }

        false
    }

    fn on_wheel(&mut self, ctx: &mut EventContext, event: WheelEvent) -> bool {
        if event.zoom {
            let zoom = *self.zoom;
            let point = (ctx.state.cursor + *self.position) / zoom;

            let zoom = (self.zoom.get_target() + zoom * event.delta.y / 256.)
                .clamp(Self::ZOOM_MIN, Self::ZOOM_MAX);

            self.zoom.set(zoom);
            self.position.set(point * zoom - ctx.state.cursor);
        } else {
            let (mut x, mut y) = event.delta.into();
            if event.reverse {
                (x, y) = (y, x);
            }

            let target = *self.position - Vec2::new(x, y);

            if event.mouse {
                self.position.set(target);
            } else {
                self.position.reset(target);
            }
        }

        ctx.state.request_redraw();

        true
    }
}

impl Node for Workspace {
    fn layout(&self) -> &Layout {
        &self.layout
    }

    fn layout_mut(&mut self) -> &mut Layout {
        &mut self.layout
    }
}

impl Element for Workspace {
    fn setup(tree: &mut Tree, ctx: &mut EventContext) -> NodeId {
        tree.add_element(ctx, Self::STYLE, None, |node_id, ctx| {
            ctx.state.key_listeners.insert(node_id);

            Self {
                layout: Default::default(),
                node_id,

                position: AnimatedProperty::new(DeltaAnimation::initialized(
                    Default::default(),
                    30.,
                )),
                zoom: AnimatedProperty::new(DeltaAnimation::initialized(1., 30.)),

                previous_tool: None,
                hovered_entity: None,
                hovered_connection: None,
                move_start_point: None,
            }
        })
    }
}
