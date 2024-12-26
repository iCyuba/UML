use super::primitives::traits::Draw;
use crate::animations::animated_property::AnimatedProperty;
use crate::animations::standard_animation::Easing::EaseInOut;
use crate::animations::standard_animation::StandardAnimation;
use crate::app::{EventTarget, Renderer, State, Tree};
use crate::elements::primitives::simple_box::SimpleBox;
use crate::elements::toolbox_item_icon::ToolboxItemIcon;
use crate::elements::Element;
use crate::geometry::rect::Rect;
use derive_macros::AnimatedElement;
use std::time::Duration;
use taffy::prelude::length;
use taffy::{AlignContent, AlignItems, Layout, NodeId, Style};
use vello::peniko::Color;
use winit::window::CursorIcon;

#[derive(Eq, PartialEq, Hash, Debug, Default, Copy, Clone)]
pub enum Tool {
    #[default]
    Select,
    Entity,
    Relation,
}

#[derive(AnimatedElement)]
pub struct ToolboxItem {
    layout: Layout,

    tool_type: Tool,
    background: AnimatedProperty<StandardAnimation<Color>>,
    hover_opacity: AnimatedProperty<StandardAnimation<f32>>,

    active: bool,
    hovered: bool,
    initialized: bool,
}

impl ToolboxItem {
    pub fn setup(tree: &mut Tree, tool_type: Tool) -> NodeId {
        let icon = ToolboxItemIcon::setup(tree, tool_type, 20.);
        let duration = Duration::from_millis(50);
        
        let this = Self {
            layout: Default::default(),
            tool_type,
            active: false,
            hovered: false,
            initialized: false,
            background: AnimatedProperty::new(StandardAnimation::new(
                Default::default(),
                duration,
                EaseInOut,
            )),
            hover_opacity: AnimatedProperty::new(StandardAnimation::new(
                0.,
                duration,
                EaseInOut,
            )),
        };

        let node = tree
            .new_with_children(
                Style {
                    size: length(32.),
                    justify_content: Some(AlignContent::Center),
                    align_items: Some(AlignItems::Center),
                    ..Default::default()
                },
                &[icon],
            )
            .unwrap();

        tree.set_node_context(node, Some(Box::new(this))).unwrap();

        node
    }
}

impl EventTarget for ToolboxItem {
    fn update(&mut self, r: &Renderer, state: &mut State) {
        self.active = state.tool == self.tool_type;
        let color = if self.active {
            r.colors.accent
        } else {
            r.colors.toolbox_background
        };

        if self.initialized {
            self.background.set(color);
        } else {
            self.background.reset(color);
            self.initialized = true;
        }

        if self.animate() {
            state.request_redraw();
        }
    }

    fn render(&self, r: &mut Renderer, _: &State) {
        let scale = r.scale();
        let rect: Rect = self.layout.into();
        let hover = r.colors.hover.multiply_alpha(*self.hover_opacity);

        SimpleBox::new(scale, rect, 5., *self.background).draw(&mut r.scene);
        SimpleBox::new(scale, rect, 5., hover).draw(&mut r.scene);
    }

    fn cursor(&self, _: &State) -> Option<CursorIcon> {
        if self.hovered {
            Some(CursorIcon::Pointer)
        } else {
            None
        }
    }

    fn on_click(&mut self, state: &mut State) -> bool {
        state.tool = self.tool_type;
        state.request_redraw();

        true
    }

    fn on_mouseenter(&mut self, state: &mut State) -> bool {
        self.hovered = true;
        self.hover_opacity.set(0.1);
        state.request_redraw();
        state.request_cursor_update();

        true
    }

    fn on_mouseleave(&mut self, state: &mut State) -> bool {
        self.hovered = false;
        self.hover_opacity.set(0.);
        state.request_redraw();
        state.request_cursor_update();

        true
    }
}

impl Element for ToolboxItem {
    fn layout(&self) -> &Layout {
        &self.layout
    }

    fn layout_mut(&mut self) -> &mut Layout {
        &mut self.layout
    }
}
