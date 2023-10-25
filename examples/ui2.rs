use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

use gtk::{gdk::EventMask, prelude::*};
use plotters::prelude::*;
use plotters::style::full_palette;
use plotters_cairo::CairoBackend;
use tp::trapezoidal_non_zero::{Lim, Segment, Times};

const GLADE_UI_SOURCE: &'static str = include_str!("ui.glade");

struct PlottingState {
    q0: f64,
    q1: f64,
    v0: f64,
    v1: f64,
    lim_vel: f64,
    lim_acc: f64,
    lim_jerk: f64,
    show_pos: bool,
    show_vel: bool,
    show_acc: bool,
    show_jerk: bool,
    seg: Segment,
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

        let max = lim
            .vel
            .max(lim.acc)
            .max(lim.jerk)
            .max(self.q0.abs() as f32)
            .max(self.q1.abs() as f32);
        let min = lim
            .vel
            .min(lim.acc)
            .min(lim.jerk)
            .min(self.q0.abs() as f32)
            .min(self.q1.abs() as f32);

        let min = -min.max(max);

        let Times { total_time, .. } = self.seg.times();

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

                let out = self.seg.tp(t).unwrap_or_default();

                (t, out.pos)
            }),
            &full_palette::DEEPORANGE,
        );

        let vel = LineSeries::new(
            (0..=(total_time * points) as u32).map(|t| {
                let t = (t as f32) / points;

                let out = self.seg.tp(t).unwrap_or_default();

                (t, out.vel)
            }),
            &full_palette::GREEN,
        );

        let acc = LineSeries::new(
            (0..=(total_time * points) as u32).map(|t| {
                let t = (t as f32) / points;

                let out = self.seg.tp(t).unwrap_or_default();

                (t, out.acc)
            }),
            &full_palette::BLUE,
        );

        let jerk = LineSeries::new(
            (0..=(total_time * points) as u32).map(|t| {
                let t = (t as f32) / points;

                let out = self.seg.tp(t).unwrap_or_default();

                (t, out.jerk)
            }),
            &full_palette::BROWN,
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

        if self.show_jerk {
            chart
                .draw_series(jerk)?
                .label("Jerk")
                .legend(|(x, y)| Rectangle::new([(x, y + 1), (x + 8, y)], full_palette::BROWN));
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

    let q0_scale = builder.object::<gtk::Scale>("Q0Scale").unwrap();
    let q1_scale = builder.object::<gtk::Scale>("Q1Scale").unwrap();
    let v0_scale = builder.object::<gtk::Scale>("V0Scale").unwrap();
    let v1_scale = builder.object::<gtk::Scale>("V1Scale").unwrap();
    let lim_vel_scale = builder.object::<gtk::Scale>("VELScale").unwrap();
    let lim_acc_scale = builder.object::<gtk::Scale>("ACCScale").unwrap();
    let lim_jerk_scale = builder.object::<gtk::Scale>("JERKScale").unwrap();

    let show_pos = builder.object::<gtk::ToggleButton>("POSShow").unwrap();
    let show_vel = builder.object::<gtk::ToggleButton>("VELShow").unwrap();
    let show_acc = builder.object::<gtk::ToggleButton>("ACCShow").unwrap();
    let show_jerk = builder.object::<gtk::ToggleButton>("JERKShow").unwrap();
    let times = builder.object::<gtk::Label>("Times").unwrap();

    let app_state = Rc::new(RefCell::new(PlottingState {
        q0: q0_scale.value(),
        q1: q1_scale.value(),
        v0: v0_scale.value(),
        v1: v1_scale.value(),
        lim_vel: lim_vel_scale.value(),
        lim_acc: lim_acc_scale.value(),
        lim_jerk: lim_jerk_scale.value(),
        show_pos: show_pos.is_active(),
        show_vel: show_vel.is_active(),
        show_acc: show_acc.is_active(),
        show_jerk: show_jerk.is_active(),
        seg: Segment::new(
            q0_scale.value() as f32,
            q1_scale.value() as f32,
            v0_scale.value() as f32,
            v1_scale.value() as f32,
            &Lim {
                vel: lim_vel_scale.value() as f32,
                acc: lim_acc_scale.value() as f32,
                jerk: lim_jerk_scale.value() as f32,
            },
        ),
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
        let app_state = state_cloned.borrow();

        let times = app_state.seg.times();

        widget.set_text(&format!(
            "Total {:>5}, t_j1 {:>5}, t_a {:>5}, t_v {:>5}, t_j2 {:>5}, t_d {:>5}",
            times.total_time, times.t_j1, times.t_a, times.t_v, times.t_j2, times.t_d
        ));

        Inhibit(false)
    });

    let handle_change =
        |what: &gtk::Scale, how: Box<dyn Fn(&mut PlottingState) -> &mut f64 + 'static>| {
            let app_state = app_state.clone();
            let drawing_area = drawing_area.clone();
            let times = times.clone();
            what.connect_value_changed(move |target| {
                let mut state = app_state.borrow_mut();
                *how(&mut *state) = target.value();

                state.seg = Segment::new(
                    state.q0 as f32,
                    state.q1 as f32,
                    state.v0 as f32,
                    state.v1 as f32,
                    &Lim {
                        vel: state.lim_vel as f32,
                        acc: state.lim_acc as f32,
                        jerk: state.lim_jerk as f32,
                    },
                );

                drawing_area.queue_draw();
                times.queue_draw();
            });

            // Reset to 0 on double click
            what.connect_button_press_event(move |target, event| {
                if event.button() == 1 && event.click_count() == Some(2) {
                    target.set_value(0.0);
                }

                Inhibit(false)
            })
        };

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

    handle_change(&q0_scale, Box::new(|s| &mut s.q0));
    handle_change(&q1_scale, Box::new(|s| &mut s.q1));
    handle_change(&v0_scale, Box::new(|s| &mut s.v0));
    handle_change(&v1_scale, Box::new(|s| &mut s.v1));
    handle_change(&lim_vel_scale, Box::new(|s| &mut s.lim_vel));
    handle_change(&lim_acc_scale, Box::new(|s| &mut s.lim_acc));
    handle_change(&lim_jerk_scale, Box::new(|s| &mut s.lim_jerk));
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
