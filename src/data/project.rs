#![allow(dead_code)]

use super::{
    connection::{Relation, RelationType},
    entity::EntityType,
    Connection, Entity,
};
use serde::{Deserialize, Serialize};
use slotmap::{new_key_type, SlotMap};
use std::fmt::Display;

new_key_type! {
    pub struct ConnectionKey;
    pub struct EntityKey;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    pub name: String,

    pub entities: SlotMap<EntityKey, Entity>,
    // Used for keeping track of the order in which entities are displayed on the screen
    pub ordered_entities: Vec<EntityKey>,
    pub connections: SlotMap<ConnectionKey, Connection>,
}

impl Project {
    pub fn new(name: String) -> Self {
        Self {
            name,

            entities: SlotMap::with_key(),
            ordered_entities: Vec::new(),
            connections: SlotMap::with_key(),
        }
    }

    pub fn add_entity(&mut self, entity: Entity) -> EntityKey {
        self.entities.insert_with_key(|key| {
            self.ordered_entities.push(key);
            let mut entity = entity;
            entity.key = key;
            entity
        })
    }

    pub fn remove_entity(&mut self, key: EntityKey) {
        self.ordered_entities.retain(|&k| k != key);
        let entity = self.entities.remove(key).unwrap();

        for &connection in entity.connections.iter() {
            self.disconnect(connection);
        }
    }

    pub fn set_parent(&mut self, entity: EntityKey, parent: Option<EntityKey>) -> bool {
        if let Some(parent) = parent {
            let ent = &self.entities[entity];
            if ent.entity_type == EntityType::Interface {
                return false;
            }

            if ent.parent.is_some() {
                self.set_parent(entity, None);
            }

            match self.entities[parent].entity_type {
                EntityType::Interface | EntityType::SealedClass => return false,
                _ => {}
            }

            let conn = self.connect(Connection::new(
                RelationType::Generalization,
                Relation::new(entity),
                Relation::new(parent),
                vec![],
                self.entities[entity].get_rect(),
                self.entities[parent].get_rect(),
            ));

            if self.entities[parent].parent == Some(conn) {
                self.connections[conn].swap();
                self.entities[parent].parent = None;
            }

            self.entities[entity].parent = Some(conn);

            true
        } else if let Some(parent) = self.entities[entity].parent.take() {
            self.disconnect(parent);
            true
        } else {
            false
        }
    }

    pub fn implement(&mut self, entity: EntityKey, interface: EntityKey) -> bool {
        if self.entities[interface].entity_type != EntityType::Interface {
            return false;
        }

        let conn = self.connect(Connection::new(
            RelationType::Realization,
            Relation::new(entity),
            Relation::new(interface),
            vec![],
            self.entities[entity].get_rect(),
            self.entities[interface].get_rect(),
        ));

        if self.entities[interface].implements.contains(&conn) {
            return false;
        }

        self.entities[entity].implements.push(conn);

        true
    }

    pub fn associate(&mut self, from: EntityKey, to: EntityKey) -> bool {
        let conn = self.connect(Connection::new(
            RelationType::Association,
            Relation::new(from),
            Relation::new(to),
            vec![],
            self.entities[from].get_rect(),
            self.entities[to].get_rect(),
        ));

        self.entities[from].connections.insert(conn);
        self.entities[to].connections.insert(conn);

        true
    }

    pub fn connect(&mut self, connection: Connection) -> ConnectionKey {
        let from = connection.from.entity;
        let to = connection.to.entity;

        // Check if connection already exists
        for &key in self.entities[from].connections.iter() {
            let existing = &self.connections[key];
            if (existing.from.entity == from && existing.to.entity == to)
                || (existing.from.entity == to && existing.to.entity == from)
            {
                return key;
            }
        }

        let key = self.connections.insert_with_key(|key| {
            let mut connection = connection;
            connection.key = key;
            connection
        });

        self.entities[from].connections.insert(key);
        self.entities[to].connections.insert(key);

        key
    }

    pub fn disconnect(&mut self, key: ConnectionKey) {
        let connection = self.connections.remove(key).unwrap();

        let from = connection.from.entity;
        let to = connection.to.entity;

        if let Some(from) = self.entities.get_mut(from) {
            from.connections.shift_remove(&key);

            if connection.relation == RelationType::Generalization {
                from.parent = None;
            } else if connection.relation == RelationType::Realization {
                from.implements.retain(|&k| k != key);
            }
        }

        if let Some(to) = self.entities.get_mut(to) {
            to.connections.shift_remove(&key);
        }
    }

