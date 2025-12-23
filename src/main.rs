use eframe::egui;
use sysinfo::Disks;
use std::path::{Path, PathBuf};
use std::fs;
use std::time::SystemTime;

#[derive(Clone, Copy, PartialEq, Debug)]
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
    size: u64,
    is_dir: bool,
    category: FileCategory,
    usefulness: f32, // 0-100 score
    modified: Option<SystemTime>,
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
    hovered_item: Option<usize>,
}

impl Default for DiskDashboard {
    fn default() -> Self {
        Self {
            disks: Disks::new_with_refreshed_list(),
            refresh_interval: 1.0,
            time_since_refresh: 0.0,
            current_path: None,
            current_disk: None,
            file_items: Vec::new(),
            filtered_items: Vec::new(),
            loading: false,
            sort_column: SortColumn::Usefulness,
            sort_direction: SortDirection::Ascending,
            navigation_history: Vec::new(),
            history_index: 0,
            search_query: String::new(),
            hovered_item: None,
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_title("Disk Capacity Dashboard")
            .with_decorations(true)
            .with_resizable(true),
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
        self.time_since_refresh += ctx.input(|i| i.stable_dt);
        if self.time_since_refresh >= self.refresh_interval {
            self.disks.refresh();
            self.time_since_refresh = 0.0;
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

        // Refresh file list if path changed
        let path_to_load = self.current_path.clone();
        if let Some(ref path) = path_to_load {
            if !self.loading {
                self.load_directory(path);
            }
        }

        // Apply modern theme
        self.apply_modern_theme(ctx);

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
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.heading("Disks");
                ui.separator();
                
                let mut disk_data: Vec<(PathBuf, u64, u64, f64)> = self.disks.list().iter()
                    .map(|d| {
                        let mount = d.mount_point().to_path_buf();
                        let total = d.total_space();
                        let available = d.available_space();
                        let used = total - available;
                        let percent = if total > 0 {
                            (used as f64 / total as f64) * 100.0
                        } else {
                            0.0
                        };
                        (mount, total, available, percent)
                    })
                    .collect();
                
                disk_data.sort_by(|a, b| a.0.to_string_lossy().cmp(&b.0.to_string_lossy()));

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (mount_point, total, available, percent) in &disk_data {
                        // Skip empty/invalid entries
                        if *total == 0 {
                            continue;
                        }
                        let mount_clone = mount_point.clone();
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
                        
                        // Different styling for selected disk
                        let card_fill = if is_selected {
                            egui::Color32::from_rgb(35, 45, 60)
                        } else {
                            egui::Color32::from_rgb(28, 30, 38)
                        };
                        
                        let border_color = if is_selected {
                            egui::Color32::from_rgb(100, 150, 255)
                        } else {
                            egui::Color32::from_rgb(45, 48, 55)
                        };
                        
                        let border_width = if is_selected { 2.0 } else { 1.0 };
                        
                        let disk_card_response = egui::Frame::default()
                            .fill(card_fill)
                            .stroke(egui::Stroke::new(border_width, border_color))
                            .rounding(8.0)
                            .inner_margin(egui::Margin::same(14.0))
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    // Drive name
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new("💿").size(20.0));
                                        ui.label(egui::RichText::new(mount_point.to_string_lossy().as_ref())
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
                        
                        // Hover effect handling
                        let is_hovered = disk_card_response.response.hovered();
                        
                        if is_selected {
                            // Selected state - subtle enhancement on hover
                            if is_hovered {
                                // Slightly brighter when hovering selected item
                                ui.painter().rect_filled(
                                    disk_card_response.response.rect,
                                    8.0,
                                    egui::Color32::from_rgb(38, 48, 62),
                                );
                            }
                            // Selected border always visible
                            ui.painter().rect_stroke(
                                disk_card_response.response.rect,
                                8.0,
                                egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 150, 255)),
                            );
                        } else if is_hovered {
                            // Hover effect for non-selected items only
                            ui.painter().rect_filled(
                                disk_card_response.response.rect,
                                8.0,
                                egui::Color32::from_rgb(38, 42, 50),
                            );
                            ui.painter().rect_stroke(
                                disk_card_response.response.rect,
                                8.0,
                                egui::Stroke::new(1.5, usage_color),
                            );
                        }
                        
                        // Make entire card clickable
                        if disk_card_response.response.clicked() {
                            self.navigate_to(mount_clone.clone());
                            self.current_disk = Some(mount_clone);
                            self.file_items.clear();
                            self.search_query.clear();
                        }
                        
                        ui.add_space(12.0);
                    }
                });

                // Calculate totals for summary and pie chart
                let total_disks = disk_data.len();
                let total_space: u64 = disk_data.iter().map(|(_, total, _, _)| *total).sum();
                let total_used: u64 = disk_data.iter().map(|(_, total, available, _)| *total - *available).sum();
                let total_available: u64 = disk_data.iter().map(|(_, _, available, _)| *available).sum();
                let avg_usage = if total_space > 0 {
                    (total_used as f64 / total_space as f64) * 100.0
                } else {
                    0.0
                };

                // Modern Summary panel
                ui.add_space(15.0);
                egui::Frame::default()
                    .fill(egui::Color32::from_rgb(25, 27, 35))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(50, 55, 65)))
                    .rounding(8.0)
                    .inner_margin(egui::Margin::same(12.0))
                    .show(ui, |ui| {
                        ui.heading(egui::RichText::new("Summary")
                            .size(16.0)
                            .color(egui::Color32::from_rgb(180, 200, 255)));
                        ui.add_space(10.0);

                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new(format!("{}", total_disks))
                                    .size(20.0)
                                    .strong()
                                    .color(egui::Color32::from_rgb(100, 200, 255)));
                                ui.label(egui::RichText::new("Disks")
                                    .size(11.0)
                                    .color(egui::Color32::from_gray(150)));
                            });
                            ui.add_space(20.0);
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new(format!("{:.1}", total_space as f64 / 1_000_000_000.0))
                                    .size(20.0)
                                    .strong()
                                    .color(egui::Color32::from_rgb(100, 200, 255)));
                                ui.label(egui::RichText::new("Total Space (GB)")
                                    .size(11.0)
                                    .color(egui::Color32::from_gray(150)));
                            });
                            ui.add_space(20.0);
                            ui.vertical(|ui| {
                                let used_color = if avg_usage > 90.0 {
                                    egui::Color32::from_rgb(220, 50, 50)
                                } else if avg_usage > 75.0 {
                                    egui::Color32::from_rgb(255, 165, 0)
                                } else {
                                    egui::Color32::from_rgb(50, 200, 50)
                                };
                                ui.label(egui::RichText::new(format!("{:.1}%", avg_usage))
                                    .size(20.0)
                                    .strong()
                                    .color(used_color));
                                ui.label(egui::RichText::new("Used")
                                    .size(11.0)
                                    .color(egui::Color32::from_gray(150)));
                            });
                        });
                    });
                
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);
                
                // Pie chart visualization
                if total_space > 0 {
                    let avg_usage = if total_space > 0 {
                        (total_used as f64 / total_space as f64) * 100.0
                    } else {
                        0.0
                    };
                    self.render_pie_chart(ui, &disk_data, total_space, total_used, total_available, avg_usage);
                }
            });

        let current_path_clone = self.current_path.clone();
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(ref path) = current_path_clone {
                let path_clone = path.clone();
                self.render_file_browser(ui, &path_clone);
            } else {
                self.render_disk_overview(ui);
            }
        });
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
                let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
                let modified = metadata.as_ref().and_then(|m| m.modified().ok());

                let name = entry_path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string();

                let (category, usefulness) = self.analyze_file(&entry_path, &name, is_dir, size);
                
                self.file_items.push(FileItem {
                    path: entry_path,
                    name,
                    size,
                    is_dir,
                    category,
                    usefulness,
                    modified,
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

    fn analyze_file(&self, path: &Path, name: &str, _is_dir: bool, size: u64) -> (FileCategory, f32) {
        let name_lower = name.to_lowercase();
        let path_str = path.to_string_lossy().to_lowercase();

        // System and critical files
        if path_str.contains("windows\\system32") ||
           path_str.contains("windows\\syswow64") ||
           path_str.contains("program files") ||
           path_str.contains("programdata") ||
           name == "boot" ||
           name == "bootmgr" ||
           name == "pagefile.sys" ||
           name == "hiberfil.sys" {
            return (FileCategory::MustKeep, 100.0);
        }

        // Temp files and cache - useless
        if name_lower.contains("temp") ||
           name_lower.contains("cache") ||
           name_lower.contains("tmp") ||
           name_lower.ends_with(".tmp") ||
           name_lower.ends_with(".log") ||
           name_lower.contains("recycle") ||
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

        // Large files that might be deletable
        if size > 1_000_000_000 && ( // > 1GB
           name_lower.ends_with(".zip") ||
           name_lower.ends_with(".rar") ||
           name_lower.ends_with(".7z") ||
           name_lower.ends_with(".iso") ||
           name_lower.ends_with(".dmg")) {
            return (FileCategory::Regular, 40.0);
        }

        // Regular files
        (FileCategory::Regular, 60.0)
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

        // Modern file list header with sortable columns
        egui::Frame::default()
            .fill(egui::Color32::from_rgb(25, 25, 32))
            .inner_margin(egui::Margin::same(10.0))
            .rounding(4.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
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
                // Usefulness column
                let usefulness_clicked = ui.selectable_label(
                    self.sort_column == SortColumn::Usefulness,
                    format!("Usefulness {}", if self.sort_column == SortColumn::Usefulness && self.sort_direction == SortDirection::Ascending { "▲" } else if self.sort_column == SortColumn::Usefulness { "▼" } else { "" })
                ).clicked();
                if usefulness_clicked {
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
                ui.add_space(20.0);

                // Size column
                let size_clicked = ui.selectable_label(
                    self.sort_column == SortColumn::Size,
                    format!("Size {}", if self.sort_column == SortColumn::Size && self.sort_direction == SortDirection::Ascending { "▲" } else if self.sort_column == SortColumn::Size { "▼" } else { "" })
                ).clicked();
                if size_clicked {
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
                ui.add_space(20.0);

                // Category column
                let category_clicked = ui.selectable_label(
                    self.sort_column == SortColumn::Category,
                    format!("Category {}", if self.sort_column == SortColumn::Category && self.sort_direction == SortDirection::Ascending { "▲" } else if self.sort_column == SortColumn::Category { "▼" } else { "" })
                ).clicked();
                if category_clicked {
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
                    });
                });
            });
        ui.add_space(8.0);

        // File list
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
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

        let size_str = if item.is_dir {
            "—".to_string()
        } else {
            format_size(item.size)
        };

        let hover_color = egui::Color32::from_rgb(50, 50, 65); // More visible hover color

        let item_rect = ui.available_rect_before_wrap();
        let is_hovered = ui.ctx().pointer_latest_pos()
            .map(|pos| item_rect.contains(pos))
            .unwrap_or(false);

        // Draw background highlight
        if is_hovered {
            ui.painter().rect_filled(
                item_rect,
                4.0,
                hover_color,
            );
        }

        // Modern clean file item design
        let frame_response = egui::Frame::default()
            .fill(if is_hovered { egui::Color32::from_rgb(35, 38, 45) } else { egui::Color32::TRANSPARENT })
            .stroke(if is_hovered { 
                egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 120, 180)) 
            } else { 
                egui::Stroke::NONE 
            })
            .rounding(4.0)
            .inner_margin(egui::Margin::same(10.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Icon column - cleaner icons
                    let icon_size = 18.0;
                    let icon_text = if item.is_dir {
                        "📁"
                    } else {
                        match item.category {
                            FileCategory::MustKeep => "🔒",
                            FileCategory::System => "⚙️",
                            FileCategory::Regular => "📄",
                            FileCategory::Useless => "🗑️",
                            FileCategory::Unknown => "❓",
                        }
                    };
                    
                    ui.label(egui::RichText::new(icon_text).size(icon_size));
                    ui.add_space(12.0);
                    
                    // Name column - cleaner, no extra icons cluttering
                    let name_label = ui.selectable_label(false, egui::RichText::new(&item.name)
                        .size(13.0)
                        .color(if item.is_dir { 
                            egui::Color32::from_rgb(150, 200, 255) 
                        } else { 
                            egui::Color32::from_rgb(220, 220, 220) 
                        }));
                    
                    if name_label.clicked() {
                        if item.is_dir {
                            self.navigate_to(item.path.clone());
                        }
                    }
                    
                    // Right-click context menu
                    name_label.context_menu(|ui| {
                        if ui.button("📂 Open in Explorer").clicked() {
                            ui.close_menu();
                        }
                        if !item.is_dir {
                            ui.separator();
                            if ui.button("📋 Copy Path").clicked() {
                                ui.close_menu();
                            }
                            if item.category == FileCategory::Useless {
                                ui.separator();
                                if ui.button("🗑️ Delete File").clicked() {
                                    ui.close_menu();
                                }
                            }
                        }
                        ui.separator();
                        if ui.button("ℹ️ Properties").clicked() {
                            ui.close_menu();
                        }
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Usefulness score - cleaner display
                        ui.label(egui::RichText::new(format!("{:.0}%", item.usefulness))
                            .size(12.0)
                            .color(usefulness_color)
                            .strong());
                        ui.add_space(25.0);

                        // Size - cleaner
                        ui.label(egui::RichText::new(size_str)
                            .size(12.0)
                            .color(egui::Color32::from_gray(180)));
                        ui.add_space(25.0);

                        // Category badge - modern pill design
                        let badge_frame = egui::Frame::default()
                            .fill(egui::Color32::from_rgb(25, 25, 35))
                            .stroke(egui::Stroke::new(1.0, category_color))
                            .rounding(8.0)
                            .inner_margin(egui::Margin::symmetric(8.0, 4.0));
                        
                        badge_frame.show(ui, |ui| {
                            ui.label(egui::RichText::new(category_text)
                                .size(10.0)
                                .color(category_color));
                        });
                    });
                });
            });

        // Hover tooltip with file information
        if frame_response.response.hovered() {
            frame_response.response.on_hover_ui(|ui| {
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
    }

    fn render_pie_chart(&self, ui: &mut egui::Ui, disk_data: &[(PathBuf, u64, u64, f64)], total_space: u64, total_used: u64, _total_available: u64, avg_usage: f64) {
        ui.add_space(15.0);
        egui::Frame::default()
            .fill(egui::Color32::from_rgb(25, 27, 35))
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(50, 55, 65)))
            .rounding(8.0)
            .inner_margin(egui::Margin::same(12.0))
            .show(ui, |ui| {
                ui.heading(egui::RichText::new("Disk Usage Breakdown")
                    .size(16.0)
                    .color(egui::Color32::from_rgb(180, 200, 255)));
                ui.add_space(10.0);
                
                let chart_size = 180.0;
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
                    
                    for (i, (mount_point, total, available, _percent)) in disk_data.iter().enumerate() {
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
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("●").color(color).size(12.0));
                            ui.label(format!("{}: {:.1}% ({:.1}% of total)", 
                                mount_point.to_string_lossy(), 
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
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}
