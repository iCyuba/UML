use std::iter;

use crate::app::{Renderer, State};
use crate::geometry::{Point, Vec2};

pub trait Element {
    // Child elements

    fn children(&self) -> Box<dyn Iterator<Item = &dyn Element> + '_> {
        Box::new(iter::empty())
    }

    fn children_mut(&mut self) -> Box<dyn Iterator<Item = &mut dyn Element> + '_> {
        Box::new(iter::empty())
    }

    // Lifecycle

    fn update(&mut self, state: &mut State) {
        for child in self.children_mut() {
            child.update(state);
        }
    }

    fn render(&self, r: &mut Renderer) {
        for child in self.children() {
            child.render(r);
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

    fn on_mousemove(&mut self, state: &State, cursor: Point) {
        for child in self.children_mut() {
            child.on_mousemove(state, cursor);
        }
    }
}
