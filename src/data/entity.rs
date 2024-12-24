#![allow(dead_code)]

use super::project::ConnectionKey;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Serialize, Deserialize)]
pub struct Entity {
    pub name: String,

    pub connections: HashSet<ConnectionKey>,
}

impl Entity {
    pub fn new(name: String) -> Self {
        Self {
            name,
            connections: HashSet::new(),
        }
    }
}
