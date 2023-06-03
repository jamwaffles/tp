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

fn is_feasible(q0: f32, q1: f32, v0: f32, v1: f32, lim: &Lim) -> bool {
    let Lim {
        acc: amax,
        jerk: jmax,
        ..
    } = lim;

    let t_j_star = (f32::abs(v1 - v0) / jmax).sqrt().min(amax / jmax);

    let delta = q1 - q0;

    let limit = amax / jmax;

    let comp = if t_j_star < limit {
        t_j_star * (v0 + v1)
    } else if t_j_star == limit {
        0.5 * (v0 + v1) * (t_j_star + (v1 - v0).abs() / amax)
    } else {
        return false;
    };

    delta > comp
}

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
        let delta = q1 - q0;

        // 3.31
        // ---
        let sign = delta.signum();

        let q0 = sign * q0;
        let q1 = sign * q1;
        let v0 = sign * v0;
        let v1 = sign * v1;

        let lim = Lim {
            vel: sign * lim.vel,
            acc: sign * lim.acc,
            jerk: sign * lim.jerk,
        };

        if !is_feasible(q0, q1, v0, v1, &lim) {
            return Self::default();
        }

        let Lim {
            vel: vmax,
            acc: amax,
            jerk: jmax,
        } = lim;

        // Symmetrical profiles for now
        let vmin = -vmax;
        let amin = -amax;
        let jmin = -jmax;

        let max_accel_not_reached = (vmax - v0) * jmax < amax.powi(2);
        let max_decel_not_reached = (vmax - v1) * jmax < amax.powi(2);

        // Acceleration time Ta
        let (mut t_j1, mut t_a) = if max_accel_not_reached {
            // The time that jerk is constant during accel
            let t_j1 = f32::sqrt((vmax - v0) / jmax);
            // Acceleration period
            let t_a = 2.0 * t_j1;

            (t_j1, t_a)
        } else {
            // The time that jerk is constant during accel
            let t_j1 = amax / jmax;
            // Acceleration period
            let t_a = t_j1 + ((vmax - v0) / amax);

            (t_j1, t_a)
        };

        // Deceleration time Td
        let (mut t_j2, mut t_d) = if max_decel_not_reached {
            // The time that jerk is constant during accel
            let t_j2 = f32::sqrt((vmax - v1) / jmax);
            // Deceleration period
            let t_d = 2.0 * t_j2;

            (t_j2, t_d)
        } else {
            // The time that jerk is constant during accel
            let t_j2 = amax / jmax;
            // Deceleration period
            let t_d = t_j2 + ((vmax - v1) / amax);

            (t_j2, t_d)
        };

        // 3.25 duration of constant velocity
        let mut t_v =
            (delta / vmax) - (t_a / 2.0) * (1.0 + v0 / vmax) - (t_d / 2.0) * (1.0 + v1 / vmax);

        // Greatest velocity reached
        let vlim;

        // No constant velocity section
        if t_v < 0.0 {
            t_j1 = amax / jmax;
            t_j2 = amax / jmax;

            let delta = amax.powi(4) / jmax.powi(2)
                + 2.0 * (v0.powi(2) + v1.powi(2))
                + amax * (4.0 * (q1 - q0) - 2.0 * amax / jmax * (v0 + v1));

            t_a = (amax.powi(2) / jmax - 2.0 * v0 + delta.sqrt()) / 2.0 * amax;
            t_d = (amax.powi(2) / jmax - 2.0 * v1 + delta.sqrt()) / 2.0 * amax;

            t_v = 0.0;

            vlim = v0 + (t_a - t_j1) * jmax * t_j1;
        } else {
            vlim = vmax;
        }

        let total_time = t_a + t_v + t_d;

        // Acceleration reached
        let a_lim_a = jmax * t_j1;
        let a_lim_d = -jmax * t_j2;

        Self {
            q0,
            q1,
            v0,
            v1,
            t_j1,
            t_a,
            a_lim_a,
            a_lim_d,
            t_j2,
            t_d,
            t_v,
            feasible: true,
            lim,
            vlim,
            start_t,
            t: total_time,
        }
    }

    fn tp(&self, t: f32) -> Option<Out> {
        let t = t - self.start_t;

        if t < 0.0 {
            return None;
        }

        let Self {
            q0,
            q1,
            v0,
            v1,
            lim,
            t_j1,
            t_a,
            a_lim_a,
            a_lim_d,
            t_j2,
            t_d,
            t_v,
            t: total_time,
            vlim,
            ..
        } = *self;

        let Lim {
            vel: vmax,
            acc: amax,
            jerk: jmax,
        } = lim;

        // Symmetrical profiles for now
        let vmin = -vmax;
        let amin = -amax;
        let jmin = -jmax;

        // Accel phase, max jerk
        if t < t_j1 {
            let pos = q0 + (v0 * t) + (jmax * t.powi(3) / 6.0);
            let vel = v0 + jmax * t.powi(2) / 2.0;
            let acc = jmax * t;
            let jerk = jmax;

            Some(Out {
                pos,
                vel,
                acc,
                jerk,
            })
        }
        // Accel phase, zero jerk
        else if t < (t_a - t_j1) {
            let pos =
                q0 + (v0 * t) + (a_lim_a / 6.0) * (3.0 * t.powi(2) - 3.0 * t_j1 * t + t_j1.powi(2));
            let vel = v0 + a_lim_a * (t - t_j1 / 2.0);
            let acc = a_lim_a;
            let jerk = 0.0;

            Some(Out {
                pos,
                vel,
                acc,
                jerk,
            })
        }
        // Accel phase, min jerk
        else if t < t_a {
            let pos =
                q0 + (vlim + v0) * t_a / 2.0 - vlim * (t_a - t) - jmin * (t_a - t).powi(3) / 6.0;
            let vel = vlim + jmin * (t_a - t).powi(2) / 2.0;
            let acc = -jmin * (t_a - t);
            let jerk = jmin;

            Some(Out {
                pos,
                vel,
                acc,
                jerk,
            })
        }
        // Coast
        else if t < t_a + t_v {
            let pos = q0 + (vlim + v0) * t_a / 2.0 + vlim * (t - t_a);
            let vel = vlim;
            let acc = 0.0;
            let jerk = 0.0;

            Some(Out {
                pos,
                vel,
                acc,
                jerk,
            })
        }
        // Decel, max jerk
        else if t < total_time - t_d + t_j2 {
            let pos = q1 - (vlim + v1) * t_d / 2.0 + vlim * (t - total_time + t_d)
                - jmax * (t - total_time + t_d).powi(3) / 6.0;
            let vel = vlim - jmax * (t - total_time + t_d).powi(2) / 2.0;
            let acc = -jmax * (t - total_time + t_d);
            let jerk = jmax;

            Some(Out {
                pos,
                vel,
                acc,
                jerk,
            })
        }
        // Decel, zero jerk
        else if t < total_time - t_j2 {
            let pos = q1 - (vlim + v1) * t_d / 2.0
                + vlim * (t - total_time + t_d)
                + a_lim_d / 6.0
                    * (3.0 * (t - total_time + t_d).powi(2) - 3.0 * t_j2 * (t - total_time + t_d)
                        + t_j2.powi(2));
            let vel = vlim + a_lim_d * (t - total_time + t_d - t_j2 / 2.0);
            let acc = a_lim_d;
            let jerk = 0.0;

            Some(Out {
                pos,
                vel,
                acc,
                jerk,
            })
        }
        // Decel, min jerk
        else if t <= total_time {
            let pos = q1 - v1 * (total_time - t) - jmax * (total_time - t).powi(3) / 6.0;
            let vel = v1 + jmax * (total_time - t).powi(2) / 2.0;
            let acc = -jmax * (total_time - t);
            let jerk = jmin;

            Some(Out {
                pos,
                vel,
                acc,
                jerk,
            })
        }
        // Out of bounds!
        else {
            None
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it() {
        // These values give two S curves with constant acceleration sections AND a coast section.
        let q0 = 0.0;
        let q1 = 20.0;
        let v0 = 0.0;
        let v1 = 0.0;
        let lim = Lim {
            vel: 10.0,
            acc: 10.0,
            jerk: 40.0,
        };

        let mut t = 0.0f32;

        let mut times = Times::default();

        let (total_time, _) = tp(t, q0, q1, v0, v1, &lim, &mut times);

        while t <= total_time {
            let (_, values) = tp(t, q0, q1, v0, v1, &lim, &mut times);

            println!(
                "pos {}, vel {} acc {} jerk {}",
                values.pos, values.vel, values.acc, values.jerk
            );

            t += 0.1;
        }
    }
}
