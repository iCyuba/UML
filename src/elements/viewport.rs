use super::{workspace::Workspace, Element};
use crate::app::State;
use std::iter;
use taffy::{NodeId, Size, Style, TaffyTree};

const LAYOUT: Style = {
    let mut style = Style::DEFAULT;
    style.size = Size::from_percent(1., 1.);

    style
};

pub struct Viewport {
    pub node_id: NodeId,

    workspace: Workspace,
}

impl Viewport {
    pub fn new(flex_tree: &mut TaffyTree) -> Self {
        Self {
            node_id: flex_tree.new_leaf(LAYOUT).unwrap(),

            workspace: Workspace::new(),
        }
    }
}

impl Element for Viewport {
    fn children(&self) -> Box<dyn Iterator<Item = &dyn Element> + '_> {
        Box::new(iter::once(&self.workspace as &dyn Element))
    }

    fn children_mut(&mut self) -> Box<dyn Iterator<Item = &mut dyn Element> + '_> {
        Box::new(iter::once(&mut self.workspace as &mut dyn Element))
    }

    fn update(&mut self, state: &mut State) {
        state
            .flex_tree
            .compute_layout(self.node_id, state.size.into())
            .expect("Couldn't compute layout");

        self.workspace.update(state);
    }
}
