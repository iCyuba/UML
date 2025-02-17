use crate::animations::animated_property::AnimatedProperty;
use crate::animations::standard_animation::{Easing, StandardAnimation};
use crate::animations::traits::Magnitude;
use crate::geometry::Point;
use crate::geometry::{Rect, Vec2};
use derive_macros::AnimatedElement;
use std::collections::VecDeque;
use std::f64::consts::SQRT_2;
use std::time::Duration;
use vello::kurbo::BezPath;
use vello::kurbo::PathEl;

// https://stackoverflow.com/questions/1734745/how-to-create-circle-with-b%C3%A9zier-curves
const CONTROL_POINT_DISTANCE: f64 = 0.552284749831;

#[derive(Debug, AnimatedElement)]
pub struct ConnectionItemData {
    pub start_rect: AnimatedProperty<StandardAnimation<Rect>>,
    pub end_rect: AnimatedProperty<StandardAnimation<Rect>>,
    pub opacity: AnimatedProperty<StandardAnimation<f32>>,

    pub ghost_point: Option<Point>,

    pub path: BezPath,
    pub path_points: Vec<PathPoint>,
}

impl Default for ConnectionItemData {
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
            opacity: AnimatedProperty::new(StandardAnimation::initialized(
                0.5,
                Duration::from_millis(100),
                Easing::EaseOut,
            )),
            path: BezPath::new(),
            path_points: Vec::new(),
            ghost_point: None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PathPoint {
    Edge(Point),      // Directly connected to the edge of an entity
    ArrowTail(Point), // Arrow tail - only on one side, just between Edge and Margin points
    Margin(Point),    // Offset from the closest edge point, aligned to grid
    Implicit(Point),  // Created automatically to keep lines orthogonal
    Explicit(Point),  // Created by the user
}

impl From<&PathPoint> for Point {
    fn from(point: &PathPoint) -> Self {
        match point {
            PathPoint::Edge(p) => *p,
            PathPoint::ArrowTail(p) => *p,
            PathPoint::Margin(p) => *p,
            PathPoint::Implicit(p) => *p,
            PathPoint::Explicit(p) => *p,
        }
    }
}

#[derive(Clone)]
pub enum PathUpdate {
    MoveStartRect(Rect, bool),
    AddPoint(usize, Point),
    MovePoint(usize, Point),
    RemovePoint(usize),
    MoveEndRect(Rect, bool),
}

impl ConnectionItemData {
    pub const STROKE_THICKNESS: f64 = 0.05;
    pub const ARROW_SIZE: f64 = SQRT_2 * 0.5;

    pub fn new(points: &[(i32, i32)], start: Rect, end: Rect) -> Self {
        let points: Vec<Point> = points.iter().map(|&p| p.into()).collect();

        let path_points = Self::points_to_path_points(&points, &start, &end);
        let path = Self::points_to_path(&path_points);

        Self {
            path,
            path_points,
            start_rect: AnimatedProperty::new(StandardAnimation::initialized(
                start,
                Duration::from_millis(100),
                Easing::EaseOut,
            )),
            end_rect: AnimatedProperty::new(StandardAnimation::initialized(
                end,
                Duration::from_millis(100),
                Easing::EaseOut,
            )),
            ..Default::default()
        }
    }
    
    pub fn update(&mut self) -> bool {
        self.animate()
    }

    fn round_vector(v: Vec2, dir: Vec2) -> Vec2 {
        match dir {
            Vec2 { x: _, y: 1. } => Vec2::new(v.x, v.y.ceil()),
            Vec2 { x: _, y: -1. } => Vec2::new(v.x, v.y.floor()),
            Vec2 { x: 1., y: _ } => Vec2::new(v.x.ceil(), v.y),
            Vec2 { x: -1., y: _ } => Vec2::new(v.x.floor(), v.y),
            _ => v,
        }
    }

    fn get_rect_edges(rect: &Rect) -> [Point; 4] {
        let center = rect.center();
        let half_size = rect.size / 2.;

        [
            center + Vec2::new(half_size.x, 0.),
            center + Vec2::new(-half_size.x, 0.),
            center + Vec2::new(0., half_size.y),
            center + Vec2::new(0., -half_size.y),
        ]
    }

    fn get_closest_edge_to_point(point: Point, rect: &Rect) -> Point {
        let center = rect.center();
        let mut closest = center;
        let mut min_distance = f64::MAX;

        for edge in Self::get_rect_edges(rect) {
            let distance = (point - edge).magnitude();
            if distance < min_distance {
                min_distance = distance;
                closest = edge;
            }
        }

        closest
    }

    fn get_closest_edges(first: &Rect, second: &Rect) -> (Point, Point) {
        let first_edges = Self::get_rect_edges(first);
        let second_edges = Self::get_rect_edges(second);

        let mut min_distance = f64::MAX;
        let mut closest_edges = (first_edges[0], second_edges[0]);

        for &first_edge in &first_edges {
            for &second_edge in &second_edges {
                let distance = (first_edge - second_edge).magnitude();
                if distance < min_distance {
                    min_distance = distance;
                    closest_edges = (first_edge, second_edge);
                }
            }
        }

        closest_edges
    }

