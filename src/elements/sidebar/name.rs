use super::sidebar_entity;
use crate::{
    elements::{
        node::{CurriedSetup, ElementWithProps},
        text_input::{TextInput, TextInputProps},
    },
    presentation::fonts,
};

pub fn sidebar_name() -> Box<CurriedSetup> {
    TextInput::create(TextInputProps {
        getter: Box::new(|ctx| {
            if let Some(entity) = sidebar_entity!(ctx => get) {
                entity.name.clone()
            } else {
                "".to_string()
            }
        }),
        setter: Box::new(|ctx, str| {
            if let Some(entity) = sidebar_entity!(ctx => get_mut) {
                entity.name = str.to_string();
            }
        }),
        placeholder: Some("Untitled".to_string()),
        size: 24.,
        font: fonts::jbmono_bold(),
    })
}
