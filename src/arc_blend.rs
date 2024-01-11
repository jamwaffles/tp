//! Based on the paper "Time-Optimal Trajectory Generation for Path Following with Bounded
//! Acceleration and Velocity", Kunz and Stilman.
//!
//! Problems with circular blends:
//!
//! - Start and end points have discontinuous acceleration.

use std::f32::consts::PI;

use nalgebra::{Point2, Vector2};

pub type Coord2 = Point2<f32>;

#[derive(Debug, Copy, Clone)]
pub struct ArcBlend {
    pub prev: Coord2,
    pub mid: Coord2,
    pub next: Coord2,
    pub max_deviation: f32,

    pub arc_start: Coord2,
    pub arc_center: Coord2,
    pub arc_radius: f32,
    pub arc_end: Coord2,
    pub arc_len: f32,
    pub velocity_limit: f32,
    // deviation: f32,
}

impl ArcBlend {
    pub fn new(prev: Coord2, mid: Coord2, next: Coord2, max_deviation: f32) -> Self {
        // Qi
        let prev_delta: Vector2<f32> = mid - prev;
        // Qi+1
        let next_delta: Vector2<f32> = next - mid;

        // Yi
        let prev_delta_norm: Vector2<f32> = prev_delta.normalize();
        // Yi+1
        let next_delta_norm: Vector2<f32> = next_delta.normalize();

        // Lengths of both line segments
        let prev_len = prev_delta.norm();
        let next_len = next_delta.norm();

        // ⍺i: Outside angle between segments in radians
        let outside_angle = prev_delta.angle(&next_delta);

        let half_angle = outside_angle / 2.0;

        // Li: The maximum arc radius that is within the maximum deviation from the midpoint
        let deviation_limit_max_radius =
            (max_deviation * half_angle.sin()) / (1.0 - half_angle.cos());

        // Arc may at most contain half of the smallest path segment, or be a maximum distance
        // away from the midpoint, specified by the given configuration.
        let radius_limit = (prev_len / 2.0)
            .min(next_len / 2.0)
            .min(deviation_limit_max_radius);

        // Ri
        let arc_radius = radius_limit / half_angle.tan();

        // Ci
        let arc_center: nalgebra::OPoint<f32, nalgebra::Const<2>> =
            mid + (next_delta_norm - prev_delta_norm).normalize() * (arc_radius / half_angle.cos());

        // Xi
        let start_point = {
            // Xi: Vector pointing from arc center to start point
            let x_i = (mid - radius_limit * prev_delta_norm - arc_center).normalize();

            arc_center + (x_i * arc_radius)
        };

        let end_point = {
            let x_i = (mid + radius_limit * next_delta_norm - arc_center).normalize();

            arc_center + (x_i * arc_radius)
        };

        // s: Length of arc
        let arc_len = outside_angle * arc_radius;

        // TODO: Configurable from global trajectory limits
        // TODO: This would be the smaller of the 3 axis acceleration limits
        // TODO: Need to take into account arc rotation
        let accel_limit = 5.0;

        // For a trajectory, this will be the min of this value, and the global velocity limit
        let velocity_limit = f32::sqrt(arc_radius * accel_limit);

        dbg!(velocity_limit);

        Self {
            prev,
            mid,
            next,
            max_deviation,
            arc_center,
            arc_start: Coord2::new(start_point.x, start_point.y),
            arc_end: Coord2::new(end_point.x, end_point.y),
            arc_radius,
            arc_len,
            velocity_limit,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colinear() {
        let p1 = Coord2::new(0.0, 0.0);
        let p2 = Coord2::new(2.0, 0.0);
        let p3 = Coord2::new(5.0, 0.0);

        ArcBlend::new(p1, p2, p3, 0.1);
    }

    #[test]
    fn right_angle_with_limit() {
        let p1 = Coord2::new(0.0, 10.0);
        let p2 = Coord2::new(0.0, 0.0);
        let p3 = Coord2::new(10.0, 0.0);

        ArcBlend::new(p1, p2, p3, 0.1);
    }

    #[test]
    fn right_angle_no_limit() {
        let p1 = Coord2::new(0.0, 0.0);
        let p2 = Coord2::new(0.0, 10.0);
        let p3 = Coord2::new(10.0, 10.0);

        ArcBlend::new(p1, p2, p3, f32::INFINITY);
    }
}
