//! Show the computed trajectory along a single straight line segment.
//!
//! Designed to test multi-axis synchronisation.

use kiss3d::event::WindowEvent;
use kiss3d::window::Window;
use kiss3d::{camera::ArcBall, light::Light};
use nalgebra::{Point3, Translation3, UnitQuaternion, Vector3};
use std::f32::consts::PI;
use tp::trapezoidal_non_zero_3d::{Coord3, Lim, Out, Phase, Segment};

struct State {
    p1: Coord3,
    p2: Coord3,
    lim: Lim,
    seg: Segment,
}

fn main() {
    let eye = Point3::new(8.0f32, 8.0, 8.0);
    let at = Point3::origin();
    let mut arc_ball = ArcBall::new(eye, at);

    let p1 = Coord3::new(2.0, 0.0, 0.0);
    let p2 = Coord3::new(0.0, 4.0, 2.0);

    let lim = Lim {
        acc: Coord3::new(5.0, 5.0, 5.0),
        vel: Coord3::new(2.0, 2.0, 2.0),
    };

    let seg = Segment::new(p1, p2, Coord3::zeros(), Coord3::zeros(), 0.0, &lim);

    let mut window = Window::new("Following a straight line");

    let align_z_up = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), PI / 2.0);

    let mut floor = window.add_quad(7.0, 7.0, 1, 1);
    floor.set_color(0.3, 0.3, 0.3);
    floor.append_rotation_wrt_center(&align_z_up);
    floor.append_translation(&Translation3::new(0.0, -1.0, 0.0));

    window.set_light(Light::StickToCamera);

    window.set_line_width(2.0);

    let mut state = State { p1, p2, lim, seg };

    while window.render_with_camera(&mut arc_ball) {
        for event in window.events().iter() {
            match event.value {
                WindowEvent::Key(key, action, _modif) => match (key, action) {
                    // (Key::R, Action::Press) => {
                    //     let p = Coord3::new_random() * 3.0f32;

                    //     println!("New point comin up: {}", p);

                    //     state.set_p2(p);

                    //     state
                    //         .ball
                    //         .set_local_translation(Translation3::new(0.0, 1.0, 0.0));
                    // }
                    // (Key::B, Action::Press) => {
                    //     println!("Toggle balls");

                    //     state.toggle_start_end();
                    // }
                    // (Key::V, Action::Press) => {
                    //     println!("Toggle acceleration/velocity vectors");

                    //     state.toggle_vel_acc();
                    // }
                    _ => (),
                },
                _ => {}
            }
        }

        window.draw_line(
            &Point3::from(state.p1),
            &Point3::from(state.p2),
            &Point3::new(1.0, 1.0, 1.0),
        );

        let mut prev_point = Point3::from(state.p1);

        let points = 100u16;

        for t in 0..100u16 {
            let t = f32::from(t) / (f32::from(points) / state.seg.total_time);

            let (Out { pos, acc, vel }, phase) = state.seg.tp(t).unwrap();

            let pos_point = Point3::new(pos.x, pos.y, pos.z);

            let a = acc.norm();

            let colour = match phase {
                Phase::Accel => Point3::new(1.0, 0.0, 0.0),
                Phase::Cruise => Point3::new(0.0, 1.0, 0.0),
                Phase::Decel => Point3::new(0.0, 0.0, 1.0),
            };

            window.draw_line(&prev_point, &pos_point, &colour);

            prev_point = pos_point;
        }
    }
}
