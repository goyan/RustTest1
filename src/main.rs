#![windows_subsystem = "windows"]

mod models;
mod theme;
mod analysis;
mod navigation;
mod utils;
mod ui;
mod tests;

use eframe::egui;
use sysinfo::Disks;
use std::path::PathBuf;
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{channel, Receiver, Sender};

use models::*;

pub struct DiskDashboard {
    disks: Disks,
    refresh_interval: f32,
    time_since_refresh: f32,
    pub(crate) current_path: Option<PathBuf>,
    pub(crate) current_disk: Option<PathBuf>,
    pub(crate) file_items: Vec<FileItem>,
    pub(crate) filtered_items: Vec<FileItem>,
    pub(crate) loading: bool,
    pub(crate) sort_column: SortColumn,
    pub(crate) sort_direction: SortDirection,
    pub(crate) navigation_history: Vec<PathBuf>,
    pub(crate) history_index: usize,
    pub(crate) search_query: String,
    // Deletion confirmation
    pub(crate) pending_delete: Option<PathBuf>,
    pub(crate) delete_error: Option<String>,
    pub(crate) needs_refresh: bool,
    // Toast notifications
    pub(crate) toast_message: Option<(String, f32)>, // (message, time_remaining)
    // Folder size cache for efficient recursive size calculation
    pub(crate) folder_size_cache: HashMap<PathBuf, u64>,
    // Async folder size calculation
    pub(crate) size_sender: Sender<(PathBuf, u64)>,
    size_receiver: Receiver<(PathBuf, u64)>,
    pub(crate) pending_size_calculations: HashSet<PathBuf>,
    // Multi-file selection
    pub(crate) selected_items: HashSet<PathBuf>,
    // Batch delete confirmation
    pub(crate) pending_batch_delete: bool,
    // Track loaded path to avoid reloading every frame
    pub(crate) last_loaded_path: Option<PathBuf>,
}

impl Default for DiskDashboard {
    fn default() -> Self {
        let (sender, receiver) = channel();
        Self {
            disks: Disks::new_with_refreshed_list(),
            refresh_interval: 1.0,
            time_since_refresh: 0.0,
            current_path: None,
            current_disk: None,
            file_items: Vec::new(),
            filtered_items: Vec::new(),
            loading: false,
            sort_column: SortColumn::Size,
            sort_direction: SortDirection::Descending,
            navigation_history: Vec::new(),
            history_index: 0,
            search_query: String::new(),
            pending_delete: None,
            delete_error: None,
            needs_refresh: false,
            toast_message: None,
            folder_size_cache: HashMap::new(),
            size_sender: sender,
            size_receiver: receiver,
            pending_size_calculations: HashSet::new(),
            selected_items: HashSet::new(),
            pending_batch_delete: false,
            last_loaded_path: None,
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 700.0])
            .with_min_inner_size([900.0, 550.0])
            .with_title("Disk Capacity Dashboard")
            .with_decorations(true)
            .with_resizable(true)
            .with_maximized(false),
        ..Default::default()
    };

    eframe::run_native(
        "Disk Dashboard",
        options,
        Box::new(|_cc| Box::new(DiskDashboard::default())),
    )
}

impl eframe::App for DiskDashboard {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Auto-refresh disks at specified interval
        let dt = ctx.input(|i| i.stable_dt);
        self.time_since_refresh += dt;
        if self.time_since_refresh >= self.refresh_interval {
            self.disks.refresh();
            self.time_since_refresh = 0.0;
            ctx.request_repaint();
        }

        // Update toast timer
        if let Some((_, ref mut time_left)) = self.toast_message {
            *time_left -= dt;
            if *time_left <= 0.0 {
                self.toast_message = None;
            } else {
                ctx.request_repaint(); // Keep animating
            }
        }

        // Check for completed async folder size calculations
        let mut sizes_updated = false;
        while let Ok((path, size)) = self.size_receiver.try_recv() {
            self.folder_size_cache.insert(path.clone(), size);
            self.pending_size_calculations.remove(&path);
            sizes_updated = true;
        }
        // Update file items with new sizes
        if sizes_updated {
            for item in &mut self.file_items {
                if item.is_dir {
                    if let Some(&size) = self.folder_size_cache.get(&item.path) {
                        item.size = size;
                    }
                }
            }
            self.apply_filter_and_sort();
            ctx.request_repaint();
        }
        // Request repaint if calculations pending
        if !self.pending_size_calculations.is_empty() {
            ctx.request_repaint();
        }

