#![allow(dead_code)]

use super::project::{ConnectionKey, EntityKey};
use crate::animations::animated_property::AnimatedProperty;
use crate::animations::traits::{Animatable, DotMul};
use crate::app::renderer::Canvas;
use crate::app::State;
use crate::elements::primitives::icon::{Icon, Symbol};
use crate::elements::primitives::traits::Draw;
use crate::elements::toolbox_item::Tool;
use crate::elements::workspace::connection::{ConnectionItemData, PathPoint, PathUpdate};
use crate::elements::workspace::item::Item;
use crate::elements::workspace::Workspace;
use crate::geometry::{Point, Rect, Vec2};
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, FRAC_PI_6};
use vello::kurbo::{Affine, BezPath, Cap, Circle, Join, Stroke};
use vello::peniko::{Color, Fill};

#[derive(Debug, Serialize, Deserialize)]
pub enum Multiplicity {
    One,
    Many,
}

impl From<&Multiplicity> for Symbol {
    fn from(multiplicity: &Multiplicity) -> Self {
        match multiplicity {
            Multiplicity::One => Symbol::One,
            Multiplicity::Many => Symbol::Many,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum RelationType {
    Association,
    OneWayAssociation,
    Aggregation,
    Composition,
    Generalization,
    Realization,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Relation {
    pub entity: EntityKey,
    pub multiplicity: Multiplicity,
}

impl Relation {
    pub fn new(entity: EntityKey) -> Self {
        Self {
            entity,
            multiplicity: Multiplicity::One,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Connection {
    pub key: ConnectionKey,

    pub relation: RelationType,
    pub from: Relation,
    pub to: Relation,
    pub points: Vec<(i32, i32)>,

    #[serde(skip)]
    pub data: ConnectionItemData,
}

impl Connection {
    pub fn new(
        relation: RelationType,
        from: Relation,
        to: Relation,
        points: Vec<(i32, i32)>,
        start: Rect,
        end: Rect,
    ) -> Self {
        Self {
            key: Default::default(),
            relation,
            from,
            to,
            data: ConnectionItemData::new(&points, start, end),
            points,
        }
    }

    /// Returns the relation that is not the given entity
    ///
    /// If the entity is not part of the connection, it returns the from relation
    pub fn other(&self, entity: EntityKey) -> &Relation {
        if self.from.entity == entity {
            &self.to
        } else {
            &self.from
        }
    }

    /// Returns the mutable relation that is not the given entity
    pub fn other_mut(&mut self, entity: EntityKey) -> &mut Relation {
        if self.from.entity == entity {
            &mut self.to
        } else {
            &mut self.from
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

    pub fn update_origin(&mut self, entity: EntityKey, rect: Rect, reset: bool) -> bool {
        self.update_data(if self.from.entity == entity {
            Some(PathUpdate::MoveStartRect(rect, reset))
        } else if self.to.entity == entity {
            Some(PathUpdate::MoveEndRect(rect, reset))
        } else {
            None
        })
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
                if index <= self.points.len() {
                    self.points.insert(index, point.into());
                    true
                } else {
                    false
                }
            }
            Some(PathUpdate::MovePoint(index, value)) => {
                let value = value.into();

                if index < self.points.len() && self.points[index] != value {
                    self.points[index] = value;
                    true
                } else {
                    false
                }
            }
            Some(PathUpdate::RemovePoint(index)) => {
                if index < self.points.len() {
                    let point = Point::from(self.points.remove(index));
                    if self.data.ghost_point == Some(point) {
                        self.data.ghost_point = None;
                    }

                    true
                } else {
                    false
                }
            }
            Some(PathUpdate::MoveEndRect(rect, reset)) => {
                Self::animate_property(&mut self.data.end_rect, rect, reset)
            }
            _ => false,
        };

        if self.data.update() | updated {
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

    pub fn swap(&mut self) {
        std::mem::swap(&mut self.from, &mut self.to);

        self.points.reverse();
        self.data =
            ConnectionItemData::new(&self.points, *self.data.end_rect, *self.data.start_rect);
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

    fn render_arrow(&self, c: &mut Canvas, affine: Affine, color: Color, stroke: &Stroke) {
        let points = &self.data.path_points;
        let relation = &self.relation;

        let head = Point::from(&points[points.len() - 1]);
        let tail = Point::from(&points[points.len() - 2]);

        let direction = (head - tail).normalize();

        let arrow_size = ConnectionItemData::ARROW_SIZE;

        let mut path = BezPath::new();
        path.move_to(tail);

        match relation {
            RelationType::Association => {
                path.line_to(head);
            }
            RelationType::OneWayAssociation => {
                path.line_to(head);
                path.line_to(head - direction.rotate_by_angle(FRAC_PI_6) * arrow_size);
                path.move_to(head);
                path.line_to(head - direction.rotate_by_angle(-FRAC_PI_6) * arrow_size);
            }
            RelationType::Aggregation | RelationType::Composition => {
                path.line_to(
                    head - (direction + direction.rotate_by_angle(FRAC_PI_2)) * arrow_size / 2.,
                );
                path.line_to(head);
                path.line_to(
                    head - (direction - direction.rotate_by_angle(FRAC_PI_2)) * arrow_size / 2.,
                );
                path.close_path();
            }
            RelationType::Generalization | RelationType::Realization => {
                path.line_to(tail + direction * (1. - 3f64.sqrt() / 2.) * arrow_size);
                path.move_to(head);
                path.line_to(head - direction.rotate_by_angle(FRAC_PI_6) * arrow_size);
                path.line_to(head - direction.rotate_by_angle(-FRAC_PI_6) * arrow_size);
                path.close_path();
            }
        }

        match relation {
            RelationType::Composition => c.scene().fill(Fill::NonZero, affine, color, None, &path),
            _ => c.scene().stroke(stroke, affine, color, None, &path),
        }
    }

    fn render_icon(&self, c: &mut Canvas, pos: Vec2, scale: f64, line_color: Color, reverse: bool) {
        let points = &self.data.path_points;

        let (head, tail, multiplicity, offset) = if reverse {
            (
                Point::from(&points[0]),
                Point::from(&points[1]),
                &self.from.multiplicity,
                false,
            )
        } else {
            (
                Point::from(&points[points.len() - 1]),
                Point::from(&points[points.len() - 2]),
                &self.to.multiplicity,
                self.relation != RelationType::Association,
            )
        };

        let direction = (head - tail).normalize();
        let size = 0.5;
        let rect = Point::new(size, size);

        let origin = if offset {
            tail - rect / 2. - direction.rotate_by_angle(2. * FRAC_PI_6) * 0.3
        } else {
            head - rect / 2. - direction.rotate_by_angle(FRAC_PI_4) * 0.4
        };

        let rect = (Rect::new(origin, rect) * scale).translate(-pos);
        Icon::new(Symbol::from(multiplicity), rect, size * scale, line_color).draw(c);
    }
}

impl Item for Connection {
    fn update(&mut self, state: &State, ws: &Workspace) -> bool {
        let highlighted = state.selected_point.is_some_and(|(key, _)| key == self.key)
            || ws.hovered_connection == Some(self.key)
            || [self.from.entity, self.to.entity].iter().any(|&entity| {
                ws.hovered_entity == Some(entity) || state.selected_entity == Some(entity)
            });

        self.data.opacity.set(if highlighted { 0.8 } else { 0.5 });

        self.update_data(None)
    }

    fn render(&self, c: &mut Canvas, state: &State, ws: &Workspace) {
        let pos = ws.position();
        let zoom_adjustment = c.scale() * ws.zoom();
        let scale = zoom_adjustment * Workspace::GRID_SIZE;

        let affine = Affine::scale(scale).then_translate((-pos * c.scale()).into());

        let line_color = c.colors().text.multiply_alpha(*self.data.opacity);
        let accent_color = c.colors().accent;
        let stroke = Stroke::new(ConnectionItemData::STROKE_THICKNESS)
            .with_caps(Cap::Butt)
            .with_join(Join::Round);
        let dashed_stroke = Stroke::new(ConnectionItemData::STROKE_THICKNESS)
            .with_caps(Cap::Round)
            .with_join(Join::Round)
            .with_dashes(0., vec![0.75, 0.5]);

        // Draw line
        c.scene().stroke(
            if self.relation == RelationType::Realization {
                &dashed_stroke
            } else {
                &stroke
            },
            affine,
            line_color,
            None,
            &self.data.path,
        );

        // Draw arrow
        self.render_arrow(c, affine, line_color, &stroke);

        // Draw icons
        self.render_icon(c, pos, scale, line_color, false);
        self.render_icon(c, pos, scale, line_color, true);

        let mut render_point = |point: Point, accent: Color, border: Color| {
            for (color, radius) in &[(accent, 0.22), (border, 0.20), (accent, 0.14)] {
                let circle = Circle::new(point, *radius / zoom_adjustment);
                c.scene().fill(Fill::NonZero, affine, *color, None, &circle);
            }
        };

        if state.tool == Tool::Pen || ws.previous_tool == Some(Tool::Pen) {
            let mut show_ghost_point = true;
            let ghost_point = self.data.ghost_point;

            // Draw explicit path points
            for point in self.data.path_points.iter() {
                if let PathPoint::Explicit(point) = point {
                    render_point(*point, accent_color, Color::WHITE);

                    if ghost_point == Some(*point) {
                        show_ghost_point = false
                    }
                }
            }

            // Draw ghost point
            if ws.hovered_connection == Some(self.key) && show_ghost_point {
                if let Some(ghost_point) = ghost_point {
                    render_point(ghost_point, Color::DARK_GRAY, Color::WHITE);
                }
            }
        }
    }
}
