//! Trapezoidal trajectory with non-zero initial velocity.

use nalgebra::Vector3;

pub type Coord3 = Vector3<f32>;

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Lim {
    pub vel: Coord3,
    pub acc: Coord3,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Out {
    pub pos: Coord3,
    pub vel: Coord3,
    pub acc: Coord3,
}

impl core::ops::Add for Out {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            pos: self.pos + rhs.pos,
            vel: self.vel + rhs.vel,
            acc: self.acc + rhs.acc,
        }
    }
}

// #[derive(Debug, Default, Clone, Copy)]
// pub struct Times {
//     pub t_j1: f32,
//     pub t_j2: f32,
//     pub t_d: f32,
//     pub t_a: f32,
//     pub t_v: f32,
//     pub total_time: f32,
// }

#[derive(Debug, Default)]
pub struct Segment {
    /// Start time of this segment.
    pub start_t: f32,
    /// Initial position.
    pub q0: Coord3,
    /// Final position.
    pub q1: Coord3,
    /// Initial velocity.
    pub v0: Coord3,
    /// Final velocity.
    pub v1: Coord3,

    /// Total time.
    pub total_time: f32,

    /// Acceleration time.
    t_a: f32,

    /// Deceleration time.
    t_d: f32,

    /// Highest velocity reached in this segment.
    vlim: Coord3,

    /// Sign of displacement.
    sign: Coord3,
}

impl Segment {
    // FIXME: This assumes the same acc/vel limit for all axes.
    pub fn new(q0: Coord3, q1: Coord3, v0: Coord3, v1: Coord3, start_t: f32, lim: &Lim) -> Self {
        assert!(
            lim.acc > Coord3::zeros() && lim.vel > Coord3::zeros(),
            "Limits must all be positive values, got {:?}",
            lim
        );

        // let sign = (q1 - q0).map(|axis| axis.signum());
        let sign = Coord3::new(1.0, 1.0, 1.0);

        let q0 = q0.component_mul(&sign);
        let q1 = q1.component_mul(&sign);
        let v0 = v0.component_mul(&sign);
        let v1 = v1.component_mul(&sign);

        // Displacement
        let h = q1 - q0;

        let largest_axis = h.imax();

        let process_axis = |axis: usize, limits: &Lim| {
            let h = h[axis];
            let a_max = limits.acc[axis];
            let v_max = limits.vel[axis];
            let v0 = v0[axis];
            let v1 = v1[axis];

            // Was the given max velocity reached? If so, this segment will contain a cruise phase.
            let v_lim_reached = {
                let lhs = h * a_max;

                let rhs = v_max.powi(2) - (v0.powi(2) + v1.powi(2)) / 2.0;

                lhs > rhs
            };

            let vlim = if v_lim_reached {
                // We reached max allowed velocity
                v_max
            } else {
                // Didn't reach max velocity, so reduce it by how much acceleration we can get away with
                f32::sqrt(h * a_max + (v0.powi(2) + v1.powi(2)) / 2.0)
            };

            // dbg!(v_lim_reached, h, a_max);

            let t_a = (vlim - v0) / a_max;
            let t_d = (vlim - v1) / a_max;

            // Total duration of this segment
            let total_time = if v_lim_reached {
                h / v_max
                    + v_max / (2.0 * a_max) * (1.0 - v0 / v_max).powi(2)
                    + v_max / (2.0 * a_max) * (1.0 - v1 / v_max).powi(2)
            } else {
                // No cruise, so just sum accel + decel
                t_a + t_d
            };

            // dbg!(v0, v1, a_max, t_a, t_d, total_time, vlim, h);
            (t_a, t_d, total_time, vlim)
        };

        // Book section 3.2.2: Compute accel period Ta and total duration T for axis with largest
        // displacement.
        // TODO: How do we handle axes with different max velocities/accelerations?
        let (largest_axis_accel_time, largest_axis_decel_time, largest_axis_total_time, _) =
            process_axis(largest_axis, &lim);

        // Compute new limits based on largest axis. This synchronises all other axes.
        let lim = {
            Lim {
                vel: h.map(|axis| axis / (largest_axis_total_time - largest_axis_accel_time)),
                acc: h.map(|axis| {
                    axis / (largest_axis_accel_time
                        * (largest_axis_total_time - largest_axis_accel_time))
                }),
            }
        };

        let mut vlim = Coord3::zeros();

        for i in 0..q0.len() {
            let (_, _, _, limit) = process_axis(i, &lim);

            vlim[i] = limit;

            // dbg!(i, seg);
        }

        // let displacement = displacement.abs();

        // dbg!(displacement);

        // let largest_axis = displacement.imax();

        // dbg!(largest_axis, displacement.normalize());

        // The displacement of each axis relative to the largest displacement (1.0)
        // let relative_displacement = displacement / displacement[largest_axis];

        // dbg!(relative_displacement, "old lim", lim);

        // let largest_traj = crate::trapezoidal_non_zero::Segment::new(
        //     q0[largest_axis],
        //     q1[largest_axis],
        //     v0[largest_axis],
        //     v1[largest_axis],
        //     &crate::trapezoidal_non_zero::Lim {
        //         vel: lim.vel[largest_axis],
        //         acc: lim.acc[largest_axis],
        //     },
        // );

        // dbg!(largest_traj.t, largest_traj.t_a);

        // Book section 3.2.3: Scale limits for each axis to stay on the line.
        // TODO: Take into account different velocity/acceleration limits per axis. Might just need to acc / acc[largest_axis]?
        // let lim = {
        //     Lim {
        //         vel: displacement.map(|axis| axis / (largest_traj.t - largest_traj.t_a)),
        //         acc: displacement
        //             .map(|axis| axis / (largest_traj.t_a * (largest_traj.t - largest_traj.t_a))),
        //     }
        // };

        // dbg!("new lim", lim);

        // let mut vlim = Coord3::zeros();

        // for i in 0..q0.len() {
        //     let seg = crate::trapezoidal_non_zero::Segment::new(
        //         q0[i],
        //         q1[i],
        //         v0[i],
        //         v1[i],
        //         &crate::trapezoidal_non_zero::Lim {
        //             vel: lim.vel[i],
        //             acc: lim.acc[i],
        //         },
        //     );

        //     vlim[i] = seg.vlim;

        //     // dbg!(i, seg);
        // }

        Self {
            start_t,
            q0,
            q1,
            v0,
            v1,
            total_time: largest_axis_total_time,
            t_a: largest_axis_accel_time,
            t_d: largest_axis_decel_time,
            vlim,
            sign,
        }
    }

