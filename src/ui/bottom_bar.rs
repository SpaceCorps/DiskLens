use egui::{RichText, Ui};

use crate::model::{FileEntry, VolumeInfo, format_bytes, format_count};
use crate::ui::theme::ThemeColors;

pub struct BottomBarAction {
    pub reveal_selected: bool,
    pub trash_selected: bool,
}

pub fn render_bottom_bar(
    ui: &mut Ui,
    entries_count: usize,
    current_folder_size: u64,
    scanning_count: usize,
    volume: &VolumeInfo,
    selected_entry: Option<&FileEntry>,
    status_notice: Option<&str>,
) -> BottomBarAction {
    let mut action = BottomBarAction {
        reveal_selected: false,
        trash_selected: false,
    };

    ui.horizontal(|ui| {
        // Status overview
        let folder_pct = if volume.total_bytes > 0 {
            (current_folder_size as f64 / volume.total_bytes as f64) * 100.0
        } else {
            0.0
        };

        ui.label(
            RichText::new(format!(
                "Total: {} items ({}) • {:.2}% of disk",
                entries_count,
                format_bytes(current_folder_size),
                folder_pct
            ))
            .size(11.0)
            .color(ThemeColors::TEXT_PRIMARY),
        );

        ui.separator();

        if let Some(notice) = status_notice {
            ui.label(
                RichText::new(notice)
                    .size(11.0)
                    .strong()
                    .color(ThemeColors::CYAN),
            );
        } else if scanning_count > 0 {
            ui.label(
                RichText::new(format!("Computing size for {} folders...", scanning_count))
                    .size(11.0)
                    .color(ThemeColors::WARNING_AMBER),
            );
        } else {
            ui.label(
                RichText::new("All sizes calculated")
                    .size(11.0)
                    .color(ThemeColors::LIGHT_GREEN),
            );
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if let Some(entry) = selected_entry {
                let entry_disk_pct = entry.disk_percent(volume.total_bytes);
                let entry_folder_pct = entry.folder_percent(current_folder_size);

                if ui
                    .button(
                        RichText::new("Trash")
                            .size(11.0)
                            .color(ThemeColors::CRITICAL_RED),
                    )
                    .clicked()
                {
                    action.trash_selected = true;
                }

                if ui.button(RichText::new("Reveal").size(11.0)).clicked() {
                    action.reveal_selected = true;
                }

                ui.separator();

                let count_str = if entry.is_dir() {
                    format!(" ({})", format_count(entry.file_count))
                } else {
                    String::new()
                };

                ui.label(
                    RichText::new(format!(
                        "Selected: {} • {}{} • {:.2}% Disk • {:.1}% Folder",
                        entry.name,
                        format_bytes(entry.size_bytes),
                        count_str,
                        entry_disk_pct,
                        entry_folder_pct
                    ))
                    .size(11.0)
                    .color(ThemeColors::TEXT_MUTED),
                );
            } else {
                ui.label(
                    RichText::new("Select any item to inspect details or clean up")
                        .size(11.0)
                        .color(ThemeColors::TEXT_FAINT),
                );
            }
        });
    });

    action
}
