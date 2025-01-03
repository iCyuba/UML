use super::sidebar_entity;
use crate::{
    app::{context::EventContext, Tree},
    data::entity::EntityType,
    elements::{
        button::{Button, ButtonProps, ButtonStyle},
        node::{Element, ElementWithProps},
        primitives::icon::Symbol,
        segmented_control::{SegmentedControl, SegmentedControlProps},
        text_element::{TextElement, TextElementProps},
    },
    presentation::fonts,
};
use taffy::{
    prelude::{auto, length, percent},
    AlignItems, JustifyContent, NodeId, Size, Style,
};

pub struct SidebarType;

impl Element for SidebarType {
    fn setup(tree: &mut Tree, ctx: &mut EventContext) -> NodeId {
        let type_name = TextElement::create(TextElementProps {
            getter: Box::new(|ctx| {
                if let Some(entity) = sidebar_entity!(ctx => get) {
                    format!("{}", entity.entity_type)
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
                sidebar_entity!(ctx => get)
                    .map(|e| e.entity_type as usize)
                    .unwrap_or(0)
            }),

            setter: Box::new(|ctx, index| {
                let Ok(ty) = EntityType::try_from(index) else {
                    return;
                };

                if let Some(entity) = sidebar_entity!(ctx => get_mut) {
                    entity.entity_type = ty;
                }
            }),
        });

        // This isn't related to the type, but it's on the same line
        let delete_button = Button::create(ButtonProps {
            tooltip: "Delete entity",
            icon: Symbol::Trash,
            on_click: Box::new(|ctx| {
                if let Some(entity) = ctx.state.sidebar.entity {
                    ctx.project.remove_entity(entity);
                    ctx.state.selected_entity = None;
                    ctx.state.request_redraw();
                }
            }),
            style: ButtonStyle::Segmented,
        });

        let type_name = type_name(tree, ctx);
        let segmented_control = segmented_control(tree, ctx);
        let delete_button = delete_button(tree, ctx);

        let right = tree
            .new_with_children(
                Style {
                    gap: length(4.),
                    ..<_>::default()
                },
                &[segmented_control, delete_button],
            )
            .unwrap();

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
            &[type_name, right],
        )
        .unwrap()
    }
}
