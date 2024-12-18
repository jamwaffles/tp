//! A single segment with synchronised axes.

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

    /// Maximum velocity for each axis.
    vlim: Coord3,

    /// Sign of displacement.
    sign: Coord3,
}

impl Segment {
    pub fn new(q0: Coord3, q1: Coord3, v0: Coord3, v1: Coord3, start_t: f32, lim: &Lim) -> Self {
        assert!(
            lim.acc > Coord3::zeros() && lim.vel > Coord3::zeros(),
            "Limits must all be positive values, got {:?}",
            lim
        );

        let sign = (q1 - q0).map(|axis| axis.signum());
        // let sign = Coord3::new(1.0, 1.0, 1.0);

        let q0 = q0.component_mul(&sign);
        let q1 = q1.component_mul(&sign);
        let v0 = v0.component_mul(&sign);
        let v1 = v1.component_mul(&sign);

        // Displacement
        let h = q1 - q0;

        // Velocity delta
        let v_delta = v1 - v0;

        // Largest axis, i.e. the one everything else will be adjusted against
        let largest_axis = dbg!(h.component_div(&v_delta)).abs().imax();

        dbg!(largest_axis);

        // "Trajectory with preassigned acceleration and velocity", page 73
        let preassigned_acc_vel = |axis: usize, limits: &Lim| {
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

            // Eq. (3.14)
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
        let (
            largest_axis_accel_time,
            largest_axis_decel_time,
            largest_axis_total_time,
            largest_axis_v_max,
        ) = preassigned_acc_vel(largest_axis, &lim);

        // Compute new limits based on largest axis. This synchronises all other axes.
        let vlim = h / (largest_axis_total_time - largest_axis_accel_time);

        dbg!(
            vlim,
            largest_axis_accel_time,
            largest_axis_decel_time,
            largest_axis_total_time,
            largest_axis_v_max
        );

        Self {
            start_t,
            q0,
            q1,
            v0,
            v1,
            total_time: largest_axis_total_time,
            t_a: largest_axis_accel_time,
            t_d: largest_axis_decel_time,
            sign,
            vlim,
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
            total_time,
            start_t,
            vlim,
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

    pub fn q0(&self) -> Coord3 {
        self.q0.component_mul(&self.sign)
    }

    pub fn q1(&self) -> Coord3 {
        self.q1.component_mul(&self.sign)
    }

    pub fn v0(&self) -> Coord3 {
        self.v0.component_mul(&self.sign)
    }

    pub fn v1(&self) -> Coord3 {
        self.v1.component_mul(&self.sign)
    }
}

pub enum Phase {
    Accel,
    Cruise,
    Decel,
}

#[cfg(test)]
mod tests {
    use float_cmp::assert_approx_eq;

    use super::*;

    #[test]
    fn book_example_3_3() {
        let q0 = Coord3::new(0.0, 00.0, 0.0);
        let q1 = Coord3::new(50.0, -40.0, 20.0);

        let v0 = Coord3::new(0.0, 0.0, 0.0);
        let v1 = Coord3::new(0.0, 0.0, 0.0);

        let lim = Lim {
            vel: Coord3::new(20.0, 20.0, 20.0),
            acc: Coord3::new(20.0, 20.0, 20.0),
        };

        let seg = Segment::new(q0, q1, v0, v1, 0.0, &lim);

        dbg!(seg);
    }

    // Single axis: ignore Y and Z
    #[test]
    fn book_example_3_7_a() {
        let q0 = Coord3::new(0.0, 0.0, 0.0);
        let q1 = Coord3::new(30.0, 0.0, 20.0);

        let v0 = Coord3::new(5.0, 0.0, 0.0);
        let v1 = Coord3::new(2.0, 0.0, 0.0);

        let lim = Lim {
            vel: Coord3::new(10.0, 10.0, 10.0),
            acc: Coord3::new(10.0, 10.0, 10.0),
        };

        let seg = Segment::new(q0, q1, v0, v1, 0.0, &lim);

        dbg!(&seg);

        // FIXME: The constants are out of the book but are missing a few DP
        assert_approx_eq!(f32, seg.vlim[0], lim.vel[0]);
        assert_approx_eq!(f32, seg.t_a, 0.5);
        assert_approx_eq!(f32, seg.t_d, 0.8);
        assert_approx_eq!(f32, seg.total_time, 3.44);
    }

    // Single axis: ignore Y and Z
    #[test]
    fn book_example_3_7_b() {
        let q0 = Coord3::new(0.0, 0.0, 0.0);
        let q1 = Coord3::new(30.0, 0.0, 20.0);

        let v0 = Coord3::new(5.0, 0.0, 0.0);
        let v1 = Coord3::new(2.0, 0.0, 0.0);

        let lim = Lim {
            vel: Coord3::new(20.0, 20.0, 20.0),
            acc: Coord3::new(10.0, 10.0, 10.0),
        };

        let seg = Segment::new(q0, q1, v0, v1, 0.0, &lim);

        dbg!(&seg);

        // FIXME: The constants are out of the book but are missing a few DP
        assert_approx_eq!(f32, seg.vlim[0], 17.7);
        assert_approx_eq!(f32, seg.t_a, 1.27);
        assert_approx_eq!(f32, seg.t_d, 1.57);
        assert_approx_eq!(f32, seg.total_time, 2.84);
    }
}
