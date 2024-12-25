use super::{EventTarget, Renderer, State};
use crate::{
    elements::{viewport::Viewport, Element},
    geometry::{rect::Rect, Point},
};
use std::ops::{Deref, DerefMut};
use taffy::{AvailableSpace, NodeId, Size, TaffyTree};
use winit::dpi::PhysicalSize;

pub struct Tree {
    taffy_tree: TaffyTree<Box<dyn Element>>,
    root: NodeId,

    map: Vec<(Rect, NodeId)>,
    scale: f32,
}

impl Tree {
    pub fn new() -> Self {
        let mut taffy_tree = TaffyTree::new();
        taffy_tree.disable_rounding();

        let root = taffy_tree.new_leaf(Default::default()).unwrap();

        let mut this = Self {
            taffy_tree,
            root,
            map: Vec::new(),
            scale: 1.,
        };

        Viewport::setup(&mut this, root);

        this
    }

    pub fn node_at_point(&self, point: Point) -> Option<NodeId> {
        self.map
            .iter()
            .rev()
            .find(|(rect, _)| rect.contains(point))
            .map(|(_, node)| *node)
    }

    pub fn compute_layout(&mut self, size: PhysicalSize<u32>, scale: f32) {
        // Store the scale for later use (when updating the layout)
        self.scale = scale;

        // Apply the scale
        let size = size.to_logical(scale as f64);
        let size = Size {
            width: AvailableSpace::Definite(size.width),
            height: AvailableSpace::Definite(size.height),
        };

        self.taffy_tree.compute_layout(self.root, size).unwrap();
    }
}

impl EventTarget for Tree {
    fn update(&mut self, state: &mut State) {
        fn update_children(
            node: NodeId,
            tree: &mut Tree,
            state: &mut State,
            location: taffy::Point<f32>,
        ) {
            // Scale up the layout and update the layout location to be relative to the root
            let mut layout = *tree.layout(node).unwrap();

            {
                macro_rules! scale {
                    ($e:expr) => {
                        $e *= tree.scale;
                    };
                }

                macro_rules! scale_rect {
                    ($e:expr) => {
                        scale!($e.left);
                        scale!($e.right);
                        scale!($e.top);
                        scale!($e.bottom);
                    };
                }

                scale!(layout.location.x);
                scale!(layout.location.y);
                scale!(layout.size.width);
                scale!(layout.size.height);
                scale!(layout.scrollbar_size.width);
                scale!(layout.scrollbar_size.height);
                scale_rect!(layout.border);
                scale_rect!(layout.padding);
                scale_rect!(layout.margin);
            }

            layout.location = layout.location + location;

            // Call the element's update method
            if let Some(element) = tree.get_node_context_mut(node) {
                *element.layout_mut() = layout;

                element.update(state);

                // Store the rect for hover detection
                tree.map.push((Rect::from(layout), node));
            }

            for node in tree.children(node).unwrap() {
                update_children(node, tree, state, layout.location);
            }
        }

        self.map.clear();
        update_children(self.root, self, state, Default::default());
    }

    fn render(&self, r: &mut Renderer, state: &State) {
        fn render_children(node: NodeId, tree: &Tree, r: &mut Renderer, state: &State) {
            if let Some(element) = tree.get_node_context(node) {
                element.render(r, state);
            }

            for node in tree.children(node).unwrap() {
                render_children(node, tree, r, state);
            }
        }

        render_children(self.root, self, r, state);
    }

    fn on_scroll(
        &mut self,
        state: &mut State,
        delta: crate::geometry::Vec2,
        mouse: bool,
        zoom: bool,
        reverse: bool,
    ) {
        // Only scroll the element under the cursor
        let node = self.node_at_point(state.cursor);
        let element = node.and_then(|node| self.get_node_context_mut(node));

        if let Some(element) = element {
            element.on_scroll(state, delta, mouse, zoom, reverse);
        }
    }

    fn on_mousemove(&mut self, state: &mut State, cursor: Point) {
        // Update the active element or the element under the cursor
        let node = state.focused.or_else(|| self.node_at_point(cursor));

        // Set the hovered element
        state.hovered = node;

        // Call the element's on_mousemove method, if it exists
        if let Some(element) = node.and_then(|node| self.get_node_context_mut(node)) {
            element.on_mousemove(state, cursor);
        }
    }
}

impl Deref for Tree {
    type Target = TaffyTree<Box<dyn Element>>;

    fn deref(&self) -> &Self::Target {
        &self.taffy_tree
    }
}

impl DerefMut for Tree {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.taffy_tree
    }
}