    /// Get trajectory parameters at the given time `t`.
    pub fn tp(&self, t: f32) -> Option<(Out, Phase)> {
        let Self {
            q0,
            q1,
            v0,
            v1,
            t_a,
            t_d,
            vlim,
            total_time,
            start_t,
            ..
        } = *self;

        let t0 = start_t;
        let t1 = t0 + total_time;
        let t_delta = t - t0;

        let mut phase = Phase::Accel;

        // Accel (3.13a)
        let out = if t_delta < t_a {
            phase = Phase::Accel;

            Some(Out {
                pos: q0 + v0 * (t - t0) + (vlim - v0) / (2.0 * t_a) * (t - t0).powi(2),
                vel: v0 + (vlim - v0) / t_a * (t - t0),
                acc: (vlim - v0) / t_a,
            })
        }
        // Coast (3.13b)
        else if t_delta < (total_time - t_d) {
            phase = Phase::Cruise;

            Some(Out {
                pos: q0 + v0 * t_a / 2.0 + vlim * (t - t0 - t_a / 2.0),
                vel: vlim,
                acc: Coord3::zeros(),
            })
        }
        // Decel (3.13c) (non-inclusive)
        else if t_delta <= total_time {
            phase = Phase::Decel;

            Some(Out {
                pos: q1 - v1 * (t1 - t) - (vlim - v1) / (2.0 * t_d) * (t1 - t).powi(2),
                vel: v1 + (vlim - v1) / t_d * (t1 - t),
                acc: -(vlim - v1) / t_d,
            })
        }
        // Out of range
        else {
            None
        };

        out.map(|out| {
            (
                Out {
                    pos: out.pos.component_mul(&self.sign),
                    vel: out.vel.component_mul(&self.sign),
                    acc: out.acc.component_mul(&self.sign),
                },
                phase,
            )
        })
    }
}

pub enum Phase {
    Accel,
    Cruise,
    Decel,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let q0 = Coord3::new(0.0, 0.0, 0.0);
        let q1 = Coord3::new(10.0, 15.0, 20.0);
        let v0 = Coord3::new(0.0, 0.0, 0.0);
        let v1 = Coord3::new(0.0, 0.0, 0.0);

        let lim = Lim {
            vel: Coord3::new(2.0, 2.0, 2.0),
            acc: Coord3::new(5.0, 5.0, 5.0),
        };

        let seg = Segment::new(q0, q1, v0, v1, 0.0, &lim);

        dbg!(seg);
    }
}
