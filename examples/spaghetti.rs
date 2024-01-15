//! Generate a bunch of random points in space, with blends.

use kiss3d::event::WindowEvent;
use kiss3d::window::Window;
use kiss3d::{camera::ArcBall, light::Light};
use nalgebra::{Point3, Translation3, UnitQuaternion, Vector3};
use std::f32::consts::PI;
use tp::arc_blend::{Coord3, Out};
use tp::segments_blends::Trajectory;
use tp::trapezoidal_non_zero_3d::Lim;

struct State {
    trajectory: Trajectory,
    lim: Lim,
}

fn main() {
    let eye = Point3::new(5.0f32, 5.0, 5.0);
    let at = Point3::origin();
    let mut arc_ball = ArcBall::new(eye, at);

    let range = 4.0;

    let lim = Lim {
        acc: Coord3::new(5.0, 5.0, 5.0),
        vel: Coord3::new(2.0, 2.0, 2.0),
    };

    let mut trajectory = Trajectory::new();

    for i in 0..10 {
        trajectory.push_point((Coord3::new_random() * range).map(|axis| axis - 2.0));
    }

    let mut window = Window::new("Spaghetti!");

    let align_z_up = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), PI / 2.0);

    let mut floor = window.add_quad(7.0, 7.0, 1, 1);
    floor.set_color(0.3, 0.3, 0.3);
    floor.append_rotation_wrt_center(&align_z_up);
    floor.append_translation(&Translation3::new(0.0, -range / 2.0 - 0.5, 0.0));

    window.set_light(Light::StickToCamera);

    let state = State { trajectory, lim };

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

        for [a, b] in state
            .trajectory
            .points
            .windows(2)
            .map(|chunk| <&[Coord3; 2]>::try_from(chunk).unwrap())
        {
            let start = Point3::from(*a);
            let end = Point3::from(*b);

            window.draw_line(&start, &end, &Point3::new(1.0, 0.0, 0.0));

            sph(&mut window, start, Point3::new(0.0, 1.0, 0.0));
            sph(&mut window, end, Point3::new(1.0, 0.0, 0.0));
        }

        for blend in state.trajectory.blends.iter() {
            let mut prev_point =
                Point3::new(blend.arc_start.x, blend.arc_start.y, blend.arc_start.z);
            let mut t = 0.0;

            // // Normal of plane passing through the 3 points of the trajectory
            // let colour = {
            //     let a = blend.mid - blend.prev;
            //     let b = blend.next - blend.mid;

            //     let result = a.cross(&b).normalize();

            //     Point3::from(result)
            // };

            let colour = Point3::new(0.0, 1.0, 1.0);

            for t in 0..50u16 {
                let t = f32::from(t) / (50.0 / blend.time);

                let Out {
                    pos,
                    acc: _,
                    vel: _,
                } = blend.tp(t).unwrap();

                let pos_point = Point3::new(pos.x, pos.y, pos.z);

                // let colour = Point3::new(0.0, 1.0, 1.0);

                window.draw_line(&prev_point, &pos_point, &colour);

                prev_point = pos_point;
            }
        }
    }
}

fn sph(window: &mut Window, at: Point3<f32>, colour: Point3<f32>) {
    let mut sphere = window.add_sphere(0.05);
    sphere.set_color(colour.x, colour.y, colour.z);
    sphere.append_translation(&Translation3::from(at));
}
