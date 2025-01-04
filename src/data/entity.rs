#![allow(dead_code)]

use super::project::{AsToken, ConnectionKey, EntityKey, Keyword, Token};
use crate::elements::workspace::entity::EntityItemData;
use crate::elements::workspace::Workspace;
use crate::geometry::Rect;
use indexmap::IndexSet;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::fmt::Formatter;
use std::vec;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default)]
pub enum AccessModifier {
    #[default]
    Public,
    Private,
    Protected,
}

impl AccessModifier {
    pub fn as_char(&self) -> char {
        match self {
            AccessModifier::Public => '+',
            AccessModifier::Protected => '#',
            AccessModifier::Private => '-',
        }
    }
}

impl AsToken for AccessModifier {
    fn as_token(&self) -> Vec<Token> {
        match self {
            AccessModifier::Public => vec![Token::Keyword(Keyword::Public)],
            AccessModifier::Protected => vec![Token::Keyword(Keyword::Protected)],
            AccessModifier::Private => vec![Token::Keyword(Keyword::Private)],
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Field {
    pub name: String,
    pub modifier: AccessModifier,
    pub r#type: String,
}

impl Display for Field {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}: {}",
            self.modifier.as_char(),
            self.name,
            self.r#type
        )
    }
}

impl AsToken for Field {
    fn as_token(&self) -> Vec<Token> {
        [
            self.modifier.as_token(),
            vec![
                Token::Identifier(self.r#type.clone()),
                Token::Space,
                Token::Identifier(self.name.clone()),
                Token::Accessors,
            ],
        ]
        .concat()
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Method {
    pub name: String,
    pub modifier: AccessModifier,
    pub return_type: String,
    pub arguments: Vec<String>,
}

impl AsToken for Method {
    fn as_token(&self) -> Vec<Token> {
        [
            self.modifier.as_token(),
            vec![
                Token::Identifier(self.return_type.clone()),
                Token::Space,
                Token::Identifier(if self.name.trim().is_empty() {
                    "void".to_string()
                } else {
                    self.name.clone()
                }),
            ],
            vec![Token::MethodArguments(
                self.arguments
                    .iter()
                    .enumerate()
                    .map(|(i, arg)| {
                        (
                            Token::Identifier(arg.clone()),
                            Token::Identifier(format!("arg{i}")),
                        )
                    })
                    .collect(),
            )],
            vec![Token::Block(vec![Token::NewLine, Token::Placeholder])],
        ]
        .concat()
    }
}

impl Display for Method {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let args = self.arguments.join(", ");
        write!(
            f,
            "{}{}({}): {}",
            self.modifier.as_char(),
            self.name,
            args,
            self.return_type
        )
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, Copy, PartialEq)]
pub enum EntityType {
    #[default]
    Class = 0,
    AbstractClass = 1,
    SealedClass = 2,
    Interface = 3,
}

impl AsToken for EntityType {
    fn as_token(&self) -> Vec<Token> {
        match self {
            EntityType::Class => vec![Token::Keyword(Keyword::Class)],
            EntityType::AbstractClass => vec![
                Token::Keyword(Keyword::Abstract),
                Token::Keyword(Keyword::Class),
            ],
            EntityType::SealedClass => vec![
                Token::Keyword(Keyword::Sealed),
                Token::Keyword(Keyword::Class),
            ],
            EntityType::Interface => vec![Token::Keyword(Keyword::Interface)],
        }
    }
}

impl TryFrom<usize> for EntityType {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(EntityType::Class),
            1 => Ok(EntityType::AbstractClass),
            2 => Ok(EntityType::SealedClass),
            3 => Ok(EntityType::Interface),
            _ => Err(()),
        }
    }
}

impl Display for EntityType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityType::Class => write!(f, "Class"),
            EntityType::AbstractClass => write!(f, "Abstract class"),
            EntityType::SealedClass => write!(f, "Sealed class"),
            EntityType::Interface => write!(f, "Interface"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Entity {
    pub key: EntityKey,

    pub name: String,
    pub entity_type: EntityType,

    pub parent: Option<ConnectionKey>, // Connection of type "Generalization"
    pub implements: Vec<ConnectionKey>, // Connections of type "Implementation"

    pub fields: Vec<Field>,
    pub methods: Vec<Method>,

    pub connections: IndexSet<ConnectionKey>,

    /// Position of the entity in the workspace.
    pub position: (i32, i32),

    /// Extra data used for rendering.
    #[serde(skip)]
    pub data: EntityItemData,
}

impl Entity {
    pub fn new(name: String, entity_type: EntityType, pos: (i32, i32)) -> Self {
        Entity {
            key: Default::default(),
            name,
            entity_type,
            parent: None,
            implements: vec![],
            fields: vec![],
            methods: vec![],
            connections: IndexSet::new(),
            position: pos,
            data: EntityItemData::new(pos),
        }
    }

    pub fn get_rect(&self) -> Rect {
        (*self.data.rect).translate(self.data.move_pos.unwrap_or_default()) / Workspace::GRID_SIZE
    }
}
