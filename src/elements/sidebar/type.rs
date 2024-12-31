use super::sidebar_entity;
use crate::{
    app::{context::EventContext, Tree},
    data::entity::EntityType,
    elements::{
        node::{Element, ElementWithProps},
        primitives::icon::Symbol,
        segmented_control::{SegmentedControl, SegmentedControlProps},
        text_element::{TextElement, TextElementProps},
    },
    presentation::fonts,
};
use taffy::{
    prelude::{auto, percent},
    AlignItems, JustifyContent, NodeId, Size, Style,
};

pub struct SidebarType;

impl Element for SidebarType {
    fn setup(tree: &mut Tree, ctx: &mut EventContext) -> NodeId {
        let type_name = TextElement::create(TextElementProps {
            text: Box::new(|ctx| {
                if let Some(entity) = ctx
                    .state
                    .sidebar
                    .entity
                    .and_then(|e| ctx.project.entities.get(e))
                {
                    format!("{}", entity.ty)
                } else {
                    "".to_string()
                }
            }),
            size: 16.,
            font: fonts::jbmono_regular(),
        });

        let segmented_control = SegmentedControl::create(SegmentedControlProps {
            items: vec![
                (Symbol::Class, "Class"),
                (Symbol::AbstractClass, "Abstract class"),
                (Symbol::SealedClass, "Sealed class"),
                (Symbol::Interface, "Interface"),
            ],

            getter: Box::new(|ctx| {
                if let Some(entity) = sidebar_entity!(ctx => get) {
                    match entity.ty {
                        EntityType::Class => 0,
                        EntityType::AbstractClass => 1,
                        EntityType::SealedClass => 2,
                        EntityType::Interface => 3,
                    }
                } else {
                    0
                }
            }),

            setter: Box::new(|ctx, index| {
                if let Some(entity) = sidebar_entity!(ctx => get_mut) {
                    entity.ty = match index {
                        0 => EntityType::Class,
                        1 => EntityType::AbstractClass,
                        2 => EntityType::SealedClass,
                        3 => EntityType::Interface,
                        _ => return,
                    };
                }
            }),
        });

        let type_name = type_name(tree, ctx);
        let segmented_control = segmented_control(tree, ctx);

        tree.new_with_children(
            Style {
                size: Size {
                    width: percent(1.),
                    height: auto(),
                },
                align_items: Some(AlignItems::Center),
                justify_content: Some(JustifyContent::SpaceBetween),
                ..<_>::default()
            },
            &[type_name, segmented_control],
        )
        .unwrap()
    }
}
