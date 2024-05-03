//! Graphs the position, velocity and acceleration of a trajectory generated with
//! `segments_blends.rs`.

use eframe::egui;
use egui::epaint::Hsva;
use egui::{Color32, TextStyle, Ui};
use egui_extras::{Column, TableBuilder};
use egui_extras::{Size, StripBuilder};
use egui_plot::{Legend, Line, LineStyle, Plot, PlotPoints, VLine};
use nalgebra::{Point3, Translation3, UnitQuaternion, Vector3};
use std::{path::PathBuf, sync::Arc, thread, time::Duration};
use tp::arc_blend::Coord3;
use tp::segments_blends::{Item, Trajectory};
use tp::trapezoidal_non_zero_3d::{Lim, Out};

struct MyApp {
    trajectory: Trajectory,
}

impl MyApp {
    /// Returns `(start count, end count, stride)`. Used for showing a subset of some data on the
    /// graph to improve performance.
    fn compute_bounds(&self, plot_ui: &mut egui_plot::PlotUi) -> (usize, usize, usize) {
        // Bounds of the plot by data values, not pixels
        let plot_bounds = plot_ui.plot_bounds();

        let (start_count, end_count) = if plot_bounds.min()[0] <= 0.0 {
            (0usize, self.trajectory.total_time.ceil() as usize)
        } else {
            (plot_bounds.min()[0] as usize, plot_bounds.max()[0] as usize)
        };

        let values_width = plot_bounds.width();

        let pixels_width = {
            plot_ui.screen_from_plot(plot_bounds.max().into())[0]
                - plot_ui.screen_from_plot(plot_bounds.min().into())[0]
        } as f64;

        let stride = (values_width / pixels_width).max(1.0) as usize;

        (start_count, end_count, stride)
    }

    /// Take a series of points and filter them down to a subset where:
    ///
    /// - Only visible points are shown.
    /// - If the data is dense enough that multiple points span a single pixel, two points (min,
    ///   max) are created for that pixel.
    fn aggregate(
        &self,
        (start_count, end_count, stride): (usize, usize, usize),
        series: &[[f64; 2]],
    ) -> Vec<[f64; 2]> {
        let display_range = start_count.min(series.len())..end_count.min(series.len());

        series[display_range]
            .chunks(stride)
            .into_iter()
            .map(|chunk| {
                let ys = chunk.iter().map(|[_x, y]| *y);
                let xs = chunk.iter().map(|[x, _y]| *x);

                // Put X coord in middle of chunk
                let x = xs.sum::<f64>() / chunk.len() as f64;

                [
                    [
                        x,
                        ys.clone()
                            .min_by(|a, b| (*a as u32).cmp(&(*b as u32)))
                            .unwrap(),
                    ],
                    [x, ys.max_by(|a, b| (*a as u32).cmp(&(*b as u32))).unwrap()],
                ]
            })
            .flatten()
            .collect::<Vec<_>>()
    }

    fn chart(&mut self, heading_text_size: f32, ui: &mut Ui) {
        StripBuilder::new(ui)
            // Heading
            .size(Size::exact(heading_text_size))
            // Chart
            .size(Size::remainder())
            .vertical(|mut strip| {
                strip.cell(|ui| {
                    ui.heading("Trajectory plot");
                });
                strip.cell(|ui| {
                    let n_points = 500u16;

                    let mut points = Vec::new();

                    for t in 0..n_points {
                        let t = f32::from(t) / (f32::from(n_points) / self.trajectory.total_time);

                        let Some((out, _is_arc)) = self.trajectory.tp(t) else {
                            continue;
                        };

                        points.push((f64::from(t), out));
                    }

                    Plot::new("trajectory_x")
                        .x_axis_label("Time")
                        // .y_axis_label("Value")
                        .legend(Legend::default())
                        .show(ui, |plot_ui| {
                            // let bounds = self.compute_bounds(plot_ui);

                            // let mut points = Vec::new();

                            // for t in 0..n_points {
                            //     let t = f32::from(t)
                            //         / (f32::from(n_points) / self.trajectory.total_time);

                            //     let Some((Out { pos, acc, vel }, _is_arc)) = self.trajectory.tp(t)
                            //     else {
                            //         continue;
                            //     };

                            //     points.push([f64::from(t), f64::from(pos.x)]);
                            // }

                            // let points = self.aggregate(bounds, &points);

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
            });
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("left_panel")
            // .resizable(true)
            .default_width(200.0)
            // .width_range(200.0..=500.0)
            .show(ctx, |ui| {
                // ui.vertical_centered(|ui| {
                ui.heading("TODO");
                // });

                // egui::ScrollArea::vertical().show(ui, |ui| {
                //     self.file_list(ui);
                // });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            // if ui.button("Save Plot").clicked() {
            //     ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot);
            // }

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
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1920.0, 1080.0])
            .with_min_inner_size([1280.0, 720.0]),
        ..Default::default()
    };

    let mut trajectory = Trajectory::new();

    trajectory.push_point(Coord3::new(0.0, 0.0, 0.0));
    trajectory.push_point(Coord3::new(5.0, 0.0, 0.0));

    eframe::run_native(
        "Visualiser",
        native_options,
        Box::new(|_cc| {
            // let ctx = cc.egui_ctx.clone();

            Box::new(MyApp { trajectory })
        }),
    )
}

// Nicked from <https://github.com/emilk/egui/blob/e29022efc4783fe06842a46371d5bd88e3f13bdd/crates/egui_plot/src/plot_ui.rs#L16C5-L22C6>
fn idx_to_colour(idx: usize) -> Color32 {
    let i = idx as f32;
    let golden_ratio = (5.0_f32.sqrt() - 1.0) / 2.0; // 0.61803398875
    let h = i as f32 * golden_ratio;
    Hsva::new(h, 0.85, 0.5, 1.0).into() // TODO(emilk): OkLab or some other perspective color space
}
