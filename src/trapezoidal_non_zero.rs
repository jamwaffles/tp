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

    /// Limits provided by the user.
    lim: Lim,
}

impl Segment {
    pub fn new(q0: f32, q1: f32, v0: f32, v1: f32, lim: &Lim) -> Self {
        let sign = (q1 - q0).signum();

        // Correct signs for trajectories with negative positions at start and/or end
        let lim = {
            Lim {
                vel: sign * lim.vel,
                acc: sign * lim.acc,
                jerk: sign * lim.jerk,
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

        dbg!(v_max, v_lim_reached, vlim);

        let t_a = (vlim - v0) / a_max;
        let t_d = (vlim - v1) / a_max;

        // // Don't allow trajectories with initial deceleration. For now this is handled by
        // // decelerating at the end of the previous segment.
        // // FIXME: This
        // if t_a < 0.0 {
        //     return Self::default();
        // }

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
            lim,
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
        if t_delta < *t_a {
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
        }
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

/// Returns a tuple of total trajectory time + segment properties at `t`.
pub fn tp_seg(t: f32, segments: &[Segment]) -> (f32, Out) {
    // let mut segs = segments.iter().filter(|segment| {
    //     // Any segment where start time is less than or equal to `t` AND the segment's end
    //     // time (s.start_t + s.total_time) is than `t`. The range end is non-inclusive.

    //     // TODO: Optimise
    //     segment.tp(t).is_some()
    // });

    // let num_segs = segs.clone().count();

    // let mut outs = segs
    //     .clone()
    //     .filter_map(|segment| segment.tp(t))
    //     .fold(Out::default(), |accum, seg| accum + seg);

    // // We're in the overlap region. Integrate sum of velocities (added together in fold() above) to
    // // get displacement
    // if num_segs > 1 {
    //     // The first segment is the previous one (i.e. the one we're at the decel phase for)
    //     let prev_seg = segs.next().unwrap();

    //     // Create a time at beginning of decel phase (beginning of entire trajectory is t = 0)
    //     let decel_start = prev_seg.start_t + prev_seg.total_time - prev_seg.t_a;

    //     // Time since beginning decel
    //     let delta_t = t - decel_start;

    //     // Velocity during the transition phase (= prev decel + curr accel)
    //     let vel = outs.vel;

    //     let Out {
    //         pos: pos_at_decel_start,
    //         ..
    //     } = prev_seg.tp(decel_start).expect("Bad seg");

    //     outs.pos = pos_at_decel_start + (vel * delta_t);
    // }

    let outs = segments
        .iter()
        .find_map(|segment| segment.tp(t))
        .unwrap_or_default();

    // Total time is segment's last time plus its duration. There is no time reduction
    // due to adjacent segment overlap for the last segment, so that doesn't need to be
    // accounted for.
    let total_time = segments
        .last()
        .map(|seg| seg.start_t + seg.total_time)
        .unwrap_or(0.0);

    (total_time, outs)
}

/// Generate test data for multiple segments
pub fn make_segments(lim: &Lim, enable_overlap: bool) -> Vec<Segment> {
    let q0 = 0.0;
    let q1 = 1.0;
    let q2 = 3.0;
    let q3 = 10.0;

    // Velocities at start of these segments
    let v2 = 1.0;
    let v3 = 0.5;

    // End velocity of a segment should be the same as the next segment's start velocity.

    // NOTE: Set overlap times to 0 if "come to full stop" option is desired

    let s1 = Segment::new(q0, q1, 0.0, v2, &lim);

    let mut s2 = Segment::new(q1, q2, v2, v3, &lim);

    // Disable overlap if desired
    let overlap_time = if !enable_overlap {
        0.0
    } else {
        f32::min(s1.t_a, s2.t_a)
    };

    s2.start_t = s1.start_t + s1.total_time - overlap_time;

    // End segment final velocity must always be zero
    let mut s3 = Segment::new(q2, q3, v3, 0.0, &lim);

    // Disable overlap if desired
    let overlap_time = if !enable_overlap {
        0.0
    } else {
        f32::min(s2.t_a, s3.t_a)
    };

    s3.start_t = s2.start_t + s2.total_time - overlap_time;

    vec![s1, s2, s3]
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn multi() {
        //
    }
}
