use super::{State, Tree};
use crate::elements::tooltip::Tooltip;
use crate::elements::{toolbox::Toolbox, workspace::Workspace};
use taffy::{NodeId, Style};
use taffy::AlignContent::SpaceBetween;
use taffy::Position::Relative;
use crate::elements::sidebar::Sidebar;

pub struct Viewport;

/// The viewport isn't an element. It's just a container for the workspace and ui.
impl Viewport {
    const STYLE: Style = {
        let mut style = Style::DEFAULT;
        style.size = taffy::Size::from_percent(1., 1.);
        style.justify_content = Some(SpaceBetween);
        style.position = Relative;

        style
    };

    // Unlike elements, the viewport overrides an existing node (the root node) instead of creating a new one
    pub fn setup(tree: &mut Tree, state: &mut State, node: NodeId) -> NodeId {
        let workspace = Workspace::setup(tree, state);
        let toolbox = Toolbox::setup(tree, state);
        let tooltip = Tooltip::setup(tree, state);
        let sidebar = Sidebar::setup(tree, state);

        tree.set_style(node, Self::STYLE).unwrap();
        tree.set_children(node, &[workspace, toolbox, tooltip])
            .unwrap();
        tree.set_children(node, &[workspace, toolbox, sidebar]).unwrap();

        node
    }
}
