use super::{
    primitives::simple_box::SimpleBox, primitives::traits::Draw,
    toolbox_item_icon::ToolboxItemIcon, tooltip::TooltipPosition, tooltip::TooltipState, Node,
};
use crate::elements::node::ElementWithProps;
use crate::elements::toolbox_item_icon::ToolboxItemIconProps;
use crate::{
    animations::{
        animated_property::AnimatedProperty,
        standard_animation::{Easing::EaseInOut, StandardAnimation},
        traits::Interpolate,
    },
    app::{
        context::{EventContext, GetterContext, RenderContext},
        EventTarget, Tree,
    },
    geometry::Rect,
};
use derive_macros::AnimatedElement;
use std::{fmt::Display, time::Duration};
use taffy::{prelude::length, AlignContent, AlignItems, Layout, NodeId, Style};
use winit::window::CursorIcon;

#[derive(Eq, PartialEq, Hash, Debug, Default, Copy, Clone)]
pub enum Tool {
    #[default]
    Select,
    Hand,
    Entity,
    Relation,
    Pen
}

impl Display for Tool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tool::Select => write!(f, "Select"),
            Tool::Hand => write!(f, "Hand tool"),
            Tool::Entity => write!(f, "Entity"),
            Tool::Relation => write!(f, "Relation"),
            Tool::Pen => write!(f, "Pen tool"),
        }
    }
}

#[derive(AnimatedElement)]
pub struct ToolboxItem {
    layout: Layout,

    tool_type: Tool,
    background: AnimatedProperty<StandardAnimation<f64>>,
    hover_opacity: AnimatedProperty<StandardAnimation<f32>>,

    hovered: bool,
}

impl EventTarget for ToolboxItem {
    fn update(&mut self, ctx: &mut EventContext) {
        self.background.set(if ctx.state.tool == self.tool_type {
            1.
        } else {
            0.
        });

        if self.animate() {
            ctx.state.request_redraw();
        }
    }

    fn render(&self, RenderContext { c, .. }: &mut RenderContext) {
        let rect: Rect = self.layout.into();
        let hover = c.colors().hover.multiply_alpha(*self.hover_opacity);
        let bg = Interpolate::interpolate(
            &c.colors().floating_background,
            &c.colors().accent,
            *self.background,
        );

        SimpleBox::new(rect, 5., bg).draw(c);
        SimpleBox::new(rect, 5., hover).draw(c);
    }

    fn cursor(&self, _: &GetterContext) -> Option<CursorIcon> {
        if self.hovered {
            Some(CursorIcon::Pointer)
        } else {
            None
        }
    }

    fn tooltip(&self, _: &GetterContext) -> Option<TooltipState> {
        Some(TooltipState {
            text: self.tool_type.to_string(),
            anchor: self.layout.into(),
            position: TooltipPosition::Right,
        })
    }

    fn on_click(&mut self, ctx: &mut EventContext) -> bool {
        ctx.state.tool = self.tool_type;
        ctx.state.tooltip_state = None; // Hide the tooltip after clicking
        ctx.state.request_redraw();

        true
    }

    fn on_mouseenter(&mut self, ctx: &mut EventContext) -> bool {
        self.hovered = true;
        self.hover_opacity.set(0.1);
        ctx.state.request_tooltip_update(); // This requests a redraw automatically

        true
    }

    fn on_mouseleave(&mut self, ctx: &mut EventContext) -> bool {
        self.hovered = false;
        self.hover_opacity.set(0.);
        ctx.state.request_tooltip_update();

        true
    }
}

impl Node for ToolboxItem {
    fn layout(&self) -> &Layout {
        &self.layout
    }

    fn layout_mut(&mut self) -> &mut Layout {
        &mut self.layout
    }
}

impl ElementWithProps for ToolboxItem {
    type Props = Tool;

    fn setup(tree: &mut Tree, ctx: &mut EventContext, tool_type: Tool) -> NodeId {
        let duration = Duration::from_millis(50);

        tree.add_element(
            ctx,
            Style {
                size: length(32.),
                justify_content: Some(AlignContent::Center),
                align_items: Some(AlignItems::Center),
                ..Default::default()
            },
            Some(vec![ToolboxItemIcon::create(ToolboxItemIconProps {
                tool: tool_type,
                size: 20.,
            })]),
            |_, _| Self {
                layout: Default::default(),
                tool_type,
                hovered: false,
                background: AnimatedProperty::new(StandardAnimation::new(duration, EaseInOut)),
                hover_opacity: AnimatedProperty::new(StandardAnimation::initialized(
                    0., duration, EaseInOut,
                )),
            },
        )
    }
}
