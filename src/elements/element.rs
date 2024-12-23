use crate::app::{Renderer, State};
use crate::geometry::{Point, Vec2};
use std::iter;
use taffy::{Layout, NodeId};
use crate::geometry::rect::Rect;

pub trait Element {
    fn node_id(&self) -> NodeId;

    // Box model

    fn get_layout(&self) -> &Layout;

    fn set_layout(&mut self, layout: Layout);

    fn get_hitbox(&self) -> Rect {
        let layout = self.get_layout();
        Rect::new(layout.location, layout.size)
    }

    // Child elements

    fn children(&self) -> Box<dyn Iterator<Item = &dyn Element> + '_> {
        Box::new(iter::empty())
    }

    fn children_mut(&mut self) -> Box<dyn Iterator<Item = &mut dyn Element> + '_> {
        Box::new(iter::empty())
    }

    // Lifecycle

    fn update(&mut self, state: &mut State, pos: Point) {
        let new_pos = self.update_layout(state, pos);
        self.update_children(state, new_pos);
    }

    fn update_layout(&mut self, state: &mut State, pos: Point) -> Point {
        let mut layout = state.flex_tree.layout(self.node_id()).unwrap().to_owned();
        let pos = pos + layout.location.into();
        
        layout.location = pos.into();
        self.set_layout(layout);
        
        pos
    }

    fn update_children(&mut self, state: &mut State, pos: Point) {
        for child in self.children_mut() {
            child.update(state, pos);
        }
    }

    fn render(&self, r: &mut Renderer, state: &State) {
        self.render_children(r, state);
    }

    fn render_children(&self, r: &mut Renderer, state: &State) {
        for child in self.children() {
            child.render(r, state);
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
