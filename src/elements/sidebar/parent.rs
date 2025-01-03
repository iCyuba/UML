use super::{
    category::{Category, CategoryProps},
    sidebar_entity,
};
use crate::{
    app::{context::EventContext, EventTarget, Tree},
    elements::{
        button::{Button, ButtonProps, ButtonStyle},
        node::{Element, ElementWithProps},
        primitives::icon::Symbol,
        text_element::{TextElement, TextElementProps},
        Node,
    },
    presentation::fonts,
};
use taffy::{
    prelude::{auto, length, percent},
    Display, FlexDirection, Layout, NodeId, Size, Style,
};

pub struct SidebarParent {
    layout: Layout,
    content: NodeId,
}

impl EventTarget for SidebarParent {
    fn update(&mut self, ctx: &mut EventContext) {
        let content = self.content;

        ctx.state.modify_tree(move |tree, ctx| {
            let style = tree.style(content).unwrap();
            let mut display = style.display;

            if sidebar_entity!(ctx => get).is_some_and(|e| e.parent.is_some()) {
                if display == Display::Flex {
                    return;
                }

                display = Display::Flex;
            } else {
                if display == Display::None {
                    return;
                }

                display = Display::None;
            }

            let style = Style {
                display,
                gap: length(4.),
                ..<_>::default()
            };

            tree.set_style(content, style).unwrap();
        });
    }
}

impl Node for SidebarParent {
    fn layout(&self) -> &Layout {
        &self.layout
    }

    fn layout_mut(&mut self) -> &mut Layout {
        &mut self.layout
    }
}

impl Element for SidebarParent {
    fn setup(tree: &mut Tree, ctx: &mut EventContext) -> NodeId {
        let category = Category::create(CategoryProps {
            name: "Parent".to_string(),
            icon: Symbol::Generalization,
        });

        let name = TextElement::create(TextElementProps {
            getter: Box::new(|ctx| {
                if let Some(entity) = sidebar_entity!(ctx => get)
                    .and_then(|e| e.parent)
                    .map(|id| ctx.project.connections[id].to.entity)
                    .and_then(|id| ctx.project.entities.get(id))
                {
                    entity.name.clone()
                } else {
                    "".to_string()
                }
            }),
            size: 16.,
            font: fonts::jbmono_semi_bold(),
        });

        let remove_button = Button::create(ButtonProps {
            tooltip: "Remove parent",
            icon: Symbol::Trash,
            on_click: Box::new(|ctx| {
                if let Some(entity) = ctx.state.sidebar.entity {
                    ctx.project.set_parent(entity, None);
                    ctx.state.request_redraw();
                }
            }),
            style: ButtonStyle::Segmented,
        });

        let name = name(tree, ctx);
        let delete_button = remove_button(tree, ctx);

        let content = tree
            .new_with_children(
                Style {
                    gap: length(4.),
                    ..<_>::default()
                },
                &[name, delete_button],
            )
            .unwrap();

        tree.add_element(
            ctx,
            Style {
                size: Size {
                    width: percent(1.),
                    height: auto(),
                },
                gap: length(4.),
                flex_direction: FlexDirection::Column,
                ..<_>::default()
            },
            Some(vec![category, Box::new(move |_, _| content)]),
            |_, _| Self {
                layout: <_>::default(),
                content,
            },
        )
    }
}
