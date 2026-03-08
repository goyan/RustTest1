use eframe::egui;
use std::path::PathBuf;
use crate::DiskDashboard;

impl DiskDashboard {
    pub(crate) fn render_disk_overview(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.heading(egui::RichText::new("Select a disk to browse")
                .size(24.0)
                .color(egui::Color32::from_gray(200)));
            ui.add_space(20.0);
            ui.label(egui::RichText::new("Click on a disk in the left panel to explore its contents")
                .size(14.0)
                .color(egui::Color32::from_rgb(120, 100, 160)));
        });
    }

    pub(crate) fn render_pie_chart(&self, ui: &mut egui::Ui, disk_data: &[(PathBuf, String, u64, u64, f64)], total_space: u64, total_used: u64, avg_usage: f64) {
        ui.add_space(15.0);
        egui::Frame::default()
            .fill(egui::Color32::from_rgb(25, 27, 35))
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(50, 55, 65)))
            .rounding(8.0)
            .inner_margin(egui::Margin::same(12.0))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.heading(egui::RichText::new("Disk Usage Breakdown")
                    .size(16.0)
                    .color(egui::Color32::from_rgb(180, 200, 255)));
                ui.add_space(10.0);

                // Use available width to determine chart size, with max of 180
                let chart_size = ui.available_width().min(180.0);
                let (response, painter) = ui.allocate_painter(
                    egui::Vec2::new(chart_size, chart_size),
                    egui::Sense::hover()
                );

                let center = response.rect.center();
                let radius = chart_size * 0.4;

                // Calculate angles
                let used_angle = if total_space > 0 {
                    (total_used as f64 / total_space as f64) * 2.0 * std::f64::consts::PI
                } else {
                    0.0
                };

                // Draw pie slices
                let mut current_angle = -std::f64::consts::PI / 2.0; // Start from top

                // Used space (neon colors based on usage)
                let used_color = if avg_usage > 90.0 {
                    egui::Color32::from_rgb(255, 51, 102)   // Neon red
                } else if avg_usage > 75.0 {
                    egui::Color32::from_rgb(255, 136, 0)    // Neon orange
                } else {
                    egui::Color32::from_rgb(0, 255, 136)    // Neon green
                };

                if used_angle > 0.0 {
                    let mut used_path = self.create_pie_slice_path(center, radius, current_angle, current_angle + used_angle);
                    used_path.fill = used_color;
                    painter.add(egui::Shape::Path(used_path));
                    current_angle += used_angle;
                }

                // Available space (green)
                let available_angle = 2.0 * std::f64::consts::PI - used_angle;
                if available_angle > 0.0 {
                    let mut available_path = self.create_pie_slice_path(center, radius, current_angle, current_angle + available_angle);
                    available_path.fill = egui::Color32::from_rgb(50, 200, 50);
                    painter.add(egui::Shape::Path(available_path));
                }

                // Draw legend
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("●").color(used_color).size(16.0));
                    ui.label(format!("Used: {:.1}%", avg_usage));
                    ui.add_space(15.0);
                    ui.label(egui::RichText::new("●").color(egui::Color32::from_rgb(50, 200, 50)).size(16.0));
                    ui.label(format!("Available: {:.1}%", 100.0 - avg_usage));
                });

                // Breakdown by disk
                if disk_data.len() > 1 {
                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(5.0);
                    ui.label(egui::RichText::new("By Disk:").strong());

                    let colors = vec![
                        egui::Color32::from_rgb(100, 150, 255),
                        egui::Color32::from_rgb(255, 150, 100),
                        egui::Color32::from_rgb(150, 255, 150),
                        egui::Color32::from_rgb(255, 200, 100),
                        egui::Color32::from_rgb(200, 150, 255),
                    ];

                    for (i, (mount_point, disk_name, total, available, _percent)) in disk_data.iter().enumerate() {
                        let used = total - available;
                        let disk_percent = if *total > 0 {
                            (used as f64 / *total as f64) * 100.0
                        } else {
                            0.0
                        };
                        let space_percent = if total_space > 0 {
                            (*total as f64 / total_space as f64) * 100.0
                        } else {
                            0.0
                        };

                        let color = colors[i % colors.len()];
                        let display = if disk_name.is_empty() {
                            mount_point.to_string_lossy().to_string()
                        } else {
                            format!("{} ({})", mount_point.to_string_lossy(), disk_name)
                        };
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("●").color(color).size(12.0));
                            ui.label(format!("{}: {:.1}% ({:.1}% of total)",
                                display,
                                disk_percent,
                                space_percent));
                        });
                    }
                }
            });
    }

    fn create_pie_slice_path(&self, center: egui::Pos2, radius: f32, start_angle: f64, end_angle: f64) -> egui::epaint::PathShape {
        let mut points = vec![center];

        // Add points along the arc
        let num_points = 32;
        for i in 0..=num_points {
            let angle = start_angle + (end_angle - start_angle) * (i as f64 / num_points as f64);
            let x = center.x + radius * (angle.cos() as f32);
            let y = center.y + radius * (angle.sin() as f32);
            points.push(egui::Pos2::new(x, y));
        }

        egui::epaint::PathShape {
            points,
            closed: true,
            fill: egui::Color32::TRANSPARENT,
            stroke: egui::Stroke::NONE,
        }
    }
}
