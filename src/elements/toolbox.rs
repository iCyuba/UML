use super::primitives::traits::Draw;
use crate::app::context::{EventContext, RenderContext};
use crate::app::{EventTarget, Tree};
use crate::elements::node::{Element, ElementWithProps};
use crate::elements::primitives::fancy_box::{BorderOptions, FancyBox, ShadowOptions};
use crate::elements::toolbox_item::{Tool, ToolboxItem};
use crate::elements::Node;
use taffy::prelude::length;
use taffy::Display::Flex;
use taffy::FlexDirection::Column;
use taffy::{Layout, NodeId, Style};
use winit::keyboard::{KeyCode, PhysicalKey};

pub struct Toolbox(Layout);

impl EventTarget for Toolbox {
    fn render(&self, RenderContext { c, .. }: &mut RenderContext) {
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

    fn on_keydown(&mut self, ctx: &mut EventContext, event: winit::event::KeyEvent) -> bool {
        let pk = event.physical_key;
        let char = event.text.and_then(|t| t.chars().next());

        ctx.state.set_tool(
            if matches!(pk, PhysicalKey::Code(KeyCode::Digit1))
                || matches!(char, Some('v') | Some('V'))
            {
                Tool::Select
            } else if matches!(pk, PhysicalKey::Code(KeyCode::Digit2))
                || matches!(char, Some('h') | Some('H'))
            {
                Tool::Hand
            } else if matches!(pk, PhysicalKey::Code(KeyCode::Digit3))
                || matches!(char, Some('e') | Some('E'))
            {
                Tool::Entity
            } else if matches!(pk, PhysicalKey::Code(KeyCode::Digit4))
                || matches!(char, Some('r') | Some('R'))
            {
                Tool::Relation
            } else if matches!(pk, PhysicalKey::Code(KeyCode::Digit5))
                // G for Generalization
                || matches!(char, Some('g') | Some('G'))
            {
                Tool::Parent
            } else if matches!(pk, PhysicalKey::Code(KeyCode::Digit6))
                || matches!(char, Some('i') | Some('I'))
            {
                Tool::Implementation
            } else if matches!(pk, PhysicalKey::Code(KeyCode::Digit7))
                || matches!(char, Some('p') | Some('P'))
            {
                Tool::Pen
            } else {
                return false;
            },
        );

        ctx.state.request_redraw();
        ctx.state.request_cursor_update();

        true
    }
}

impl Node for Toolbox {
    fn layout(&self) -> &Layout {
        &self.0
    }

    fn layout_mut(&mut self) -> &mut Layout {
        &mut self.0
    }
}

impl Element for Toolbox {
    fn setup(tree: &mut Tree, ctx: &mut EventContext) -> NodeId {
        tree.add_element(
            ctx,
            Style {
                display: Flex,
                flex_direction: Column,
                border: length(1.),
                margin: length(12.),
                padding: length(8.),
                gap: length(8.),
                align_self: Some(taffy::AlignSelf::Center),
                ..Default::default()
            },
            Some(vec![
                ToolboxItem::create(Tool::Select),
                ToolboxItem::create(Tool::Hand),
                ToolboxItem::create(Tool::Entity),
                ToolboxItem::create(Tool::Relation),
                ToolboxItem::create(Tool::Parent),
                ToolboxItem::create(Tool::Implementation),
                ToolboxItem::create(Tool::Pen),
            ]),
            |node_id, ctx| {
                ctx.state.key_listeners.insert(node_id);

                Self(Layout::new())
            },
        )
    }
}
