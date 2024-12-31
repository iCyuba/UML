use super::item::Item;
use crate::animations::animated_property::AnimatedProperty;
use crate::animations::standard_animation::{Easing, StandardAnimation};
use crate::animations::traits::Magnitude;
use crate::app::renderer::Canvas;
use crate::app::State;
use crate::elements::workspace::Workspace;
use crate::geometry::{Rect, Vec2};
use crate::{data::Connection, geometry::Point};
use derive_macros::AnimatedElement;
use std::collections::VecDeque;
use std::time::Duration;
use vello::kurbo::{BezPath, Circle};
use vello::{
    kurbo::{Affine, PathEl, Stroke},
    peniko::Fill,
};

// https://stackoverflow.com/questions/1734745/how-to-create-circle-with-b%C3%A9zier-curves
const CONTROL_POINT_DISTANCE: f64 = 0.552284749831;

#[derive(Debug, AnimatedElement)]
pub struct PathItemData {
    pub start_rect: AnimatedProperty<StandardAnimation<Rect>>,
    pub end_rect: AnimatedProperty<StandardAnimation<Rect>>,
    pub points: Vec<Point>,
    pub path: BezPath,
    pub path_points: Vec<PathPoint>,
}

impl Default for PathItemData {
    fn default() -> Self {
        Self {
            start_rect: AnimatedProperty::new(StandardAnimation::new(
                Duration::from_millis(100),
                Easing::EaseOut,
            )),
            end_rect: AnimatedProperty::new(StandardAnimation::new(
                Duration::from_millis(100),
                Easing::EaseOut,
            )),
            points: Vec::new(),
            path: BezPath::new(),
            path_points: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PathPoint {
    Edge(Point),     // Directly connected to the edge of an entity
    Margin(Point),   // Offset from the closest edge point, aligned to grid
    Implicit(Point), // Created automatically to keep lines orthogonal
    Explicit(Point), // Created by the user
}

impl From<&PathPoint> for Point {
    fn from(point: &PathPoint) -> Self {
        match point {
            PathPoint::Edge(p) => *p,
            PathPoint::Margin(p) => *p,
            PathPoint::Implicit(p) => *p,
            PathPoint::Explicit(p) => *p,
        }
    }
}

#[derive(Clone)]
pub enum PathUpdate {
    Start(Rect, bool),
    Point(usize, Point),
    End(Rect, bool),
}

impl PathItemData {
    const STROKE_THICKNESS: f64 = 0.05;

    pub fn new(points: &[(i32, i32)], start: Point, end: Point) -> Self {
        let points: Vec<Point> = points.iter().map(|&p| p.into()).collect();
        let start_rect = Rect::new(start, Vec2::ZERO);
        let end_rect = Rect::new(end, Vec2::ZERO);

        let path_points = Self::convert_path_points(&points, &start_rect, &end_rect);
        let path = Self::convert_bez_path(&path_points);

        Self {
            points,
            path,
            path_points,
            start_rect: AnimatedProperty::new(StandardAnimation::initialized(
                start_rect,
                Duration::from_millis(100),
                Easing::EaseOut,
            )),
            end_rect: AnimatedProperty::new(StandardAnimation::initialized(
                end_rect,
                Duration::from_millis(100),
                Easing::EaseOut,
            )),
        }
    }

    pub fn update(&mut self, value: Option<PathUpdate>) {
        if let Some(value) = value {
            match value {
                PathUpdate::Start(rect, reset) => {
                    if reset {
                        self.start_rect.reset(rect);
                    } else {
                        self.start_rect.set(rect);
                    }
                }
                PathUpdate::Point(index, value) => {
                    self.points[index] = value;
                }
                PathUpdate::End(rect, reset) => {
                    if reset {
                        self.end_rect.reset(rect);
                    } else {
                        self.end_rect.set(rect);
                    }
                }
            }
        }

        self.path_points =
            Self::convert_path_points(&self.points, &self.start_rect, &self.end_rect);

        self.path = Self::convert_bez_path(&self.path_points);
    }

    fn round_vector(v: Vec2, dir: Vec2) -> Vec2 {
        match dir {
            Vec2 { x: 0., y: 1. } => Vec2::new(v.x, v.y.ceil()),
            Vec2 { x: 0., y: -1. } => Vec2::new(v.x, v.y.floor()),
            Vec2 { x: 1., y: 0. } => Vec2::new(v.x.ceil(), v.y),
            Vec2 { x: -1., y: 0. } => Vec2::new(v.x.floor(), v.y),
            _ => v,
        }
    }

    fn get_closest_edge(point: Point, rect: &Rect) -> Point {
        let center = rect.center();
        let half_size = rect.size / 2.;

        let mut closest = center;
        let mut min_distance = f64::MAX;

        for &edge in &[
            center + Vec2::new(half_size.x, 0.),
            center + Vec2::new(-half_size.x, 0.),
            center + Vec2::new(0., half_size.y),
            center + Vec2::new(0., -half_size.y),
        ] {
            let distance = (point - edge).magnitude();
            if distance < min_distance {
                min_distance = distance;
                closest = edge;
            }
        }

        closest
    }

    fn merge_path_points(first: &PathPoint, second: &PathPoint) -> Option<PathPoint> {
        fn merge(first: &PathPoint, second: &PathPoint) -> Option<PathPoint> {
            let center = (Point::from(first) + Point::from(second)) / 2.;

            match (first, second) {
                (PathPoint::Explicit(p1), PathPoint::Implicit(_)) => Some(PathPoint::Explicit(*p1)),
                (PathPoint::Explicit(p1), PathPoint::Margin(_)) => Some(PathPoint::Explicit(*p1)),
                (PathPoint::Edge(p1), PathPoint::Implicit(_)) => Some(PathPoint::Edge(*p1)),
                (PathPoint::Margin(_), PathPoint::Implicit(p2)) => Some(PathPoint::Implicit(*p2)),
                (PathPoint::Implicit(_), PathPoint::Implicit(_)) => {
                    Some(PathPoint::Implicit(center))
                }
                (PathPoint::Margin(_), PathPoint::Margin(_)) => Some(PathPoint::Margin(center)),
                _ => None,
            }
        }

        merge(first, second).or_else(|| merge(second, first))
    }

    fn merge_close_points(points: &[PathPoint], min_distance: f64) -> Vec<PathPoint> {
        if points.is_empty() {
            return Vec::new();
        }

        let mut merged_points = VecDeque::new();
        merged_points.push_back(points[0]);

        for &current_point in &points[1..] {
            let last_point = merged_points.back().unwrap();

            // If the current point is too close to the last merged point, replace the last point
            let diff = Point::from(&current_point) - Point::from(last_point);
            match Self::merge_path_points(last_point, &current_point) {
                Some(merged) if diff.length() < min_distance => {
                    merged_points.pop_back();
                    merged_points.push_back(merged);
                }
                _ => merged_points.push_back(current_point),
            }
        }

        merged_points.into_iter().collect()
    }

    fn convert_path_points(points: &[Point], start: &Rect, end: &Rect) -> Vec<PathPoint> {
        let mut result = Vec::with_capacity(points.len() * 2 + 3); // Estimate capacity

        // Helper function that helps avoid 360 degree and non 90 degree turns
        fn add_implicit_point(result: &mut Vec<PathPoint>, first: Point, second: Point) {
            if (first.x - second.x).abs() >= 0.5 && (first.y - second.y).abs() >= 0.5 {
                let mut implicit_point = Point::new(first.x, second.y);
                if let Some(second_last) = result.get(result.len().wrapping_sub(2)) {
                    let second_last: Point = second_last.into();

                    // If the second to last point goes in the opposite direction, invert the implicit point position
                    if (((second_last.x > first.x) == (implicit_point.x > first.x)
                        && (second_last.y - implicit_point.y).abs() <= 1.5)
                        || ((second_last.y > first.y) == (implicit_point.y > first.y)
                            && (second_last.x - implicit_point.x).abs() <= 1.5))
                        && (first - implicit_point).length() >= 2.
                    {
                        implicit_point = Point::new(second.x, first.y);
                    }
                }
                result.push(PathPoint::Implicit(implicit_point));
            }
        }

        let first = if points.is_empty() {
            end.center()
        } else {
            points[0]
        };

        let edge = Self::get_closest_edge(first, start);
        let direction = (edge - start.center()).normalize();

        result.push(PathPoint::Edge(edge));

        let start_margin = Self::round_vector(edge + direction / 2., direction);
        result.push(PathPoint::Margin(start_margin));

        if !points.is_empty() {
            add_implicit_point(&mut result, first, start_margin);

            result.push(PathPoint::Explicit(first));

            for window in points.windows(2) {
                let prev_point = window[0];
                let current_point = window[1];

                add_implicit_point(&mut result, prev_point, current_point);

                result.push(PathPoint::Explicit(current_point));
            }
        }

        let last = if points.is_empty() {
            start.center()
        } else {
            points[points.len() - 1]
        };

        let edge = Self::get_closest_edge(last, end);
        let direction = (edge - end.center()).normalize();

        let end_margin = Self::round_vector(edge + direction / 2., direction);

        if !points.is_empty() {
            add_implicit_point(&mut result, last, end_margin);
        } else {
            add_implicit_point(&mut result, start_margin, end_margin);
        }

        result.push(PathPoint::Margin(end_margin));

        result.push(PathPoint::Edge(edge));

        Self::merge_close_points(&result, 1.)
    }

    fn convert_bez_path(points: &[PathPoint]) -> BezPath {
        let points: Vec<Point> = points.iter().map(Into::into).collect();

        match points.len() {
            0 | 1 => BezPath::new(),
            2 => BezPath::from_vec(vec![
                PathEl::MoveTo(points[0].into()),
                PathEl::LineTo(points[1].into()),
            ]),
            _ => PathBuilder::new(&points, Self::STROKE_THICKNESS).build(),
        }
    }

    pub fn draw(&self, c: &mut Canvas, affine: Affine) {
        let color = c.colors().workspace_text;
        let accent = c.colors().accent;
        let stroke = Stroke::new(Self::STROKE_THICKNESS);

        c.scene()
            .stroke(&stroke, affine, color.multiply_alpha(0.7), None, &self.path);

        for point in &self.path_points {
            if let PathPoint::Explicit(pos) = point {
                c.scene()
                    .fill(Fill::NonZero, affine, accent, None, &Circle::new(*pos, 0.1));
            }
        }
    }
}

struct PathBuilder<'a> {
    path: BezPath,
    points: &'a [Point],
    control_point_offset: f64,
}

impl<'a> PathBuilder<'a> {
    fn new(points: &'a [Point], stroke_thickness: f64) -> Self {
        let control_point_offset = (CONTROL_POINT_DISTANCE - stroke_thickness) / 2.;
        Self {
            path: BezPath::new(),
            points,
            control_point_offset,
        }
    }

    fn build(mut self) -> BezPath {
        let mut prev = self.points[0];

        self.path.move_to(prev);
        let mut direction = Vec2::ZERO;

        for i in 1..self.points.len() {
            let current = self.points[i];

            if i > 1 {
                self.add_curve_segment(prev, current, direction);
            }

            direction = (current - prev).normalize();

            if i == self.points.len() - 1 {
                self.path.line_to(current)
            } else {
                self.add_line_segment(current, direction);
            }

            prev = current;
        }

        self.path
    }

    fn add_curve_segment(&mut self, prev: Point, current: Point, direction: Point) {
        let prev_control_point = prev - direction * self.control_point_offset;
        let next_direction = (current - prev).normalize();
        let next_control_point = prev + next_direction * self.control_point_offset;

        self.path.curve_to(
            prev_control_point,
            next_control_point,
            prev + next_direction / 2.,
        );
    }

    fn add_line_segment(&mut self, current: Point, direction: Point) {
        self.path.line_to(current - direction / 2.);
    }
}

impl Item for Connection {
    fn update(&mut self) -> bool {
        if self.data.animate() {
            self.data.update(None);
            true
        } else {
            false
        }
    }

    fn render(&self, c: &mut Canvas, _: &State, ws: &Workspace) {
        let pos = ws.position();
        let scale = c.scale() * ws.zoom() * Workspace::GRID_SIZE;

        let affine = Affine::scale(scale).then_translate((-pos * c.scale()).into());

        self.data.draw(c, affine);
    }
}