        // Handle keyboard shortcuts and scroll selection
        ctx.input(|i| {
            // Mouse forward/backward buttons
            if i.pointer.button_pressed(egui::PointerButton::Extra1) {
                self.navigate_back();
            }
            if i.pointer.button_pressed(egui::PointerButton::Extra2) {
                self.navigate_forward();
            }

            // Keyboard shortcuts
            if i.key_pressed(egui::Key::Backspace) && i.modifiers.ctrl {
                self.navigate_back();
            }
            if i.key_pressed(egui::Key::ArrowLeft) && i.modifiers.alt {
                self.navigate_back();
            }
            if i.key_pressed(egui::Key::ArrowRight) && i.modifiers.alt {
                self.navigate_forward();
            }
            if i.key_pressed(egui::Key::Home) && i.modifiers.ctrl {
                self.current_path = None;
                self.current_disk = None;
                self.file_items.clear();
            }
            // Focus search with Ctrl+F
            if i.key_pressed(egui::Key::F) && i.modifiers.ctrl {
                // Search will be focused in UI
            }

        });

        // Refresh file list if needed (after deletion)
        if self.needs_refresh {
            self.needs_refresh = false;
            if let Some(ref path) = self.current_path.clone() {
                self.load_directory(path);
                self.last_loaded_path = Some(path.clone());
            }
        }

        // Load directory only when path changes (not every frame!)
        if self.current_path != self.last_loaded_path {
            if let Some(ref path) = self.current_path.clone() {
                self.load_directory(path);
                self.last_loaded_path = Some(path.clone());
                self.selected_items.clear(); // Clear selection on navigation
            } else {
                self.last_loaded_path = None;
            }
        }

        // Apply modern theme
        theme::apply_modern_theme(ctx);

        // Show dialogs
        self.render_delete_dialog(ctx);
        self.render_error_dialog(ctx);
        self.render_batch_delete_dialog(ctx);

