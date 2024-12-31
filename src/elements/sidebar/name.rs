use super::sidebar_entity;
use crate::{
    elements::{
        node::{CurriedSetup, ElementWithProps},
        text_element::{TextElement, TextElementProps},
    },
    presentation::fonts,
};

pub fn sidebar_name() -> Box<CurriedSetup> {
    TextElement::create(TextElementProps {
        text: Box::new(|ctx| {
            if let Some(entity) = sidebar_entity!(ctx => get) {
                entity.name.clone()
            } else {
                "".to_string()
            }
        }),
        size: 24.,
        font: fonts::jbmono_bold(),
    })
}
