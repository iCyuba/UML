use super::{toolbox::Toolbox, workspace::Workspace, Element};
use crate::app::{EventTarget, Renderer, State, Tree};
use taffy::{Layout, NodeId, Style};

pub struct Viewport(Layout);

impl Viewport {
    const STYLE: Style = {
        let mut style = Style::DEFAULT;
        style.size = taffy::Size::from_percent(1., 1.);

        style
    };

    // Unlike other elements, the viewport overrides an existing node (the root node) instead of creating a new one
    pub fn setup(tree: &mut Tree, node: NodeId) -> NodeId {
        let workspace = Workspace::setup(tree);
        let toolbox = Toolbox::setup(tree);

        tree.set_style(node, Self::STYLE).unwrap();
        tree.set_children(node, &[workspace, toolbox]).unwrap();
        tree.set_node_context(node, Some(Box::new(Self(Layout::new()))))
            .unwrap();

        node
    }
}

impl EventTarget for Viewport {
    // Nothing to render
    fn render(&self, _: &mut Renderer, _: &State) {}
}

impl Element for Viewport {
    fn layout(&self) -> &Layout {
        &self.0
    }

    fn layout_mut(&mut self) -> &mut Layout {
        &mut self.0
    }
}
