#[derive(Debug, Clone, Copy)]
struct Lim {
    vel: f32,
    acc: f32,
    jerk: f32,
}

fn tp(t: f32, q0: f32, q1: f32, v0: f32, v1: f32, lim: &Lim) -> (f32, Lim, bool) {
    print!("t {}", t);

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

    // Accel phase, max jerk
    if t < t_j1 {
        println!("--> Accel, max jerk");
    }
    // Accel phase, zero jerk
    else if t < (t_a - t_j1) {
        println!("--> Accel, zero jerk");
    }
    // Accel phase, min jerk
    else if t < t_a {
        println!("--> Accel, min jerk");
    }
    // Coast
    else if t < t_a + t_v {
        println!("--> Coast");
    }
    // Decel, max jerk
    else if t < total_time - (t_d + t_j2) {
        println!("--> Decel, max jerk");
    }
    // Decel, zero jerk
    else if t < total_time - t_j2 {
        println!("--> Decel, zero jerk");
    }
    // Decel, min jerk
    else if t <= total_time {
        println!("--> Decel, min jerk");
    }
    // Out of bounds!
    else {
        return (total_time, lim, false);
    }

    (total_time, lim, true)
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

            println!("vel {} acc {} jerk {}", values.vel, values.acc, values.jerk);

            t += 0.1;
        }
    }
}
