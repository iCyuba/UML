use super::item::Item;
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
        Element,
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

    pub fn setup(tree: &mut Tree, ctx: &mut EventContext) -> NodeId {
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

        ctx.state.key_listeners.insert(node_id);

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
        f: impl FnOnce(&mut Entity) -> bool,
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
            .find(|(_, e)| (*e.data.rect * *self.zoom).contains(point))
            .map(|(k, _)| k)
    }
}

impl EventTarget for Workspace {
    fn update(&mut self, ctx: &mut EventContext) {
        if self.animate() {
            ctx.state.request_redraw();
        }

        // Entities
        let selection = ctx.state.selected_entity;
        let mut redraw = false;
        for (key, entity) in ctx.project.entities.iter_mut() {
            entity.data.is_selected = selection == Some(key);
            redraw |= entity.update()
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

            let gap = 32.0 * scale_zoom;
            let position = *self.position * scale;

            let mut x = 0.;
            let mut y = 0.;

            let transform = Affine::translate(Point::new(gap - position.x, gap - position.y) % gap);

            while x < width as f64 {
                while y < height as f64 {
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
                y = 0.;
            }
        }

        // Render workspace items

        // Entities
        for (_, entity) in project.entities.iter() {
            entity.render(c, state, self);
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
            colors.workspace_text,
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

    fn on_click(&mut self, ctx: &mut EventContext) -> bool {
        ctx.state.screenshot();

        if let Some(selection) = ctx.state.selected_entity {
            let entity = ctx.project.entities.get_mut(selection).unwrap();
            entity.name += "!";
            ctx.state.request_redraw();
        }

        true
    }

    fn on_keydown(&mut self, ctx: &mut EventContext, event: KeyEvent) -> bool {
        if matches!(event.logical_key, Key::Named(NamedKey::Space)) {
            self.select_hand(ctx.state);
            return true;
        }

        false
    }

    fn on_keyup(&mut self, ctx: &mut EventContext, event: KeyEvent) -> bool {
        if matches!(event.logical_key, Key::Named(NamedKey::Space)) {
            self.deselect_hand(ctx.state);
            return true;
        }

        false
    }

    fn on_mousedown(&mut self, ctx: &mut EventContext, button: MouseButton) -> bool {
        let middle = button == MouseButton::Middle;
        let left = button == MouseButton::Left;

        if middle {
            self.select_hand(ctx.state);
        }

        if (left || middle) && ctx.state.tool == Tool::Hand {
            ctx.state.focused = Some(self.node_id);
            ctx.state.request_cursor_update();

            return true;
        }

        if !self.entity_mut(ctx.project, self.hovered, |entity| {
            entity.on_mousedown(ctx.state, button)
        }) {
            ctx.state.selected_entity = None;
            ctx.state.request_redraw();
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

        let entity = self.entity_at_point(ctx.project, cursor + *self.position);
        if entity != self.hovered {
            self.entity_mut(ctx.project, self.hovered, |entity| {
                entity.on_mouseleave(ctx.state)
            });

            self.entity_mut(ctx.project, entity, |entity| {
                entity.on_mouseenter(ctx.state)
            });

            ctx.state.request_cursor_update();

            self.hovered = entity;
        }

        self.entity_mut(ctx.project, self.hovered, |entity| {
            entity.on_mousemove(ctx.state, cursor)
        })
    }

    fn on_mouseup(&mut self, ctx: &mut EventContext, mb: MouseButton) -> bool {
        let left = ctx.state.mouse_buttons.contains(&MouseButton::Left);
        let middle = ctx.state.mouse_buttons.contains(&MouseButton::Middle);
        let space = ctx.state.keys.contains(&NamedKey::Space.into());

        if self.is_dragging(ctx.state) && !left && !middle {
            ctx.state.focused = None;
            ctx.state.request_cursor_update();

            if !space {
                self.deselect_hand(ctx.state);
            }

            return true;
        }

        self.entity_mut(ctx.project, self.hovered, |entity| {
            entity.on_mouseup(ctx.state, mb)
        })
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

impl Element for Workspace {
    fn layout(&self) -> &Layout {
        &self.layout
    }

    fn layout_mut(&mut self) -> &mut Layout {
        &mut self.layout
    }
}
