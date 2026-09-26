use egui::{Button, Context, RichText, Window};

use crate::model::{FileEntry, format_bytes};
use crate::ui::theme::ThemeColors;

pub enum TrashModalResult {
    Confirmed(FileEntry),
    Cancelled,
}

pub fn render_trash_modal(ctx: &Context, entry: &FileEntry) -> Option<TrashModalResult> {
    let mut result = None;

    Window::new("Confirm Move to Trash")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.set_width(380.0);
            ui.spacing_mut().item_spacing.y = 12.0;

            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("[!]")
                        .size(16.0)
                        .strong()
                        .color(ThemeColors::CRITICAL_RED),
                );
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("Move to System Trash?")
                            .strong()
                            .size(15.0)
                            .color(ThemeColors::TEXT_PRIMARY),
                    );
                    ui.label(
                        RichText::new("The item will be moved to your Trash / Recycle Bin.")
                            .size(12.0)
                            .color(ThemeColors::TEXT_MUTED),
                    );
                });
            });

            ui.group(|ui| {
                ui.set_width(ui.available_width());
                ui.label(
                    RichText::new(format!("Target: {}", entry.name))
                        .strong()
                        .size(13.0)
                        .color(ThemeColors::TEXT_PRIMARY),
                );
                ui.label(
                    RichText::new(format!("Path: {}", entry.path.to_string_lossy()))
                        .size(11.0)
                        .color(ThemeColors::TEXT_FAINT),
                );
                ui.label(
                    RichText::new(format!(
                        "Reclaimable Space: {}",
                        format_bytes(entry.size_bytes)
                    ))
                    .strong()
                    .size(13.0)
                    .color(ThemeColors::CRITICAL_RED),
                );
            });

            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let confirm_btn = Button::new(
                        RichText::new("Move to Trash")
                            .strong()
                            .color(egui::Color32::WHITE),
                    )
                    .fill(ThemeColors::CRITICAL_RED);

                    if ui.add(confirm_btn).clicked() {
                        result = Some(TrashModalResult::Confirmed(entry.clone()));
                    }

                    if ui.button("Cancel").clicked() {
                        result = Some(TrashModalResult::Cancelled);
                    }
                });
            });
        });

    result
}
