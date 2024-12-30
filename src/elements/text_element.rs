use super::{
    node::ElementWithProps,
    primitives::{text::Text, traits::Draw},
    Node,
};
use crate::{
    app::{
        context::{EventContext, GetterContext, RenderContext},
        EventTarget, Tree,
    },
    presentation::FontResource,
};
use taffy::{
    prelude::{auto, length},
    AvailableSpace, Layout, NodeId, Size, Style,
};

pub type TextGetter = Box<dyn Fn(&GetterContext) -> String>;

pub struct TextElementProps {
    pub text: TextGetter,
    pub size: f64,
    pub font: &'static FontResource<'static>,
}

pub struct TextElement {
    pub props: TextElementProps,
    pub layout: Layout,
}

impl EventTarget for TextElement {
    fn render(&self, ctx: &mut RenderContext) {
        let text = self.props.text.as_ref()(&GetterContext {
            state: &ctx.state,
            project: &ctx.project,
            c: &ctx.c,
        });

        Text::new(
            &text,
            self.layout,
            self.props.size,
            self.props.font,
            ctx.c.colors().workspace_text,
        )
        .draw(ctx.c);
    }
}

impl Node for TextElement {
    fn layout(&self) -> &Layout {
        &self.layout
    }

    fn layout_mut(&mut self) -> &mut Layout {
        &mut self.layout
    }

    fn measure(
        &self,
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
        _: &Style,
        ctx: &mut EventContext,
    ) -> taffy::Size<f32> {
        let text = self.props.text.as_ref()(&GetterContext {
            state: ctx.state,
            project: ctx.project,
            c: ctx.c,
        });

        let size = Text::measure(&text, self.props.size, self.props.font);

        let mut width = size.x as f32;
        if let AvailableSpace::Definite(max) = available_space.width {
            width = width.min(max);
        };

        Size {
            width,
            height: known_dimensions
                .height
                .unwrap_or(self.props.size as f32 * 1.2),
        }
    }
}

impl ElementWithProps for TextElement {
    type Props = TextElementProps;

    fn setup(tree: &mut Tree, ctx: &mut EventContext, props: TextElementProps) -> NodeId {
        tree.add_element(
            ctx,
            Style {
                size: Size {
                    width: auto(),
                    height: length((props.size * 1.2) as f32),
                },
                ..<_>::default()
            },
            None,
            |_, _| TextElement {
                props,
                layout: Default::default(),
            },
        )
    }
}
