#![allow(dead_code)]

use crate::app::context::EventContext;
use crate::app::{EventTarget, Tree};
use taffy::{Layout, NodeId};

pub trait Node: EventTarget {
    // Box model
    fn layout(&self) -> &Layout;
    fn layout_mut(&mut self) -> &mut Layout;
}

pub type CurriedSetup = dyn FnOnce(&mut Tree, &mut EventContext) -> NodeId;

pub trait Element {
    // Default setup
    fn setup(tree: &mut Tree, ctx: &mut EventContext) -> NodeId;

    fn create() -> Box<CurriedSetup>
    {
        Box::new(move |tree, ctx| Self::setup(tree, ctx))
    }
}

pub trait ElementWithProps {
    type Props: PartialEq + 'static;

    // Setup with props
    fn setup(tree: &mut Tree, ctx: &mut EventContext, props: Self::Props) -> NodeId;

    fn create(props: Self::Props) -> Box<CurriedSetup>
    {
        Box::new(move |tree, ctx| Self::setup(tree, ctx, props))
    }
}
