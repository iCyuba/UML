use crate::app::{Renderer, State};
use crate::elements::element_style::ElementStyle;
use crate::geometry::rect::Rect;
use crate::geometry::{Point, Vec2};
use std::iter;
use taffy::NodeId;

pub trait Element {
    fn node_id(&self) -> NodeId;

    fn get_style(&self) -> &ElementStyle;
    fn get_mut_style(&mut self) -> &mut ElementStyle;

    // Box model

    fn get_hitbox(&self) -> Rect {
        let pos = self.get_style().get_pos();
        let size = self.get_style().get_layout().size;
        Rect::new(pos, size)
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
        let new_pos = self.update_element_style(state, pos);
        self.update_children(state, new_pos);
    }

    fn update_element_style(&mut self, state: &mut State, pos: Point) -> Point {
        let node_id = self.node_id();
        let style = self.get_mut_style();
        let pos = pos + style.get_layout().location.into();
        
        style.update(node_id, state, &pos);
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
