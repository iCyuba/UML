use crate::app::{EventTarget, Renderer, State, Tree};
use crate::data::project::EntityKey;
use crate::data::Project;
use crate::elements::primitives::fancy_box::{BorderOptions, FancyBox, ShadowOptions};
use crate::elements::primitives::traits::Draw;
use crate::elements::text_element::TextElement;
use crate::elements::Element;
use crate::presentation::fonts;
use taffy::prelude::{auto, length};
use taffy::Display::Flex;
use taffy::FlexDirection::Column;
use taffy::{Layout, NodeId, Size, Style};

pub struct Sidebar {
    node_id: NodeId,
    layout: Layout,

    entity: Option<EntityKey>,
}

impl Sidebar {
    pub fn setup(tree: &mut Tree, _: &mut State) -> NodeId {
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

    fn remove_children(&self, state: &mut State) {
        let node_id = self.node_id;

        state.modify_tree(move |tree| {
            tree.children(node_id)
                .unwrap()
                .into_iter()
                .for_each(|child| {
                    tree.remove(child).unwrap();
                });
        });
    }

    fn update_content(
        &mut self,
        r: &Renderer,
        state: &mut State,
        project: &mut Project,
        key: EntityKey,
    ) {
        let parent = self.node_id;
        let entity = &project.entities[key];
        let name = entity.name.clone();
        let color = r.colors.workspace_text;

        if self.entity.is_some() {
            self.remove_children(state);
        }

        state.modify_tree(move |tree| {
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
    fn update(&mut self, r: &Renderer, state: &mut State, project: &mut Project) {
        if self.entity == state.selected_entity {
            return;
        }

        if let Some(key) = state.selected_entity {
            self.update_content(r, state, project, key);
        } else {
            self.remove_children(state);
        }

        self.entity = state.selected_entity;
    }

    fn render(&self, r: &mut Renderer, state: &State, _: &Project) {
        if state.selected_entity.is_none() {
            return;
        }

        FancyBox::from_element(
            self,
            r.scale(),
            13.,
            r.colors.floating_background,
            Some(BorderOptions {
                color: r.colors.border,
            }),
            Some(ShadowOptions {
                color: r.colors.drop_shadow,
                offset: (0., 1.).into(),
                blur_radius: 5.,
            }),
        )
        .draw(&mut r.scene);
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
