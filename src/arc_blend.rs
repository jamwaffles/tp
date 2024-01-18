//! Based on the paper "Time-Optimal Trajectory Generation for Path Following with Bounded
//! Acceleration and Velocity", Kunz and Stilman.
//!
//! Problems with circular blends:
//!
//! - Start and end points have discontinuous acceleration.

use crate::trapezoidal_non_zero_3d::{Lim, Out};
use nalgebra::Vector3;

pub type Coord3 = Vector3<f32>;

#[derive(Debug, Copy, Clone, Default)]
pub struct ArcBlend {
    pub prev: Coord3,
    pub mid: Coord3,
    pub next: Coord3,
    pub max_deviation: f32,

    pub arc_start: Coord3,
    pub arc_center: Coord3,
    pub arc_radius: f32,
    pub arc_end: Coord3,
    pub arc_len: f32,
    pub velocity_limit: Coord3,
    pub time: f32,
    pub start_t: f32,
    // Actual deviation from the midpoint
    // deviation: f32
}

impl ArcBlend {
    pub fn new(
        prev: Coord3,
        mid: Coord3,
        next: Coord3,
        max_deviation: f32,
        start_t: f32,
        Lim {
            acc: max_acceleration,
            vel: max_velocity,
        }: Lim,
    ) -> Self {
        // Qi
        let prev_delta: Vector3<f32> = mid - prev;
        // Qi+1
        let next_delta: Vector3<f32> = next - mid;

        // Yi
        let prev_delta_norm: Vector3<f32> = prev_delta.normalize();
        // Yi+1
        let next_delta_norm: Vector3<f32> = next_delta.normalize();

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
        let arc_center =
            mid + (next_delta_norm - prev_delta_norm).normalize() * (arc_radius / half_angle.cos());

        // Xi
        let start_point = {
            // Xi: Vector pointing from arc center to start point
            let x_i = (mid - radius_limit * prev_delta_norm - arc_center).normalize();

            arc_center + (x_i * arc_radius)
        };

        // Another way of finding start/end points from
        // <https://math.stackexchange.com/questions/2343931/how-to-calculate-3d-arc-between-two-lines>
        // let start_point = {
        //     // Na
        //     let mid_to_prev_normalised = (prev - mid).normalize();

        //     let p_a = mid + (arc_radius / half_angle.tan()) * mid_to_prev_normalised;

        //     p_a
        // };

        let end_point = {
            // Xi+1: Vector pointing from arc center to end point
            let x_i = (mid + radius_limit * next_delta_norm - arc_center).normalize();

            arc_center + (x_i * arc_radius)
        };

        // s: Length of arc
        let arc_len = outside_angle * arc_radius;

        let accel_limit = max_acceleration;

        // Equation from <https://openstax.org/books/physics/pages/6-2-uniform-circular-motion> `a_c
        // = v^2 / r` rearranged.
        let velocity_limit = {
            let lim = arc_radius * accel_limit;

            // Clamp limit to maximum allowed velocity for each axis
            Coord3::new(
                lim.x.sqrt().min(max_velocity.x),
                lim.y.sqrt().min(max_velocity.y),
                lim.z.sqrt().min(max_velocity.z),
            )
        };

        Self {
            prev,
            mid,
            next,
            max_deviation,
            arc_center,
            arc_start: start_point,
            arc_end: end_point,
            arc_radius,
            arc_len,
            velocity_limit,
            time: velocity_limit.norm() * arc_len,
            start_t,
        }
    }

    pub fn tp(&self, t: f32) -> Option<Out> {
        let t = t - self.start_t;

        if t >= self.time || t < 0.0 {
            return None;
        }

        let t = t / self.time;

        let pos = (self.arc_start - self.arc_center).slerp(&(self.arc_end - self.arc_center), t);
        let pos = self.arc_center + pos * self.arc_radius;

        // Centripetal acceleration: it always points towards center of circle
        // TODO: Magnitude
        let acc = (self.arc_center - pos).normalize();

        // Instantaneous velocity is always tangent to the arc
        // TODO: Magnitude
        let vel = {
            let a = self.mid - self.prev;
            let b = self.next - self.mid;

            let normal = b.cross(&a).normalize();

            (normal.cross(&acc)).normalize()
        };

        Some(Out { pos, vel, acc })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colinear() {
        let p1 = Coord3::new(0.0, 0.0, 0.0);
        let p2 = Coord3::new(2.0, 0.0, 0.0);
        let p3 = Coord3::new(5.0, 0.0, 0.0);

        ArcBlend::new(
            p1,
            p2,
            p3,
            0.1,
            0.0,
            Lim {
                acc: Coord3::new(5.0, 5.0, 5.0),
                vel: Coord3::new(2.0, 2.0, 2.0),
            },
        );
    }

    #[test]
    fn right_angle_with_limit() {
        let p1 = Coord3::new(0.0, 10.0, 0.0);
        let p2 = Coord3::new(0.0, 0.0, 0.0);
        let p3 = Coord3::new(10.0, 0.0, 0.0);

        ArcBlend::new(
            p1,
            p2,
            p3,
            0.1,
            0.0,
            Lim {
                acc: Coord3::new(5.0, 5.0, 5.0),
                vel: Coord3::new(2.0, 2.0, 2.0),
            },
        );
    }

    #[test]
    fn right_angle_no_limit() {
        let p1 = Coord3::new(0.0, 0.0, 0.0);
        let p2 = Coord3::new(0.0, 10.0, 0.0);
        let p3 = Coord3::new(10.0, 10.0, 0.0);

        ArcBlend::new(
            p1,
            p2,
            p3,
            f32::INFINITY,
            0.0,
            Lim {
                acc: Coord3::new(5.0, 5.0, 5.0),
                vel: Coord3::new(2.0, 2.0, 2.0),
            },
        );
    }
}
