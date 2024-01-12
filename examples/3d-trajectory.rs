use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

use gtk::{gdk::EventMask, prelude::*};
use plotters::prelude::*;
use plotters::style::full_palette;
use plotters_cairo::CairoBackend;
use tp::trapezoidal_non_zero_3d::{Coord3, Lim, Segment};

const GLADE_UI_SOURCE: &'static str = include_str!("3d-trajectory.glade");

struct PlottingState {
    show_pos: bool,
    show_vel: bool,
    show_acc: bool,
    show_jerk: bool,
}

impl PlottingState {
    fn plot_pdf<'a, DB: DrawingBackend + 'a>(
        &self,
        backend: DB,
    ) -> Result<(), Box<dyn Error + 'a>> {
        let root = backend.into_drawing_area();

        root.fill(&WHITE)?;

        let q0 = Coord3::new(0.0, 0.0, 0.0);
        let q1 = Coord3::new(10.0, 0.0, 0.0);
        let v0 = Coord3::new(0.0, 0.0, 0.0);
        let v1 = Coord3::new(0.0, 0.0, 0.0);

        let lim = Lim {
            vel: Coord3::new(1.0, 1.0, 1.0),
            acc: Coord3::new(5.0, 5.0, 5.0),
        };

        let seg = Segment::new(q0, q1, v0, v1, &lim);

        let max = lim
            .vel
            .max()
            .max(lim.acc.max())
            .max(q0.max().abs() as f32)
            .max(q1.max().abs() as f32);
        let min = lim
            .vel
            .min()
            .min(lim.acc.min())
            .min(q0.min().abs() as f32)
            .min(q1.min().abs() as f32);

        let min = -min.max(max);

        let total_time = seg.total_time;

        let mut chart = ChartBuilder::on(&root)
            // .caption("y=x^2", ("sans-serif", 50).into_font())
            .margin(5)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_cartesian_2d(0.0f32..total_time, (min - 0.2)..(max + 0.2))?;

        chart.configure_mesh().max_light_lines(0).draw()?;

        // Number of X samples
        let points = 500.0;

        let pos = LineSeries::new(
            (0..=(total_time * points) as u32).map(|t| {
                let t = (t as f32) / points;

                let out = seg.tp(t).unwrap_or_default();

                (t, out.pos.x)
            }),
            &full_palette::DEEPORANGE,
        );

        let vel = LineSeries::new(
            (0..=(total_time * points) as u32).map(|t| {
                let t = (t as f32) / points;

                let out = seg.tp(t).unwrap_or_default();

                (t, out.vel.x)
            }),
            &full_palette::GREEN,
        );

        let acc = LineSeries::new(
            (0..=(total_time * points) as u32).map(|t| {
                let t = (t as f32) / points;

                let out = seg.tp(t).unwrap_or_default();

                (t, out.acc.x)
            }),
            &full_palette::BLUE,
        );

        if self.show_pos {
            chart.draw_series(pos)?.label("Pos").legend(|(x, y)| {
                Rectangle::new([(x, y + 1), (x + 8, y)], full_palette::DEEPORANGE)
            });
        }

        if self.show_vel {
            chart
                .draw_series(vel)?
                .label("Vel")
                .legend(|(x, y)| Rectangle::new([(x, y + 1), (x + 8, y)], full_palette::GREEN));
        }

        if self.show_acc {
            chart
                .draw_series(acc)?
                .label("Acc")
                .legend(|(x, y)| Rectangle::new([(x, y + 1), (x + 8, y)], full_palette::BLUE));
        }

        chart
            .configure_series_labels()
            .position(SeriesLabelPosition::UpperRight)
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
    window.set_events(window.events() | EventMask::POINTER_MOTION_MASK);

    let drawing_area: gtk::DrawingArea = builder.object("MainDrawingArea").unwrap();

    let show_pos = builder.object::<gtk::ToggleButton>("POSShow").unwrap();
    let show_vel = builder.object::<gtk::ToggleButton>("VELShow").unwrap();
    let show_acc = builder.object::<gtk::ToggleButton>("ACCShow").unwrap();
    let show_jerk = builder.object::<gtk::ToggleButton>("JERKShow").unwrap();
    let times = builder.object::<gtk::Label>("Times").unwrap();

    let drawing_area: gtk::DrawingArea = builder.object("MainDrawingArea").unwrap();
    let app_state = Rc::new(RefCell::new(PlottingState {
        show_pos: show_pos.is_active(),
        show_vel: show_vel.is_active(),
        show_acc: show_acc.is_active(),
        show_jerk: show_jerk.is_active(),
    }));

    window.set_application(Some(app));

    let state_cloned = app_state.clone();
    drawing_area.connect_draw(move |widget, cr| {
        let state = state_cloned.borrow();
        let w = widget.allocated_width();
        let h = widget.allocated_height();
        let backend = CairoBackend::new(cr, (w as u32, h as u32)).unwrap();
        state.plot_pdf(backend).unwrap();
        Inhibit(false)
    });

    // let state_cloned = app_state.clone();
    drawing_area.set_events(drawing_area.events() | EventMask::POINTER_MOTION_MASK);
    drawing_area.connect_motion_notify_event(move |_widget, _cr| {
        // TODO: Find a way to get value from chart. This method is currently a noop but it was a
        // bit challenging to get it working so I'll leave it in.

        Inhibit(false)
    });

    let state_cloned = app_state.clone();
    times.connect_draw(move |widget, _cr| {
        // let app_state = state_cloned.borrow();

        // let times = app_state.seg.times();

        // widget.set_text(&format!(
        //     "Total {:>5}, t_j1 {:>5}, t_a {:>5}, t_v {:>5}, t_j2 {:>5}, t_d {:>5}",
        //     times.total_time, times.t_j1, times.t_a, times.t_v, times.t_j2, times.t_d
        // ));

        Inhibit(false)
    });

    let handle_bool_change =
        |what: &gtk::ToggleButton, how: Box<dyn Fn(&mut PlottingState) -> &mut bool + 'static>| {
            let app_state = app_state.clone();
            let drawing_area = drawing_area.clone();
            let times = times.clone();
            what.connect_toggled(move |target| {
                let mut state = app_state.borrow_mut();
                *how(&mut *state) = target.is_active();
                drawing_area.queue_draw();
                times.queue_draw();
            });
        };

    handle_bool_change(&show_pos, Box::new(|s| &mut s.show_pos));
    handle_bool_change(&show_vel, Box::new(|s| &mut s.show_vel));
    handle_bool_change(&show_acc, Box::new(|s| &mut s.show_acc));
    handle_bool_change(&show_jerk, Box::new(|s| &mut s.show_jerk));

    window.show_all();
}

fn main() {
    let application = gtk::Application::new(Some("io.tp-debugger"), Default::default());

    application.connect_activate(|app| {
        build_ui(app);
    });

    application.run();
}
