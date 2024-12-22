use crate::app::{Renderer, State};
use crate::geometry::rect::Rect;
use crate::geometry::{Point, Vec2};
use std::iter;
use std::ops::Add;
use taffy::{BoxSizing, Layout, NodeId, Style};

pub trait Element {
    fn node_id(&self) -> NodeId;

    fn get_layout<'a>(&self, state: &'a State) -> &'a Layout {
        state.flex_tree.layout(self.node_id()).unwrap()
    }

    fn get_style<'a>(&self, state: &'a State) -> &'a Style {
        state.flex_tree.style(self.node_id()).unwrap()
    }

    // Box model

    fn hitbox(&self, state: &State, pos: Point) -> Rect {
        let Layout {
            location,
            margin,
            padding,
            border,
            size,
            ..
        } = self.get_layout(state);
        let style = self.get_style(state);

        let (x, y) = (
            pos.x as f32 + location.x + margin.left,
            pos.y as f32 + location.y + margin.top,
        );

        let (width, height) = match style.box_sizing {
            BoxSizing::BorderBox => (size.width, size.height),
            BoxSizing::ContentBox => (
                size.width + padding.left + padding.right + border.left + border.right,
                size.height + padding.top + padding.bottom + border.top + border.bottom,
            ),
        };

        Rect::from_origin_size((x, y), (width, height))
    }

    fn content_offset(&self, state: &State) -> Point {
        let Layout {
            location,
            margin,
            padding,
            border,
            ..
        } = self.get_layout(state);
        let style = self.get_style(state);

        let (x, y) = match style.box_sizing {
            BoxSizing::BorderBox => (location.x + margin.left, location.y + margin.top),
            BoxSizing::ContentBox => (
                location.x + margin.left + padding.left + border.left,
                location.y + margin.top + padding.top + border.top,
            ),
        };

        (x as f64, y as f64).into()
    }

    // Child elements

    fn children(&self) -> Box<dyn Iterator<Item = &dyn Element> + '_> {
        Box::new(iter::empty())
    }

    fn children_mut(&mut self) -> Box<dyn Iterator<Item = &mut dyn Element> + '_> {
        Box::new(iter::empty())
    }

    // Lifecycle

    fn update(&mut self, _state: &mut State) {
        self.update_children();
    }

    fn update_children(&mut self) {
        for child in self.children_mut() {
            child.update_children();
        }
    }

    fn render(&self, r: &mut Renderer, state: &State, pos: Point) {
        self.render_children(r, state, pos);
    }

    fn render_children(&self, r: &mut Renderer, state: &State, pos: Point) {
        let pos = pos.add(self.content_offset(state));

        for child in self.children() {
            child.render(r, state, pos);
        }
    }

    // Events

    fn on_scroll(
        &mut self,
        state: &mut State,
        delta: Vec2,
        mouse: bool,
        zoom: bool,
        reverse: bool,
    ) {
        for child in self.children_mut() {
            child.on_scroll(state, delta, mouse, zoom, reverse);
        }
    }

    fn on_mousemove(&mut self, state: &mut State, cursor: Point) {
        for child in self.children_mut() {
            child.on_mousemove(state, cursor);
        }
    }
}
