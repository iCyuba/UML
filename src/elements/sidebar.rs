use super::{
    primitives::{
        fancy_box::{BorderOptions, FancyBox, ShadowOptions},
        traits::Draw,
    },
    text_element::TextElement,
    Element,
};
use crate::{
    app::{
        context::{EventContext, RenderContext},
        EventTarget, Tree,
    },
    data::project::EntityKey,
    presentation::fonts,
};
use taffy::{
    prelude::{auto, length},
    Display::Flex,
    FlexDirection::Column,
    Layout, NodeId, Size, Style,
};

pub struct Sidebar {
    node_id: NodeId,
    layout: Layout,

    entity: Option<EntityKey>,
}

impl Sidebar {
    pub fn setup(tree: &mut Tree, _: &mut EventContext) -> NodeId {
        let style = Style {
            display: Flex,
            flex_direction: Column,
            border: length(1.),
            margin: length(12.),
            padding: length(16.),
            gap: length(8.),
            size: Size {
                width: length(300.),
                height: auto(),
            },
            ..Default::default()
        };

        let node = tree.new_leaf(style).unwrap();

        tree.set_node_context(
            node,
            Some(Box::new(Self {
                node_id: node,
                layout: Layout::new(),
                entity: None,
            })),
        )
        .unwrap();

        node
    }

    fn remove_children(&self, ctx: &mut EventContext) {
        let node_id = self.node_id;

        ctx.state.modify_tree(move |tree| {
            tree.children(node_id)
                .unwrap()
                .into_iter()
                .for_each(|child| {
                    tree.remove(child).unwrap();
                });
        });
    }

    fn update_content(&mut self, ctx: &mut EventContext, key: EntityKey) {
        let parent = self.node_id;
        let entity = &ctx.project.entities[key];
        let name = entity.name.clone();
        let color = ctx.c.colors().workspace_text;

        if self.entity.is_some() {
            self.remove_children(ctx);
        }

        ctx.state.modify_tree(move |tree| {
            let title = TextElement::setup(
                tree,
                name,
                24.,
                fonts::jbmono_regular(),
                color,
                Style::DEFAULT,
            );

            tree.add_child(parent, title).unwrap();
        });
    }
}

impl EventTarget for Sidebar {
    fn update(&mut self, ctx: &mut EventContext) {
        if self.entity == ctx.state.selected_entity {
            return;
        }

        if let Some(key) = ctx.state.selected_entity {
            self.update_content(ctx, key);
        } else {
            self.remove_children(ctx);
        }

        self.entity = ctx.state.selected_entity;
    }

    fn render(&self, RenderContext { c, state, .. }: &mut RenderContext) {
        if state.selected_entity.is_none() {
            return;
        }

        FancyBox::from_element(
            self,
            c.scale(),
            13.,
            c.colors().floating_background,
            Some(BorderOptions {
                color: c.colors().border,
            }),
            Some(ShadowOptions {
                color: c.colors().drop_shadow,
                offset: (0., 1.).into(),
                blur_radius: 5.,
            }),
        )
        .draw(c.scene());
    }
}

impl Element for Sidebar {
    fn layout(&self) -> &Layout {
        &self.layout
    }

    fn layout_mut(&mut self) -> &mut Layout {
        &mut self.layout
    }
}
