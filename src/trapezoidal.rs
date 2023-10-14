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

#[derive(Debug, Default, Clone, Copy)]
pub struct Times {
    pub t_j1: f32,
    pub t_j2: f32,
    pub t_d: f32,
    pub t_a: f32,
    pub t_v: f32,
    pub total_time: f32,
}

#[derive(Debug, Default)]
struct Segment {
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
    fn new(start_t: f32, q0: f32, q1: f32, v0: f32, v1: f32, lim: &Lim) -> Self {
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
            start_t,
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

    /// Get trajectory parameters at the given time `t`.
    fn tp(&self, t: f32) -> Option<Out> {
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
    let segment = Segment::new(0.0, q0, q1, v0, v1, &lim);

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
