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
}

impl AsRef<Project> for Project {
    fn as_ref(&self) -> &Project {
        self
    }
}
