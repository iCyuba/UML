#![allow(dead_code)]

use super::project::{ConnectionKey, EntityKey};
use crate::elements::workspace::entity::EntityItemData;
use serde::{Deserialize, Serialize};
use slotmap::{new_key_type, SlotMap};
use std::collections::{HashMap, HashSet};

new_key_type! {
    pub struct InternalTypeKey;
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AccessModifier {
    Public,
    Protected,
    Private,
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

#[derive(Debug, Serialize, Deserialize)]
pub enum ImplementationModifier {
    Abstract,
    Virtual,
    Sealed,
    Override,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TypeArgument(pub String, pub Option<Type>);

#[derive(Debug, Serialize, Deserialize)]
pub struct Argument(pub String, pub Type);

#[derive(Debug, Serialize, Deserialize)]
pub enum Type {
    Custom(EntityKey),
    Internal(InternalTypeKey),
    External(String, Vec<Type>),
}

impl Type {
    pub fn uses_internal(&self, key: InternalTypeKey) -> bool {
        match self {
            Type::Internal(internal_key) => *internal_key == key,
            Type::External(_, types) => types.iter().any(|ty| ty.uses_internal(key)),
            _ => false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Attribute {
    Field(AccessModifier, Type),
    Method(
        AccessModifier,
        Option<ImplementationModifier>,
        Type,
        Vec<Argument>,
    ),
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub enum EntityType {
    #[default]
    Class,
    AbstractClass,
    SealedClass,
    Interface,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Entity {
    pub key: EntityKey,

    pub name: String,
    pub ty: EntityType,
    pub generics: SlotMap<InternalTypeKey, TypeArgument>,

    pub inherits: Option<Type>,
    pub implements: Vec<Type>,

    pub attributes: HashMap<String, Attribute>,

    pub connections: HashSet<ConnectionKey>,

    /// Position of the entity in the workspace.
    pub position: (i32, i32),

    /// Extra data used for rendering.
    #[serde(skip)]
    pub data: EntityItemData,
}

impl Entity {
    pub fn new(name: String, ty: EntityType, pos: (i32, i32)) -> Self {
        Entity {
            key: Default::default(),
            name,
            ty,
            generics: SlotMap::with_key(),
            inherits: None,
            implements: vec![],
            attributes: HashMap::new(),
            connections: HashSet::new(),
            position: pos,
            data: EntityItemData::new(pos),
        }
    }
}

pub enum TypeInUseError {
    Generic,
    Attribute(String),
    Interface,
    BaseType,
}

impl Entity {
    pub fn add_generic(&mut self, name: String) -> InternalTypeKey {
        self.generics.insert(TypeArgument(name, None))
    }

    pub fn get_generic(&self, key: InternalTypeKey) -> &TypeArgument {
        &self.generics[key]
    }

    pub fn remove_generic(
        &mut self,
        key: InternalTypeKey,
    ) -> Result<Option<TypeArgument>, TypeInUseError> {
        let uses_internal = |ty: &Type| ty.uses_internal(key);

        if self
            .generics
            .values()
            .any(|generic| generic.1.as_ref().map_or(false, uses_internal))
        {
            return Err(TypeInUseError::Generic);
        }

        if let Some((name, _)) = self.attributes.iter().find(|(_, attr)| match attr {
            Attribute::Field(_, ty) | Attribute::Method(_, _, ty, _) => uses_internal(ty),
        }) {
            return Err(TypeInUseError::Attribute(name.to_string()));
        }

        if self.implements.iter().any(uses_internal) {
            return Err(TypeInUseError::Interface);
        }

        if self.inherits.as_ref().map_or(false, uses_internal) {
            return Err(TypeInUseError::BaseType);
        }

        Ok(self.generics.remove(key))
    }
}
