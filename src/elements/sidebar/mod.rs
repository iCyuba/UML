use super::{
    node::ElementWithProps,
    primitives::{
        fancy_box::{BorderOptions, FancyBox, ShadowOptions},
        icon::Symbol,
        simple_box::SimpleBox,
        traits::Draw,
    },
    Node,
};
use crate::{
    animations::{
        animated_property::AnimatedProperty,
        delta_animation::DeltaAnimation,
        standard_animation::{Easing, StandardAnimation},
    },
    app::{
        context::{EventContext, RenderContext},
        EventTarget, Tree,
    },
    data::project::EntityKey,
    elements::{node::Element, toolbox_item::Tool},
    geometry::Rect,
};
use category::CategoryProps;
use connection::SidebarConnection;
use derive_macros::AnimatedElement;
use field::SidebarField;
use implementation::SidebarImplementation;
use list::List;
use methods::SidebarMethod;
use name::sidebar_name;
use parent::SidebarParent;
use r#type::SidebarType;
use std::time::Duration;
use taffy::{
    prelude::{auto, length, zero},
    Display::Flex,
    FlexDirection::Column,
    Layout, NodeId, Position, Size, Style,
};

mod category;
mod connection;
mod field;
mod implementation;
mod list;
mod methods;
mod name;
mod parent;
mod r#type;

#[derive(Default)]
pub struct SidebarState {
    // Used for animating the sidebar in and out
    // Will contain the old entity id when the sidebar is closing
    pub entity: Option<EntityKey>,
}

// Macro for getting the entity from the sidebar state, cuz it's long
macro_rules! sidebar_entity {
    ($ctx:expr => $get:ident) => {
        $ctx.state
            .sidebar
            .entity
            .and_then(|e| $ctx.project.entities.$get(e))
    };
}

pub(super) use sidebar_entity;

#[derive(AnimatedElement)]
pub struct Sidebar {
    layout: Layout,

    position: AnimatedProperty<StandardAnimation<f32>>,
    scroll: AnimatedProperty<DeltaAnimation<f32>>,
}

impl EventTarget for Sidebar {
    fn update(&mut self, ctx: &mut EventContext) {
        let animate = self.animate();
        let offset = *self.position;
        self.layout.location.x += offset * 372.; // Offscreen = (360 width + 12 margin)

        // Clamp the scroll offset
        let max = self.layout.content_size.height - self.layout.size.height;
        self.scroll
            .set(self.scroll.get_target().clamp(0., max.max(0.)));

        macro_rules! cached {
            () => {
                ctx.state.sidebar.entity
            };
        }

        macro_rules! real {
            () => {
                match ctx.state.tool {
                    Tool::Relation | Tool::Parent | Tool::Implementation | Tool::Pen => None,
                    _ => ctx.state.selected_entity,
                }
            };
        }

        if animate {
            ctx.state.request_redraw();

            // If the entity changes mid-animation, stop the animation
            if real!() != cached!() {
                self.position.set(1.);
            }

            return;
        }

        // If the animation is finished, remove the old entity and rerun update
        if cached!().is_some() && offset == 1. {
            cached!() = None;
            self.scroll.reset(0.);

            return self.update(ctx);
        }

        // If a new entity is selected, update the sidebar
        if real!().is_some() && real!() != cached!() {
            if cached!().is_some() {
                self.position.set(1.);
            } else {
                cached!() = real!();
                self.position.set(0.);
            }

            ctx.state.request_redraw();
        }

        // Hide the sidebar if no entity is selected
        if cached!().is_none() {
            self.layout.size = zero();
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

        // Scrollbar
        let rect = Rect::from(self.layout);

        let height = self.layout.size.height as f64;
        let co_height = self.layout.content_size.height as f64;

        if co_height <= height {
            return;
        }

        let ratio = height / co_height;
        let scroll = *self.scroll as f64;

        let scrollbar = Rect::new(
            (rect.end().x - 12., rect.origin.y + scroll * ratio + 16.),
            (4., height * ratio - 32.),
        );

        SimpleBox::new(scrollbar, 2., c.colors().border).draw(c);
    }

    fn on_wheel(
        &mut self,
        ctx: &mut EventContext,
        event: crate::app::event_target::WheelEvent,
    ) -> bool {
        let max = self.layout.content_size.height - self.layout.size.height;
        let target = (*self.scroll - event.delta.y as f32).clamp(0., max.max(0.));

        if event.mouse {
            self.scroll.set(target);
        } else {
            self.scroll.reset(target);
        }

        ctx.state.request_redraw();

        true
    }
}

impl Node for Sidebar {
    fn layout(&self) -> &Layout {
        &self.layout
    }

    fn layout_mut(&mut self) -> &mut Layout {
        &mut self.layout
    }

    fn scrollable(&self) -> bool {
        true
    }

    fn scroll_offset(&self) -> (f32, f32) {
        (0., *self.scroll)
    }
}

impl Element for Sidebar {
    fn setup(tree: &mut Tree, ctx: &mut EventContext) -> NodeId {
        tree.add_element(
            ctx,
            Style {
                display: Flex,
                position: Position::Absolute,
                inset: taffy::Rect {
                    top: length(0.),
                    right: length(0.),
                    bottom: length(0.),
                    left: auto(),
                },
                flex_direction: Column,
                border: length(1.),
                margin: length(12.),
                padding: length(16.),
                gap: length(8.),
                size: Size {
                    width: length(360.),
                    height: auto(),
                },
                ..Default::default()
            },
            Some(vec![
                // Type
                SidebarType::create(),
                // Name
                sidebar_name(),
                // Parent
                SidebarParent::create(),
                // Implementations
                List::<SidebarImplementation>::create(CategoryProps {
                    icon: Symbol::Interface,
                    name: "Implements".to_string(),
                    add: Box::new(|ctx| {
                        ctx.state.set_tool(Tool::Implementation);
                        ctx.state.request_redraw();
                    }),
                }),
                // Connections
                List::<SidebarConnection>::create(CategoryProps {
                    icon: Symbol::Workflow,
                    name: "Relations".to_string(),
                    add: Box::new(|ctx| {
                        ctx.state.set_tool(Tool::Relation);
                        ctx.state.request_redraw();
                    }),
                }),
                // Fields
                List::<SidebarField>::create(CategoryProps {
                    icon: Symbol::Field,
                    name: "Fields".to_string(),
                    add: Box::new(|ctx| {
                        if let Some(entity) = sidebar_entity!(ctx => get_mut) {
                            entity.fields.push(Default::default());
                            ctx.state.request_redraw();
                        }
                    }),
                }),
                // Methods
                List::<SidebarMethod>::create(CategoryProps {
                    icon: Symbol::Method,
                    name: "Methods".to_string(),
                    add: Box::new(|ctx| {
                        if let Some(entity) = sidebar_entity!(ctx => get_mut) {
                            entity.methods.push(Default::default());
                            ctx.state.request_redraw();
                        }
                    }),
                }),
            ]),
            |_, _| Self {
                layout: Default::default(),

                position: AnimatedProperty::new(StandardAnimation::initialized(
                    1., // 1 = offscreen, 0 = fully visible
                    Duration::from_millis(200),
                    Easing::EaseInOut,
                )),

                scroll: AnimatedProperty::new(DeltaAnimation::initialized(0., 30.)),
            },
        )
    }
}
