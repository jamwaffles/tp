#[derive(Debug, Clone, Copy)]
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

pub fn tp(t: f32, q0: f32, q1: f32, v0: f32, v1: f32, lim: &Lim) -> (f32, Out, bool) {
    println!("t {}", t);

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

    let Lim {
        vel: vmax,
        acc: amax,
        jerk: jmax,
    } = lim;

    // Symmetrical profiles for now
    let vmin = -vmax;
    let amin = -amax;
    let jmin = -jmax;

    let max_accel_reached = (vmax - v0) * jmax < amax.powi(2);
    let max_decel_reached = (vmax - v1) * jmax < amax.powi(2);

    // Acceleration time Ta
    let (t_j1, t_a) = if max_accel_reached {
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
    let (t_j2, t_d) = if max_decel_reached {
        // The time that jerk is constant during accel
        let t_j2 = f32::sqrt((vmax - v1) / jmax);
        // Acceleration period
        let t_d = 2.0 * t_j2;

        (t_j2, t_d)
    } else {
        // The time that jerk is constant during accel
        let t_j2 = amax / jmax;
        // Acceleration period
        let t_d = t_j2 + ((vmax - v1) / amax);

        (t_j2, t_d)
    };

    // 3.25 duration of constant velocity
    let t_v = (delta / vmax) - (t_a / 2.0) * (1.0 + v0 / vmax) - (t_d / 2.0) * (1.0 + v1 / vmax);

    // No constant velocity section
    if t_v < 0.0 {
        todo!("Max velocity not reached");
    }

    let total_time = 2.0 * t_j1 + t_a + t_v + 2.0 * t_j2 + t_d;

    // Acceleration reached
    let a_lim_a = jmax * t_j1;
    let a_lim_d = -jmax * t_j2;

    // Velocity reached
    let vlim = v0 + (t_a - t_j1) * a_lim_a;

    // Accel phase, max jerk
    if t < t_j1 {
        println!("--> Accel, max jerk");

        let pos = q0 + (v0 * t) + (jmax * t.powi(3) / 6.0);
        let vel = v0 + jmax * t.powi(2) / 2.0;
        let acc = jmax.powf(t);
        let jerk = jmax;

        (
            total_time,
            Out {
                pos,
                vel,
                acc,
                jerk,
            },
            true,
        )
    }
    // Accel phase, zero jerk
    else if t < (t_a - t_j1) {
        println!("--> Accel, zero jerk");

        let pos =
            q0 + (v0 * t) + (a_lim_a / 6.0) * (3.0 * t.powi(2) - 3.0 * t_j1 * t + t_j1.powi(2));
        let vel = v0 + a_lim_a * (t - t_j1 / 2.0);
        let acc = a_lim_a;
        let jerk = 0.0;

        (
            total_time,
            Out {
                pos,
                vel,
                acc,
                jerk,
            },
            true,
        )
    }
    // Accel phase, min jerk
    else if t < t_a {
        println!("--> Accel, min jerk");

        let pos = q0 + (vlim + v0) * t_a / 2.0 - vlim * (t_a - t) - jmin * (t_a - t).powi(3) / 6.0;
        let vel = vlim + jmin * (t_a - t).powi(2) / 2.0;
        let acc = -jmin * (t_a - t);
        let jerk = -jmin;

        (
            total_time,
            Out {
                pos,
                vel,
                acc,
                jerk,
            },
            true,
        )
    }
    // Coast
    else if t < t_a + t_v {
        println!("--> Coast");

        let pos = q0 + (vlim + v0) * t_a / 2.0 + vlim * (t - t_a);
        let vel = vlim;
        let acc = 0.0;
        let jerk = 0.0;

        (
            total_time,
            Out {
                pos,
                vel,
                acc,
                jerk,
            },
            true,
        )
    }
    // Decel, max jerk
    else if t < total_time - (t_d + t_j2) {
        println!("--> Decel, max jerk");

        let pos = q1 - (vlim + v1) * t_d / 2.0 + vlim * (t - total_time + t_d)
            - jmax * (t - total_time + t_d).powi(3) / 6.0;
        let vel = vlim - jmax * (t - total_time + t_d).powi(2) / 2.0;
        let acc = -jmax * (t - total_time + t_d);
        let jerk = jmin;

        (
            total_time,
            Out {
                pos,
                vel,
                acc,
                jerk,
            },
            true,
        )
    }
    // Decel, zero jerk
    else if t < total_time - t_j2 {
        println!("--> Decel, zero jerk");

        let pos = q1 - (vlim + v1) * t_d / 2.0
            + vlim * (t - total_time + t_d)
            + a_lim_d / 6.0
                * (3.0 * (t - total_time + t_d).powi(2) - 3.0 * t_j2 * (t - total_time + t_d)
                    + t_j2.powi(2));
        let vel = vlim + a_lim_d * (t - total_time + t_d - t_j2 / 2.0);
        let acc = a_lim_d;
        let jerk = 0.0;

        (
            total_time,
            Out {
                pos,
                vel,
                acc,
                jerk,
            },
            true,
        )
    }
    // Decel, min jerk
    else if t <= total_time {
        println!("--> Decel, min jerk");

        let pos = q1 - v1 * (total_time - t) - jmax * (total_time - t).powi(3) / 6.0;
        let vel = v1 + jmax * (total_time - t).powi(2) / 2.0;
        let acc = -jmax * (total_time - t);
        let jerk = jmin;

        (
            total_time,
            Out {
                pos,
                vel,
                acc,
                jerk,
            },
            true,
        )
    }
    // Out of bounds!
    else {
        (total_time, Out::default(), false)
    }
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

        let (total_time, _, _) = tp(t, q0, q1, v0, v1, &lim);

        while t <= total_time {
            let (_, values, _) = tp(t, q0, q1, v0, v1, &lim);

            println!(
                "pos {}, vel {} acc {} jerk {}",
                values.pos, values.vel, values.acc, values.jerk
            );

            t += 0.1;
        }
    }
}
