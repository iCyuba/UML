use super::category::{Category, CategoryProps};
use crate::{
    app::{context::EventContext, EventTarget, Tree},
    elements::{node::ElementWithProps, Node},
};
use taffy::{
    prelude::{auto, length, percent},
    FlexDirection, Layout, NodeId, Size, Style, TraversePartialTree,
};

pub trait Countable {
    fn count(ctx: &EventContext) -> usize;
}

pub struct List<T: ElementWithProps<Props = usize> + Countable> {
    layout: Layout,
    container: NodeId,

    _phantom: std::marker::PhantomData<T>,
}

impl<T: ElementWithProps<Props = usize> + Countable> EventTarget for List<T> {
    fn update(&mut self, ctx: &mut EventContext) {
        // Make sure the child count matches the connection count
        let node_id = self.container;

        ctx.state.modify_tree(move |tree, ctx| {
            let target = T::count(ctx);
            let count = tree.child_count(node_id);

            #[allow(clippy::comparison_chain)]
            if count < target {
                for i in count..target {
                    let new = T::setup(tree, ctx, i);

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
        });
    }
}

impl<T: ElementWithProps<Props = usize> + Countable> Node for List<T> {
    fn layout(&self) -> &Layout {
        &self.layout
    }

    fn layout_mut(&mut self) -> &mut Layout {
        &mut self.layout
    }
}

impl<T: ElementWithProps<Props = usize> + Countable + 'static> ElementWithProps for List<T> {
    type Props = CategoryProps;

    fn setup(tree: &mut Tree, ctx: &mut EventContext, label: CategoryProps) -> NodeId {
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
                // Label
                Category::create(label),
                // Items
                Box::new(move |_, _| container),
            ]),
            |_, _| Self {
                layout: <_>::default(),
                container,
                _phantom: std::marker::PhantomData,
            },
        )
    }
}
