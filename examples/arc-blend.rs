use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

use gtk::prelude::*;
use plotters::prelude::*;
use plotters::style::full_palette;
use plotters_cairo::CairoBackend;
use tp::trapezoidal_non_zero::{Lim, Segments};

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

        root.draw(&Circle::new(
            (100, 100),
            50,
            Into::<ShapeStyle>::into(&GREEN).filled(),
        ))?;

        root.fill(&WHITE)?;

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
        let w = widget.allocated_width();
        let h = widget.allocated_height();
        let backend = CairoBackend::new(cr, (w as u32, h as u32)).expect("Cairo no");
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
