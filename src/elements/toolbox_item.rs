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

    selected: bool,
    initialized: bool,
}

impl ToolboxItem {
    pub fn setup(tree: &mut Tree, tool_type: Tool) -> NodeId {
        let icon = ToolboxItemIcon::setup(tree, tool_type, 20.);
        let duration = Duration::from_millis(100);
        
        let this = Self {
            layout: Default::default(),
            tool_type,
            selected: false,
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
        let selected = state.tool == self.tool_type;
        let color = if selected {
            r.colors.accent
        } else {
            r.colors.toolbox_background
        };

        if !self.initialized {
            self.background.reset(color);
        }
        else if self.selected != selected {
            self.background.set(color);
        }
        
        self.selected = selected;
        self.initialized = true;

        if self.animate() {
            state.request_redraw();
        }
    }

    fn render(&self, r: &mut Renderer, _: &State) {
        let rect: Rect = self.layout.into();
        let hover = Color::WHITE.multiply_alpha(*self.hover_opacity);

        SimpleBox::new(rect, 5., *self.background).draw(&mut r.scene);
        SimpleBox::new(rect, 5., hover).draw(&mut r.scene);
    }

    fn on_click(&mut self, state: &mut State) -> bool {
        state.tool = self.tool_type;
        state.request_redraw();

        true
    }

    fn on_mouseenter(&mut self, state: &mut State) -> bool {
        self.hover_opacity.set(0.1);
        state.request_redraw();

        true
    }

    fn on_mouseleave(&mut self, state: &mut State) -> bool {
        self.hover_opacity.set(0.);
        state.request_redraw();

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
