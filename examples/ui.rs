use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

use gtk::prelude::*;
use plotters::prelude::*;
use plotters_cairo::CairoBackend;
use tp::Lim;

const GLADE_UI_SOURCE: &'static str = include_str!("ui.glade");

#[derive(Clone, Copy)]
struct PlottingState {
    q0: f64,
    q1: f64,
    v0: f64,
    v1: f64,
    lim_vel: f64,
    lim_acc: f64,
    lim_jerk: f64,
}

impl PlottingState {
    fn plot_pdf<'a, DB: DrawingBackend + 'a>(
        &self,
        backend: DB,
    ) -> Result<(), Box<dyn Error + 'a>> {
        let root = backend.into_drawing_area();

        root.fill(&WHITE)?;

        // TODO

        // let mut chart = ChartBuilder::on(&root).build_cartesian_3d(
        //     -10.0f64..10.0,
        //     0.0f64..1.2,
        //     -10.0f64..10.0,
        // )?;

        // chart.with_projection(|mut p| {
        //     p.pitch = self.pitch;
        //     p.yaw = self.roll;
        //     p.scale = 0.7;
        //     p.into_matrix() // build the projection matrix
        // });

        // chart
        //     .configure_axes()
        //     .light_grid_style(BLACK.mix(0.15))
        //     .max_light_lines(3)
        //     .draw()?;
        // let self_cloned = self.clone();
        // chart.draw_series(
        //     SurfaceSeries::xoz(
        //         (-50..=50).map(|x| x as f64 / 5.0),
        //         (-50..=50).map(|x| x as f64 / 5.0),
        //         move |x, y| self_cloned.guassian_pdf(x, y),
        //     )
        //     .style_func(&|&v| (&HSLColor(240.0 / 360.0 - 240.0 / 360.0 * v, 1.0, 0.7)).into()),
        // )?;

        root.present()?;
        Ok(())
    }
}

fn build_ui(app: &gtk::Application) {
    let builder = gtk::Builder::from_string(GLADE_UI_SOURCE);
    let window = builder.object::<gtk::Window>("MainWindow").unwrap();

    window.set_title("Gaussian PDF Plotter");

    let drawing_area: gtk::DrawingArea = builder.object("MainDrawingArea").unwrap();

    let q0_scale = builder.object::<gtk::Scale>("Q0Scale").unwrap();
    let q1_scale = builder.object::<gtk::Scale>("Q1Scale").unwrap();
    let v0_scale = builder.object::<gtk::Scale>("V0Scale").unwrap();
    let v1_scale = builder.object::<gtk::Scale>("V1Scale").unwrap();
    let lim_vel_scale = builder.object::<gtk::Scale>("VELscale").unwrap();
    let lim_acc_scale = builder.object::<gtk::Scale>("ACCscale").unwrap();
    let lim_jerk_scale = builder.object::<gtk::Scale>("JERKscale").unwrap();

    let app_state = Rc::new(RefCell::new(PlottingState {
        q0: q0_scale.value(),
        q1: q1_scale.value(),
        v0: v0_scale.value(),
        v1: v1_scale.value(),
        lim_vel: lim_vel_scale.value(),
        lim_acc: lim_acc_scale.value(),
        lim_jerk: lim_jerk_scale.value(),
    }));

    window.set_application(Some(app));

    let state_cloned = app_state.clone();
    drawing_area.connect_draw(move |widget, cr| {
        let state = state_cloned.borrow().clone();
        let w = widget.allocated_width();
        let h = widget.allocated_height();
        let backend = CairoBackend::new(cr, (w as u32, h as u32)).unwrap();
        state.plot_pdf(backend).unwrap();
        Inhibit(false)
    });

    let handle_change =
        |what: &gtk::Scale, how: Box<dyn Fn(&mut PlottingState) -> &mut f64 + 'static>| {
            let app_state = app_state.clone();
            let drawing_area = drawing_area.clone();
            what.connect_value_changed(move |target| {
                let mut state = app_state.borrow_mut();
                *how(&mut *state) = target.value();
                drawing_area.queue_draw();
            });
        };

    handle_change(&q0_scale, Box::new(|s| &mut s.q0));
    handle_change(&q1_scale, Box::new(|s| &mut s.q1));
    handle_change(&v0_scale, Box::new(|s| &mut s.v0));
    handle_change(&v1_scale, Box::new(|s| &mut s.v1));
    handle_change(&lim_vel_scale, Box::new(|s| &mut s.lim_vel));
    handle_change(&lim_acc_scale, Box::new(|s| &mut s.lim_acc));
    handle_change(&lim_jerk_scale, Box::new(|s| &mut s.lim_jerk));

    window.show_all();
}

fn main() {
    let application = gtk::Application::new(Some("io.tp-debugger"), Default::default());

    application.connect_activate(|app| {
        build_ui(app);
    });

    application.run();
}
