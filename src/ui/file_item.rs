use eframe::egui;
use std::time::SystemTime;
use crate::DiskDashboard;
use crate::models::{FileItem, FileCategory};
use crate::utils::format_size;

impl DiskDashboard {
    pub(crate) fn render_file_item(&mut self, ui: &mut egui::Ui, item: &FileItem, _index: usize) {
        let is_selected = self.selected_items.contains(&item.path);

        let category_text = match item.category {
            FileCategory::MustKeep => "Must Keep",
            FileCategory::System => "System",
            FileCategory::Regular => "Regular",
            FileCategory::Useless => "Useless",
            FileCategory::Unknown => "Unknown",
        };

        let category_color = match item.category {
            FileCategory::MustKeep => egui::Color32::from_rgb(0, 255, 136),   // Neon green
            FileCategory::System => egui::Color32::from_rgb(170, 85, 255),    // Neon purple
            FileCategory::Regular => egui::Color32::from_rgb(0, 212, 255),    // Electric blue
            FileCategory::Useless => egui::Color32::from_rgb(255, 51, 102),   // Neon red
            FileCategory::Unknown => egui::Color32::from_rgb(100, 80, 140),   // Dim purple
        };

        let usefulness_color = if item.usefulness < 20.0 {
            egui::Color32::from_rgb(255, 51, 102)  // Neon red
        } else if item.usefulness < 50.0 {
            egui::Color32::from_rgb(255, 136, 0)   // Neon orange
        } else if item.usefulness < 80.0 {
            egui::Color32::from_rgb(0, 255, 255)   // Neon cyan
        } else {
            egui::Color32::from_rgb(0, 255, 136)   // Neon green
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

        // Cyberpunk neon file item design
        let base_fill = if is_selected {
            egui::Color32::from_rgb(60, 20, 80)  // Purple glow for selected
        } else if is_empty_folder {
            egui::Color32::from_rgb(15, 12, 20) // Darker for empty
        } else {
            egui::Color32::from_rgb(18, 16, 26)  // Dark purple base
        };

        let hover_fill = if is_selected {
            egui::Color32::from_rgb(80, 30, 100) // Brighter purple for selected+hover
        } else if is_empty_folder {
            egui::Color32::from_rgb(25, 20, 35)  // Dim purple hover for empty
        } else {
            egui::Color32::from_rgb(30, 25, 45)  // Purple hover
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
                    egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 40, 80))  // Dim purple for empty
                } else if is_selected {
                    egui::Stroke::new(1.5, egui::Color32::from_rgb(255, 0, 255)) // Magenta for selected
                } else {
                    egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 255, 255)) // Cyan hover
                }
            } else if is_selected {
                egui::Stroke::new(1.0, egui::Color32::from_rgb(180, 0, 180)) // Dim magenta for selected
            } else {
                egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 30, 60))  // Dark purple border
            })
            .rounding(6.0)
            .inner_margin(egui::Margin::same(10.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Checkbox for selection
                    let mut checked = is_selected;
                    let checkbox_response = ui.add(
                        egui::Checkbox::without_text(&mut checked)
                    );
                    if checkbox_response.changed() {
                        if checked {
                            self.selected_items.insert(item.path.clone());
                        } else {
                            self.selected_items.remove(&item.path);
                        }
                    }

                    // Icon column
                    let icon_size = 18.0;
                    let icon_text = crate::analysis::get_file_icon(&item.name, item.is_dir, is_empty_folder, item.category);

                    ui.label(egui::RichText::new(icon_text).size(icon_size));
                    ui.add_space(12.0);

                    // Name column - neon colors for cyberpunk theme
                    let name_color = if item.is_dir {
                        if is_empty_folder {
                            egui::Color32::from_rgb(80, 60, 100)   // Dim purple for empty
                        } else {
                            egui::Color32::from_rgb(0, 255, 255)   // Neon cyan for folders
                        }
                    } else {
                        egui::Color32::from_rgb(200, 180, 255)     // Light purple for files
                    };

                    // Name label (click handled by row interact_response)
                    let name_label = ui.add(
                        egui::Label::new(egui::RichText::new(&item.name)
                            .size(13.0)
                            .color(name_color))
                        .sense(egui::Sense::click())
                    );

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

                        // Usefulness score - fixed 60px (rightmost)
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

                        // Category badge - fixed 90px (middle)
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

                        // Size column - fixed 75px (leftmost, most important)
                        ui.allocate_ui_with_layout(
                            egui::Vec2::new(75.0, 20.0),
                            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                            |ui| {
                                ui.label(egui::RichText::new(&size_str)
                                    .size(11.0)
                                    .color(egui::Color32::from_gray(160)));
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
                egui::Color32::from_rgba_unmultiplied(255, 51, 102, 40)  // Neon red glow
            } else if size_ratio > 0.5 {
                egui::Color32::from_rgba_unmultiplied(255, 136, 0, 35)   // Neon orange glow
            } else {
                egui::Color32::from_rgba_unmultiplied(0, 255, 255, 30)   // Neon cyan glow
            };
            ui.painter().rect_filled(bar_rect, 4.0, bar_color);
        }

        // Row click = navigate/open (NOT selection — checkboxes handle that)
        if interact_response.clicked() {
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
}
