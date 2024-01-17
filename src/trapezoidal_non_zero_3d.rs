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
    start_t: f32,
    /// Initial position.
    q0: Coord3,
    /// Final position.
    q1: Coord3,
    /// Initial velocity.
    v0: Coord3,
    /// Final velocity.
    v1: Coord3,

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
    pub fn new(q0: Coord3, q1: Coord3, v0: Coord3, v1: Coord3, lim: &Lim) -> Self {
        assert!(
            lim.acc > Coord3::zeros() && lim.vel > Coord3::zeros(),
            "Limits must all be positive values, got {:?}",
            lim
        );

        let displacement = q1 - q0;

        let sign = displacement.map(|axis| axis.signum());

        // TODO: This is reversed again within `trapezoidal_non_zero`. Need to optimise this
        // double-negation out.
        let q0 = q0.component_mul(&sign);
        let q1 = q1.component_mul(&sign);

        // let displacement = displacement.abs();

        dbg!(displacement);

        let largest_axis = displacement.imax();

        dbg!(largest_axis, displacement.normalize());

        // The displacement of each axis relative to the largest displacement (1.0)
        let relative_displacement = displacement / displacement[largest_axis];

        dbg!(relative_displacement, "old lim", lim);

        let largest_traj = crate::trapezoidal_non_zero::Segment::new(
            q0[largest_axis],
            q1[largest_axis],
            v0[largest_axis],
            v1[largest_axis],
            &crate::trapezoidal_non_zero::Lim {
                vel: lim.vel[largest_axis],
                acc: lim.acc[largest_axis],
            },
        );

        dbg!(largest_traj.t, largest_traj.t_a);

        // Book section 3.2.3: Scale limits for each axis to stay on the line.
        let lim = {
            Lim {
                vel: displacement.map(|axis| axis / (largest_traj.t - largest_traj.t_a)),
                acc: displacement
                    .map(|axis| axis / (largest_traj.t_a * (largest_traj.t - largest_traj.t_a))),
            }
        };

        dbg!("new lim", lim);

        let mut vlim = Coord3::zeros();

        for i in 0..q0.len() {
            let seg = crate::trapezoidal_non_zero::Segment::new(
                q0[i],
                q1[i],
                v0[i],
                v1[i],
                &crate::trapezoidal_non_zero::Lim {
                    vel: lim.vel[i],
                    acc: lim.acc[i],
                },
            );

            vlim[i] = seg.vlim;

            dbg!(i, seg);
        }

        Self {
            start_t: 0.0,
            q0,
            q1,
            v0,
            v1,
            total_time: largest_traj.t,
            t_a: largest_traj.t_a,
            t_d: largest_traj.t_d,
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

        let seg = Segment::new(q0, q1, v0, v1, &lim);

        dbg!(seg);
    }
}
