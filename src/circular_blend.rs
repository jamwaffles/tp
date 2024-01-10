//! Based on the paper "Time-Optimal Trajectory Generation for Path Following with Bounded
//! Acceleration and Velocity", Kunz and Stilman.

use nalgebra::{Point2, Vector2};

type Coord2 = Point2<f32>;

pub struct LinearSegment {
    start: Coord2,
    end: Coord2,
}

pub struct CircularBlend {
    prev: Coord2,
    mid: Coord2,
    next: Coord2,
    max_deviation: f32,

    arc_start: Coord2,
    arc_end: Coord2,
    arc_center: Coord2,
    deviation: f32,
}

impl CircularBlend {
    pub fn new(prev: Coord2, mid: Coord2, next: Coord2, max_deviation: f32) -> Self {
        // Yi
        let prev_delta: Vector2<f32> = (mid - prev).normalize();
        // Yi+1
        let next_delta: Vector2<f32> = (next - mid).normalize();

        // Lengths of both line segments
        let prev_len = prev_delta.norm();
        let next_len = next_delta.norm();

        // ⍺i: Outside angle between segments in radians
        let outside_angle = prev_delta.angle(&next_delta);

        // TODO: Remove this when more tests are added and this still passes. Using non-normalised
        // lengths should be more accurate I think?
        assert_eq!(
            outside_angle,
            prev_delta.normalize().angle(&next_delta.normalize())
        );

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
            mid + (next_delta - prev_delta).normalize() * (arc_radius / half_angle.cos());

        // TODO: This is possibly completely wrong lol
        let start_point = {
            // Xi: Vector pointing from arc center to start point
            let x_i = (mid - deviation_limit_max_radius * prev_delta - arc_center).normalize();

            // TODO: Might not need this
            // x_i * arc_radius

            x_i
        };

        dbg!(
            prev_len,
            next_len,
            outside_angle,
            radius_limit,
            arc_radius,
            arc_center,
            start_point,
        );

        todo!()

        // Self {
        //     prev,
        //     mid,
        //     next,
        //     max_deviation,
        //     arc_center,
        // }
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

        CircularBlend::new(p1, p2, p3, 0.1);
    }

    #[test]
    fn right_angle_with_limit() {
        let p1 = Coord2::new(0.0, 10.0);
        let p2 = Coord2::new(0.0, 0.0);
        let p3 = Coord2::new(10.0, 0.0);

        CircularBlend::new(p1, p2, p3, 0.1);
    }

    #[test]
    fn right_angle_no_limit() {
        let p1 = Coord2::new(0.0, 0.0);
        let p2 = Coord2::new(0.0, 10.0);
        let p3 = Coord2::new(10.0, 10.0);

        CircularBlend::new(p1, p2, p3, f32::INFINITY);
    }
}
