use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

use gtk::prelude::*;
use plotters::prelude::*;
use plotters::style::full_palette;
use plotters_cairo::CairoBackend;
use tp::{
    arc_blend::{ArcBlend, Coord3},
    trapezoidal_non_zero_3d::Lim,
};

const GLADE_UI_SOURCE: &'static str = include_str!("arc-blend.glade");

#[derive(Debug, Default)]
struct PlottingState {
    deviation_limit: f64,
    accel_limit: f64,
    start_x: f64,
    p1: Coord3,
    p2: Coord3,
    p3: Coord3,
}

impl PlottingState {
    fn plot_pdf<'a, DB: DrawingBackend + 'a>(
        &self,
        backend: DB,
    ) -> Result<(), Box<dyn Error + 'a>> {
        let root = backend.into_drawing_area();

        root.fill(&WHITE)?;

        let margin = 50;

        let Self { p1, p2, p3, .. } = *self;

        let root = root.margin(margin, margin, margin, margin);

        let lim = Lim {
            acc: Coord3::new(
                self.accel_limit as f32,
                self.accel_limit as f32,
                self.accel_limit as f32,
            ),
            vel: Coord3::new(2.0, 2.0, 2.0),
        };

        let blend = ArcBlend::new(p1, p2, p3, self.deviation_limit as f32, 0.0, lim);

        // Chart must be square to get circle in the right position
        let range = p1.y.min(p2.y).min(p3.y).min(p1.x).min(p2.x).min(p3.x)
            ..p1.y.max(p2.y).max(p3.y).max(p1.x).max(p2.x).max(p3.x);

        let mut chart = ChartBuilder::on(&root).build_cartesian_2d(range.clone(), range)?;

        chart.draw_series(LineSeries::new(
            vec![
                (p1.x, p1.y),
                (p2.x, p2.y),
                (p3.x, p3.y),
                // (blend.arc_center.x, blend.arc_center.y),
            ],
            &full_palette::DEEPORANGE,
        ))?;

        let (arc_radius, _) = chart.backend_coord(&(blend.arc_radius, 0.0));
        let arc_radius = arc_radius - margin;

        chart.plotting_area().draw(&Circle::new(
            (p1.x, p1.y),
            3,
            Into::<ShapeStyle>::into(&full_palette::GREY).filled(),
        ))?;
        chart.plotting_area().draw(&Circle::new(
            (p2.x, p2.y),
            3,
            Into::<ShapeStyle>::into(&full_palette::GREY).filled(),
        ))?;
        chart.plotting_area().draw(&Circle::new(
            (p3.x, p3.y),
            3,
            Into::<ShapeStyle>::into(&full_palette::GREY).filled(),
        ))?;

        chart.plotting_area().draw(&Circle::new(
            (blend.arc_center.x, blend.arc_center.y),
            3,
            Into::<ShapeStyle>::into(&full_palette::BLACK).filled(),
        ))?;

        chart.plotting_area().draw(&Circle::new(
            (blend.arc_center.x, blend.arc_center.y),
            arc_radius,
            Into::<ShapeStyle>::into(&full_palette::BLUE),
        ))?;

        // Arc start in green
        chart.plotting_area().draw(&Circle::new(
            (blend.arc_start.x, blend.arc_start.y),
            3,
            Into::<ShapeStyle>::into(&full_palette::GREEN_500),
        ))?;

        // Arc end in red
        chart.plotting_area().draw(&Circle::new(
            (blend.arc_end.x, blend.arc_end.y),
            3,
            Into::<ShapeStyle>::into(&full_palette::RED_500),
        ))?;

        root.present()?;