        egui::TopBottomPanel::top("top_panel")
            .show(ctx, |ui| {
                egui::Frame::default()
                    .fill(egui::Color32::from_rgb(12, 10, 18))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 255, 255)))
                    .inner_margin(egui::Margin::same(15.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.heading(egui::RichText::new("⚡ DISK DASHBOARD")
                                .size(24.0)
                                .color(egui::Color32::from_rgb(0, 255, 255))
                                .strong());
                            ui.add_space(20.0);
                            ui.label(egui::RichText::new("// SYSTEM ANALYSIS ACTIVE")
                                .size(12.0)
                                .color(egui::Color32::from_rgb(255, 0, 255)));

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if self.current_path.is_some() {
                                    if ui.add(egui::Button::new("⌂ HOME")
                                        .fill(egui::Color32::from_rgb(30, 20, 50))
                                        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 255, 255))))
                                        .clicked() {
                                        self.current_path = None;
                                        self.current_disk = None;
                                        self.file_items.clear();
                                        self.search_query.clear();
                                    }
                                }
                            });
                        });
                    });
            });

        egui::SidePanel::left("disks_panel")
            .resizable(true)
            .default_width(280.0)
            .min_width(220.0)
            .max_width(400.0)
            .show(ctx, |ui| {
                ui.heading("Disks");
                ui.separator();

                let mut disk_data: Vec<(PathBuf, String, u64, u64, f64)> = self.disks.list().iter()
                    .map(|d| {
                        let mount = d.mount_point().to_path_buf();
                        let name = d.name().to_string_lossy().to_string();
                        let total = d.total_space();
                        let available = d.available_space();
                        let used = total - available;
                        let percent = if total > 0 {
                            (used as f64 / total as f64) * 100.0
                        } else {
                            0.0
                        };
                        (mount, name, total, available, percent)
                    })
                    .collect();

                disk_data.sort_by(|a, b| a.0.to_string_lossy().cmp(&b.0.to_string_lossy()));

                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                    // Set minimum width to fill panel
                    ui.set_min_width(ui.available_width());

                    for (mount_point, disk_name, total, available, percent) in &disk_data {
                        // Skip empty/invalid entries
                        if *total == 0 {
                            continue;
                        }
                        let mount_clone = mount_point.clone();
                        let name_clone = disk_name.clone();
                        let total_clone = *total;
                        let available_clone = *available;
                        let percent_clone = *percent;

                        // Check if this disk is currently selected
                        let is_selected = self.current_disk.as_ref()
                            .map(|d| d == mount_point)
                            .unwrap_or(false);

                        // Cyberpunk disk card with neon colors
                        let usage_color = if percent_clone > 90.0 {
                            egui::Color32::from_rgb(255, 51, 102)   // Neon red
                        } else if percent_clone > 75.0 {
                            egui::Color32::from_rgb(255, 136, 0)    // Neon orange
                        } else {
                            egui::Color32::from_rgb(0, 255, 136)    // Neon green
                        };

                        // Create an interactive area to detect hover BEFORE drawing
                        let card_id = ui.make_persistent_id(format!("disk_card_{}", mount_point.to_string_lossy()));
                        let card_rect = ui.available_rect_before_wrap();
                        let interact_rect = egui::Rect::from_min_size(card_rect.min, egui::Vec2::new(ui.available_width(), 120.0));
                        let sense = egui::Sense::click().union(egui::Sense::hover());
                        let interact_response = ui.interact(interact_rect, card_id, sense);
                        let is_hovered = interact_response.hovered();

                        // Different styling for selected/hovered disk
                        let card_fill = if is_selected {
                            egui::Color32::from_rgb(40, 55, 75)
                        } else if is_hovered {
                            egui::Color32::from_rgb(38, 42, 55)
                        } else {
                            egui::Color32::from_rgb(28, 30, 38)
                        };

                        let border_color = if is_selected {
                            egui::Color32::from_rgb(100, 150, 255)
                        } else if is_hovered {
                            usage_color
                        } else {
                            egui::Color32::from_rgb(45, 48, 55)
                        };

                        let border_width = if is_selected { 2.0 } else if is_hovered { 1.5 } else { 1.0 };

                        let disk_card_response = egui::Frame::default()
                            .fill(card_fill)
                            .stroke(egui::Stroke::new(border_width, border_color))
                            .rounding(8.0)
                            .inner_margin(egui::Margin::same(14.0))
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    // Drive letter and name
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new("💿").size(20.0));
                                        let display_name = if name_clone.is_empty() {
                                            mount_point.to_string_lossy().to_string()
                                        } else {
                                            format!("{} ({})", mount_point.to_string_lossy(), name_clone)
                                        };
                                        ui.label(egui::RichText::new(display_name)
                                            .size(16.0)
                                            .strong()
                                            .color(egui::Color32::from_rgb(220, 230, 255)));
                                    });

                                    ui.add_space(8.0);

                                    // Usage percentage
                                    ui.label(egui::RichText::new(format!("{:.1}% used", percent_clone))
                                        .size(14.0)
                                        .color(usage_color)
                                        .strong());

                                    ui.add_space(6.0);

                                    // Progress bar
                                    let progress_bar = egui::ProgressBar::new(percent_clone as f32 / 100.0)
                                        .fill(usage_color)
                                        .show_percentage();
                                    ui.add(progress_bar);

                                    ui.add_space(4.0);

                                    // Size info
                                    ui.label(egui::RichText::new(format!("{:.2} GB / {:.2} GB",
                                        (total_clone - available_clone) as f64 / 1_000_000_000.0,
                                        total_clone as f64 / 1_000_000_000.0))
                                        .size(11.0)
                                        .color(egui::Color32::from_gray(160)));
                                });
                            });

                        // Handle click on the interactive area
                        if interact_response.clicked() {
                            self.navigate_to(mount_clone.clone());
                            self.current_disk = Some(mount_clone);
                            self.file_items.clear();
                            self.search_query.clear();
                        }

                        // Ensure we don't consume the frame response click as well
                        let _ = disk_card_response.response;

                        ui.add_space(12.0);
                    }

                    // Calculate totals for summary and pie chart
                    let total_disks = disk_data.len();
                    let total_space: u64 = disk_data.iter().map(|(_, _, total, _, _)| *total).sum();
                    let total_used: u64 = disk_data.iter().map(|(_, _, total, available, _)| *total - *available).sum();
                    let avg_usage = if total_space > 0 {
                        (total_used as f64 / total_space as f64) * 100.0
                    } else {
                        0.0
                    };

                    // Modern Summary panel (inside scroll area)
                    ui.add_space(15.0);
                    egui::Frame::default()
                        .fill(egui::Color32::from_rgb(25, 27, 35))
                        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(50, 55, 65)))
                        .rounding(8.0)
                        .inner_margin(egui::Margin::same(12.0))
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.heading(egui::RichText::new("Summary")
                                .size(16.0)
                                .color(egui::Color32::from_rgb(180, 200, 255)));
                            ui.add_space(10.0);

                            // Use columns for better layout
                            ui.columns(3, |columns| {
                                columns[0].vertical_centered(|ui| {
                                    ui.label(egui::RichText::new(format!("{}", total_disks))
                                        .size(18.0)
                                        .strong()
                                        .color(egui::Color32::from_rgb(100, 200, 255)));
                                    ui.label(egui::RichText::new("Disks")
                                        .size(10.0)
                                        .color(egui::Color32::from_rgb(120, 100, 160)));
                                });
                                columns[1].vertical_centered(|ui| {
                                    ui.label(egui::RichText::new(format!("{:.0} GB", total_space as f64 / 1_000_000_000.0))
                                        .size(18.0)
                                        .strong()
                                        .color(egui::Color32::from_rgb(100, 200, 255)));
                                    ui.label(egui::RichText::new("Total")
                                        .size(10.0)
                                        .color(egui::Color32::from_rgb(120, 100, 160)));
                                });
                                columns[2].vertical_centered(|ui| {
                                    let used_color = if avg_usage > 90.0 {
                                        egui::Color32::from_rgb(255, 51, 102)   // Neon red
                                    } else if avg_usage > 75.0 {
                                        egui::Color32::from_rgb(255, 136, 0)    // Neon orange
                                    } else {
                                        egui::Color32::from_rgb(0, 255, 136)    // Neon green
                                    };
                                    ui.label(egui::RichText::new(format!("{:.1}%", avg_usage))
                                        .size(18.0)
                                        .strong()
                                        .color(used_color));
                                    ui.label(egui::RichText::new("Used")
                                        .size(10.0)
                                        .color(egui::Color32::from_rgb(120, 100, 160)));
                                });
                            });
                        });

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    // Pie chart visualization
                    if total_space > 0 {
                        self.render_pie_chart(ui, &disk_data, total_space, total_used, avg_usage);
                    }
                }); // Close ScrollArea
            }); // Close SidePanel

        let current_path_clone = self.current_path.clone();
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(ref path) = current_path_clone {
                let path_clone = path.clone();
                self.render_file_browser(ui, &path_clone);
            } else {
                self.render_disk_overview(ui);
            }
        });

        // Render cyberpunk toast notification overlay
        if let Some((ref message, time_left)) = self.toast_message {
            let opacity = (time_left.min(0.3) / 0.3).min(1.0); // Fade out in last 0.3s
            egui::Area::new(egui::Id::new("toast_notification"))
                .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -50.0])
                .show(ctx, |ui| {
                    egui::Frame::default()
                        .fill(egui::Color32::from_rgba_unmultiplied(20, 10, 35, (230.0 * opacity) as u8))
                        .stroke(egui::Stroke::new(1.5, egui::Color32::from_rgba_unmultiplied(0, 255, 255, (220.0 * opacity) as u8)))
                        .rounding(8.0)
                        .inner_margin(egui::Margin::symmetric(20.0, 12.0))
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new(message)
                                .size(14.0)
                                .color(egui::Color32::from_rgba_unmultiplied(0, 255, 255, (255.0 * opacity) as u8)));
                        });
                });
        }
    }
}
