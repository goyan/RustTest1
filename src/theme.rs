use eframe::egui;

pub fn apply_modern_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    // Cyberpunk neon color palette
    style.visuals.dark_mode = true;
    style.visuals.panel_fill = egui::Color32::from_rgb(18, 16, 26);       // Dark purple
    style.visuals.window_fill = egui::Color32::from_rgb(10, 10, 15);      // Deep black
    style.visuals.extreme_bg_color = egui::Color32::from_rgb(8, 8, 12);   // Darker
    style.visuals.faint_bg_color = egui::Color32::from_rgb(25, 22, 35);   // Purple tint
    style.visuals.hyperlink_color = egui::Color32::from_rgb(0, 255, 255); // Neon cyan

    // Neon selection styling
    style.visuals.button_frame = true;
    style.visuals.selection.bg_fill = egui::Color32::from_rgb(80, 0, 120);  // Purple glow
    style.visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 255, 255));

    // Widget styling with neon accents
    style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(20, 18, 28);
    style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 40, 80));
    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(30, 25, 45);
    style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.5, egui::Color32::from_rgb(0, 255, 255));
    style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(50, 30, 70);
    style.visuals.widgets.active.bg_stroke = egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 0, 255));

    // Spacing
    style.spacing.item_spacing = egui::Vec2::new(8.0, 6.0);
    style.spacing.window_margin = egui::Margin::same(8.0);

    ctx.set_style(style);
}
