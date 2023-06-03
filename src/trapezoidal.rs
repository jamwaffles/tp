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

    t_j1: f32,
    /// Acceleration duration.
    t_a: f32,
    /// Maximum accel reached during acceleration phase.
    a_lim_a: f32,
    /// Maximum accel reached during deceleration phase.
    a_lim_d: f32,
    t_j2: f32,
    /// Deceleration duration.
    t_d: f32,
    /// Constant velocity duration.
    t_v: f32,

    /// Highest velocity reached in this segment.
    vlim: f32,

    /// Limits provided by the user.
    lim: Lim,
    /// Whether this segment is feasible/valid or not.
    feasible: bool,
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
        let mut total_time = (h * a_max + v_max.powi(2)) / (a_max * v_max);

        // Max velocity cannot be reached
        if h < v_max.powi(2) / a_max {
            t_a = f32::sqrt(h / a_max);
            total_time = 2.0 * t_a;
            v_max = a_max * t_a;
        }

        todo!()
    }

    fn tp(&self, t: f32) -> Option<Out> {
        todo!()
    }
}

pub fn tp(t: f32, q0: f32, q1: f32, v0: f32, v1: f32, lim: &Lim, times: &mut Times) -> (f32, Out) {
    let segment = Segment::new(0.0, q0, q1, v0, v1, &lim);

    let total_time = segment.t;

    *times = Times {
        t_j1: segment.t_j1,
        t_j2: segment.t_j2,
        t_d: segment.t_d,
        t_a: segment.t_a,
        t_v: segment.t_v,
        total_time,
    };

    (total_time, segment.tp(t).unwrap_or_default())
}
