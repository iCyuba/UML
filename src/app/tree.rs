use super::{
    context::{EventContext, GetterContext},
    event_target::WheelEvent,
    viewport::Viewport,
    EventTarget,
};
use crate::{
    app::context::RenderContext,
    elements::{tooltip::TooltipState, Element},
    geometry::{Point, Rect},
};
use std::{
    collections::HashSet,
    ops::{Deref, DerefMut},
};
use taffy::{AvailableSpace, NodeId, Size, TaffyTree};
use winit::{
    event::{KeyEvent, MouseButton},
    window::CursorIcon,
};

pub struct Tree {
    taffy_tree: TaffyTree<Box<dyn Element>>,
    root: NodeId,

    map: Vec<(Rect, NodeId)>,

    // User state, which shouldn't be accessible from the elements
    hovered_on_mouse_down: Option<NodeId>,
}

impl Tree {
    pub fn new(ctx: &mut EventContext) -> Self {
        let mut taffy_tree = TaffyTree::new();
        taffy_tree.disable_rounding();

        let root = taffy_tree.new_leaf(Default::default()).unwrap();

        let mut this = Self {
            taffy_tree,
            root,
            map: Vec::new(),
            hovered_on_mouse_down: None,
        };

        Viewport::setup(&mut this, ctx, root);

        this
    }

    pub fn node_at_point(&self, point: Point) -> Option<NodeId> {
        self.map
            .iter()
            .rfind(|(rect, _)| rect.contains(point))
            .map(|(_, node)| *node)
    }

    pub fn compute_layout(&mut self, (width, height): (u32, u32), scale: f32) {
        // Apply the scale
        let size = Size {
            width: AvailableSpace::Definite(width as f32 / scale),
            height: AvailableSpace::Definite(height as f32 / scale),
        };

        self.taffy_tree.compute_layout(self.root, size).unwrap();
    }

