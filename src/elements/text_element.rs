use super::{
    primitives::{text::Text, traits::Draw},
    Element,
};
use crate::{
    app::{context::RenderContext, EventTarget, Tree},
    presentation::FontResource,
};
use taffy::{Layout, NodeId, Style};
use vello::peniko::BrushRef;

pub struct TextElement {
    pub text: String,
    pub size: f64,
    pub font: &'static FontResource<'static>,
    pub brush: BrushRef<'static>,
    pub layout: Layout,
}

impl TextElement {
    pub fn setup(
        tree: &mut Tree,
        text: String,
        size: f64,
        font: &'static FontResource<'static>,
        brush: impl Into<BrushRef<'static>>,
        style: Style,
    ) -> NodeId {
        let node = tree
            .new_leaf(Style {
                size: Text::measure(&text, size, font).size.into(),
                ..style
            })
            .unwrap();

        let this = Self {
            text,
            size,
            font,
            brush: brush.into(),
            layout: Default::default(),
        };

        tree.set_node_context(node, Some(Box::new(this))).unwrap();

        node
    }
}

impl EventTarget for TextElement {
    fn render(&self, RenderContext { c, .. }: &mut RenderContext) {
        Text::new(&self.text, self.layout, self.size, self.font, self.brush).draw(c);
    }
}

impl Element for TextElement {
    fn layout(&self) -> &Layout {
        &self.layout
    }

    fn layout_mut(&mut self) -> &mut Layout {
        &mut self.layout
    }
}
