use super::{EventTarget, Renderer, State};
use crate::{
    elements::{viewport::Viewport, Element},
    geometry::{rect::Rect, Point},
};
use std::{
    collections::HashSet,
    ops::{Deref, DerefMut},
};
use taffy::{AvailableSpace, NodeId, Size, TaffyTree};
use winit::{dpi::PhysicalSize, event::MouseButton, window::CursorIcon};

pub struct Tree {
    taffy_tree: TaffyTree<Box<dyn Element>>,
    root: NodeId,

    map: Vec<(Rect, NodeId)>,
    scale: f32,

    // User state, which shouldn't be accessible from the elements
    hovered_on_mouse_down: Option<NodeId>,
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
            hovered_on_mouse_down: None,
        };

        Viewport::setup(&mut this, root);

        this
    }

    pub fn node_at_point(&self, point: Point) -> Option<NodeId> {
        self.map
            .iter()
            .rfind(|(rect, _)| rect.contains(point))
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

    /// Get the lowest common ancestor of two nodes.
    pub fn lowest_common_ancestor(&self, a: NodeId, b: NodeId) -> Option<NodeId> {
        // Fast path for the same node
        if a == b {
            return Some(a);
        }

        // A
        let mut ancestors = HashSet::new();
        let mut a = Some(a);
        while let Some(node) = a {
            ancestors.insert(node);
            a = self.parent(node);
        }

        // B
        let mut b = Some(b);
        while let Some(node) = b {
            if ancestors.contains(&node) {
                return Some(node);
            }

            b = self.parent(node);
        }

        None
    }

    /// Bubble an event up the tree.
    ///
    /// The closure returns an **immutable** reference to the element.
    /// If it returns true, the event will stop bubbling.
    ///
    /// Returns true if the event was stopped. (i.e. handled)
    pub fn bubble(
        &self,
        node: Option<NodeId>,
        mut f: impl FnMut(&Box<dyn Element>) -> bool,
    ) -> bool {
        let Some(node) = node else {
            return false;
        };

        if let Some(element) = self.get_node_context(node) {
            if f(element) {
                return true;
            }
        }

        self.bubble(self.parent(node), f)
    }

    /// Bubble an event up the tree.
    ///
    /// The closure returns a **mutable** reference to the element.
    /// If it returns true, the event will stop bubbling.
    ///
    /// Returns true if the event was stopped. (i.e. handled)
    fn bubble_mut(
        &mut self,
        node: Option<NodeId>,
        mut f: impl FnMut(&mut Box<dyn Element>) -> bool,
    ) -> bool {
        let Some(node) = node else {
            return false;
        };

        if let Some(element) = self.get_node_context_mut(node) {
            if f(element) {
                return true;
            }
        }

        self.bubble_mut(self.parent(node), f)
    }
}

impl EventTarget for Tree {
    // Lifecycle

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

    // Getters

    fn cursor(&self, state: &State) -> Option<CursorIcon> {
        let mut result = None;

        self.bubble(state.hovered, |el| {
            result = el.cursor(state);

            result.is_some()
        });

        result
    }

    // Events

    fn on_click(&mut self, state: &mut State) -> bool {
        let (Some(hovered), Some(md)) = (state.hovered, self.hovered_on_mouse_down) else {
            return false;
        };

        let node = self.lowest_common_ancestor(hovered, md);
        self.bubble_mut(node, |el| el.on_click(state))
    }

    fn on_mousedown(&mut self, state: &mut State, button: MouseButton) -> bool {
        // Store the last mouse down position
        if button == MouseButton::Left {
            self.hovered_on_mouse_down = state.hovered;
        }

        self.bubble_mut(state.hovered, |el| el.on_mousedown(state, button))
    }

    // `on_mouseenter` doesn't need to be handled here when the cursor enters the window
    // cuz a mouse move event will be fired immediately after that

    fn on_mouseleave(&mut self, state: &mut State) -> bool {
        self.bubble_mut(state.hovered, |el| {
            el.on_mouseleave(state);

            // These events should always bubble up
            false
        });

        // Clear the hovered element
        state.hovered = None;

        true
    }

    fn on_mousemove(&mut self, state: &mut State, cursor: Point) -> bool {
        // Update the active element or the element under the cursor
        let node = state.focused.or_else(|| self.node_at_point(cursor));

        // Fire the mouse enter and leave events
        if state.hovered != node {
            self.bubble_mut(state.hovered, |el| {
                el.on_mouseleave(state);

                false
            });
            self.bubble_mut(node, |el| {
                el.on_mouseenter(state);

                false
            });
        }

        // Set the hovered element
        state.hovered = node;

        self.bubble_mut(node, |el| el.on_mousemove(state, cursor))
    }

    fn on_mouseup(&mut self, state: &mut State, button: MouseButton) -> bool {
        let mut captured = self.bubble_mut(state.hovered, |el| el.on_mouseup(state, button));

        // Fire the on_click event
        if button == MouseButton::Left && self.hovered_on_mouse_down.is_some() {
            captured |= self.on_click(state);

            // Clear the last mouse down position
            self.hovered_on_mouse_down = None;
        }

        captured
    }

    fn on_wheel(
        &mut self,
        state: &mut State,
        delta: crate::geometry::Vec2,
        mouse: bool,
        zoom: bool,
        reverse: bool,
    ) -> bool {
        self.bubble_mut(state.hovered, |el| {
            el.on_wheel(state, delta, mouse, zoom, reverse)
        })
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