    /// Get the lowest common ancestor of two nodes.
    pub fn lowest_common_ancestor(&self, a: NodeId, b: NodeId) -> Option<NodeId> {
        // Fast path for the same node
        if a == b {
            return Some(a);
        }

        // Another fast path if one of the nodes is the parent of the other
        if self.children(a).unwrap().contains(&b) {
            return Some(a);
        }

        if self.children(b).unwrap().contains(&a) {
            return Some(b);
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
        mut f: impl FnMut(NodeId, &Box<dyn Element>) -> bool,
    ) -> bool {
        let Some(node) = node else {
            return false;
        };

        if let Some(element) = self.get_node_context(node) {
            if f(node, element) {
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
        mut f: impl FnMut(NodeId, &mut Box<dyn Element>) -> bool,
    ) -> bool {
        let Some(node) = node else {
            return false;
        };

        if let Some(element) = self.get_node_context_mut(node) {
            if f(node, element) {
                return true;
            }
        }

        self.bubble_mut(self.parent(node), f)
    }
}

impl EventTarget for Tree {
    // Lifecycle

    fn update(&mut self, ctx: &mut EventContext) {
        fn update_children(
            node: NodeId,
            tree: &mut Tree,
            ctx: &mut EventContext,
            location: taffy::Point<f32>,
        ) {
            // Update the layout location to be relative to the root
            let mut layout = *tree.layout(node).unwrap();
            layout.location = layout.location + location;

            // Call the element's update method
            if let Some(element) = tree.get_node_context_mut(node) {
                *element.layout_mut() = layout;

                element.update(ctx);

                // Store the rect for hover detection
                tree.map.push((Rect::from(layout), node));
            }

            for node in tree.children(node).unwrap() {
                update_children(node, tree, ctx, layout.location);
            }
        }

        self.map.clear();

        // Update layout
        self.compute_layout(ctx.c.size(), ctx.c.scale() as f32);

        update_children(self.root, self, ctx, Default::default());
    }

    fn render(&self, ctx: &mut RenderContext) {
        fn render_children(node: NodeId, tree: &Tree, ctx: &mut RenderContext) {
            if let Some(element) = tree.get_node_context(node) {
                element.render(ctx);
            }

            for node in tree.children(node).unwrap() {
                render_children(node, tree, ctx);
            }
        }

        render_children(self.root, self, ctx);
    }

    // Getters

    fn cursor(&self, ctx: &GetterContext) -> Option<CursorIcon> {
        let mut result = None;

        self.bubble(ctx.state.hovered, |_, el| {
            result = el.cursor(ctx);

            result.is_some()
        });

        result
    }

    fn tooltip(&self, ctx: &GetterContext) -> Option<TooltipState> {
        let mut result = None;

        self.bubble(ctx.state.hovered, |_, el| {
            result = el.tooltip(ctx);

            result.is_some()
        });

        result
    }

    // Events

    fn on_click(&mut self, ctx: &mut EventContext) -> bool {
        let (Some(hovered), Some(md)) = (ctx.state.hovered, self.hovered_on_mouse_down) else {
            return false;
        };

        let node = self.lowest_common_ancestor(hovered, md);
        self.bubble_mut(node, |_, el| el.on_click(ctx))
    }

    fn on_keydown(&mut self, ctx: &mut EventContext, event: KeyEvent) -> bool {
        // If there's a focused element, fire the event there. If not, fire it on all key listeners.
        if let Some(focused) = ctx
            .state
            .focused
            .and_then(|node| self.get_node_context_mut(node))
        {
            focused.on_keydown(ctx, event)
        } else {
            let mut handled = false;

            let key_listeners = ctx.state.key_listeners.iter().cloned().collect::<Vec<_>>();
            for node in key_listeners {
                if let Some(element) = self.get_node_context_mut(node) {
                    handled |= element.on_keydown(ctx, event.clone());
                }
            }

            handled
        }
    }

    fn on_keyup(&mut self, ctx: &mut EventContext, event: KeyEvent) -> bool {
        if let Some(focused) = ctx
            .state
            .focused
            .and_then(|node| self.get_node_context_mut(node))
        {
            focused.on_keyup(ctx, event)
        } else {
            let mut handled = false;

            let key_listeners = ctx.state.key_listeners.iter().cloned().collect::<Vec<_>>();
            for node in key_listeners {
                if let Some(element) = self.get_node_context_mut(node) {
                    handled |= element.on_keyup(ctx, event.clone());
                }
            }

            handled
        }
    }

    fn on_mousedown(&mut self, ctx: &mut EventContext, button: MouseButton) -> bool {
        // Store the last mouse down position
        if button == MouseButton::Left {
            self.hovered_on_mouse_down = ctx.state.hovered;
        }

        self.bubble_mut(ctx.state.hovered, |_, el| el.on_mousedown(ctx, button))
    }

    // `on_mouseenter` doesn't need to be handled here when the cursor enters the window
    // cuz a mouse move event will be fired immediately after that

    fn on_mouseleave(&mut self, ctx: &mut EventContext) -> bool {
        self.bubble_mut(ctx.state.hovered, |_, el| {
            el.on_mouseleave(ctx);

            // This event should always bubble up
            false
        });

        // Clear the hovered element
        ctx.state.hovered = None;

        true
    }

    fn on_mousemove(&mut self, ctx: &mut EventContext, cursor: Point) -> bool {
        // Update the active element or the element under the cursor
        let node = ctx.state.focused.or_else(|| self.node_at_point(cursor));

        // Set the hovered element
        let old = ctx.state.hovered;
        ctx.state.hovered = node;

        // Find the lowest common ancestor of the old and new hovered elements
        // We don't want to refire on the LCA and its ancestors
        let lca = if let (Some(old), Some(node)) = (old, node) {
            self.lowest_common_ancestor(old, node)
        } else {
            None
        };

        // Fire the mouse enter and leave events
        if old != node {
            self.bubble_mut(old, |node, el| {
                if Some(node) == lca {
                    return true;
                }

                el.on_mouseleave(ctx);

                false
            });
            self.bubble_mut(node, |node, el| {
                if Some(node) == lca {
                    return true;
                }

                el.on_mouseenter(ctx);

                false
            });

            ctx.state.request_cursor_update();
        }

        self.bubble_mut(node, |_, el| el.on_mousemove(ctx, cursor))
    }

    fn on_mouseup(&mut self, ctx: &mut EventContext, button: MouseButton) -> bool {
        let mut captured = self.bubble_mut(ctx.state.hovered, |_, el| el.on_mouseup(ctx, button));

        // Fire the on_click event
        if button == MouseButton::Left && self.hovered_on_mouse_down.is_some() {
            captured |= self.on_click(ctx);

            // Clear the last mouse down position
            self.hovered_on_mouse_down = None;
        }

        captured
    }

    fn on_wheel(&mut self, ctx: &mut EventContext, event: WheelEvent) -> bool {
        self.bubble_mut(ctx.state.hovered, |_, el| el.on_wheel(ctx, event))
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
