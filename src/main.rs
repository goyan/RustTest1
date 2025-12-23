use eframe::egui;
use sysinfo::Disks;
use std::path::{Path, PathBuf};
use std::fs;
use std::time::SystemTime;
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

#[derive(Clone, Copy, PartialEq, Debug)]
#[allow(dead_code)]
enum FileCategory {
    MustKeep,    // Critical system files, important user data
    System,      // System files that should generally be kept
    Regular,     // Normal files
    Useless,     // Temp files, cache, logs, etc.
    Unknown,     // Can't determine
}

#[derive(Clone, Copy, PartialEq)]
enum SortColumn {
    Name,
    Size,
    Category,
    Usefulness,
}

#[derive(Clone, Copy, PartialEq)]
enum SortDirection {
    Ascending,
    Descending,
}

#[derive(Clone)]
struct FileItem {
    path: PathBuf,
    name: String,
    size: u64,              // For files: file size. For folders: total size of contents
    is_dir: bool,
    category: FileCategory,
    usefulness: f32,        // 0-100 score
    modified: Option<SystemTime>,
    child_count: Option<usize>, // For directories: number of items inside
}

struct DiskDashboard {
    disks: Disks,
    refresh_interval: f32,
    time_since_refresh: f32,
    current_path: Option<PathBuf>,
    current_disk: Option<PathBuf>,
    file_items: Vec<FileItem>,
    filtered_items: Vec<FileItem>,
    loading: bool,
    sort_column: SortColumn,
    sort_direction: SortDirection,
    navigation_history: Vec<PathBuf>,
    history_index: usize,
    search_query: String,
    // Deletion confirmation
    pending_delete: Option<PathBuf>,
    delete_error: Option<String>,
    needs_refresh: bool,
    // Toast notifications
    toast_message: Option<(String, f32)>, // (message, time_remaining)
    // Folder size cache for efficient recursive size calculation
    folder_size_cache: HashMap<PathBuf, u64>,
    // Async folder size calculation
    size_sender: Sender<(PathBuf, u64)>,
    size_receiver: Receiver<(PathBuf, u64)>,
    pending_size_calculations: HashSet<PathBuf>,
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