        Ok(())
    }

    fn plot_chart<'a, DB: DrawingBackend + 'a>(
        &self,
        backend: DB,
    ) -> Result<(), Box<dyn Error + 'a>> {
        let root = backend.into_drawing_area();

        root.fill(&WHITE)?;

        let margin = 50;

        let Self { p1, p2, p3, .. } = *self;

        let root = root.margin(margin, margin, margin, margin);

        let lim = Lim {
            acc: Coord3::new(
                self.accel_limit as f32,
                self.accel_limit as f32,
                self.accel_limit as f32,
            ),
            vel: Coord3::new(2.0, 2.0, 2.0),
        };

        let blend = ArcBlend::new(p1, p2, p3, self.deviation_limit as f32, 0.0, lim);

        let y_range = blend.arc_start.max().max(blend.arc_end.max())
            ..blend.arc_start.min().min(blend.arc_end.min());

        let mut chart = ChartBuilder::on(&root)
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_cartesian_2d(0.0..blend.time, y_range)?;

        chart.configure_mesh().max_light_lines(0).draw()?;

        let points = 500.0f32;
        let total_time = blend.time;

        let pos_iter = (0..=(total_time * points) as u32).map(|t| {
            let t = (t as f32) / points;

            let out = blend.tp(t).unwrap_or_default();

            (t, out)
        });

        // Position

        chart
            .draw_series(LineSeries::new(
                pos_iter.clone().map(|(t, out)| (t, out.pos.x)),
                &full_palette::RED_A100,
            ))?
            .label("Pos X")
            .legend(|(x, y)| Rectangle::new([(x, y + 1), (x + 8, y)], full_palette::RED_A100));

        chart
            .draw_series(LineSeries::new(
                pos_iter.clone().map(|(t, out)| (t, out.pos.y)),
                &full_palette::PINK_A100,
            ))?
            .label("Pos Y")
            .legend(|(x, y)| Rectangle::new([(x, y + 1), (x + 8, y)], full_palette::PINK_A100));

        chart
            .draw_series(LineSeries::new(
                pos_iter.clone().map(|(t, out)| (t, out.pos.z)),
                &full_palette::DEEPPURPLE_A100,
            ))?
            .label("Pos Z")
            .legend(|(x, y)| {
                Rectangle::new([(x, y + 1), (x + 8, y)], full_palette::DEEPPURPLE_A100)
            });

        // Velocity

        // TODO

        // Acceleration

        chart
            .draw_series(LineSeries::new(
                pos_iter.clone().map(|(t, out)| (t, out.acc.x)),
                &full_palette::PURPLE,
            ))?
            .label("Acc X")
            .legend(|(x, y)| Rectangle::new([(x, y + 1), (x + 8, y)], full_palette::PURPLE));

        chart
            .draw_series(LineSeries::new(
                pos_iter.clone().map(|(t, out)| (t, out.acc.y)),
                &full_palette::BLUE,
            ))?
            .label("Acc Y")
            .legend(|(x, y)| Rectangle::new([(x, y + 1), (x + 8, y)], full_palette::BLUE));

        chart
            .draw_series(LineSeries::new(
                pos_iter.clone().map(|(t, out)| (t, out.acc.z)),
                &full_palette::TEAL,
            ))?
            .label("Acc Z")
            .legend(|(x, y)| Rectangle::new([(x, y + 1), (x + 8, y)], full_palette::TEAL));

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

    window.set_title("Circular arc blend debugger");

    let drawing_area: gtk::DrawingArea = builder.object("MainDrawingArea").unwrap();
    let chart_area: gtk::DrawingArea = builder.object("ChartDrawingArea").unwrap();

    let stats = builder.object::<gtk::Label>("Stats").unwrap();
    let deviation_limit_scale = builder.object::<gtk::Scale>("DeviationLimit").unwrap();
    let accel_limit_scale = builder.object::<gtk::Scale>("AccelLimit").unwrap();
    let start_x_scale = builder.object::<gtk::Scale>("StartX").unwrap();

    let app_state = Rc::new(RefCell::new(PlottingState {
        deviation_limit: deviation_limit_scale.value(),
        accel_limit: accel_limit_scale.value(),
        start_x: start_x_scale.value(),
        p1: Coord3::new(start_x_scale.value() as f32, 5.0, 0.0),
        p2: Coord3::new(0.0, 0.0, 0.0),
        p3: Coord3::new(7.0, 0.0, 0.0),
    }));

    window.set_application(Some(app));

    let state_cloned = app_state.clone();
    drawing_area.connect_draw(move |widget, cr| {
        let state = state_cloned.borrow();
        let w = widget.allocated_width() as u32;
        let h = widget.allocated_height() as u32;

        let backend = CairoBackend::new(cr, (w.min(h), w.min(h))).expect("Cairo no");
        state.plot_pdf(backend).expect("Bad plot");
        Inhibit(false)
    });

    let state_cloned = app_state.clone();
    chart_area.connect_draw(move |widget, cr| {
        let state = state_cloned.borrow();
        let w = widget.allocated_width() as u32;
        let h = widget.allocated_height() as u32;

        let backend = CairoBackend::new(cr, (w, h)).expect("Cairo no");
        state.plot_chart(backend).expect("Bad plot");
        Inhibit(false)
    });

    let state_cloned = app_state.clone();
    stats.connect_draw(move |widget, _cr| {
        let state = state_cloned.borrow();

        let lim = Lim {
            acc: Coord3::new(
                state.accel_limit as f32,
                state.accel_limit as f32,
                state.accel_limit as f32,
            ),
            vel: Coord3::new(2.0, 2.0, 2.0),
        };

        let blend = ArcBlend::new(
            state.p1,
            state.p2,
            state.p3,
            state.deviation_limit as f32,
            0.0,
            lim,
        );

        widget.set_text(&format!(
            "Deviation limit {}, accel limit {}, velocity limit {}",
            state.deviation_limit, state.accel_limit, blend.velocity_limit
        ));

        Inhibit(false)
    });

    let handle_change =
        |what: &gtk::Scale, how: Box<dyn Fn(&mut PlottingState) -> &mut f64 + 'static>| {
            let app_state = app_state.clone();
            let drawing_area = drawing_area.clone();
            let chart_area = chart_area.clone();
            let stats = stats.clone();
            what.connect_value_changed(move |target| {
                let mut state = app_state.borrow_mut();
                *how(&mut *state) = target.value();

                state.p1.x = state.start_x as f32;

                drawing_area.queue_draw();
                chart_area.queue_draw();
                stats.queue_draw();
            });
        };

    let _handle_bool_change =
        |what: &gtk::ToggleButton, how: Box<dyn Fn(&mut PlottingState) -> &mut bool + 'static>| {
            let app_state = app_state.clone();
            let drawing_area = drawing_area.clone();
            let chart_area = chart_area.clone();
            let stats = stats.clone();
            what.connect_toggled(move |target| {
                let mut state = app_state.borrow_mut();
                *how(&mut *state) = target.is_active();

                state.p1.x = state.start_x as f32;

                drawing_area.queue_draw();
                chart_area.queue_draw();
                stats.queue_draw();
            });
        };

    handle_change(&deviation_limit_scale, Box::new(|s| &mut s.deviation_limit));
    handle_change(&start_x_scale, Box::new(|s| &mut s.start_x));
    handle_change(&accel_limit_scale, Box::new(|s| &mut s.accel_limit));

    window.show_all();
}

fn main() {
    let application = gtk::Application::new(Some("io.tp-multi-debugger"), Default::default());

    application.connect_activate(|app| {
        build_ui(app);
    });

    application.run();
}
