//! Graphs the position, velocity and acceleration of a trajectory segment using the
//! [`synchronised`] module.

use eframe::egui;
use egui::epaint::Hsva;
use egui::{Color32, TextStyle, Ui};
use egui_extras::{Size, StripBuilder};
use egui_plot::{Legend, Line, Plot, PlotPoints};
use env_logger::Env;
use tp::synchronised::{Coord3, Lim, Segment};

struct MyApp {
    segment: Segment,
}

impl MyApp {
    fn chart(&mut self, _heading_text_size: f32, ui: &mut Ui) {
        StripBuilder::new(ui)
            // Charts
            .size(Size::remainder())
            .size(Size::remainder())
            .size(Size::remainder())
            .vertical(|mut strip| {
                let n_points = 5000u16;

                let mut points = Vec::new();

                for t in 0..n_points {
                    let t = f32::from(t) / (f32::from(n_points) / self.segment.total_time);

                    let Some((out, _is_arc)) = self.segment.tp(t) else {
                        continue;
                    };

                    points.push((f64::from(t), out));
                }

                // X axis
                strip.cell(|ui| {
                    Plot::new("trajectory_x")
                        .x_axis_label("Time")
                        .legend(Legend::default())
                        .show(ui, |plot_ui| {
                            let pos = points
                                .iter()
                                .map(|(t, out)| [*t, f64::from(out.pos.x)])
                                .collect::<Vec<_>>();

                            let vel = points
                                .iter()
                                .map(|(t, out)| [*t, f64::from(out.vel.x)])
                                .collect::<Vec<_>>();

                            let acc = points
                                .iter()
                                .map(|(t, out)| [*t, f64::from(out.acc.x)])
                                .collect::<Vec<_>>();

                            plot_ui.line(
                                Line::new(PlotPoints::new(pos))
                                    .color(idx_to_colour(0))
                                    .name("Position"),
                            );

                            plot_ui.line(
                                Line::new(PlotPoints::new(vel))
                                    .color(idx_to_colour(1))
                                    .name("Velocity"),
                            );

                            plot_ui.line(
                                Line::new(PlotPoints::new(acc))
                                    .color(idx_to_colour(2))
                                    .name("Acceleration"),
                            );
                        });
                });

                // Y axis
                strip.cell(|ui| {
                    Plot::new("trajectory_y")
                        .x_axis_label("Time")
                        .legend(Legend::default())
                        .show(ui, |plot_ui| {
                            let pos = points
                                .iter()
                                .map(|(t, out)| [*t, f64::from(out.pos.y)])
                                .collect::<Vec<_>>();

                            let vel = points
                                .iter()
                                .map(|(t, out)| [*t, f64::from(out.vel.y)])
                                .collect::<Vec<_>>();

                            let acc = points
                                .iter()
                                .map(|(t, out)| [*t, f64::from(out.acc.y)])
                                .collect::<Vec<_>>();

                            plot_ui.line(
                                Line::new(PlotPoints::new(pos))
                                    .color(idx_to_colour(0))
                                    .name("Position"),
                            );

                            plot_ui.line(
                                Line::new(PlotPoints::new(vel))
                                    .color(idx_to_colour(1))
                                    .name("Velocity"),
                            );

                            plot_ui.line(
                                Line::new(PlotPoints::new(acc))
                                    .color(idx_to_colour(2))
                                    .name("Acceleration"),
                            );
                        });
                });

                // Z axis
                strip.cell(|ui| {
                    Plot::new("trajectory_z")
                        .x_axis_label("Time")
                        .legend(Legend::default())
                        .show(ui, |plot_ui| {
                            let pos = points
                                .iter()
                                .map(|(t, out)| [*t, f64::from(out.pos.z)])
                                .collect::<Vec<_>>();

                            let vel = points
                                .iter()
                                .map(|(t, out)| [*t, f64::from(out.vel.z)])
                                .collect::<Vec<_>>();

                            let acc = points
                                .iter()
                                .map(|(t, out)| [*t, f64::from(out.acc.z)])
                                .collect::<Vec<_>>();

                            plot_ui.line(
                                Line::new(PlotPoints::new(pos))
                                    .color(idx_to_colour(0))
                                    .name("Position"),
                            );

                            plot_ui.line(
                                Line::new(PlotPoints::new(vel))
                                    .color(idx_to_colour(1))
                                    .name("Velocity"),
                            );

                            plot_ui.line(
                                Line::new(PlotPoints::new(acc))
                                    .color(idx_to_colour(2))
                                    .name("Acceleration"),
                            );
                        });
                });
            });
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let heading_text_size = TextStyle::Heading.resolve(ui.style()).size;

            StripBuilder::new(ui)
                .size(Size::remainder())
                .vertical(|mut strip| {
                    strip.cell(|ui| {
                        self.chart(heading_text_size, ui);
                    });
                });
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1920.0, 1080.0])
            .with_min_inner_size([1280.0, 720.0]),
        ..Default::default()
    };

    let q0 = Coord3::new(0.0, 10.0, 0.0);
    let q1 = Coord3::new(50.0, -40.0, 20.0);

    let v0 = Coord3::new(0.0, 0.0, 0.0);
    let v1 = Coord3::new(0.0, 0.0, 0.0);

    let lim = Lim {
        vel: Coord3::new(10.0, 10.0, 10.0),
        acc: Coord3::new(20.0, 15.0, 5.0),
    };

    let segment = Segment::new(q0, q1, v0, v1, 0.0, &lim);

    log::info!("Duration {}", segment.total_time);

    eframe::run_native(
        "Visualiser",
        native_options,
        Box::new(|_cc| {
            // let ctx = cc.egui_ctx.clone();

            Box::new(MyApp { segment })
        }),
    )
}

// Nicked from <https://github.com/emilk/egui/blob/e29022efc4783fe06842a46371d5bd88e3f13bdd/crates/egui_plot/src/plot_ui.rs#L16C5-L22C6>
fn idx_to_colour(idx: usize) -> Color32 {
    let i = idx as f32;
    let golden_ratio = (5.0_f32.sqrt() - 1.0) / 2.0; // 0.61803398875
    let h = i as f32 * golden_ratio;
    Hsva::new(h, 0.85, 0.5, 1.0).into()
}
