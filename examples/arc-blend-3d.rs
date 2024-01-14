use kiss3d::event::{Action, Key, WindowEvent};
use kiss3d::ncollide3d;
use kiss3d::ncollide3d::math::Translation;
use kiss3d::scene::SceneNode;
use kiss3d::window::Window;
use kiss3d::{camera::ArcBall, light::Light};
use nalgebra::{Point3, Translation3, UnitQuaternion, Vector3};
use std::f32::consts::PI;
use tp::arc_blend::{ArcBlend, Out};
use tp::trapezoidal_non_zero_3d::{Coord3, Lim};

struct State {
    p1: Coord3,
    p2: Coord3,
    p3: Coord3,
    lim: Lim,
    blend: ArcBlend,
    arc_center: SceneNode,
    arc_end: SceneNode,
    arc_start: SceneNode,
    ball: SceneNode,

    draw_vel_acc: bool,
}

impl State {
    fn set_p2(&mut self, new_random: Coord3) {
        self.p2 = new_random;

        self.blend = ArcBlend::new(self.p1, self.p2, self.p3, 0.5, self.lim);

        self.arc_center
            .set_local_translation(Translation::from(self.blend.arc_center));
        self.arc_start
            .set_local_translation(Translation::from(self.blend.arc_start));
        self.arc_end
            .set_local_translation(Translation::from(self.blend.arc_end));

        // I can't get the translation to work properly so I'll just hide the ball for now
        self.ball.set_visible(false);

        // self.ball
        //     .set_local_translation(Translation::from(self.blend.arc_center));
        // self.ball.set_local_scale(
        //     self.blend.arc_radius,
        //     self.blend.arc_radius,
        //     self.blend.arc_radius,
        // );
    }

    fn toggle_start_end(&mut self) {
        self.arc_end.set_visible(!self.arc_end.is_visible());
        self.arc_start.set_visible(!self.arc_start.is_visible());
    }

    fn toggle_vel_acc(&mut self) {
        self.draw_vel_acc = !self.draw_vel_acc
    }
}

fn main() {
    let eye = Point3::new(5.0f32, 5.0, 5.0);
    let at = Point3::origin();
    let mut arc_ball = ArcBall::new(eye, at);

    let p1 = Coord3::new(0.0, 5.0, 0.0);
    let p2 = Coord3::new(0.0, 0.0, 2.0);
    let p3 = Coord3::new(7.0, 0.0, 1.0);

    let lim = Lim {
        acc: Coord3::new(5.0, 5.0, 5.0),
        vel: Coord3::new(2.0, 2.0, 2.0),
    };

    let blend = ArcBlend::new(p1, p2, p3, 0.5, lim);

    let mut window = Window::new("Arc blend with two segments");

    let align_z_up = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), PI / 2.0);

    let mut floor = window.add_quad(7.0, 7.0, 1, 1);
    floor.set_color(0.3, 0.3, 0.3);
    floor.append_rotation_wrt_center(&align_z_up);
    floor.append_translation(&Translation3::new(0.0, -1.0, 0.0));

    window.set_light(Light::StickToCamera);

    // window.set_line_width(5.0);
    // window.set_point_size(5.0);

    let ball = ncollide3d::procedural::sphere(blend.arc_radius * 2.0, 20, 20, false);
    let mut ball = window.add_trimesh(ball, Vector3::from_element(1.0));
    ball.set_color(0.7, 0.7, 0.7);
    ball.set_lines_width(1.0);
    ball.set_surface_rendering_activation(false);
    ball.append_translation(&Translation3::new(
        blend.arc_center.x,
        blend.arc_center.y,
        blend.arc_center.z,
    ));

    let mut arc_center = window.add_sphere(0.1);
    arc_center.set_color(1.0, 1.0, 1.0);
    arc_center.append_translation(&Translation3::new(
        blend.arc_center.x,
        blend.arc_center.y,
        blend.arc_center.z,
    ));

    let mut arc_start = window.add_sphere(0.1);
    arc_start.set_color(0.0, 1.0, 0.0);
    arc_start.append_translation(&Translation3::new(
        blend.arc_start.x,
        blend.arc_start.y,
        blend.arc_start.z,
    ));

    let mut arc_end = window.add_sphere(0.1);
    arc_end.set_color(1.0, 0.0, 0.0);
    arc_end.append_translation(&Translation3::new(
        blend.arc_end.x,
        blend.arc_end.y,
        blend.arc_end.z,
    ));

    let mut state = State {
        p1,
        p2,
        p3,
        blend,
        lim,
        ball,
        arc_start,
        arc_end,
        arc_center,
        draw_vel_acc: true,
    };

    while window.render_with_camera(&mut arc_ball) {
        for event in window.events().iter() {
            match event.value {
                WindowEvent::Key(key, action, _modif) => match (key, action) {
                    (Key::R, Action::Press) => {
                        let p = Coord3::new_random() * 3.0f32;

                        println!("New point comin up: {}", p);

                        state.set_p2(p);

                        state
                            .ball
                            .set_local_translation(Translation3::new(0.0, 1.0, 0.0));
                    }
                    (Key::B, Action::Press) => {
                        println!("Toggle balls");

                        state.toggle_start_end();
                    }
                    (Key::V, Action::Press) => {
                        println!("Toggle acceleration/velocity vectors");

                        state.toggle_vel_acc();
                    }
                    _ => (),
                },
                _ => {}
            }
        }

        window.draw_line(
            &Point3::new(state.p1.x, state.p1.y, state.p1.z),
            &Point3::new(state.p2.x, state.p2.y, state.p2.z),
            &Point3::new(1.0, 0.0, 0.0),
        );

        window.draw_line(
            &Point3::new(state.p2.x, state.p2.y, state.p2.z),
            &Point3::new(state.p3.x, state.p3.y, state.p3.z),
            &Point3::new(0.0, 1.0, 0.0),
        );

        // Normal of plane passing through the 3 points of the trajectory
        {
            let a = state.blend.mid - state.blend.prev;
            let b = state.blend.next - state.blend.mid;

            let result = state.p2 + a.cross(&b).normalize();

            window.draw_line(
                &Point3::new(state.p2.x, state.p2.y, state.p2.z),
                &Point3::new(result.x, result.y, result.z),
                &Point3::new(1.0, 1.0, 1.0),
            );
        }

        let mut prev_point = Point3::new(
            state.blend.arc_start.x,
            state.blend.arc_start.y,
            state.blend.arc_start.z,
        );
        let mut t = 0.0;

        while t <= state.blend.time {
            let Out { pos, acc, vel } = state.blend.tp(t).unwrap();

            let pos_point = Point3::new(pos.x, pos.y, pos.z);

            window.draw_line(&prev_point, &pos_point, &Point3::new(0.0, 1.0, 1.0));

            if state.draw_vel_acc {
                // Velocity vector
                {
                    let vector = pos + vel * 0.3;
                    let end_point = Point3::new(vector.x, vector.y, vector.z);

                    // Cyan
                    window.draw_line(&pos_point, &end_point, &Point3::from(vel.normalize()));
                }

                // Acceleration vector
                {
                    let vector = pos + acc * 0.3;
                    let end_point = Point3::new(vector.x, vector.y, vector.z);

                    // Magenta
                    window.draw_line(&pos_point, &end_point, &Point3::from(acc.normalize()));
                }
            }

            prev_point = pos_point;

            t += 0.1;
        }
    }
}
