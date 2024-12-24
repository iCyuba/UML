#![allow(dead_code)]

use super::project::EntityKey;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Connection {
    pub from: EntityKey,
    pub to: EntityKey,
}

impl Connection {
    pub fn new(from: EntityKey, to: EntityKey) -> Self {
        Self { from, to }
    }
}
