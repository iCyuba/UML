use super::context::EventContext;
use super::Tree;
use crate::elements::node::Element;
use crate::elements::sidebar::Sidebar;
use crate::elements::tooltip::Tooltip;
use crate::elements::{toolbox::Toolbox, workspace::Workspace};
use taffy::AlignContent::SpaceBetween;
use taffy::Position::Relative;
use taffy::{NodeId, Style};

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
    pub fn setup(tree: &mut Tree, ctx: &mut EventContext, node: NodeId) -> NodeId {
        let workspace = Workspace::setup(tree, ctx);
        let toolbox = Toolbox::setup(tree, ctx);
        let tooltip = Tooltip::setup(tree, ctx);
        let sidebar = Sidebar::setup(tree, ctx);

        tree.set_style(node, Self::STYLE).unwrap();
        tree.set_children(node, &[workspace, toolbox, tooltip, sidebar])
            .unwrap();

        node
    }
}
