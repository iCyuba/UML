use super::{workspace::Workspace, Element};
use crate::app::State;
use crate::elements::toolbox::Toolbox;
use crate::geometry::{Point, Size};
use taffy::{NodeId, Style, TaffyTree};
use crate::elements::element_style::ElementStyle;

const LAYOUT: Style = {
    let mut style = Style::DEFAULT;
    style.size = taffy::Size::from_percent(1., 1.);

    style
};

pub struct Viewport {
    node_id: NodeId,
    element_style: ElementStyle,

    workspace: Workspace,
    toolbox: Toolbox,
}

impl Viewport {
    pub fn new(flex_tree: &mut TaffyTree, _: &Size) -> Self {
        let toolbox = Toolbox::new(flex_tree);
        let workspace = Workspace::new(flex_tree);

        let node_id = flex_tree
            .new_with_children(LAYOUT, &[toolbox.node_id(), workspace.node_id()])
            .unwrap();

        Self {
            element_style: Default::default(),
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

    fn get_style(&self) -> &ElementStyle {
        &self.element_style
    }

    fn get_mut_style(&mut self) -> &mut ElementStyle {
        &mut self.element_style
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

        let new_pos = self.update_element_style(state, pos);
        self.update_children(state, new_pos);
    }
}
