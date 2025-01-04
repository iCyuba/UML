#![allow(dead_code)]

use crate::app::{context::EventContext, event_target::noop, EventTarget, Tree};
use taffy::{AvailableSpace, Layout, NodeId, Style};

pub trait Node: EventTarget {
    // Box model
    fn layout(&self) -> &Layout;
    fn layout_mut(&mut self) -> &mut Layout;

    fn scrollable(&self) -> bool {
        false
    }

    fn scroll_offset(&self) -> (f32, f32) {
        (0., 0.)
    }

    fn measure(
        &self,
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<AvailableSpace>,
        style: &Style,
        ctx: &mut EventContext,
    ) -> taffy::Size<f32> {
        noop!(known_dimensions, available_space, style, ctx);
    }
}

pub type CurriedSetup = dyn FnOnce(&mut Tree, &mut EventContext) -> NodeId;

pub trait Element {
    // Default setup
    fn setup(tree: &mut Tree, ctx: &mut EventContext) -> NodeId;

    fn create() -> Box<CurriedSetup> {
        Box::new(move |tree, ctx| Self::setup(tree, ctx))
    }
}

pub trait ElementWithProps {
    type Props: 'static;

    // Setup with props
    fn setup(tree: &mut Tree, ctx: &mut EventContext, props: Self::Props) -> NodeId;

    fn create(props: Self::Props) -> Box<CurriedSetup> {
        Box::new(move |tree, ctx| Self::setup(tree, ctx, props))
    }
}
