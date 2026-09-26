use egui::{RichText, ScrollArea, Ui, Vec2};
use std::path::{Path, PathBuf};

use crate::model::{
    FileCategory, FileEntry, FilterPreset, PercentageDisplayMode, SortColumn, SortDirection,
    VolumeInfo, format_bytes, format_count,
};
use crate::ui::theme::{ThemeColors, draw_percent_bar};

pub struct ExplorerAction {
    pub navigate_to: Option<PathBuf>,
    pub open_file: Option<PathBuf>,
    pub reveal_in_finder: Option<PathBuf>,
    pub open_terminal: Option<PathBuf>,
    pub prompt_trash: Option<FileEntry>,
    pub select_entry: Option<FileEntry>,
}

#[allow(clippy::too_many_arguments)]
pub fn render_explorer(
    ui: &mut Ui,
    entries: &[FileEntry],
    volume: &VolumeInfo,
    current_folder_size: u64,
    display_mode: PercentageDisplayMode,
    active_filter: FilterPreset,
    search_query: &str,
    show_hidden: bool,
    sort_col: &mut SortColumn,
    sort_dir: &mut SortDirection,
    selected_path: Option<&Path>,
) -> ExplorerAction {
    let mut action = ExplorerAction {
        navigate_to: None,
        open_file: None,
        reveal_in_finder: None,
        open_terminal: None,
        prompt_trash: None,
        select_entry: None,
    };

    // Filter entries
    let query_lower = search_query.to_lowercase();
    let mut filtered_entries: Vec<&FileEntry> = entries
        .iter()
        .filter(|e| {
            if !show_hidden && e.is_hidden {
                return false;
            }
            if !query_lower.is_empty() && !e.name.to_lowercase().contains(&query_lower) {
                return false;
            }
            match active_filter {
                FilterPreset::All => true,
                FilterPreset::FoldersOnly => e.is_dir(),
                FilterPreset::HeavyOnly => e.size_bytes >= 100 * 1024 * 1024,
                FilterPreset::MassiveOnly => e.size_bytes >= 1024 * 1024 * 1024,
                FilterPreset::DevArtifacts => {
                    matches!(e.category, FileCategory::Git | FileCategory::BuildArtifact)
                }
            }
        })
        .collect();

    // Sort entries
    filtered_entries.sort_by(|a, b| {
        let cmp = match sort_col {
            SortColumn::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            SortColumn::Size => a.size_bytes.cmp(&b.size_bytes),
            SortColumn::FillPercent => a.size_bytes.cmp(&b.size_bytes),
            SortColumn::ItemCount => a.file_count.cmp(&b.file_count),
            SortColumn::Modified => a.modified.cmp(&b.modified),
        };
        match sort_dir {
            SortDirection::Ascending => cmp,
            SortDirection::Descending => cmp.reverse(),
        }
    });

    let max_size = filtered_entries
        .iter()
        .map(|e| e.size_bytes)
        .max()
        .unwrap_or(1)
        .max(1);

    // Table Header
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;

        // Name column header
        let name_label = format!(
            "Name {}",
            sort_indicator(*sort_col == SortColumn::Name, *sort_dir)
        );
        if ui
            .button(
                RichText::new(name_label)
                    .strong()
                    .size(12.0)
                    .color(ThemeColors::TEXT_PRIMARY),
            )
            .clicked()
        {
            toggle_sort(sort_col, sort_dir, SortColumn::Name);
        }

        ui.add_space(140.0);

        // Fill % Gauge column header
        let fill_title = match display_mode {
            PercentageDisplayMode::DiskPercent => "Fill % of Disk",
            PercentageDisplayMode::FolderPercent => "Fill % of Folder",
            PercentageDisplayMode::Dual => "Disk % & Folder %",
        };
        let fill_label = format!(
            "{} {}",
            fill_title,
            sort_indicator(*sort_col == SortColumn::FillPercent, *sort_dir)
        );
        if ui
            .button(
                RichText::new(fill_label)
                    .strong()
                    .size(12.0)
                    .color(ThemeColors::CYAN),
            )
            .clicked()
        {
            toggle_sort(sort_col, sort_dir, SortColumn::FillPercent);
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                RichText::new("Actions")
                    .size(11.0)
                    .color(ThemeColors::TEXT_MUTED),
            );
            ui.add_space(20.0);

            // Date Modified header
            let date_label = format!(
                "Date {}",
                sort_indicator(*sort_col == SortColumn::Modified, *sort_dir)
            );
            if ui
                .button(
                    RichText::new(date_label)
                        .size(12.0)
                        .color(ThemeColors::TEXT_MUTED),
                )
                .clicked()
            {
                toggle_sort(sort_col, sort_dir, SortColumn::Modified);
            }
            ui.add_space(10.0);

            // Item count header
            let count_label = format!(
                "Items {}",
                sort_indicator(*sort_col == SortColumn::ItemCount, *sort_dir)
            );
            if ui
                .button(
                    RichText::new(count_label)
                        .size(12.0)
                        .color(ThemeColors::TEXT_MUTED),
                )
                .clicked()
            {
                toggle_sort(sort_col, sort_dir, SortColumn::ItemCount);
            }
            ui.add_space(10.0);

            // Size header
            let size_label = format!(
                "Size {}",
                sort_indicator(*sort_col == SortColumn::Size, *sort_dir)
            );
            if ui
                .button(
                    RichText::new(size_label)
                        .strong()
                        .size(12.0)
                        .color(ThemeColors::TEXT_PRIMARY),
                )
                .clicked()
            {
                toggle_sort(sort_col, sort_dir, SortColumn::Size);
            }
        });
    });

    ui.separator();

    // Table Body with ScrollArea
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 4.0;

            if filtered_entries.is_empty() {
                ui.add_space(40.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new("Empty Folder or No Matches")
                            .size(16.0)
                            .color(ThemeColors::TEXT_MUTED),
                    );
                    if !search_query.is_empty() {
                        ui.label(
                            RichText::new(format!("No items match '{}'", search_query))
                                .size(12.0)
                                .color(ThemeColors::TEXT_FAINT),
                        );
                    }
                });
                return;
            }

            for entry in filtered_entries {
                let is_selected = selected_path.is_some_and(|p| p == entry.path);
                let disk_pct = entry.disk_percent(volume.total_bytes);
                let folder_pct = entry.folder_percent(current_folder_size);

                let row_response = ui.group(|ui| {
                    ui.set_width(ui.available_width());

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;

                        // Icon & Name
                        let icon = entry.category.icon();
                        ui.label(RichText::new(icon).size(14.0));

                        let name_color = if is_selected {
                            ThemeColors::CYAN
                        } else if entry.is_hidden {
                            ThemeColors::TEXT_MUTED
                        } else {
                            ThemeColors::TEXT_PRIMARY
                        };

                        let name_text = RichText::new(&entry.name).size(13.0).color(name_color);
                        let name_resp = ui.selectable_label(is_selected, name_text);

                        if name_resp.clicked() {
                            action.select_entry = Some((*entry).clone());
                        }
                        if name_resp.double_clicked() {
                            if entry.is_dir() {
                                action.navigate_to = Some(entry.path.clone());
                            } else {
                                action.open_file = Some(entry.path.clone());
                            }
                        }

                        // Fill % Bar Widget
                        let bar_color = match display_mode {
                            PercentageDisplayMode::DiskPercent => {
                                ThemeColors::color_for_disk_percent(disk_pct)
                            }
                            PercentageDisplayMode::FolderPercent => {
                                ThemeColors::color_for_folder_percent(folder_pct)
                            }
                            PercentageDisplayMode::Dual => {
                                ThemeColors::color_for_disk_percent(disk_pct)
                            }
                        };

                        // Compute fill fraction:
                        // In FolderPercent or Dual mode: relative to current folder (or heaviest in view)
                        // In DiskPercent mode: visual width is proportional to max in current view,
                        // while color and text reflect the disk capacity!
                        let fill_fraction =
                            (entry.size_bytes as f64 / max_size as f64).clamp(0.0, 1.0);

                        let bar_label = match display_mode {
                            PercentageDisplayMode::DiskPercent => {
                                if entry.is_scanning {
                                    "calculating...".to_string()
                                } else {
                                    format!("{:.2}% of Disk", disk_pct)
                                }
                            }
                            PercentageDisplayMode::FolderPercent => {
                                if entry.is_scanning {
                                    "calculating...".to_string()
                                } else {
                                    format!("{:.1}% of Folder", folder_pct)
                                }
                            }
                            PercentageDisplayMode::Dual => {
                                if entry.is_scanning {
                                    "calculating...".to_string()
                                } else {
                                    format!("{:.1}% dir • {:.2}% disk", folder_pct, disk_pct)
                                }
                            }
                        };

                        ui.allocate_ui_with_layout(
                            Vec2::new(220.0, 20.0),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                draw_percent_bar(ui, fill_fraction, bar_color, &bar_label, 14.0);
                            },
                        );

                        // Right side: Size, Items, Quick Actions
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // Quick Action Buttons
                            if ui
                                .button(
                                    RichText::new("Del")
                                        .size(10.0)
                                        .color(ThemeColors::CRITICAL_RED),
                                )
                                .on_hover_text("Move to Trash")
                                .clicked()
                            {
                                action.prompt_trash = Some((*entry).clone());
                            }

                            if ui
                                .button(
                                    RichText::new("Copy")
                                        .size(10.0)
                                        .color(ThemeColors::TEXT_MUTED),
                                )
                                .on_hover_text("Copy Path to Clipboard")
                                .clicked()
                            {
                                ui.ctx().copy_text(entry.path.to_string_lossy().to_string());
                            }

                            if ui
                                .button(
                                    RichText::new("Term")
                                        .size(10.0)
                                        .color(ThemeColors::TEXT_MUTED),
                                )
                                .on_hover_text("Open in Terminal")
                                .clicked()
                            {
                                action.open_terminal = Some(entry.path.clone());
                            }

                            if ui
                                .button(
                                    RichText::new("Open")
                                        .size(10.0)
                                        .color(ThemeColors::TEXT_MUTED),
                                )
                                .on_hover_text("Reveal in Finder")
                                .clicked()
                            {
                                action.reveal_in_finder = Some(entry.path.clone());
                            }

                            ui.add_space(8.0);

                            // File / item count
                            if entry.is_dir() {
                                let count_text = if entry.is_scanning {
                                    "scanning...".to_string()
                                } else {
                                    format_count(entry.file_count)
                                };
                                ui.label(
                                    RichText::new(count_text)
                                        .size(11.0)
                                        .color(ThemeColors::TEXT_MUTED),
                                );
                            } else {
                                ui.label(
                                    RichText::new("file")
                                        .size(11.0)
                                        .color(ThemeColors::TEXT_FAINT),
                                );
                            }

                            ui.add_space(8.0);

                            // Formatted Size
                            let size_color = if disk_pct >= 5.0 || folder_pct >= 40.0 {
                                ThemeColors::CRITICAL_RED
                            } else if disk_pct >= 1.0 || folder_pct >= 15.0 {
                                ThemeColors::WARNING_AMBER
                            } else {
                                ThemeColors::TEXT_PRIMARY
                            };

                            let size_text = if entry.is_scanning && entry.size_bytes == 0 {
                                "…".to_string()
                            } else {
                                format_bytes(entry.size_bytes)
                            };

                            ui.label(
                                RichText::new(size_text)
                                    .strong()
                                    .size(13.0)
                                    .color(size_color),
                            );
                        });
                    });
                });

                if row_response.response.double_clicked() {
                    if entry.is_dir() {
                        action.navigate_to = Some(entry.path.clone());
                    } else {
                        action.open_file = Some(entry.path.clone());
                    }
                }
            }
        });

    action
}

fn toggle_sort(col: &mut SortColumn, dir: &mut SortDirection, target_col: SortColumn) {
    if *col == target_col {
        *dir = match *dir {
            SortDirection::Ascending => SortDirection::Descending,
            SortDirection::Descending => SortDirection::Ascending,
        };
    } else {
        *col = target_col;
        // Default to descending for size / percentage, ascending for name
        *dir = match target_col {
            SortColumn::Name => SortDirection::Ascending,
            _ => SortDirection::Descending,
        };
    }
}

fn sort_indicator(is_active: bool, dir: SortDirection) -> &'static str {
    if is_active {
        match dir {
            SortDirection::Ascending => "▲",
            SortDirection::Descending => "▼",
        }
    } else {
        ""
    }
}
