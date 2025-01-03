#![allow(dead_code)]

use super::{
    connection::{Relation, RelationType},
    entity::EntityType,
    Connection, Entity,
};
use serde::{Deserialize, Serialize};
use slotmap::{new_key_type, SlotMap};

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
            let connection = self.connections.remove(connection).unwrap();
            let other = connection.other(key);

            let other = self.entities.get_mut(other.entity).unwrap();

            other.connections.shift_remove(&connection.key);
        }
    }

    pub fn set_parent(&mut self, entity: EntityKey, parent: Option<EntityKey>) {
        if let Some(parent) = parent {
            if self.entities[entity].parent.is_some() {
                self.set_parent(entity, None);
            }

            match self.entities[parent].entity_type {
                EntityType::Interface | EntityType::SealedClass => return,
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

            self.entities[entity].parent = Some(conn);
        } else if let Some(parent) = self.entities[entity].parent.take() {
            self.disconnect(parent);
        }
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

        self.entities[from].connections.shift_remove(&key);
        self.entities[to].connections.shift_remove(&key);
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
}

impl AsRef<Project> for Project {
    fn as_ref(&self) -> &Project {
        self
    }
}
