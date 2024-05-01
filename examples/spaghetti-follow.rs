//! Generate a bunch of random points and plot the trajectory through them.

use kiss3d::event::WindowEvent;
use kiss3d::window::Window;
use kiss3d::{camera::ArcBall, light::Light};
use nalgebra::{Point3, Translation3, UnitQuaternion, Vector3};
use std::f32::consts::PI;
use tp::arc_blend::Coord3;
use tp::segments_blends::{Item, Trajectory};
use tp::trapezoidal_non_zero_3d::{Lim, Out};

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

    // Generate random points on every run
    for _ in 0..10 {
        trajectory.push_point((Coord3::new_random() * range).map(|axis| axis - (range / 2.0)));
    }

    // // Weird broken test case
    // trajectory.push_point(Coord3::new(0.69154215, -1.7893867, -0.38952398));
    // trajectory.push_point(Coord3::new(0.115730524, 0.83142185, -0.56099606));
    // trajectory.push_point(Coord3::new(0.89620423, 1.502274, -1.7002156));

    let mut window = Window::new("Spaghetti!");

    let align_z_up = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), PI / 2.0);

    let mut floor = window.add_quad(7.0, 7.0, 1, 1);
    floor.set_color(0.3, 0.3, 0.3);
    floor.append_rotation_wrt_center(&align_z_up);
    floor.append_translation(&Translation3::new(0.0, -range / 2.0 - 0.5, 0.0));

    window.set_light(Light::StickToCamera);

    let state = State { trajectory, lim };

    for point in state.trajectory.points.iter() {
        println!("Point {}, {}, {}", point.x, point.y, point.z);
    }

    for item in state.trajectory.items.iter() {
        match item {
            Item::Linear(line) => {
                println!(
                    "Linear   start {}, duration {} from {}, {}, {} -> {}, {}, {}",
                    line.start_t,
                    line.total_time,
                    line.q0.x,
                    line.q0.y,
                    line.q0.z,
                    line.q1.x,
                    line.q1.y,
                    line.q1.z,
                )
            }
            Item::ArcBlend(blend) => {
                println!(
                    "ArcBlend start {}, duration {}, midpoint {}, {}, {}",
                    blend.start_t, blend.time, blend.mid.x, blend.mid.y, blend.mid.z,
                )
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    window.set_line_width(2.0);

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

        let start = state.trajectory.points.first().unwrap();

        sph(
            &mut window,
            Point3::from(*start),
            Point3::new(1.0, 1.0, 1.0),
        );

        // Uncomment to draw lines between desired path points
        // for [a, b] in state
        //     .trajectory
        //     .points
        //     .windows(2)
        //     .map(|chunk| <&[Coord3; 2]>::try_from(chunk).unwrap())
        // {
        //     let start = Point3::from(*a);
        //     let end = Point3::from(*b);

        //     window.draw_line(&start, &end, &Point3::new(1.0, 1.0, 1.0));
        // }

        // let lines = state.trajectory.items.iter().filter_map(|item| match item {
        //     Item::Linear(line) => Some(line),
        //     Item::ArcBlend(_) => None,
        // });

        // Draw straight line segments between blends. Commented out for now as we want to draw the
        // lines using the TP output.
        // for line in lines {
        //     let start = Point3::from(line.q0);
        //     let end = Point3::from(line.q1);

        //     window.draw_line(&start, &end, &Point3::new(1.0, 0.0, 0.0));

        //     sph(&mut window, start, Point3::new(0.0, 1.0, 0.0));
        //     sph(&mut window, end, Point3::new(1.0, 0.0, 0.0));
        // }

        // let blends = state.trajectory.items.iter().filter_map(|item| match item {
        //     Item::Linear(_) => None,
        //     Item::ArcBlend(blend) => Some(blend),
        // });

        // // Draw blend arcs using a bunch of line segments for each one
        // for blend in blends {
        //     let mut prev_point =
        //         Point3::new(blend.arc_start.x, blend.arc_start.y, blend.arc_start.z);

        //     let colour = Point3::new(0.0, 1.0, 1.0);

        //     for t in 0..50u16 {
        //         let t = f32::from(t) / (50.0 / blend.time);

        //         let t = t + blend.start_t;

        //         let Out {
        //             pos,
        //             acc: _,
        //             vel: _,
        //         } = blend.tp(t).unwrap();

        //         let pos_point = Point3::new(pos.x, pos.y, pos.z);

        //         // let colour = Point3::new(0.0, 1.0, 1.0);

        //         window.draw_line(&prev_point, &pos_point, &colour);

        //         prev_point = pos_point;
        //     }
        // }

        // Draw straight trajectory segments using output of planner
        let n_points = 500u16;
        let mut prev_point =
            Point3::from(*state.trajectory.points.first().expect("Empty trajectory"));

        for t in 0..n_points {
            let t = f32::from(t) / (f32::from(n_points) / state.trajectory.total_time);

            let (
                Out {
                    pos,
                    acc: _,
                    vel: _,
                },
                is_arc,
            ) = state.trajectory.tp(t).expect("Out of bounds");

            let pos = Point3::from(pos);

            let colour = if is_arc {
                Point3::new(0.0, 1.0, 1.0)
            } else {
                Point3::new(1.0, 0.0, 1.0)
            };

            window.draw_line(&prev_point, &pos, &colour);

            prev_point = pos;
        }
    }
}

fn sph(window: &mut Window, at: Point3<f32>, colour: Point3<f32>) {
    let mut sphere = window.add_sphere(0.05);
    sphere.set_color(colour.x, colour.y, colour.z);
    sphere.append_translation(&Translation3::from(at));
}