        // Handle keyboard shortcuts
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
            }
        }

        // Refresh file list if path changed
        let path_to_load = self.current_path.clone();
        if let Some(ref path) = path_to_load {
            if !self.loading {
                self.load_directory(path);
            }
        }

        // Apply modern theme
        self.apply_modern_theme(ctx);

        // Show delete confirmation dialog
        if let Some(path_to_delete) = self.pending_delete.clone() {
            let is_dir = path_to_delete.is_dir();
            let file_name = path_to_delete.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path_to_delete.to_string_lossy().to_string());

            // Check if this is a protected system folder/file
            let name_lower = file_name.to_lowercase();
            let is_protected = name_lower.starts_with("$") ||
                name_lower == "system volume information" ||
                name_lower == "recovery" ||
                name_lower == "boot" ||
                name_lower == "bootmgr" ||
                name_lower == "pagefile.sys" ||
                name_lower == "hiberfil.sys" ||
                path_to_delete.to_string_lossy().to_lowercase().contains("windows\\system32") ||
                path_to_delete.to_string_lossy().to_lowercase().contains("program files");

            egui::Window::new(if is_protected { "Protected Item" } else { "Confirm Delete" })
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.set_min_width(350.0);

                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);

                        if is_protected {
                            // Protected item - show warning and only allow cancel
                            ui.label(egui::RichText::new("🔒 Protected System Item")
                                .size(18.0)
                                .strong()
                                .color(egui::Color32::from_rgb(255, 200, 50)));
                            ui.add_space(15.0);

                            ui.label(egui::RichText::new(&file_name)
                                .size(14.0)
                                .strong()
                                .color(egui::Color32::from_rgb(200, 200, 200)));
                            ui.add_space(15.0);

                            ui.label(egui::RichText::new("This is a protected Windows system item.")
                                .color(egui::Color32::from_rgb(255, 150, 100)));
                            ui.label(egui::RichText::new("Deleting it could damage your system.")
                                .color(egui::Color32::from_rgb(255, 150, 100)));
                            ui.add_space(10.0);
                            ui.label(egui::RichText::new("To empty the Recycle Bin, right-click it on your Desktop.")
                                .size(11.0)
                                .color(egui::Color32::from_gray(150)));

                            ui.add_space(20.0);

                            if ui.add(egui::Button::new("OK")
                                .fill(egui::Color32::from_rgb(60, 60, 80))
                                .min_size(egui::Vec2::new(100.0, 30.0)))
                                .clicked()
                            {
                                self.pending_delete = None;
                            }
                        } else {
                            // Normal delete confirmation
                            ui.label(egui::RichText::new(if is_dir { "🗑️ Delete Folder?" } else { "🗑️ Delete File?" })
                                .size(18.0)
                                .strong()
                                .color(egui::Color32::from_rgb(255, 100, 100)));
                            ui.add_space(15.0);

                            ui.label(format!("Are you sure you want to delete:"));
                            ui.add_space(5.0);
                            ui.label(egui::RichText::new(&file_name)
                                .size(14.0)
                                .strong()
                                .color(egui::Color32::from_rgb(255, 200, 100)));

                            if is_dir {
                                ui.add_space(10.0);
                                ui.label(egui::RichText::new("⚠️ This will delete the folder and ALL its contents!")
                                    .color(egui::Color32::from_rgb(255, 150, 50)));
                            }

                            ui.add_space(20.0);

                            ui.horizontal(|ui| {
                                ui.add_space(50.0);
                                if ui.add(egui::Button::new("Cancel")
                                    .fill(egui::Color32::from_rgb(60, 60, 80))
                                    .min_size(egui::Vec2::new(80.0, 30.0)))
                                    .clicked()
                                {
                                    self.pending_delete = None;
                                }

                                ui.add_space(20.0);

                                if ui.add(egui::Button::new("Delete")
                                    .fill(egui::Color32::from_rgb(180, 50, 50))
                                    .min_size(egui::Vec2::new(80.0, 30.0)))
                                    .clicked()
                                {
                                    // Perform deletion
                                    let result = if is_dir {
                                        fs::remove_dir_all(&path_to_delete)
                                    } else {
                                        fs::remove_file(&path_to_delete)
                                    };

                                    match result {
                                        Ok(_) => {
                                            self.delete_error = None;
                                            self.needs_refresh = true;
                                        }
                                        Err(e) => {
                                            self.delete_error = Some(format!("Failed to delete: {}", e));
                                        }
                                    }
                                    self.pending_delete = None;
                                }
                            });
                        }
                        ui.add_space(10.0);
                    });
                });
        }

        // Show error dialog if deletion failed
        if let Some(error) = self.delete_error.clone() {
            egui::Window::new("Error")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);
                        ui.label(egui::RichText::new("❌ Deletion Failed")
                            .size(16.0)
                            .color(egui::Color32::from_rgb(255, 100, 100)));
                        ui.add_space(10.0);
                        ui.label(&error);
                        ui.add_space(15.0);
                        if ui.button("OK").clicked() {
                            self.delete_error = None;
                        }
                        ui.add_space(10.0);
                    });
                });
        }

        egui::TopBottomPanel::top("top_panel")
            .show(ctx, |ui| {
                egui::Frame::default()
                    .fill(egui::Color32::from_rgb(18, 18, 24))
                    .inner_margin(egui::Margin::same(15.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.heading(egui::RichText::new("💾 Disk Dashboard")
                                .size(24.0)
                                .color(egui::Color32::from_rgb(100, 200, 255))
                                .strong());
                            ui.add_space(20.0);
                            ui.label(egui::RichText::new("Real-time disk monitoring & analysis")
                                .size(12.0)
                                .color(egui::Color32::from_gray(150)));
                            
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if self.current_path.is_some() {
                                    if ui.add(egui::Button::new("🏠 Home")
                                        .fill(egui::Color32::from_rgb(60, 60, 80))
                                        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 120))))
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
                        
                        // Modern clickable disk card with progress bar
                        let usage_color = if percent_clone > 90.0 {
                            egui::Color32::from_rgb(220, 50, 50)
                        } else if percent_clone > 75.0 {
                            egui::Color32::from_rgb(255, 165, 0)
                        } else {
                            egui::Color32::from_rgb(50, 200, 50)
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
                    let total_available: u64 = disk_data.iter().map(|(_, _, _, available, _)| *available).sum();
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
                                        .color(egui::Color32::from_gray(150)));
                                });
                                columns[1].vertical_centered(|ui| {
                                    ui.label(egui::RichText::new(format!("{:.0} GB", total_space as f64 / 1_000_000_000.0))
                                        .size(18.0)
                                        .strong()
                                        .color(egui::Color32::from_rgb(100, 200, 255)));
                                    ui.label(egui::RichText::new("Total")
                                        .size(10.0)
                                        .color(egui::Color32::from_gray(150)));
                                });
                                columns[2].vertical_centered(|ui| {
                                    let used_color = if avg_usage > 90.0 {
                                        egui::Color32::from_rgb(220, 50, 50)
                                    } else if avg_usage > 75.0 {
                                        egui::Color32::from_rgb(255, 165, 0)
                                    } else {
                                        egui::Color32::from_rgb(50, 200, 50)
                                    };
                                    ui.label(egui::RichText::new(format!("{:.1}%", avg_usage))
                                        .size(18.0)
                                        .strong()
                                        .color(used_color));
                                    ui.label(egui::RichText::new("Used")
                                        .size(10.0)
                                        .color(egui::Color32::from_gray(150)));
                                });
                            });
                        });

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    // Pie chart visualization
                    if total_space > 0 {
                        self.render_pie_chart(ui, &disk_data, total_space, total_used, total_available, avg_usage);
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

        // Render toast notification overlay
        if let Some((ref message, time_left)) = self.toast_message {
            let opacity = (time_left.min(0.3) / 0.3).min(1.0); // Fade out in last 0.3s
            egui::Area::new(egui::Id::new("toast_notification"))
                .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -50.0])
                .show(ctx, |ui| {
                    egui::Frame::default()
                        .fill(egui::Color32::from_rgba_unmultiplied(40, 45, 55, (220.0 * opacity) as u8))
                        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(100, 120, 150, (200.0 * opacity) as u8)))
                        .rounding(8.0)
                        .inner_margin(egui::Margin::symmetric(20.0, 12.0))
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new(message)
                                .size(14.0)
                                .color(egui::Color32::from_rgba_unmultiplied(255, 255, 255, (255.0 * opacity) as u8)));
                        });
                });
        }
    }
}

