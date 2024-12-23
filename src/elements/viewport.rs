use super::{workspace::Workspace, Element};
use crate::app::State;
use crate::elements::toolbox::Toolbox;
use crate::geometry::Point;
use taffy::{Layout, NodeId, Style, TaffyTree};

const LAYOUT: Style = {
    let mut style = Style::DEFAULT;
    style.size = taffy::Size::from_percent(1., 1.);

    style
};

pub struct Viewport {
    node_id: NodeId,
    layout: Layout,

    workspace: Workspace,
    toolbox: Toolbox,
}

impl Viewport {
    pub fn new(flex_tree: &mut TaffyTree) -> Self {
        let toolbox = Toolbox::new(flex_tree);
        let workspace = Workspace::new(flex_tree);

        let node_id = flex_tree
            .new_with_children(LAYOUT, &[toolbox.node_id(), workspace.node_id()])
            .unwrap();

        Self {
            layout: Default::default(),
            node_id,

            workspace,
            toolbox,
        }
    }
}

impl Element for Viewport {
    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn get_layout(&self) -> &Layout {
        &self.layout
    }

    fn set_layout(&mut self, layout: Layout) {
        self.layout = layout;
    }

    fn children(&self) -> Box<dyn Iterator<Item = &dyn Element> + '_> {
        let vec: Vec<&dyn Element> = vec![&self.workspace, &self.toolbox];
        Box::new(vec.into_iter())
    }

    fn children_mut(&mut self) -> Box<dyn Iterator<Item = &mut dyn Element> + '_> {
        let vec: Vec<&mut dyn Element> = vec![&mut self.workspace, &mut self.toolbox];
        Box::new(vec.into_iter())
    }

    fn update(&mut self, state: &mut State, pos: Point) {
        state
            .flex_tree
            .compute_layout(self.node_id, state.size.into())
            .expect("Couldn't compute layout");

        self.workspace.update(state, pos);
        self.toolbox.update(state, pos);

        let new_pos = self.update_layout(state, pos);
        self.update_children(state, new_pos);
    }
}
