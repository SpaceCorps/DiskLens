use egui::{Color32, CornerRadius, Rect, RichText, ScrollArea, Ui, Vec2};
use std::path::{Path, PathBuf};

use crate::model::{FilterPreset, VolumeInfo, format_bytes};
use crate::ui::theme::ThemeColors;

pub struct SidebarAction {
    pub navigate_to: Option<PathBuf>,
    pub pin_current: bool,
    pub unpin_index: Option<usize>,
    pub clear_cache: bool,
}

pub fn render_sidebar(
    ui: &mut Ui,
    current_path: &Path,
    volumes: &[VolumeInfo],
    bookmarks: &[PathBuf],
    active_filter: &mut FilterPreset,
) -> SidebarAction {
    let mut action = SidebarAction {
        navigate_to: None,
        pin_current: false,
        unpin_index: None,
        clear_cache: false,
    };

    ScrollArea::vertical().show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.spacing_mut().item_spacing.y = 8.0;

        // Quick Locations Section
        ui.label(
            RichText::new("QUICK ACCESS")
                .extra_letter_spacing(1.0)
                .size(10.0)
                .strong()
                .color(ThemeColors::TEXT_MUTED),
        );

        let home_dir = dirs::home_dir();

        if let Some(ref home) = home_dir {
            render_shortcut_button(ui, "Home", home, current_path, &mut action);

            let git_dir = home.join("git");
            if git_dir.exists() {
                render_shortcut_button(ui, "Git Repos", &git_dir, current_path, &mut action);
            }

            let spacecorps_dir = home.join("git").join("spacecorps");
            if spacecorps_dir.exists() {
                render_shortcut_button(
                    ui,
                    "SpaceCorps",
                    &spacecorps_dir,
                    current_path,
                    &mut action,
                );
            }

            let downloads = dirs::download_dir().unwrap_or_else(|| home.join("Downloads"));
            if downloads.exists() {
                render_shortcut_button(ui, "Downloads", &downloads, current_path, &mut action);
            }

            let documents = dirs::document_dir().unwrap_or_else(|| home.join("Documents"));
            if documents.exists() {
                render_shortcut_button(ui, "Documents", &documents, current_path, &mut action);
            }

            let desktop = dirs::desktop_dir().unwrap_or_else(|| home.join("Desktop"));
            if desktop.exists() {
                render_shortcut_button(ui, "Desktop", &desktop, current_path, &mut action);
            }
        }

        render_shortcut_button(
            ui,
            "Root Filesystem",
            Path::new("/"),
            current_path,
            &mut action,
        );

        ui.add_space(4.0);
        ui.separator();

        // Disks / Volumes Section
        ui.label(
            RichText::new("STORAGE DRIVES")
                .extra_letter_spacing(1.0)
                .size(10.0)
                .strong()
                .color(ThemeColors::TEXT_MUTED),
        );

        for vol in volumes {
            let is_selected = current_path.starts_with(&vol.mount_point);
            let btn_text = vol.name.clone();
            let total_str = format_bytes(vol.total_bytes);
            let free_str = format_bytes(vol.available_bytes);

            let resp = ui.selectable_label(is_selected, RichText::new(btn_text).size(12.0));
            if resp.clicked() {
                action.navigate_to = Some(vol.mount_point.clone());
            }

            // Disk mini bar
            let mini_bar_rect = ui.allocate_space(Vec2::new(ui.available_width(), 4.0)).1;
            let painter = ui.painter();
            let corner = CornerRadius::same(2);
            painter.rect_filled(mini_bar_rect, corner, Color32::from_rgb(30, 41, 59));

            if vol.total_bytes > 0 {
                let frac = (vol.used_bytes as f64 / vol.total_bytes as f64).clamp(0.0, 1.0) as f32;
                let fill_w = mini_bar_rect.width() * frac;
                let fill_r = Rect::from_min_size(
                    mini_bar_rect.min,
                    Vec2::new(fill_w, mini_bar_rect.height()),
                );
                let color = if frac > 0.90 {
                    ThemeColors::CRITICAL_RED
                } else if frac > 0.75 {
                    ThemeColors::WARNING_AMBER
                } else {
                    ThemeColors::CYAN
                };
                painter.rect_filled(fill_r, corner, color);
            }

            ui.label(
                RichText::new(format!("{} free of {}", free_str, total_str))
                    .size(10.0)
                    .color(ThemeColors::TEXT_FAINT),
            );
            ui.add_space(2.0);
        }

        ui.add_space(4.0);
        ui.separator();

        // Starred Bookmarks Section
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("BOOKMARKS")
                    .extra_letter_spacing(1.0)
                    .size(10.0)
                    .strong()
                    .color(ThemeColors::TEXT_MUTED),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .button(RichText::new("+ Pin").size(10.0))
                    .on_hover_text("Pin current folder to bookmarks")
                    .clicked()
                {
                    action.pin_current = true;
                }
            });
        });

        if bookmarks.is_empty() {
            ui.label(
                RichText::new("No pinned folders")
                    .size(11.0)
                    .color(ThemeColors::TEXT_FAINT),
            );
        } else {
            for (idx, bmark) in bookmarks.iter().enumerate() {
                ui.horizontal(|ui| {
                    let bmark_name = bmark.file_name().and_then(|n| n.to_str()).unwrap_or("/");
                    let is_active = current_path == bmark;
                    if ui
                        .selectable_label(is_active, bmark_name)
                        .on_hover_text(bmark.to_string_lossy())
                        .clicked()
                    {
                        action.navigate_to = Some(bmark.clone());
                    }
                    if ui
                        .button(RichText::new("x").size(10.0).color(ThemeColors::TEXT_FAINT))
                        .on_hover_text("Remove bookmark")
                        .clicked()
                    {
                        action.unpin_index = Some(idx);
                    }
                });
            }
        }

        ui.add_space(4.0);
        ui.separator();

        // Filter Presets Section
        ui.label(
            RichText::new("PRESET FILTERS")
                .extra_letter_spacing(1.0)
                .size(10.0)
                .strong()
                .color(ThemeColors::TEXT_MUTED),
        );

        if ui
            .selectable_label(*active_filter == FilterPreset::All, "All Items")
            .clicked()
        {
            *active_filter = FilterPreset::All;
        }
        if ui
            .selectable_label(*active_filter == FilterPreset::FoldersOnly, "Folders Only")
            .clicked()
        {
            *active_filter = FilterPreset::FoldersOnly;
        }
        if ui
            .selectable_label(*active_filter == FilterPreset::HeavyOnly, "Heavy (>100MB)")
            .clicked()
        {
            *active_filter = FilterPreset::HeavyOnly;
        }
        if ui
            .selectable_label(
                *active_filter == FilterPreset::MassiveOnly,
                "Massive (>1GB)",
            )
            .clicked()
        {
            *active_filter = FilterPreset::MassiveOnly;
        }
        if ui
            .selectable_label(
                *active_filter == FilterPreset::DevArtifacts,
                "Dev Caches (.git, etc.)",
            )
            .on_hover_text("Find .git, node_modules, target, build")
            .clicked()
        {
            *active_filter = FilterPreset::DevArtifacts;
        }

        ui.add_space(10.0);
        ui.separator();

        // Maintenance
        if ui
            .button(
                RichText::new("Clear Size Cache")
                    .size(11.0)
                    .color(ThemeColors::TEXT_MUTED),
            )
            .clicked()
        {
            action.clear_cache = true;
        }
    });

    action
}

fn render_shortcut_button(
    ui: &mut Ui,
    title: &str,
    target: &Path,
    current: &Path,
    action: &mut SidebarAction,
) {
    let is_selected = current == target;
    if ui
        .selectable_label(is_selected, RichText::new(title).size(12.0))
        .clicked()
    {
        action.navigate_to = Some(target.to_path_buf());
    }
}
