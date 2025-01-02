#![allow(dead_code)]

use super::project::{ConnectionKey, EntityKey};
use crate::animations::animated_property::AnimatedProperty;
use crate::animations::traits::{Animatable, DotMul};
use crate::elements::workspace::connection::{ConnectionItemData, PathPoint, PathUpdate};
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

impl Relation {
    pub fn new(entity: EntityKey) -> Self {
        Self {
            entity,
            relation: RelationType::Association,
            multiplicity: Multiplicity::One,
        }
    }
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
        start: Rect,
        end: Rect,
    ) -> Self {
        Self {
            key: Default::default(),
            from,
            to,
            data: ConnectionItemData::new(&points, start, end),
            points,
        }
    }

    pub fn add_point(&mut self, index: usize, point: Point) {
        self.update_data(Some(PathUpdate::AddPoint(index, point)));
    }

    pub fn update_point(&mut self, index: usize, value: Point) {
        self.update_data(Some(PathUpdate::MovePoint(index, value)));
    }

    pub fn remove_point(&mut self, index: usize) {
        self.update_data(Some(PathUpdate::RemovePoint(index)));
    }

    pub fn update_origin(&mut self, entity: EntityKey, rect: Rect, reset: bool) {
        self.update_data(if self.from.entity == entity {
            Some(PathUpdate::MoveStartRect(rect, reset))
        } else if self.to.entity == entity {
            Some(PathUpdate::MoveEndRect(rect, reset))
        } else {
            None
        });
    }

    pub fn get_hovered_path_point(&self, point: &Point, distance: f64) -> Option<usize> {
        self.points
            .iter()
            .position(|&p| (Point::from(p) - *point).length() < distance)
    }

    pub fn get_hovered_line(&self, point: &Point, distance: f64) -> Option<(usize, Point)> {
        let mut index = 0;
        self.data.path_points.windows(2).find_map(|pair| {
            if let PathPoint::Explicit(_) = pair[0] {
                index += 1
            }

            let (start, end) = (Point::from(&pair[0]), Point::from(&pair[1]));

            let line = end - start;
            let length = line.dot_mul(line);

            let t = ((*point - start).dot_mul(line) / length).clamp(0.0, 1.0);

            let closest_point = start + line * t;
            let dist = (*point - closest_point).length();

            if dist < distance {
                Some((index, closest_point))
            } else {
                None
            }
        })
    }

    pub(crate) fn update_data(&mut self, value: Option<PathUpdate>) -> bool {
        let updated = match value {
            Some(PathUpdate::MoveStartRect(rect, reset)) => {
                Self::animate_property(&mut self.data.start_rect, rect, reset)
            }
            Some(PathUpdate::AddPoint(index, point)) => {
                self.points.insert(index, point.into());
                true
            }
            Some(PathUpdate::MovePoint(index, value)) => {
                let value = value.into();

                if self.points[index] != value {
                    self.points[index] = value;
                    true
                } else {
                    false
                }
            }
            Some(PathUpdate::RemovePoint(index)) => {
                let point = Point::from(self.points.remove(index));
                if self.data.ghost_point == Some(point) {
                    self.data.ghost_point = None;
                }

                true
            }
            Some(PathUpdate::MoveEndRect(rect, reset)) => {
                Self::animate_property(&mut self.data.end_rect, rect, reset)
            }
            _ => false,
        };

        if self.data.start_rect.animate() | self.data.end_rect.animate() | updated {
            let points = self.points.iter().map(|&p| p.into()).collect::<Vec<_>>();
            self.data.path_points = ConnectionItemData::points_to_path_points(
                &points,
                &self.data.start_rect,
                &self.data.end_rect,
            );

            self.data.path = ConnectionItemData::points_to_path(&self.data.path_points);

            true
        } else {
            false
        }
    }

    fn animate_property<TVal, TAni>(
        prop: &mut AnimatedProperty<TAni>,
        val: TVal,
        reset: bool,
    ) -> bool
    where
        TAni: Animatable<Value = TVal>,
    {
        if reset {
            prop.reset(val)
        } else {
            prop.set(val)
        }
    }
}