    fn merge_path_points(first: &PathPoint, second: &PathPoint) -> Option<PathPoint> {
        fn merge(first: &PathPoint, second: &PathPoint) -> Option<PathPoint> {
            let center = (Point::from(first) + Point::from(second)) / 2.;

            match (first, second) {
                (PathPoint::Explicit(p1), PathPoint::Implicit(_)) => Some(PathPoint::Explicit(*p1)),
                (PathPoint::Explicit(p1), PathPoint::Margin(_)) => Some(PathPoint::Explicit(*p1)),
                (PathPoint::Edge(p1), PathPoint::Implicit(_)) => Some(PathPoint::Edge(*p1)),
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

    pub fn points_to_path_points(points: &[Point], start: &Rect, end: &Rect) -> Vec<PathPoint> {
        let mut result = Vec::with_capacity(points.len() * 2 + 5); // Estimate capacity

        // Special case where no explicit points are defined
        if points.is_empty() {
            let end_rect = &end.inset_uniform(-Self::ARROW_SIZE * 1.5);

            let (start_margin, end_margin) = Self::get_closest_edges(start, end_rect);

            let start_direction = (start_margin - start.center()).normalize();
            let end_direction = (end_margin - end_rect.center()).normalize();

            let start_margin = Self::round_vector(start_margin, start_direction);
            let end_margin = Self::round_vector(end_margin, end_direction);

            let start_edge = Self::get_closest_edge_to_point(start_margin, start);
            let end_edge = Self::get_closest_edge_to_point(end_margin, end);

            result.push(PathPoint::Edge(start_edge));
            result.push(PathPoint::Margin(start_margin));

            add_implicit_point(&mut result, start_margin, end_margin);

            result.push(PathPoint::Margin(end_margin));
            result.push(PathPoint::ArrowTail(
                end_edge + end_direction * Self::ARROW_SIZE,
            ));
            result.push(PathPoint::Edge(end_edge));

            return result;
        }

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

        let first = points[0];
        let edge = Self::get_closest_edge_to_point(first, start);
        let direction = (edge - start.center()).normalize();

        // First edge point
        result.push(PathPoint::Edge(edge));

        // First margin point
        let start_margin = Self::round_vector(edge, direction);
        result.push(PathPoint::Margin(start_margin));

        // Implicit point between the first margin point and the next explicit point
        add_implicit_point(&mut result, first, start_margin);

        // First explicit point
        result.push(PathPoint::Explicit(first));

        // Intermediate points
        for window in points.windows(2) {
            let prev_point = window[0];
            let current_point = window[1];

            add_implicit_point(&mut result, prev_point, current_point);

            result.push(PathPoint::Explicit(current_point));
        }

        let last = points[points.len() - 1];

        let edge = Self::get_closest_edge_to_point(last, end);
        let direction = (edge - end.center()).normalize();

        let end_margin = Self::round_vector(edge + direction * Self::ARROW_SIZE * 1.5, direction);

        // Last implicit point between the last explicit point and the last margin point
        add_implicit_point(&mut result, last, end_margin);

        // Last margin point
        result.push(PathPoint::Margin(end_margin));

        // Arrow tail
        result.push(PathPoint::ArrowTail(edge + direction * Self::ARROW_SIZE));

        // Last edge point
        result.push(PathPoint::Edge(edge));

        Self::merge_close_points(&result, 1.)
    }

    pub fn points_to_path(points: &[PathPoint]) -> BezPath {
        let points: Vec<Point> = points.iter().map(Into::into).collect();

        match points.len() {
            0 | 1 => BezPath::new(),
            2 => BezPath::from_vec(vec![
                PathEl::MoveTo(points[0].into()),
                PathEl::LineTo(points[1].into()),
            ]),
            _ => PathBuilder::from_points(&points[..points.len() - 1], Self::STROKE_THICKNESS)
                .build(),
        }
    }
}

#[derive(Default)]
struct PathBuilder {
    path: BezPath,
}

impl PathBuilder {
    pub fn from_points(points: &[Point], stroke_thickness: f64) -> Self {
        let mut builder = Self::default();
        let control_point_offset = (CONTROL_POINT_DISTANCE - stroke_thickness) / 2.;

        let mut prev = points[0];

        builder.path.move_to(prev);
        let mut direction = Vec2::ZERO;

        for i in 1..points.len() {
            let current = points[i];

            if i > 1 {
                builder.add_curve_segment(prev, current, direction, control_point_offset);
            }

            direction = (current - prev).normalize();

            if i == points.len() - 1 {
                builder.path.line_to(current)
            } else {
                builder.add_line_segment(current, direction);
            }

            prev = current;
        }

        builder
    }

    pub fn build(self) -> BezPath {
        self.path
    }

    pub fn add_curve_segment(
        &mut self,
        prev: Point,
        current: Point,
        direction: Point,
        control_point_offset: f64,
    ) {
        let prev_control_point = prev - direction * control_point_offset;
        let next_direction = (current - prev).normalize();
        let next_control_point = prev + next_direction * control_point_offset;

        self.path.curve_to(
            prev_control_point,
            next_control_point,
            prev + next_direction / 2.,
        );
    }

    pub fn add_line_segment(&mut self, current: Point, direction: Point) {
        self.path.line_to(current - direction / 2.);
    }
}
