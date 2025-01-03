use crate::{
    app::{context::EventContext, EventTarget, Tree},
    elements::{
        node::ElementWithProps,
        primitives::{
            icon::{Icon, Symbol},
            traits::Draw,
        },
        text_element::{TextElement, TextElementProps},
        Node,
    },
    geometry::Rect,
    presentation::fonts,
};
use taffy::{
    prelude::{length, zero},
    Layout, NodeId, Style,
};

pub struct CategoryProps {
    pub name: String,
    pub icon: Symbol,
}

pub struct Category {
    layout: Layout,

    icon: Symbol,
}

impl EventTarget for Category {
    fn render(&self, ctx: &mut crate::app::context::RenderContext) {
        let rect = Rect::from(self.layout).translate((0., 6.));

        Icon::new(self.icon, rect, 18., ctx.c.colors().text).draw(ctx.c);
    }
}

impl Node for Category {
    fn layout(&self) -> &Layout {
        &self.layout
    }

    fn layout_mut(&mut self) -> &mut Layout {
        &mut self.layout
    }
}

impl ElementWithProps for Category {
    type Props = CategoryProps;

    fn setup(tree: &mut Tree, ctx: &mut EventContext, props: CategoryProps) -> NodeId {
        tree.add_element(
            ctx,
            Style {
                padding: taffy::Rect {
                    top: length(4.),
                    bottom: length(4.),
                    left: length(24.), // Icon + gap
                    right: zero(),
                },
                gap: length(4.),
                ..<_>::default()
            },
            Some(vec![
                // Name
                TextElement::create(TextElementProps {
                    getter: Box::new(move |_| props.name.clone()),
                    size: 16.,
                    font: fonts::inter_regular(),
                }),
            ]),
            |_, _| Category {
                layout: Default::default(),

                icon: props.icon,
            },
        )
    }
}
