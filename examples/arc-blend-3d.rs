use kiss3d::window::Window;
use kiss3d::{camera::ArcBall, light::Light};
use nalgebra::{Point3, Translation3, UnitQuaternion, Vector3};
use std::f32::consts::PI;
use tp::arc_blend::ArcBlend;
use tp::trapezoidal_non_zero_3d::Coord3;

fn main() {
    let eye = Point3::new(10.0f32, 10.0, 10.0);
    let at = Point3::origin();
    let mut arc_ball = ArcBall::new(eye, at);

    let p1 = Coord3::new(0.0, 5.0, 0.0);
    let p2 = Coord3::new(0.0, 0.0, 2.0);
    let p3 = Coord3::new(7.0, 0.0, 1.0);

    let blend = ArcBlend::new(p1, p2, p3, 0.5);

    let mut window = Window::new("Arc blend with two segments");

    let align_z_up = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), PI / 2.0);

    let mut floor = window.add_quad(7.0, 7.0, 1, 1);
    floor.set_color(0.2, 0.2, 0.2);
    floor.append_rotation_wrt_center(&align_z_up);
    floor.append_translation(&Translation3::new(0.0, -0.1, 0.0));

    window.set_light(Light::StickToCamera);

    window.set_line_width(5.0);
    window.set_point_size(5.0);

    while window.render_with_camera(&mut arc_ball) {
        window.draw_line(
            &Point3::new(p1.x, p1.y, p1.z),
            &Point3::new(p2.x, p2.y, p2.z),
            &Point3::new(1.0, 0.0, 0.0),
        );

        window.draw_line(
            &Point3::new(p2.x, p2.y, p2.z),
            &Point3::new(p3.x, p3.y, p3.z),
            &Point3::new(0.0, 1.0, 0.0),
        );

        let mut prev_point = Point3::new(blend.prev.x, blend.prev.y, blend.prev.z);

        let mut t = 0.0;

        while t <= blend.time {
            let pos = blend.tp(t).unwrap().pos;

            let pos = Point3::new(pos.x, pos.y, pos.z);

            window.draw_line(&prev_point, &pos, &Point3::new(0.0, 0.0, 1.0));

            prev_point = pos;

            t += 0.1;
        }
    }
}
