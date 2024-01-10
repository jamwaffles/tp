use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

use gtk::prelude::*;
use plotters::prelude::*;
use plotters::style::full_palette;
use plotters_cairo::CairoBackend;
use tp::arc_blend::{ArcBlend, Coord2};

const GLADE_UI_SOURCE: &'static str = include_str!("arc-blend.glade");

#[derive(Debug, Default)]
struct PlottingState {
    deviation_limit: f64,
}

impl PlottingState {
    fn plot_pdf<'a, DB: DrawingBackend + 'a>(
        &self,
        backend: DB,
    ) -> Result<(), Box<dyn Error + 'a>> {
        let root = backend.into_drawing_area();

        root.fill(&WHITE)?;

        let margin = 50;

        let root = root.margin(margin, margin, margin, margin);

        // let p1 = Coord2::new(0.5, 0.5);
        // let p2 = Coord2::new(0.8, 0.8);
        // let p3 = Coord2::new(1.2, 0.6);

        // Right angle
        let p1 = Coord2::new(0.0, 10.0);
        let p2 = Coord2::new(0.0, 0.0);
        let p3 = Coord2::new(10.0, 0.0);

        let blend = ArcBlend::new(p1, p2, p3, self.deviation_limit as f32);

        let x_range = p1.x.min(p2.x).min(p3.x)..p1.x.max(p2.x).max(p3.x);
        let y_range = p1.y.min(p2.y).min(p3.y)..p1.y.max(p2.y).max(p3.y);

        let mut chart = ChartBuilder::on(&root).build_cartesian_2d(x_range, y_range)?;

        chart.draw_series(LineSeries::new(
            vec![
                (p1.x, p1.y),
                (p2.x, p2.y),
                (p3.x, p3.y),
                (blend.arc_center.x, blend.arc_center.y),
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

        root.present()?;

        Ok(())
    }
}

fn build_ui(app: &gtk::Application) {
    let builder = gtk::Builder::from_string(GLADE_UI_SOURCE);
    let window = builder.object::<gtk::Window>("MainWindow").unwrap();

    window.set_title("Circular arc blend debugger");

    let drawing_area: gtk::DrawingArea = builder.object("MainDrawingArea").unwrap();

    let stats = builder.object::<gtk::Label>("Stats").unwrap();
    let deviation_limit_scale = builder.object::<gtk::Scale>("DeviationLimit").unwrap();

    let app_state = Rc::new(RefCell::new(PlottingState {
        deviation_limit: deviation_limit_scale.value(),
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
    stats.connect_draw(move |widget, _cr| {
        let state = state_cloned.borrow();

        widget.set_text(&format!("Deviation limit {:}", state.deviation_limit));

        Inhibit(false)
    });

    let handle_change =
        |what: &gtk::Scale, how: Box<dyn Fn(&mut PlottingState) -> &mut f64 + 'static>| {
            let app_state = app_state.clone();
            let drawing_area = drawing_area.clone();
            let stats = stats.clone();
            what.connect_value_changed(move |target| {
                let mut state = app_state.borrow_mut();
                *how(&mut *state) = target.value();
                drawing_area.queue_draw();
                stats.queue_draw();
            });
        };

    let _handle_bool_change =
        |what: &gtk::ToggleButton, how: Box<dyn Fn(&mut PlottingState) -> &mut bool + 'static>| {
            let app_state = app_state.clone();
            let drawing_area = drawing_area.clone();
            let stats = stats.clone();
            what.connect_toggled(move |target| {
                let mut state = app_state.borrow_mut();
                *how(&mut *state) = target.is_active();
                drawing_area.queue_draw();
                stats.queue_draw();
            });
        };

    handle_change(&deviation_limit_scale, Box::new(|s| &mut s.deviation_limit));

    window.show_all();
}

fn main() {
    let application = gtk::Application::new(Some("io.tp-multi-debugger"), Default::default());

    application.connect_activate(|app| {
        build_ui(app);
    });

    application.run();
}
