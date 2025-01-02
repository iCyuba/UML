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
            getter: |ctx| {
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
            },
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
                sidebar_entity!(ctx => get)
                    .map(|e| e.ty as usize)
                    .unwrap_or(0)
            }),

            setter: Box::new(|ctx, index| {
                let Ok(ty) = EntityType::try_from(index) else {
                    return;
                };

                if let Some(entity) = sidebar_entity!(ctx => get_mut) {
                    entity.ty = ty;
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
