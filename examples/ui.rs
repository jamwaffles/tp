use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

use gtk::prelude::*;
use plotters::prelude::*;
use plotters::style::full_palette;
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

        let lim = Lim {
            vel: self.lim_vel as f32,
            acc: self.lim_acc as f32,
            jerk: self.lim_jerk as f32,
        };

        let max = lim.vel.max(lim.acc).max(lim.jerk);
        let min = -max;

        let (total_time, _, _) = tp::tp(
            0.0,
            self.q0 as f32,
            self.q1 as f32,
            self.v0 as f32,
            self.v1 as f32,
            &lim,
        );

        let mut chart = ChartBuilder::on(&root)
            // .caption("y=x^2", ("sans-serif", 50).into_font())
            .margin(5)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_cartesian_2d(0.0f32..total_time, (min - 0.2)..(max + 0.2))?;

        chart.configure_mesh().draw()?;

        let pos = LineSeries::new(
            (0..=(total_time * 100.0) as u32).map(|t| {
                let t = (t as f32) / 100.0;

                let (_, out, _) = tp::tp(
                    t,
                    self.q0 as f32,
                    self.q1 as f32,
                    self.v0 as f32,
                    self.v1 as f32,
                    &lim,
                );

                (t, out.pos)
            }),
            &full_palette::DEEPORANGE,
        );

        let vel = LineSeries::new(
            (0..=(total_time * 100.0) as u32).map(|t| {
                let t = (t as f32) / 100.0;

                let (_, out, _) = tp::tp(
                    t,
                    self.q0 as f32,
                    self.q1 as f32,
                    self.v0 as f32,
                    self.v1 as f32,
                    &lim,
                );

                (t, out.vel)
            }),
            &full_palette::GREEN,
        );

        let acc = LineSeries::new(
            (0..=(total_time * 100.0) as u32).map(|t| {
                let t = (t as f32) / 100.0;

                let (_, out, _) = tp::tp(
                    t,
                    self.q0 as f32,
                    self.q1 as f32,
                    self.v0 as f32,
                    self.v1 as f32,
                    &lim,
                );

                (t, out.acc)
            }),
            &full_palette::BLUE,
        );

        let jerk = LineSeries::new(
            (0..=(total_time * 100.0) as u32).map(|t| {
                let t = (t as f32) / 100.0;

                let (_, out, _) = tp::tp(
                    t,
                    self.q0 as f32,
                    self.q1 as f32,
                    self.v0 as f32,
                    self.v1 as f32,
                    &lim,
                );

                (t, out.jerk)
            }),
            &full_palette::BROWN,
        );

        chart.draw_series(pos)?;
        chart.draw_series(vel)?;
        chart.draw_series(acc)?;
        chart.draw_series(jerk)?;
        // .label("y = x^2")
        // .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

        chart
            .configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .draw()?;

        root.present()?;
        Ok(())
    }
}

fn build_ui(app: &gtk::Application) {
    let builder = gtk::Builder::from_string(GLADE_UI_SOURCE);
    let window = builder.object::<gtk::Window>("MainWindow").unwrap();

    window.set_title("TP debugger");

    let drawing_area: gtk::DrawingArea = builder.object("MainDrawingArea").unwrap();

    let q0_scale = builder.object::<gtk::Scale>("Q0Scale").unwrap();
    let q1_scale = builder.object::<gtk::Scale>("Q1Scale").unwrap();
    let v0_scale = builder.object::<gtk::Scale>("V0Scale").unwrap();
    let v1_scale = builder.object::<gtk::Scale>("V1Scale").unwrap();
    let lim_vel_scale = builder.object::<gtk::Scale>("VELScale").unwrap();
    let lim_acc_scale = builder.object::<gtk::Scale>("ACCScale").unwrap();
    let lim_jerk_scale = builder.object::<gtk::Scale>("JERKScale").unwrap();

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
