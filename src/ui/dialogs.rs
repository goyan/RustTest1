use eframe::egui;
use std::fs;
use std::path::Path;
use crate::DiskDashboard;
use crate::analysis::is_protected_full_path;

impl DiskDashboard {
    /// Invalidate folder size cache for a path and all its ancestors.
    fn invalidate_ancestor_cache(&mut self, path: &Path) {
        let mut ancestor = path.parent();
        while let Some(parent) = ancestor {
            self.folder_size_cache.remove(parent);
            self.pending_size_calculations.remove(parent);
            ancestor = parent.parent();
        }
    }

    pub(crate) fn render_delete_dialog(&mut self, ctx: &egui::Context) {
        if let Some(path_to_delete) = self.pending_delete.clone() {
            let is_dir = path_to_delete.is_dir();
            let file_name = path_to_delete.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path_to_delete.to_string_lossy().to_string());

            let is_protected = is_protected_full_path(&path_to_delete.to_string_lossy());

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
                                .color(egui::Color32::from_rgb(120, 100, 160)));

                            ui.add_space(20.0);

                            if ui.add(egui::Button::new("OK")
                                .fill(egui::Color32::from_rgb(40, 25, 60))
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
                                    .fill(egui::Color32::from_rgb(40, 25, 60))
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
                                            self.invalidate_ancestor_cache(&path_to_delete);
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
    }

    pub(crate) fn render_error_dialog(&mut self, ctx: &egui::Context) {
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
    }

    pub(crate) fn render_batch_delete_dialog(&mut self, ctx: &egui::Context) {
        if self.pending_batch_delete {
            let selected_count = self.selected_items.len();
            egui::Window::new("Confirm Batch Delete")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .frame(egui::Frame::window(&ctx.style())
                    .fill(egui::Color32::from_rgb(20, 15, 30))
                    .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 51, 102))))
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);
                        ui.label(egui::RichText::new("🗑️ Delete Selected Files?")
                            .size(18.0)
                            .color(egui::Color32::from_rgb(255, 51, 102))
                            .strong());
                        ui.add_space(10.0);
                        ui.label(egui::RichText::new(format!("{} items will be permanently deleted.", selected_count))
                            .size(14.0)
                            .color(egui::Color32::from_rgb(200, 180, 255)));

                        // List selected items (max 10 shown)
                        ui.add_space(5.0);
                        egui::Frame::default()
                            .fill(egui::Color32::from_rgb(12, 10, 18))
                            .rounding(4.0)
                            .inner_margin(egui::Margin::same(8.0))
                            .show(ui, |ui| {
                                let items: Vec<_> = self.selected_items.iter().collect();
                                for (i, path) in items.iter().enumerate() {
                                    if i >= 10 {
                                        ui.label(egui::RichText::new(format!("... and {} more", selected_count - 10))
                                            .size(11.0)
                                            .color(egui::Color32::from_rgb(120, 100, 160)));
                                        break;
                                    }
                                    let name = path.file_name()
                                        .map(|n| n.to_string_lossy().to_string())
                                        .unwrap_or_else(|| path.to_string_lossy().to_string());
                                    ui.label(egui::RichText::new(format!("  {} {}", if path.is_dir() { "📁" } else { "📄" }, name))
                                        .size(11.0)
                                        .color(egui::Color32::from_rgb(180, 160, 220)));
                                }
                            });

                        ui.add_space(15.0);
                        ui.horizontal(|ui| {
                            if ui.add(egui::Button::new(
                                egui::RichText::new("Cancel").color(egui::Color32::WHITE))
                                .fill(egui::Color32::from_rgb(60, 50, 80))
                                .min_size(egui::Vec2::new(100.0, 30.0)))
                                .clicked()
                            {
                                self.pending_batch_delete = false;
                            }

                            ui.add_space(20.0);

                            if ui.add(egui::Button::new(
                                egui::RichText::new("🗑️ Delete All").color(egui::Color32::WHITE).strong())
                                .fill(egui::Color32::from_rgb(180, 30, 60))
                                .min_size(egui::Vec2::new(120.0, 30.0)))
                                .clicked()
                            {
                                // Perform batch deletion
                                let mut deleted = 0;
                                let mut skipped = 0;
                                let mut errors = Vec::new();
                                for path in self.selected_items.clone() {
                                    if is_protected_full_path(&path.to_string_lossy()) {
                                        skipped += 1;
                                        continue;
                                    }

                                    let result = if path.is_dir() {
                                        fs::remove_dir_all(&path)
                                    } else {
                                        fs::remove_file(&path)
                                    };
                                    match result {
                                        Ok(_) => {
                                            deleted += 1;
                                            self.invalidate_ancestor_cache(&path);
                                        }
                                        Err(e) => errors.push(format!("{}: {}", path.file_name().unwrap_or_default().to_string_lossy(), e)),
                                    }
                                }
                                self.selected_items.clear();
                                self.needs_refresh = true;
                                self.pending_batch_delete = false;
                                if errors.is_empty() && skipped == 0 {
                                    self.toast_message = Some((format!("🗑️ Deleted {} items", deleted), 2.0));
                                } else if skipped > 0 {
                                    self.toast_message = Some((format!("🔒 Skipped {} protected, deleted {}", skipped, deleted), 3.0));
                                } else {
                                    self.toast_message = Some((format!("⚠️ Deleted {} items, {} failed", deleted, errors.len()), 3.0));
                                }
                            }
                        });
                        ui.add_space(10.0);
                    });
                });
        }
    }
}
