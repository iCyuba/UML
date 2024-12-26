#![allow(dead_code)]

use super::project::EntityKey;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Multiplicity {
    Zero,
    One,
    Many
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RelationType {
    Association,
    Inheritance,
    Realization,
    Dependency,
    Aggregation,
    Composition
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Relation {
    pub entity: EntityKey,
    pub multiplicity: Multiplicity,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Connection {
    pub relation: RelationType,

    pub from: Relation,
    pub to: Relation,
}
