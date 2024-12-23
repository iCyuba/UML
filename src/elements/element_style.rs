use crate::app::State;
use crate::geometry::Point;
use taffy::{Layout, NodeId, Style};

#[derive(Default)]
pub struct ElementStyle {
    layout: Layout,
    style: Style,
    pos: Point,
}

impl ElementStyle {
    pub fn update(&mut self, node_id: NodeId, state: &State, pos: &Point) {
        self.layout = *state.flex_tree.layout(node_id).unwrap();
        self.style = state.flex_tree.style(node_id).unwrap().clone();
        self.pos = *pos;
    }

    pub fn get_layout(&self) -> &Layout {
        &self.layout
    }

    pub fn get_style(&self) -> &Style {
        &self.style
    }

    pub fn get_pos(&self) -> Point {
        self.pos
    }
}