impl DiskDashboard {
    fn render_disk_overview(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.heading(egui::RichText::new("Select a disk to browse")
                .size(24.0)
                .color(egui::Color32::from_gray(200)));
            ui.add_space(20.0);
            ui.label(egui::RichText::new("Click on a disk in the left panel to explore its contents")
                .size(14.0)
                .color(egui::Color32::from_gray(150)));
        });
    }


    fn load_directory(&mut self, path: &Path) {
        self.loading = true;
        self.file_items.clear();

        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                let metadata = entry.metadata().ok();

                let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
                let modified = metadata.as_ref().and_then(|m| m.modified().ok());

                let name = entry_path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string();

                // Calculate size: for files use metadata, for dirs calculate recursive size
                let (size, child_count) = if is_dir {
                    let count = fs::read_dir(&entry_path).ok().map(|rd| rd.count());
                    // Use cached recursive size or calculate it
                    let dir_size = self.get_folder_size_recursive(&entry_path);
                    (dir_size, count)
                } else {
                    (metadata.as_ref().map(|m| m.len()).unwrap_or(0), None)
                };

                let (category, usefulness) = self.analyze_file(&entry_path, &name, is_dir, size);

                self.file_items.push(FileItem {
                    path: entry_path,
                    name,
                    size,
                    is_dir,
                    category,
                    usefulness,
                    modified,
                    child_count,
                });
            }
        }

        // Apply filtering and sorting
        self.apply_filter_and_sort();
        self.loading = false;
    }

    fn apply_filter_and_sort(&mut self) {
        // Filter items based on search query
        if self.search_query.is_empty() {
            self.filtered_items = self.file_items.clone();
        } else {
            let query_lower = self.search_query.to_lowercase();
            self.filtered_items = self.file_items.iter()
                .filter(|item| {
                    item.name.to_lowercase().contains(&query_lower) ||
                    item.path.to_string_lossy().to_lowercase().contains(&query_lower)
                })
                .cloned()
                .collect();
        }
        
        // Apply sorting to filtered items
        self.sort_file_items();
    }

    fn apply_modern_theme(&self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();
        
        // Modern color palette
        style.visuals.dark_mode = true;
        style.visuals.panel_fill = egui::Color32::from_rgb(22, 22, 28);
        style.visuals.window_fill = egui::Color32::from_rgb(18, 18, 24);
        style.visuals.extreme_bg_color = egui::Color32::from_rgb(15, 15, 20);
        style.visuals.faint_bg_color = egui::Color32::from_rgb(30, 30, 40);
        style.visuals.hyperlink_color = egui::Color32::from_rgb(100, 150, 255);
        
        // Button styling
        style.visuals.button_frame = true;
        style.visuals.selection.bg_fill = egui::Color32::from_rgb(60, 100, 200);
        
        // Spacing
        style.spacing.item_spacing = egui::Vec2::new(8.0, 6.0);
        style.spacing.window_margin = egui::Margin::same(8.0);
        
        ctx.set_style(style);
    }

    fn analyze_file(&self, path: &Path, name: &str, is_dir: bool, size: u64) -> (FileCategory, f32) {
        let name_lower = name.to_lowercase();
        let path_str = path.to_string_lossy().to_lowercase();

        // System and critical files - NEVER delete these
        if path_str.contains("windows\\system32") ||
           path_str.contains("windows\\syswow64") ||
           path_str.contains("program files") ||
           path_str.contains("programdata") ||
           name_lower == "windows" ||
           name_lower == "boot" ||
           name_lower == "bootmgr" ||
           name_lower == "pagefile.sys" ||
           name_lower == "hiberfil.sys" ||
           name_lower == "$recycle.bin" ||
           name_lower == "system volume information" ||
           name_lower == "recovery" ||
           name_lower.starts_with("$") {
            return (FileCategory::MustKeep, 100.0);
        }

        // Temp files and cache - useless (safe to delete)
        if name_lower.contains("temp") ||
           name_lower.contains("cache") ||
           name_lower.contains("tmp") ||
           name_lower.ends_with(".tmp") ||
           name_lower.ends_with(".log") ||
           path_str.contains("\\temp\\") ||
           path_str.contains("\\cache\\") ||
           path_str.contains("\\tmp\\") ||
           name_lower.starts_with("~$") {
            return (FileCategory::Useless, 5.0);
        }

        // System files
        if name_lower.ends_with(".sys") ||
           name_lower.ends_with(".dll") ||
           name_lower.ends_with(".exe") && path_str.contains("windows") ||
           name_lower.ends_with(".inf") ||
           name_lower.ends_with(".cat") {
            return (FileCategory::System, 85.0);
        }

        // Get file extension for detailed analysis
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();

        // Important user data - high usefulness
        let important_extensions = ["doc", "docx", "xls", "xlsx", "ppt", "pptx", "pdf",
                                    "txt", "md", "rtf", "odt", "ods", "odp"];
        if important_extensions.contains(&ext.as_str()) {
            return (FileCategory::Regular, 90.0);
        }

        // Photos - very important to users
        let photo_extensions = ["jpg", "jpeg", "png", "gif", "bmp", "webp", "raw", "cr2", "nef", "arw"];
        if photo_extensions.contains(&ext.as_str()) {
            return (FileCategory::Regular, 95.0);
        }

        // Videos - important but large
        let video_extensions = ["mp4", "avi", "mkv", "mov", "wmv", "flv", "webm", "m4v"];
        if video_extensions.contains(&ext.as_str()) {
            // Larger videos slightly less useful (more likely to be deletable)
            let usefulness = if size > 1_000_000_000 { 70.0 } else { 85.0 };
            return (FileCategory::Regular, usefulness);
        }

        // Audio - important
        let audio_extensions = ["mp3", "wav", "flac", "ogg", "aac", "m4a", "wma"];
        if audio_extensions.contains(&ext.as_str()) {
            return (FileCategory::Regular, 80.0);
        }

        // Code and projects - important for developers
        let code_extensions = ["rs", "py", "js", "ts", "java", "c", "cpp", "h", "cs", "go",
                              "html", "css", "json", "xml", "yaml", "toml", "sql"];
        if code_extensions.contains(&ext.as_str()) {
            return (FileCategory::Regular, 85.0);
        }

        // Archives - depends on size, often can be deleted after extraction
        let archive_extensions = ["zip", "rar", "7z", "tar", "gz", "bz2"];
        if archive_extensions.contains(&ext.as_str()) {
            let usefulness = if size > 1_000_000_000 { 30.0 }  // >1GB - likely can delete
                            else if size > 100_000_000 { 45.0 }  // >100MB
                            else { 55.0 };
            return (FileCategory::Regular, usefulness);
        }

        // ISOs and disk images - usually can be deleted
        if ext == "iso" || ext == "dmg" || ext == "img" {
            return (FileCategory::Regular, 25.0);
        }

        // Executables and installers - often safe to delete after install
        let installer_extensions = ["exe", "msi", "bat", "cmd", "ps1"];
        if installer_extensions.contains(&ext.as_str()) {
            // Installers in Downloads are less useful
            if path_str.contains("downloads") {
                return (FileCategory::Regular, 35.0);
            }
            return (FileCategory::Regular, 60.0);
        }

        // Old backup files
        if name_lower.ends_with(".bak") || name_lower.ends_with(".old") || name_lower.contains("backup") {
            return (FileCategory::Regular, 40.0);
        }

        // Folders - base usefulness on contents
        if is_dir {
            // Node modules, build folders - low usefulness
            if name_lower == "node_modules" || name_lower == "target" ||
               name_lower == "build" || name_lower == "dist" || name_lower == ".git" {
                return (FileCategory::Regular, 30.0);
            }
            // User folders - high usefulness
            if name_lower == "documents" || name_lower == "pictures" ||
               name_lower == "music" || name_lower == "videos" {
                return (FileCategory::Regular, 95.0);
            }
            // Downloads - medium, often contains deletable files
            if name_lower == "downloads" {
                return (FileCategory::Regular, 50.0);
            }
            // Default folder usefulness
            return (FileCategory::Regular, 65.0);
        }

        // Default for unknown files - base on size
        let usefulness = if size > 500_000_000 { 45.0 }  // >500MB - might want to check
                        else if size > 100_000_000 { 55.0 }  // >100MB
                        else { 60.0 };
        (FileCategory::Regular, usefulness)
    }

    fn sort_file_items(&mut self) {
        self.filtered_items.sort_by(|a, b| {
            // Directories always first
            match (a.is_dir, b.is_dir) {
                (true, false) => return std::cmp::Ordering::Less,
                (false, true) => return std::cmp::Ordering::Greater,
                _ => {}
            }

            let ordering = match self.sort_column {
                SortColumn::Name => a.name.cmp(&b.name),
                SortColumn::Size => a.size.cmp(&b.size),
                SortColumn::Category => {
                    let a_val = a.category as u8;
                    let b_val = b.category as u8;
                    a_val.cmp(&b_val)
                },
                SortColumn::Usefulness => a.usefulness.partial_cmp(&b.usefulness).unwrap_or(std::cmp::Ordering::Equal),
            };

            match self.sort_direction {
                SortDirection::Ascending => ordering,
                SortDirection::Descending => ordering.reverse(),
            }
        });
    }

    fn navigate_to(&mut self, path: PathBuf) {
        // Add to history
        if let Some(ref current) = self.current_path {
            if current != &path {
                // Remove any forward history
                self.navigation_history.truncate(self.history_index + 1);
                self.navigation_history.push(current.clone());
                self.history_index = self.navigation_history.len() - 1;
            }
        }
        self.current_path = Some(path);
    }

    fn navigate_back(&mut self) {
        if self.history_index > 0 {
            self.history_index -= 1;
            if let Some(path) = self.navigation_history.get(self.history_index) {
                self.current_path = Some(path.clone());
            }
        } else if let Some(parent) = self.current_path.as_ref().and_then(|p| p.parent()) {
            self.current_path = Some(parent.to_path_buf());
        }
    }

    fn navigate_forward(&mut self) {
        if self.history_index < self.navigation_history.len() - 1 {
            self.history_index += 1;
            if let Some(path) = self.navigation_history.get(self.history_index) {
                self.current_path = Some(path.clone());
            }
        }
    }

    /// Get folder size - returns cached value or starts async calculation
    fn get_folder_size_recursive(&mut self, path: &Path) -> u64 {
        // Check cache first
        if let Some(&size) = self.folder_size_cache.get(path) {
            return size;
        }

        // Check if calculation is already pending
        if self.pending_size_calculations.contains(path) {
            return 0; // Return 0 while calculating
        }

        // Start async calculation
        let path_buf = path.to_path_buf();
        let sender = self.size_sender.clone();
        self.pending_size_calculations.insert(path_buf.clone());

        thread::spawn(move || {
            let size = calculate_dir_size_recursive(&path_buf);
            let _ = sender.send((path_buf, size));
        });

        0 // Return 0 while calculating
    }

    fn render_file_browser(&mut self, ui: &mut egui::Ui, current_path: &Path) {
        // Modern header with breadcrumb and search
        egui::Frame::default()
            .fill(egui::Color32::from_rgb(25, 25, 32))
            .inner_margin(egui::Margin::same(12.0))
            .rounding(8.0)
            .show(ui, |ui| {
                // Breadcrumb navigation
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("📍 ").size(16.0).color(egui::Color32::from_rgb(100, 150, 255)));
                    let path_str = current_path.to_string_lossy();
                    let parts: Vec<&str> = path_str.split('\\').collect();
                    for (i, part) in parts.iter().enumerate() {
                        if i > 0 {
                            ui.label(egui::RichText::new(" / ").color(egui::Color32::from_gray(100)));
                        }
                        if ui.link(egui::RichText::new(part.to_string())
                            .color(egui::Color32::from_rgb(150, 200, 255)))
                            .clicked() {
                            let new_path = parts[..=i].join("\\");
                            self.navigate_to(PathBuf::from(&new_path));
                        }
                    }
                });
                
                ui.add_space(10.0);
                
                // Search bar
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("🔍").size(16.0));
                    let search_response = ui.add(
                        egui::TextEdit::singleline(&mut self.search_query)
                            .desired_width(300.0)
                    );
                    
                    if self.search_query.is_empty() && !search_response.has_focus() {
                        ui.painter().text(
                            search_response.rect.left_top() + egui::Vec2::new(8.0, 8.0),
                            egui::Align2::LEFT_TOP,
                            "Search files... (Ctrl+F)",
                            egui::FontId::default(),
                            egui::Color32::from_gray(100),
                        );
                    }
                    
                    if search_response.changed() {
                        self.apply_filter_and_sort();
                    }
                    
                    if !self.search_query.is_empty() {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("✕").clicked() {
                                self.search_query.clear();
                                self.apply_filter_and_sort();
                            }
                        });
                    }
                    
                    if !self.search_query.is_empty() {
                        ui.label(format!("({} results)", self.filtered_items.len()));
                    }
                });
            });
        
        ui.add_space(10.0);

        // File list with header inside ScrollArea for consistent width
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                // Header row (inside ScrollArea for same width as content)
                egui::Frame::default()
                    .fill(egui::Color32::from_rgb(25, 25, 32))
                    .inner_margin(egui::Margin::same(10.0))
                    .rounding(4.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // Add same spacing as content rows (icon 18px + space 12px = 30px)
                            ui.add_space(30.0);

                            // Name column
                            let name_clicked = ui.selectable_label(
                                self.sort_column == SortColumn::Name,
                                format!("Name {}", if self.sort_column == SortColumn::Name && self.sort_direction == SortDirection::Ascending { "▲" } else if self.sort_column == SortColumn::Name { "▼" } else { "" })
                            ).clicked();
                            if name_clicked {
                                if self.sort_column == SortColumn::Name {
                                    self.sort_direction = match self.sort_direction {
                                        SortDirection::Ascending => SortDirection::Descending,
                                        SortDirection::Descending => SortDirection::Ascending,
                                    };
                                } else {
                                    self.sort_column = SortColumn::Name;
                                    self.sort_direction = SortDirection::Ascending;
                                }
                                self.sort_file_items();
                            }

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                // Fixed width columns matching content layout (right to left)

                                // Usefulness column - 60px
                                ui.allocate_ui_with_layout(
                                    egui::Vec2::new(60.0, 20.0),
                                    egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                                    |ui| {
                                        let arrow = if self.sort_column == SortColumn::Usefulness && self.sort_direction == SortDirection::Ascending { "▲" } else if self.sort_column == SortColumn::Usefulness { "▼" } else { "" };
                                        if ui.selectable_label(self.sort_column == SortColumn::Usefulness, format!("Use {}", arrow)).clicked() {
                                            if self.sort_column == SortColumn::Usefulness {
                                                self.sort_direction = match self.sort_direction {
                                                    SortDirection::Ascending => SortDirection::Descending,
                                                    SortDirection::Descending => SortDirection::Ascending,
                                                };
                                            } else {
                                                self.sort_column = SortColumn::Usefulness;
                                                self.sort_direction = SortDirection::Ascending;
                                            }
                                            self.sort_file_items();
                                        }
                                    }
                                );

                                // Size column - 75px
                                ui.allocate_ui_with_layout(
                                    egui::Vec2::new(75.0, 20.0),
                                    egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                                    |ui| {
                                        let arrow = if self.sort_column == SortColumn::Size && self.sort_direction == SortDirection::Ascending { "▲" } else if self.sort_column == SortColumn::Size { "▼" } else { "" };
                                        if ui.selectable_label(self.sort_column == SortColumn::Size, format!("Size {}", arrow)).clicked() {
                                            if self.sort_column == SortColumn::Size {
                                                self.sort_direction = match self.sort_direction {
                                                    SortDirection::Ascending => SortDirection::Descending,
                                                    SortDirection::Descending => SortDirection::Ascending,
                                                };
                                            } else {
                                                self.sort_column = SortColumn::Size;
                                                self.sort_direction = SortDirection::Ascending;
                                            }
                                            self.sort_file_items();
                                        }
                                    }
                                );

                                // Category column - 90px (left-aligned to match badges)
                                ui.allocate_ui_with_layout(
                                    egui::Vec2::new(90.0, 20.0),
                                    egui::Layout::left_to_right(egui::Align::Center),
                                    |ui| {
                                        let arrow = if self.sort_column == SortColumn::Category && self.sort_direction == SortDirection::Ascending { "▲" } else if self.sort_column == SortColumn::Category { "▼" } else { "" };
                                        if ui.selectable_label(self.sort_column == SortColumn::Category, format!("Cat {}", arrow)).clicked() {
                                            if self.sort_column == SortColumn::Category {
                                                self.sort_direction = match self.sort_direction {
                                                    SortDirection::Ascending => SortDirection::Descending,
                                                    SortDirection::Descending => SortDirection::Ascending,
                                                };
                                            } else {
                                                self.sort_column = SortColumn::Category;
                                                self.sort_direction = SortDirection::Ascending;
                                            }
                                            self.sort_file_items();
                                        }
                                    }
                                );
                            });
                        });
                    });
                ui.add_space(8.0);

                // Back button for directories
                if let Some(parent) = current_path.parent() {
                    if ui.button(format!("⬆️ .. ({})", parent.to_string_lossy())).clicked() {
                        self.navigate_to(parent.to_path_buf());
                    }
                    ui.separator();
                }

                // Show loading indicator
                if self.loading {
                    ui.centered_and_justified(|ui| {
                        ui.spinner();
                        ui.label("Loading...");
                    });
                } else if self.filtered_items.is_empty() {
                    ui.centered_and_justified(|ui| {
                        if self.search_query.is_empty() {
                            ui.label(egui::RichText::new("No files found").color(egui::Color32::from_gray(150)));
                        } else {
                            ui.label(egui::RichText::new(format!("No results for \"{}\"", self.search_query))
                                .color(egui::Color32::from_gray(150)));
                        }
                    });
                } else {
                    let items_clone: Vec<FileItem> = self.filtered_items.clone();
                    for (idx, item) in items_clone.iter().enumerate() {
                        self.render_file_item(ui, item, idx);
                    }
                }
            });
    }

    fn render_file_item(&mut self, ui: &mut egui::Ui, item: &FileItem, _index: usize) {

        let category_text = match item.category {
            FileCategory::MustKeep => "Must Keep",
            FileCategory::System => "System",
            FileCategory::Regular => "Regular",
            FileCategory::Useless => "Useless",
            FileCategory::Unknown => "Unknown",
        };

        let category_color = match item.category {
            FileCategory::MustKeep => egui::Color32::from_rgb(50, 200, 50),
            FileCategory::System => egui::Color32::from_rgb(100, 150, 255),
            FileCategory::Regular => egui::Color32::from_rgb(200, 200, 200),
            FileCategory::Useless => egui::Color32::from_rgb(255, 100, 100),
            FileCategory::Unknown => egui::Color32::from_gray(150),
        };

        let usefulness_color = if item.usefulness < 20.0 {
            egui::Color32::from_rgb(255, 100, 100) // Red - low usefulness
        } else if item.usefulness < 50.0 {
            egui::Color32::from_rgb(255, 165, 0) // Orange
        } else if item.usefulness < 80.0 {
            egui::Color32::from_rgb(255, 255, 100) // Yellow
        } else {
            egui::Color32::from_rgb(100, 255, 100) // Green - high usefulness
        };

        let is_calculating = self.pending_size_calculations.contains(&item.path);
        let size_str = if item.is_dir {
            if item.size > 0 {
                format_size(item.size) // Show calculated folder size
            } else if is_calculating {
                "⏳".to_string() // Show loading indicator
            } else {
                match item.child_count {
                    Some(0) => "Empty".to_string(),
                    Some(n) => format!("{} items", n),
                    None => "—".to_string(),
                }
            }
        } else {
            format_size(item.size) // Show file size
        };

        let is_empty_folder = item.is_dir && item.child_count == Some(0);

        // Calculate max size in current directory for progress bar
        let max_size_in_dir = self.filtered_items.iter()
            .map(|i| i.size)
            .max()
            .unwrap_or(1)
            .max(1); // Avoid division by zero

        // Create interactive area for hover detection BEFORE drawing
        let item_id = ui.make_persistent_id(format!("file_item_{}", item.path.to_string_lossy()));
        let item_rect = ui.available_rect_before_wrap();
        let interact_rect = egui::Rect::from_min_size(item_rect.min, egui::Vec2::new(ui.available_width(), 44.0));
        let sense = egui::Sense::click().union(egui::Sense::hover());
        let interact_response = ui.interact(interact_rect, item_id, sense);
        let is_hovered = interact_response.hovered();

        // Modern clean file item design with proper hover
        // Empty folders get a muted appearance
        let base_fill = if is_empty_folder {
            egui::Color32::from_rgb(22, 24, 28) // Darker for empty
        } else {
            egui::Color32::from_rgb(25, 27, 32)
        };

        let hover_fill = if is_empty_folder {
            egui::Color32::from_rgb(35, 38, 45) // Less bright hover for empty
        } else {
            egui::Color32::from_rgb(40, 45, 58)
        };

        // Calculate size ratio for background progress bar
        let size_ratio = if item.size > 0 && max_size_in_dir > 0 {
            (item.size as f32 / max_size_in_dir as f32).min(1.0)
        } else {
            0.0
        };

        let frame_response = egui::Frame::default()
            .fill(if is_hovered { hover_fill } else { base_fill })
            .stroke(if is_hovered {
                if is_empty_folder {
                    egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 80, 90)) // Gray for empty
                } else {
                    egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 120, 180))
                }
            } else {
                egui::Stroke::new(1.0, egui::Color32::from_rgb(35, 38, 45))
            })
            .rounding(6.0)
            .inner_margin(egui::Margin::same(10.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Icon column - different icons for folders and file types
                    let icon_size = 18.0;
                    let icon_text = if item.is_dir {
                        if is_empty_folder { "📂" } else { "📁" }
                    } else {
                        // Get file extension for icon selection
                        let ext = item.path.extension()
                            .and_then(|e| e.to_str())
                            .map(|e| e.to_lowercase())
                            .unwrap_or_default();

                        match ext.as_str() {
                            // Images
                            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "svg" | "ico" => "🖼️",
                            // Videos
                            "mp4" | "avi" | "mkv" | "mov" | "wmv" | "flv" | "webm" => "🎬",
                            // Audio
                            "mp3" | "wav" | "flac" | "ogg" | "aac" | "m4a" => "🎵",
                            // Documents
                            "pdf" => "📕",
                            "doc" | "docx" => "📘",
                            "xls" | "xlsx" => "📗",
                            "ppt" | "pptx" => "📙",
                            "txt" | "md" | "rtf" => "📝",
                            // Code
                            "rs" | "py" | "js" | "ts" | "java" | "c" | "cpp" | "h" | "cs" | "go" => "💻",
                            "html" | "css" | "json" | "xml" | "yaml" | "toml" => "🌐",
                            // Archives
                            "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" => "📦",
                            // Executables
                            "exe" | "msi" | "bat" | "cmd" | "ps1" | "sh" => "⚡",
                            // Default by category
                            _ => match item.category {
                                FileCategory::MustKeep => "🔒",
                                FileCategory::System => "⚙️",
                                FileCategory::Regular => "📄",
                                FileCategory::Useless => "🗑️",
                                FileCategory::Unknown => "❓",
                            }
                        }
                    };

                    ui.label(egui::RichText::new(icon_text).size(icon_size));
                    ui.add_space(12.0);
                    
                    // Name column - different colors for empty folders
                    let name_color = if item.is_dir {
                        if is_empty_folder {
                            egui::Color32::from_rgb(120, 140, 160) // Dimmer for empty
                        } else {
                            egui::Color32::from_rgb(150, 200, 255) // Bright for non-empty
                        }
                    } else {
                        egui::Color32::from_rgb(220, 220, 220)
                    };

                    // Use regular label instead of selectable_label to avoid conflicting hover styles
                    let name_response = ui.add(
                        egui::Label::new(egui::RichText::new(&item.name)
                            .size(13.0)
                            .color(name_color))
                        .sense(egui::Sense::click())
                    );

                    // Name click - navigate for folders, open for files
                    if name_response.clicked() {
                        if item.is_dir {
                            if is_empty_folder {
                                self.toast_message = Some(("📂 This folder is empty".to_string(), 2.0));
                            } else {
                                self.navigate_to(item.path.clone());
                            }
                        } else {
                            // Open file with default application
                            #[cfg(target_os = "windows")]
                            {
                                let _ = std::process::Command::new("cmd")
                                    .args(["/C", "start", "", &item.path.to_string_lossy()])
                                    .spawn();
                            }
                            self.toast_message = Some((format!("📄 Opening {}", item.name), 1.5));
                        }
                    }

                    // Use the name_response for context menu instead of separate label
                    let name_label = name_response;
                    
                    // Right-click context menu
                    let item_path = item.path.clone();
                    let item_is_dir = item.is_dir;
                    name_label.context_menu(|ui| {
                        // Open in Explorer
                        if ui.button("📂 Open in Explorer").clicked() {
                            #[cfg(target_os = "windows")]
                            {
                                let path = if item_is_dir {
                                    item_path.clone()
                                } else {
                                    item_path.parent().unwrap_or(&item_path).to_path_buf()
                                };
                                let _ = std::process::Command::new("explorer")
                                    .arg(&path)
                                    .spawn();
                            }
                            ui.close_menu();
                        }

                        ui.separator();

                        // Copy path to clipboard
                        if ui.button("📋 Copy Path").clicked() {
                            ui.output_mut(|o| o.copied_text = item_path.to_string_lossy().to_string());
                            ui.close_menu();
                        }

                        // Only show delete option for non-protected items
                        let item_category = item.category;
                        if item_category != FileCategory::MustKeep && item_category != FileCategory::System {
                            ui.separator();

                            // Delete option (for both files and folders)
                            let delete_label = if item_is_dir {
                                "🗑️ Delete Folder"
                            } else {
                                "🗑️ Delete File"
                            };

                            if ui.add(egui::Button::new(
                                egui::RichText::new(delete_label)
                                    .color(egui::Color32::from_rgb(255, 100, 100)))
                            ).clicked() {
                                self.pending_delete = Some(item_path.clone());
                                ui.close_menu();
                            }
                        } else {
                            ui.separator();
                            ui.label(egui::RichText::new("🔒 Protected")
                                .size(11.0)
                                .color(egui::Color32::from_rgb(150, 150, 150)));
                        }
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Fixed width columns for alignment (right to left)
                        // Must match header widths: 60, 75, 50, 80

                        // Usefulness score - fixed 60px
                        ui.allocate_ui_with_layout(
                            egui::Vec2::new(60.0, 20.0),
                            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                            |ui| {
                                ui.label(egui::RichText::new(format!("{:.0}%", item.usefulness))
                                    .size(11.0)
                                    .color(usefulness_color)
                                    .strong());
                            }
                        );

                        // Size column - fixed 75px
                        ui.allocate_ui_with_layout(
                            egui::Vec2::new(75.0, 20.0),
                            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                            |ui| {
                                ui.label(egui::RichText::new(&size_str)
                                    .size(11.0)
                                    .color(egui::Color32::from_gray(160)));
                            }
                        );

                        // Category badge - fixed 90px (left-aligned to match header)
                        ui.allocate_ui_with_layout(
                            egui::Vec2::new(90.0, 20.0),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                let badge_frame = egui::Frame::default()
                                    .fill(egui::Color32::from_rgb(25, 25, 35))
                                    .stroke(egui::Stroke::new(1.0, category_color))
                                    .rounding(6.0)
                                    .inner_margin(egui::Margin::symmetric(6.0, 3.0));

                                badge_frame.show(ui, |ui| {
                                    ui.label(egui::RichText::new(category_text)
                                        .size(9.0)
                                        .color(category_color));
                                });
                            }
                        );
                    });
                });
            });

        // Draw progress bar overlay on top of the frame
        if size_ratio > 0.0 {
            let frame_rect = frame_response.response.rect;
            let bar_height = frame_rect.height() - 8.0; // Slightly smaller than row
            let bar_rect = egui::Rect::from_min_size(
                egui::Pos2::new(frame_rect.min.x + 4.0, frame_rect.min.y + 4.0),
                egui::Vec2::new((frame_rect.width() - 8.0) * size_ratio, bar_height)
            );
            let bar_color = if size_ratio > 0.8 {
                egui::Color32::from_rgba_unmultiplied(255, 80, 80, 25) // Red tint
            } else if size_ratio > 0.5 {
                egui::Color32::from_rgba_unmultiplied(255, 180, 80, 20) // Orange tint
            } else {
                egui::Color32::from_rgba_unmultiplied(80, 150, 255, 20) // Blue tint
            };
            ui.painter().rect_filled(bar_rect, 4.0, bar_color);
        }

        // Handle click on entire row
        if interact_response.clicked() {
            if item.is_dir {
                if is_empty_folder {
                    // Show toast message instead of navigating
                    self.toast_message = Some(("📂 This folder is empty".to_string(), 2.0));
                } else {
                    self.navigate_to(item.path.clone());
                }
            } else {
                // Open file with default application
                #[cfg(target_os = "windows")]
                {
                    let _ = std::process::Command::new("cmd")
                        .args(["/C", "start", "", &item.path.to_string_lossy()])
                        .spawn();
                }
                self.toast_message = Some((format!("📄 Opening {}", item.name), 1.5));
            }
        }

        // Hover tooltip with file information
        if is_hovered {
            interact_response.on_hover_ui(|ui| {
                ui.set_max_width(300.0);
                ui.label(egui::RichText::new("File Information").strong().size(14.0));
                ui.separator();
                ui.label(format!("Path: {}", item.path.to_string_lossy()));
                if !item.is_dir {
                    ui.label(format!("Size: {}", format_size(item.size)));
                }
                ui.label(format!("Category: {} {}", category_text,
                    if item.category == FileCategory::MustKeep { "🔒" }
                    else if item.category == FileCategory::Useless { "⚠️" }
                    else { "" }));
                ui.label(format!("Usefulness: {:.1}%", item.usefulness));
                if let Some(modified) = item.modified {
                    if let Ok(duration) = modified.duration_since(SystemTime::UNIX_EPOCH) {
                        let secs = duration.as_secs();
                        let days = secs / 86400;
                        ui.label(format!("Modified: {} days ago", days));
                    }
                }
                if item.category == FileCategory::Useless {
                    ui.separator();
                    ui.label(egui::RichText::new("⚠️ This file is marked as potentially useless and may be safe to delete.")
                        .color(egui::Color32::from_rgb(255, 165, 0)));
                }
                if item.category == FileCategory::MustKeep {
                    ui.separator();
                    ui.label(egui::RichText::new("🔒 This is a critical system file. Do not delete.")
                        .color(egui::Color32::from_rgb(50, 200, 50)));
                }
            });
        }

        // Add spacing between items
        ui.add_space(4.0);
    }

    fn render_pie_chart(&self, ui: &mut egui::Ui, disk_data: &[(PathBuf, String, u64, u64, f64)], total_space: u64, total_used: u64, _total_available: u64, avg_usage: f64) {
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
                
                // Used space (red/orange/green based on usage)
                let used_color = if avg_usage > 90.0 {
                    egui::Color32::from_rgb(220, 50, 50)
                } else if avg_usage > 75.0 {
                    egui::Color32::from_rgb(255, 165, 0)
                } else {
                    egui::Color32::from_rgb(100, 200, 100)
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

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

/// Calculate the total size of a directory (non-recursive, just immediate children)
fn calculate_dir_size_shallow(path: &Path) -> u64 {
    fs::read_dir(path)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter_map(|e| e.metadata().ok())
                .filter(|m| m.is_file())
                .map(|m| m.len())
                .sum()
        })
        .unwrap_or(0)
}

