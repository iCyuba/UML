use super::{
    node::ElementWithProps,
    primitives::{
        fancy_box::{BorderOptions, FancyBox, ShadowOptions},
        traits::Draw,
    },
    text_element::{TextElement, TextElementProps},
    Node,
};
use crate::{
    animations::{
        animated_property::AnimatedProperty,
        standard_animation::{Easing, StandardAnimation},
    },
    app::{
        context::{EventContext, RenderContext},
        EventTarget, Tree,
    },
    data::project::EntityKey,
    elements::node::Element,
    presentation::fonts,
};
use derive_macros::AnimatedElement;
use std::time::Duration;
use taffy::{
    prelude::{auto, length, zero},
    Display::Flex,
    FlexDirection::Column,
    Layout, NodeId, Size, Style,
};

#[derive(Default)]
pub struct SidebarState {
    // Used for animating the sidebar in and out
    // Will contain the old entity id when the sidebar is closing
    pub(self) entity: Option<EntityKey>,
}

#[derive(AnimatedElement)]
pub struct Sidebar {
    layout: Layout,

    position: AnimatedProperty<StandardAnimation<f32>>,
}

impl EventTarget for Sidebar {
    fn update(&mut self, ctx: &mut EventContext) {
        let animate = self.animate();
        let offset = *self.position;
        self.layout.location.x += offset * 312.; // Offscreen = (300 width + 12 margin)

        macro_rules! cached {
            () => {
                ctx.state.sidebar.entity
            };
        }

        macro_rules! real {
            () => {
                ctx.state.selected_entity
            };
        }

        if animate {
            ctx.state.request_redraw();

            // When the sidebar is animating in and the entity changes, we can allow it
            if *self.position.get_target() == 0. && real!().is_some() {
                cached!() = real!();
            }

            return;
        }

        // If the animation is finished, remove the old entity and rerun update
        if cached!().is_some() && offset == 1. {
            cached!() = None;

            return self.update(ctx);
        }

        // If a new entity is selected, update the sidebar
        if real!().is_some() && real!() != cached!() {
            cached!() = real!();

            self.position.set(0.);
            ctx.state.request_redraw();
        }

        // Hide the sidebar if no entity is selected
        if cached!().is_none() {
            self.layout.size = zero();
            dbg!(real!());
        }
        // Animate out the sidebar if the entity is deselected
        else if real!().is_none() {
            self.position.set(1.);
            ctx.state.request_redraw();
        }
    }

    fn render(&self, RenderContext { c, state, .. }: &mut RenderContext) {
        if state.sidebar.entity.is_none() {
            return;
        }

        FancyBox::from_node(
            self,
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
        .draw(c);
    }
}

impl Node for Sidebar {
    fn layout(&self) -> &Layout {
        &self.layout
    }

    fn layout_mut(&mut self) -> &mut Layout {
        &mut self.layout
    }
}

impl Element for Sidebar {
    fn setup(tree: &mut Tree, ctx: &mut EventContext) -> NodeId {
        tree.add_element(
            ctx,
            Style {
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
            },
            Some(vec![TextElement::create(TextElementProps {
                text: Box::new(|ctx| {
                    if let Some(entity) = ctx.state.sidebar.entity {
                        let entity = ctx.project.entities.get(entity).unwrap();
                        entity.name.clone()
                    } else {
                        "".to_string()
                    }
                }),
                size: 24.,
                font: fonts::jbmono_bold(),
            })]),
            |_, _| Self {
                layout: Default::default(),

                position: AnimatedProperty::new(StandardAnimation::initialized(
                    1., // 1 = offscreen, 0 = fully visible
                    Duration::from_millis(200),
                    Easing::EaseInOut,
                )),
            },
        )
    }
}
