/// Trapezoidal single trajectory segment.

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

    /// Highest velocity reached in this segment.
    vlim: f32,

    /// Limits provided by the user.
    lim: Lim,
}

impl Segment {
    fn new(q0: f32, q1: f32, v0: f32, v1: f32, lim: &Lim) -> Self {
        let Lim {
            vel: mut v_max,
            acc: a_max,
            ..
        } = lim;

        // Displacement
        let h = q1 - q0;

        // Acceleration (and deceleration) duration
        let mut t_a = v_max / a_max;
        // (3.2.1, eq. 3.8)
        let mut total_time = (h * a_max + v_max.powi(2)) / (a_max * v_max);

        // Max velocity cannot be reached (eq. 3.10)
        if h < v_max.powi(2) / a_max {
            t_a = f32::sqrt(h / a_max);
            total_time = 2.0 * t_a;
            v_max = a_max * t_a;
        }

        Self {
            start_t: 0.0,
            t: total_time,
            q0,
            q1,
            v0,
            v1,
            t_a,
            vlim: v_max,
            lim: *lim,
            total_time,
        }
    }

    pub fn final_pos(&self) -> f32 {
        self.q1
    }

    /// Get trajectory parameters at the given time `t`.
    fn tp(&self, t: f32) -> Option<Out> {
        let t = t - self.start_t;

        // Accel
        if t < self.t_a {
            let a0 = self.q0;
            let a1 = 0.0;
            let a2 = self.vlim / (2.0 * self.t_a);

            Some(Out {
                pos: a0 + a1 * t + a2 * t.powi(2),
                vel: a1 + 2.0 * a2 * t,
                acc: 2.0 * a2,
                jerk: 0.0,
            })
        }
        // Coast
        else if t < (self.total_time - self.t_a) {
            let b0 = self.q0 - (self.vlim * self.t_a) / 2.0;
            let b1 = self.vlim;

            Some(Out {
                pos: b0 + b1 * t,
                vel: b1,
                acc: 0.0,
                jerk: 0.0,
            })
        }
        // Decel
        else if t <= self.total_time {
            let c0 = self.q1 - (self.vlim * self.total_time.powi(2)) / (2.0 * self.t_a);
            let c1 = (self.vlim * self.total_time) / self.t_a;
            let c2 = -(self.vlim / (2.0 * self.t_a));

            Some(Out {
                pos: c0 + c1 * t + c2 * t.powi(2),
                vel: c1 + 2.0 * c2 * t,
                acc: 2.0 * c2,
                jerk: 0.0,
            })
        }
        // Out of range
        else {
            None
        }
    }
}

pub fn tp(t: f32, q0: f32, q1: f32, v0: f32, v1: f32, lim: &Lim, times: &mut Times) -> (f32, Out) {
    let segment = Segment::new(q0, q1, v0, v1, &lim);

    let total_time = segment.t;

    *times = Times {
        t_j1: 0.0,
        t_j2: 0.0,
        t_d: 0.0,
        t_a: segment.t_a,
        t_v: 0.0,
        total_time,
    };

    (total_time, segment.tp(t).unwrap_or_default())
}

/// Returns a tuple of total trajectory time + segment properties at `t`.
pub fn tp_seg(t: f32, segments: &[Segment]) -> (f32, Out) {
    let mut segs = segments.iter().filter(|segment| {
        // Any segment where start time is less than or equal to `t` AND the segment's end
        // time (s.start_t + s.total_time) is than or equal to `t`

        let in_range = segment.start_t <= t && (segment.start_t + segment.total_time) > t;

        in_range
    });

    let num_segs = segs.clone().count();

    let mut outs = segs
        .clone()
        .filter_map(|segment| segment.tp(t))
        .fold(Out::default(), |accum, seg| accum + seg);

    // We're in the overlap region. Integrate sum of velocities (added together in fold() above) to
    // get displacement
    if num_segs > 1 {
        // The first segment is the previous one (i.e. the one we're at the decel phase for)
        let prev_seg = segs.next().unwrap();

        // Create a time at beginning of decel phase (beginning of entire trajectory is t = 0)
        let decel_start = prev_seg.start_t + prev_seg.total_time - prev_seg.t_a;

        // Time since beginning decel
        let delta_t = t - decel_start;

        // Velocity during the transition phase (= prev decel + curr accel)
        let vel = outs.vel;

        let Out {
            pos: pos_at_decel_start,
            ..
        } = prev_seg.tp(decel_start).expect("Bad seg");

        outs.pos = pos_at_decel_start + (vel * delta_t);
    }

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
    let q3 = 7.0;

    // NOTE: Set overlap times to 0 if "come to full stop" option is desired

    let s1 = Segment::new(q0, q1, 0.0, 0.0, &lim);

    let mut s2 = Segment::new(q1, q2, 0.0, 0.0, &lim);

    // Disable overlap if desired
    let overlap_time = if !enable_overlap {
        0.0
    } else {
        f32::min(s1.t_a, s2.t_a)
    };

    s2.start_t = s1.start_t + s1.total_time - overlap_time;

    let mut s3 = Segment::new(q2, q3, 0.0, 0.0, &lim);

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
