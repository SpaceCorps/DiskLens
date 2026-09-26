use egui::{Color32, CornerRadius, Pos2, Rect, RichText, Stroke, Ui, Vec2};

use crate::model::{VolumeInfo, format_bytes};
use crate::ui::theme::ThemeColors;

pub fn render_disk_banner(ui: &mut Ui, volume: &VolumeInfo, current_folder_size: u64) {
    ui.group(|ui| {
        ui.set_width(ui.available_width());

        ui.horizontal(|ui| {
            // Volume identity
            ui.label(RichText::new("💾").size(14.0));
            ui.label(
                RichText::new(format!("{} ({})", volume.name, volume.fs_type))
                    .strong()
                    .size(13.0)
                    .color(ThemeColors::TEXT_PRIMARY),
            );

            ui.label(RichText::new("•").color(ThemeColors::TEXT_FAINT));

            // Disk stats
            let total_str = format_bytes(volume.total_bytes);
            let used_str = format_bytes(volume.used_bytes);
            let free_str = format_bytes(volume.available_bytes);
            let used_pct = volume.used_percent();

            ui.label(
                RichText::new(format!(
                    "Total: {} | Used: {} ({:.1}%) | Free: {}",
                    total_str, used_str, used_pct, free_str
                ))
                .size(12.0)
                .color(ThemeColors::TEXT_MUTED),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Folder share badge
                let folder_share_pct = if volume.total_bytes > 0 {
                    (current_folder_size as f64 / volume.total_bytes as f64) * 100.0
                } else {
                    0.0
                };

                let badge_color = ThemeColors::color_for_disk_percent(folder_share_pct);
                ui.label(
                    RichText::new(format!(
                        "📁 This Folder: {} ({:.2}% of Disk)",
                        format_bytes(current_folder_size),
                        folder_share_pct
                    ))
                    .strong()
                    .size(12.0)
                    .color(badge_color),
                );
            });
        });

        ui.add_space(4.0);

        // Visual segmented disk usage bar
        let bar_height = 10.0;
        let available_width = ui.available_width();
        let (rect, _response) =
            ui.allocate_exact_size(Vec2::new(available_width, bar_height), egui::Sense::hover());

        let painter = ui.painter();
        let corner_radius = CornerRadius::same(5);

        // Background track (Free space color)
        painter.rect_filled(rect, corner_radius, Color32::from_rgb(20, 83, 45)); // Dark forest green

        if volume.total_bytes > 0 {
            let total_f = volume.total_bytes as f64;

            // Current folder width
            let folder_width =
                ((current_folder_size as f64 / total_f) * rect.width() as f64) as f32;

            // Total used width (including folder)
            let used_width = ((volume.used_bytes as f64 / total_f) * rect.width() as f64) as f32;

            // Other used width
            let other_used_width = (used_width - folder_width).max(0.0);

            // Draw other used segment (Slate / Gray)
            if other_used_width > 0.0 {
                let other_rect =
                    Rect::from_min_size(rect.min, Vec2::new(other_used_width, rect.height()));
                painter.rect_filled(other_rect, corner_radius, Color32::from_rgb(51, 65, 85));
            }

            // Draw current folder segment (Glowing cyan / bright accent)
            if folder_width > 0.0 {
                let folder_start_x = rect.min.x + other_used_width;
                let folder_rect = Rect::from_min_size(
                    Pos2::new(folder_start_x, rect.min.y),
                    Vec2::new(folder_width.max(3.0), rect.height()),
                );
                painter.rect_filled(folder_rect, corner_radius, ThemeColors::CYAN);
            }
        }

        // Border outline
        painter.rect_stroke(
            rect,
            corner_radius,
            Stroke::new(1.0, ThemeColors::BORDER),
            egui::StrokeKind::Inside,
        );
    });
}
