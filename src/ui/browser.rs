use eframe::egui;
use std::path::{Path, PathBuf};
use crate::DiskDashboard;
use crate::models::{SortColumn, SortDirection, FileItem};

impl DiskDashboard {
    fn toggle_sort(&mut self, column: SortColumn) {
        if self.sort_column == column {
            self.sort_direction = match self.sort_direction {
                SortDirection::Ascending => SortDirection::Descending,
                SortDirection::Descending => SortDirection::Ascending,
            };
        } else {
            self.sort_column = column;
            self.sort_direction = SortDirection::Ascending;
        }
        self.sort_file_items();
    }

    fn sort_arrow(&self, column: SortColumn) -> &'static str {
        if self.sort_column == column {
            match self.sort_direction {
                SortDirection::Ascending => "▲",
                SortDirection::Descending => "▼",
            }
        } else {
            ""
        }
    }

    pub(crate) fn render_file_browser(&mut self, ui: &mut egui::Ui, current_path: &Path) {
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
                            if ui.selectable_label(
                                self.sort_column == SortColumn::Name,
                                format!("Name {}", self.sort_arrow(SortColumn::Name))
                            ).clicked() {
                                self.toggle_sort(SortColumn::Name);
                            }

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                // Fixed width columns matching content layout (right to left)

                                // Usefulness column - 60px (rightmost)
                                ui.allocate_ui_with_layout(
                                    egui::Vec2::new(60.0, 20.0),
                                    egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                                    |ui| {
                                        if ui.selectable_label(self.sort_column == SortColumn::Usefulness, format!("Use {}", self.sort_arrow(SortColumn::Usefulness))).clicked() {
                                            self.toggle_sort(SortColumn::Usefulness);
                                        }
                                    }
                                );

                                // Category column - 90px (middle)
                                ui.allocate_ui_with_layout(
                                    egui::Vec2::new(90.0, 20.0),
                                    egui::Layout::left_to_right(egui::Align::Center),
                                    |ui| {
                                        if ui.selectable_label(self.sort_column == SortColumn::Category, format!("Cat {}", self.sort_arrow(SortColumn::Category))).clicked() {
                                            self.toggle_sort(SortColumn::Category);
                                        }
                                    }
                                );

                                // Size column - 75px (leftmost, most important)
                                ui.allocate_ui_with_layout(
                                    egui::Vec2::new(75.0, 20.0),
                                    egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                                    |ui| {
                                        if ui.selectable_label(self.sort_column == SortColumn::Size, format!("Size {}", self.sort_arrow(SortColumn::Size))).clicked() {
                                            self.toggle_sort(SortColumn::Size);
                                        }
                                    }
                                );
                            });
                        });
                    });
                ui.add_space(8.0);

                // Back button and selection actions
                ui.horizontal(|ui| {
                    if let Some(parent) = current_path.parent() {
                        if ui.button(format!("⬆️ .. ({})", parent.to_string_lossy())).clicked() {
                            self.navigate_to(parent.to_path_buf());
                        }
                    }

                    // Select All / Deselect All toggle
                    ui.separator();
                    let all_selected = !self.filtered_items.is_empty() && self.filtered_items.iter().all(|item| self.selected_items.contains(&item.path));
                    if all_selected {
                        if ui.add(egui::Button::new("☐ Deselect All")
                            .fill(egui::Color32::from_rgb(40, 30, 60)))
                            .clicked()
                        {
                            self.selected_items.clear();
                        }
                    } else {
                        if ui.add(egui::Button::new("☑ Select All")
                            .fill(egui::Color32::from_rgb(40, 30, 60)))
                            .clicked()
                        {
                            for item in &self.filtered_items {
                                self.selected_items.insert(item.path.clone());
                            }
                        }
                    }

                    // Show selection count and delete button when items selected
                    if !self.selected_items.is_empty() {
                        ui.separator();
                        ui.label(egui::RichText::new(format!("📋 {} selected", self.selected_items.len()))
                            .color(egui::Color32::from_rgb(0, 255, 255)));

                        if ui.add(egui::Button::new(
                            egui::RichText::new("🗑️ Delete Selection")
                                .color(egui::Color32::WHITE)
                                .strong())
                            .fill(egui::Color32::from_rgb(180, 30, 60)))
                            .clicked()
                        {
                            self.pending_batch_delete = true;
                        }

                        if ui.add(egui::Button::new("❌ Clear")
                            .fill(egui::Color32::from_rgb(40, 30, 60)))
                            .clicked()
                        {
                            self.selected_items.clear();
                        }
                    }
                });
                ui.separator();

                // Show loading indicator
                if self.loading {
                    ui.centered_and_justified(|ui| {
                        ui.spinner();
                        ui.label("Loading...");
                    });
                } else if self.filtered_items.is_empty() {
                    ui.centered_and_justified(|ui| {
                        if self.search_query.is_empty() {
                            ui.label(egui::RichText::new("No files found").color(egui::Color32::from_rgb(120, 100, 160)));
                        } else {
                            ui.label(egui::RichText::new(format!("No results for \"{}\"", self.search_query))
                                .color(egui::Color32::from_rgb(120, 100, 160)));
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
}