/// Calculate the total size of a directory recursively (with depth limit)
fn calculate_dir_size_recursive(path: &Path) -> u64 {
    calculate_dir_size_recursive_limited(path, 2) // Limit to 2 levels to avoid UI freeze
}

/// Calculate directory size with depth limit to prevent crashes
fn calculate_dir_size_recursive_limited(path: &Path, max_depth: u32) -> u64 {
    if max_depth == 0 {
        // At max depth, just return shallow size
        return calculate_dir_size_shallow(path);
    }

    let mut total_size: u64 = 0;

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    total_size = total_size.saturating_add(metadata.len());
                } else if metadata.is_dir() {
                    // Skip system folders that might cause issues
                    let name = entry.file_name();
                    let name_str = name.to_string_lossy().to_lowercase();
                    if name_str.starts_with("$") ||
                       name_str == "system volume information" ||
                       name_str == "windows" {
                        continue;
                    }
                    // Recursively calculate subdirectory size
                    total_size = total_size.saturating_add(
                        calculate_dir_size_recursive_limited(&entry.path(), max_depth - 1)
                    );
                }
            }
        }
    }

    total_size
}

/// Check if a folder should block navigation (empty folder)
#[allow(dead_code)] // Used in tests
fn should_block_folder_entry(child_count: Option<usize>) -> bool {
    child_count == Some(0)
}

