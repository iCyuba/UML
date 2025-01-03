use taffy::{
    prelude::{auto, length, percent},
    FlexDirection, Layout, NodeId, Size, Style, TraversePartialTree,
};

use crate::{
    app::{context::EventContext, EventTarget, Tree},
    elements::{
        node::{Element, ElementWithProps},
        primitives::icon::Symbol,
        Node,
    },
};

use super::{
    category::{Category, CategoryProps},
    sidebar_entity,
};

mod item;

pub struct ConnectionsList {
    layout: Layout,
    container: NodeId,
}

impl EventTarget for ConnectionsList {
    fn update(&mut self, ctx: &mut EventContext) {
        // Make sure the child count matches the connection count
        let node_id = self.container;
        ctx.state.modify_tree(move |tree, ctx| {
            let count = tree.child_count(node_id);

            if let Some(selected) = sidebar_entity!(ctx => get) {
                let target = selected.connections.len();

                #[allow(clippy::comparison_chain)]
                if count < target {
                    for i in count..target {
                        let new = item::ConnectionsListItem::setup(tree, ctx, i);

                        tree.add_child(node_id, new).unwrap();
                    }
                } else if count > target {
                    for _ in target..count {
                        tree.remove_child_at_index(node_id, target).unwrap();
                    }
                } else {
                    return;
                }

                ctx.state.request_redraw();
            }
        });
    }
}

impl Node for ConnectionsList {
    fn layout(&self) -> &Layout {
        &self.layout
    }

    fn layout_mut(&mut self) -> &mut Layout {
        &mut self.layout
    }
}

impl Element for ConnectionsList {
    fn setup(tree: &mut Tree, ctx: &mut EventContext) -> NodeId {
        let style = Style {
            max_size: Size {
                width: percent(1.),
                height: auto(),
            },
            flex_direction: FlexDirection::Column,
            gap: length(8.),
            ..<_>::default()
        };

        let container = tree.new_leaf(style.clone()).unwrap();

        tree.add_element(
            ctx,
            style,
            Some(vec![
                // Category title
                Category::create(CategoryProps {
                    name: "Relations".to_string(),
                    icon: Symbol::Workflow,
                }),
                // Items
                Box::new(move |_, _| container),
            ]),
            |_, _| Self {
                layout: <_>::default(),
                container,
            },
        )
    }
}
