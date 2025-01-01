#![allow(dead_code)]

use super::project::{ConnectionKey, EntityKey};
use crate::elements::workspace::connection::{ConnectionItemData, PathUpdate};
use crate::geometry::{Point, Rect};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Multiplicity {
    One,
    Many,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RelationType {
    Association,
    Inheritance,
    Realization,
    Dependency,
    Aggregation,
    Composition,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Relation {
    pub relation: RelationType,
    pub entity: EntityKey,
    pub multiplicity: Multiplicity,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Connection {
    pub key: ConnectionKey,
    
    pub from: Relation,
    pub to: Relation,
    pub points: Vec<(i32, i32)>,
    #[serde(skip)]
    pub data: ConnectionItemData,
}

impl Connection {
    pub fn new(
        from: Relation,
        to: Relation,
        points: Vec<(i32, i32)>,
        start: Point,
        end: Point,
    ) -> Self {
        Self {
            key: Default::default(),
            from,
            to,
            data: ConnectionItemData::new(&points, start, end),
            points,
        }
    }

    pub fn update_point(&mut self, index: usize, value: Point) {
        self.data.update(Some(PathUpdate::Point(index, value)));
    }

    pub fn update_origin(&mut self, entity: EntityKey, rect: Rect, reset: bool) {
        self.data.update(if self.from.entity == entity {
            Some(PathUpdate::Start(rect, reset))
        } else if self.to.entity == entity {
            Some(PathUpdate::End(rect, reset))
        } else {
            None
        });
    }
}