/// Check if a path is a protected system path
#[allow(dead_code)] // Used in tests
fn is_protected_path(name: &str) -> bool {
    let name_lower = name.to_lowercase();
    name_lower.starts_with("$") ||
    name_lower == "system volume information" ||
    name_lower == "recovery" ||
    name_lower == "boot" ||
    name_lower == "bootmgr" ||
    name_lower == "pagefile.sys" ||
    name_lower == "hiberfil.sys"
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Size Formatting Tests ====================

    #[test]
    fn test_format_size_bytes() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1023), "1023 B");
    }

    #[test]
    fn test_format_size_kilobytes() {
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(1024 * 100), "100.0 KB");
    }

    #[test]
    fn test_format_size_megabytes() {
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_size(1024 * 1024 * 500), "500.0 MB");
    }

    #[test]
    fn test_format_size_gigabytes() {
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
        assert_eq!(format_size(1024 * 1024 * 1024 * 2), "2.00 GB");
    }

    // ==================== Empty Folder Navigation Tests ====================

    #[test]
    fn test_empty_folder_blocks_navigation() {
        // Empty folder (0 items) should block
        assert!(should_block_folder_entry(Some(0)));
    }

    #[test]
    fn test_non_empty_folder_allows_navigation() {
        // Non-empty folders should allow navigation
        assert!(!should_block_folder_entry(Some(1)));
        assert!(!should_block_folder_entry(Some(10)));
        assert!(!should_block_folder_entry(Some(100)));
    }

    #[test]
    fn test_unknown_folder_count_allows_navigation() {
        // If count is unknown, allow navigation (fail open)
        assert!(!should_block_folder_entry(None));
    }

    // ==================== Protected Path Tests ====================

    #[test]
    fn test_recycle_bin_is_protected() {
        assert!(is_protected_path("$RECYCLE.BIN"));
        assert!(is_protected_path("$Recycle.Bin"));
        assert!(is_protected_path("$recycle.bin"));
    }

    #[test]
    fn test_system_volume_info_is_protected() {
        assert!(is_protected_path("System Volume Information"));
        assert!(is_protected_path("system volume information"));
    }

    #[test]
    fn test_system_files_are_protected() {
        assert!(is_protected_path("pagefile.sys"));
        assert!(is_protected_path("hiberfil.sys"));
        assert!(is_protected_path("bootmgr"));
        assert!(is_protected_path("Recovery"));
    }

    #[test]
    fn test_dollar_prefix_is_protected() {
        assert!(is_protected_path("$WinREAgent"));
        assert!(is_protected_path("$SysReset"));
        assert!(is_protected_path("$Windows.~BT"));
    }

    #[test]
    fn test_normal_folders_not_protected() {
        assert!(!is_protected_path("Documents"));
        assert!(!is_protected_path("Users"));
        assert!(!is_protected_path("Program Files"));
        assert!(!is_protected_path("my_project"));
    }

    // ==================== File Category Tests ====================

    #[test]
    fn test_file_item_empty_folder_detection() {
        let empty_item = FileItem {
            path: PathBuf::from("C:\\test\\empty"),
            name: "empty".to_string(),
            size: 0,
            is_dir: true,
            category: FileCategory::Regular,
            usefulness: 50.0,
            modified: None,
            child_count: Some(0),
        };
        assert!(empty_item.child_count == Some(0));

        let non_empty_item = FileItem {
            path: PathBuf::from("C:\\test\\full"),
            name: "full".to_string(),
            size: 1000,
            is_dir: true,
            category: FileCategory::Regular,
            usefulness: 50.0,
            modified: None,
            child_count: Some(5),
        };
        assert!(non_empty_item.child_count != Some(0));
    }

    // ==================== UI State Tests ====================

    #[test]
    fn test_toast_message_creation() {
        let toast: Option<(String, f32)> = Some(("Test message".to_string(), 2.0));
        assert!(toast.is_some());
        let (msg, time) = toast.unwrap();
        assert_eq!(msg, "Test message");
        assert_eq!(time, 2.0);
    }

    #[test]
    fn test_toast_expiry() {
        let mut time_left: f32 = 2.0;

        // Simulate time passing
        time_left -= 0.5;
        assert!(time_left > 0.0); // Still visible

        time_left -= 1.5;
        assert!(time_left <= 0.0); // Should be hidden
    }

    // ==================== Navigation State Tests ====================

    #[test]
    fn test_navigation_history_tracking() {
        let mut history: Vec<PathBuf> = Vec::new();
        let mut index: usize = 0;

        // Navigate to first path
        history.push(PathBuf::from("C:\\"));
        index = history.len() - 1;
        assert_eq!(index, 0);

        // Navigate to second path
        history.push(PathBuf::from("C:\\Users"));
        index = history.len() - 1;
        assert_eq!(index, 1);

        // Go back
        if index > 0 {
            index -= 1;
        }
        assert_eq!(index, 0);
        assert_eq!(history[index], PathBuf::from("C:\\"));
    }
}
