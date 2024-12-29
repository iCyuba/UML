#![allow(dead_code)]

use super::{Connection, Entity};
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
    pub connections: SlotMap<ConnectionKey, Connection>,
}

impl Project {
    pub fn new(name: String) -> Self {
        Self {
            name,

            entities: SlotMap::with_key(),
            connections: SlotMap::with_key(),
        }
    }

    pub fn add_entity(&mut self, entity: Entity) -> EntityKey {
        self.entities.insert_with_key(|key| {
            let mut entity = entity;
            entity.key = key;
            entity
        })
    }

    pub fn remove_entity(&mut self, key: EntityKey) {
        let entity = self.entities.remove(key).unwrap();

        for connection in entity.connections {
            self.connections.remove(connection);
        }
    }

    pub fn connect(&mut self, connection: Connection) -> ConnectionKey {
        let from = connection.from.entity;
        let to = connection.to.entity;

        let key = self.connections.insert(connection);

        self.entities[from].connections.insert(key);
        self.entities[to].connections.insert(key);

        key
    }

    pub fn disconnect(&mut self, key: ConnectionKey) {
        let connection = self.connections.remove(key).unwrap();

        let from = connection.from.entity;
        let to = connection.to.entity;

        self.entities[from].connections.remove(&key);
        self.entities[to].connections.remove(&key);
    }
}

impl AsRef<Project> for Project {
    fn as_ref(&self) -> &Project {
        self
    }
}