    /// Modifies entity based on callback
    pub fn entity_mut(
        &mut self,
        entity: Option<EntityKey>,
        f: impl FnOnce(&mut Entity) -> bool,
    ) -> bool {
        if let Some(entity) = entity.and_then(|key| self.entities.get_mut(key)) {
            return f(entity);
        }

        false
    }

    pub fn get_entity_connections(&self, entity: EntityKey) -> Vec<ConnectionKey> {
        self.entities[entity].connections.iter().copied().collect()
    }

    pub fn entity_to_token(&self, entity: EntityKey) -> Vec<Token> {
        let mut tokens = vec![
            Token::Keyword(Keyword::Namespace),
            Token::Identifier(self.get_sanitized_name()),
            Token::SemiColon,
            Token::NewLine,
            Token::NewLine,
        ];

        let entity = &self.entities[entity];
        tokens.push(Token::Keyword(Keyword::Public));
        tokens.append(entity.entity_type.as_token().as_mut());
        tokens.push(Token::Identifier(entity.name.clone()));

        let mut implements = Vec::new();

        if let Some(parent) = entity.parent {
            let parent = &self.entities[self.connections[parent].to.entity];
            implements.push(parent.name.clone());
        }

        for &connection in entity.implements.iter() {
            let interface = &self.entities[self.connections[connection].to.entity];
            implements.push(interface.name.clone());
        }

        if !implements.is_empty() {
            tokens.push(Token::Implementation(
                implements.into_iter().map(Token::Identifier).collect(),
            ));
        }

        tokens.push(Token::Block(
            entity
                .fields
                .iter()
                .flat_map(|field| std::iter::once(Token::NewLine).chain(field.as_token()))
                .chain(
                    entity.methods.iter().flat_map(|method| {
                        std::iter::once(Token::NewLine).chain(method.as_token())
                    }),
                )
                .collect::<Vec<_>>(),
        ));

        tokens
    }

    pub fn sanitize(&self, name: &str) -> String {
        sanitize_filename::sanitize_with_options(
            name,
            sanitize_filename::Options {
                truncate: true,
                windows: true,
                replacement: "",
            },
        )
        .trim()
        .replace(" ", "_")
    }

    pub fn get_sanitized_name(&self) -> String {
        self.sanitize(&self.name)
    }
}

impl AsRef<Project> for Project {
    fn as_ref(&self) -> &Project {
        self
    }
}

#[derive(Clone)]
pub enum Keyword {
    Public,
    Private,
    Protected,
    Class,
    Interface,
    Abstract,
    Sealed,
    Namespace,
}

#[derive(Clone)]
pub enum Token {
    Keyword(Keyword),
    Identifier(String),
    Block(Vec<Token>),
    Accessors,
    MethodArguments(Vec<(Token, Token)>),
    Placeholder,
    NewLine,
    Space,
    SemiColon,
    Implementation(Vec<Token>),
}

pub struct TokenVec(pub Vec<Token>);

impl Display for TokenVec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for token in &self.0 {
            write!(f, "{}", token)?;
        }
        Ok(())
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Keyword(keyword) => write!(
                f,
                "{} ",
                match keyword {
                    Keyword::Public => "public",
                    Keyword::Private => "private",
                    Keyword::Protected => "protected",
                    Keyword::Class => "class",
                    Keyword::Interface => "interface",
                    Keyword::Abstract => "abstract",
                    Keyword::Sealed => "sealed",
                    Keyword::Namespace => "namespace",
                }
            ),
            Token::Identifier(ident) => write!(f, "{}", ident.trim()),
            Token::Block(tokens) => {
                write!(f, " {{")?;
                write!(
                    f,
                    "{}",
                    tokens
                        .iter()
                        .fold(String::new(), |mut output, t| {
                            output.push_str(&format!("{}", t));
                            output
                        })
                        .replace("\n", "\n\t")
                )?;
                write!(f, "\n}}")
            }
            Token::Accessors => write!(f, " {{ get; set; }}"),
            Token::MethodArguments(args) => {
                write!(f, "(")?;
                for (i, (r#type, name)) in args.iter().enumerate() {
                    write!(f, "{} {}", r#type, name)?;
                    if i < args.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, ")")
            }
            Token::Placeholder => write!(f, "throw new NotImplementedException();"),
            Token::NewLine => writeln!(f),
            Token::Space => write!(f, " "),
            Token::Implementation(tokens) => {
                write!(f, " : ")?;
                for (i, token) in tokens.iter().enumerate() {
                    write!(f, "{}", token)?;
                    if i < tokens.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                Ok(())
            }
            Token::SemiColon => write!(f, ";"),
        }
    }
}

pub trait AsToken {
    fn as_token(&self) -> Vec<Token>;
}
