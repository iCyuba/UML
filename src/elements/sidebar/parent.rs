use super::{
    category::{Category, CategoryProps},
    sidebar_entity,
};
use crate::{
    app::{context::EventContext, EventTarget, Tree},
    data::entity::EntityType,
    elements::{
        button::{Button, ButtonProps, ButtonStyle},
        node::{Element, ElementWithProps},
        primitives::icon::Symbol,
        text_element::{TextElement, TextElementProps},
        toolbox_item::Tool,
        Node,
    },
    presentation::fonts,
};
use taffy::{Display, FlexDirection, Layout, LengthPercentage, NodeId, Size, Style};

pub struct SidebarParent {
    layout: Layout,
    node_id: NodeId,
    content: NodeId,
}

impl SidebarParent {
    const CONTENT_STYLE: Style = Style {
        gap: Size {
            width: LengthPercentage::Length(4.),
            height: LengthPercentage::Length(4.),
        },
        ..Style::DEFAULT
    };

    const STYLE: Style = Style {
        flex_direction: FlexDirection::Column,
        ..Self::CONTENT_STYLE
    };
}

impl EventTarget for SidebarParent {
    fn update(&mut self, ctx: &mut EventContext) {
        let node_id = self.node_id;
        let content = self.content;

        // Completely hide if the entity is an interface
        // Hide the content if the entity has no parent
        ctx.state.modify_tree(move |tree, ctx| {
            let display;
            let content_display;

            if let Some(ent) = sidebar_entity!(ctx => get) {
                display = if ent.entity_type == EntityType::Interface {
                    Display::None
                } else {
                    Display::Flex
                };

                content_display = if ent.parent.is_some() {
                    Display::Flex
                } else {
                    Display::None
                };
            } else {
                display = Display::Flex;
                content_display = Display::None;
            }

            if tree.style(node_id).unwrap().display != display {
                let style = Style {
                    display,
                    ..Self::STYLE
                };

                tree.set_style(node_id, style).unwrap();
            }

            if tree.style(content).unwrap().display != content_display {
                let style = Style {
                    display: content_display,
                    ..Self::CONTENT_STYLE
                };

                tree.set_style(content, style).unwrap();
            }
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
            add: Box::new(|ctx| {
                ctx.state.set_tool(Tool::Parent);
                ctx.state.request_redraw();
            }),
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
            font: fonts::jbmono_medium(),
        });

        let remove_button = Button::create(ButtonProps {
            tooltip: "Remove parent",
            icon: Symbol::Trash,
            on_click: Box::new(|ctx| {
                if let Some(entity) = ctx.state.sidebar.entity {
                    ctx.project.set_parent(entity, None);
                    ctx.state.request_tooltip_update();
                }
            }),
            style: ButtonStyle::Segmented,
        });

        let name = name(tree, ctx);
        let delete_button = remove_button(tree, ctx);

        let content = tree
            .new_with_children(Self::CONTENT_STYLE, &[name, delete_button])
            .unwrap();

        tree.add_element(
            ctx,
            Self::STYLE,
            Some(vec![category, Box::new(move |_, _| content)]),
            |node_id, _| Self {
                layout: <_>::default(),
                node_id,
                content,
            },
        )
    }
}
