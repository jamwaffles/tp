/// Trapezoidal trajectory with multiple points using non-zero start and end velocities.

#[derive(Default, Debug, Clone, Copy)]
pub struct Lim {
    pub vel: f32,
    pub acc: f32,
    pub jerk: f32,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Out {
    pub pos: f32,
    pub vel: f32,
    pub acc: f32,
    pub jerk: f32,
}

impl core::ops::Add for Out {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            pos: self.pos + rhs.pos,
            vel: self.vel + rhs.vel,
            acc: self.acc + rhs.acc,
            jerk: self.jerk + rhs.jerk,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Times {
    pub t_j1: f32,
    pub t_j2: f32,
    pub t_d: f32,
    pub t_a: f32,
    pub t_v: f32,
    pub total_time: f32,
}

// TODO: Un-pub
#[derive(Debug, Default)]
pub struct Segment {
    /// Start time of this segment.
    start_t: f32,
    /// Duration of this segment.
    t: f32,
    /// Initial position.
    q0: f32,
    /// Final position.
    q1: f32,
    /// Initial velocity.
    v0: f32,
    /// Final velocity.
    v1: f32,

    /// Total time.
    total_time: f32,

    /// Acceleration time.
    t_a: f32,

    /// Deceleration time.
    t_d: f32,

    /// Highest velocity reached in this segment.
    vlim: f32,

    /// Sign of displacement.
    sign: f32,
}

impl Segment {
    pub fn new(q0: f32, q1: f32, v0: f32, v1: f32, lim: &Lim) -> Self {
        println!("---");
        let sign = (q1 - q0).signum();

        // Correct signs for trajectories with negative positions at start and/or end
        let lim = {
            Lim {
                // NOTE: This is verbatim out of the book, eq. 3.32. Probably a lot of room here for
                // optimisation.
                vel: { (sign + 1.0) / 2.0 * lim.vel + (sign - 1.0) / 2.0 * -lim.vel },
                acc: { (sign + 1.0) / 2.0 * lim.acc + (sign - 1.0) / 2.0 * -lim.acc },
                jerk: { (sign + 1.0) / 2.0 * lim.jerk + (sign - 1.0) / 2.0 * -lim.jerk },
            }
        };

        let q0 = sign * q0;
        let q1 = sign * q1;
        let v0 = sign * v0;
        let v1 = sign * v1;

        // FIXME: This, but allow acceleration at end of profile
        // // Assigned velocity (e.g. G0/G1 with feed rate, etc)
        // if v1 > 0.0 {
        //     lim.vel = v1.min(lim.vel);
        // }

        let Lim {
            vel: v_max,
            acc: a_max,
            ..
        } = lim;

        // Displacement
        let h = q1 - q0;

        dbg!(sign, q0, q1, a_max, v_max);

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

        dbg!(v_lim_reached, h, a_max);

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

        Self {
            start_t: 0.0,
            t: total_time,
            q0,
            q1,
            v0,
            v1,
            t_a,
            t_d,
            vlim,
            sign,
            total_time,
        }
    }

    pub fn final_pos(&self) -> f32 {
        self.q1
    }

    /// Whether the given `t` is within this segment or not.
    fn contains(&self, t: f32) -> bool {
        t >= self.start_t && t < (self.start_t + self.total_time)
    }

    /// Get trajectory parameters at the given time `t`.
    pub fn tp(&self, t: f32) -> Option<Out> {
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
        } = self;

        let t0 = start_t;
        let t1 = t0 + total_time;
        let t_delta = t - t0;

        // Accel (3.13a)
        let out = if t_delta < *t_a {
            Some(Out {
                pos: q0 + v0 * (t - t0) + (vlim - v0) / (2.0 * t_a) * (t - t0).powi(2),
                vel: v0 + (vlim - v0) / t_a * (t - t0),
                acc: (vlim - v0) / t_a,
                jerk: 0.0,
            })
        }
        // Coast (3.13b)
        else if t_delta < (total_time - t_d) {
            Some(Out {
                pos: q0 + v0 * t_a / 2.0 + vlim * (t - t0 - t_a / 2.0),
                vel: *vlim,
                acc: 0.0,
                jerk: 0.0,
            })
        }
        // Decel (3.13c) (non-inclusive)
        else if t_delta <= *total_time {
            Some(Out {
                pos: q1 - v1 * (t1 - t) - (vlim - v1) / (2.0 * t_d) * (t1 - t).powi(2),
                vel: v1 + (vlim - v1) / t_d * (t1 - t),
                acc: -(vlim - v1) / t_d,
                jerk: 0.0,
            })
        }
        // Out of range
        else {
            None
        };

        out.map(|out| Out {
            pos: out.pos * self.sign,
            vel: out.vel * self.sign,
            acc: out.acc * self.sign,
            jerk: out.jerk * self.sign,
        })
    }

    pub fn times(&self) -> Times {
        Times {
            t_j1: 0.0,
            t_j2: 0.0,
            t_d: self.t_d,
            t_a: self.t_a,
            t_v: 0.0,
            total_time: self.total_time,
        }
    }
}

#[derive(Debug, Default)]
pub struct Segments {
    segments: Vec<Segment>,
}

impl Segments {
    pub fn new() -> Self {
        Self::default()
    }

    fn add_segment(&mut self, segment: Segment) {
        // If overlap is disabled, just push the segment. This would be the "complete stop" mode.

        // If overlap is enabled

        // If next segment has a lower velocity than current vlim, set previous v1 to this lower
        // velocity and recompute.

        // Note: Newly pushed segment has v0 = vlim, v1 = 0

        // TODO: What happens if all the durations are too short?

        // Else if segment has higher velocity, set previous v1 to previous vlim (so no prev decel), and
        // set current segment v0 to prev v1, with vlim as whatever velocity it wants.

        // Else

        // No accel or decel. Ensure curr v1 / next v0 is correct

        todo!()
    }

    pub fn total_time(&self) -> f32 {
        todo!()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Segment> {
        self.segments.iter()
    }

    pub fn tp(&self, t: f32) -> Option<Out> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn multi() {
        //
    }
}
