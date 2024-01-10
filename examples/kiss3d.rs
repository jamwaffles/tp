use kiss3d::ncollide3d;
use kiss3d::renderer::{LineRenderer, Renderer};
use kiss3d::window::Window;
use kiss3d::{camera::ArcBall, light::Light};
use nalgebra::{Point3, Translation3, UnitQuaternion, Vector3};
use std::f32::consts::PI;

fn main() {
    let eye = Point3::new(2.0f32, 2.0, 2.0);
    let at = Point3::origin();
    let mut arc_ball = ArcBall::new(eye, at);

    let mut window = Window::new("Kiss3d arc blend");

    let align_z_up = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), PI / 2.0);

    // let mut c = window.add_cube(1.0, 1.0, 1.0);
    // c.set_color(1.0, 0.0, 0.0);

    let mut floor = window.add_quad(7.0, 7.0, 1, 1);
    floor.set_color(0.2, 0.2, 0.2);
    floor.append_rotation_wrt_center(&align_z_up);

    let circle = ncollide3d::procedural::cylinder(0.4f32, 0.4f32, 32);
    let mut c = window.add_trimesh(circle, Vector3::from_element(1.0));
    c.set_points_size(10.0);
    c.set_lines_width(1.0);
    c.set_surface_rendering_activation(false);
    c.append_translation(&Translation3::new(-1.0, 0.0, 0.0));
    c.set_color(0.0, 0.0, 1.0);

    window.set_light(Light::StickToCamera);

    window.set_line_width(5.0);

    while window.render_with_camera(&mut arc_ball) {
        // while window.render() {
        // c.prepend_to_local_rotation(&rot);

        window.draw_line(
            &Point3::new(0.0, 0.0, 0.0),
            &Point3::new(1.0, 1.0, 1.0),
            &Point3::new(1.0, 0.0, 0.0),
        );

        // window.render(0, &mut arc_ball);
    }
}
