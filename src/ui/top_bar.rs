use egui::{Button, Key, RichText, TextEdit, Ui};
use std::path::{Path, PathBuf};

use crate::model::PercentageDisplayMode;
use crate::ui::theme::ThemeColors;

pub struct TopBarAction {
    pub navigate_to: Option<PathBuf>,
    pub go_back: bool,
    pub go_forward: bool,
    pub go_up: bool,
    pub refresh: bool,
    pub toggle_hidden: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn render_top_bar(
    ui: &mut Ui,
    current_path: &Path,
    can_go_back: bool,
    can_go_forward: bool,
    is_editing_path: &mut bool,
    path_edit_buffer: &mut String,
    search_query: &mut String,
    display_mode: &mut PercentageDisplayMode,
    show_hidden: &mut bool,
) -> TopBarAction {
    let mut action = TopBarAction {
        navigate_to: None,
        go_back: false,
        go_forward: false,
        go_up: false,
        refresh: false,
        toggle_hidden: false,
    };

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;

        // Navigation History buttons
        let back_btn = Button::new(RichText::new("◀").size(13.0).color(if can_go_back {
            ThemeColors::TEXT_PRIMARY
        } else {
            ThemeColors::TEXT_FAINT
        }));
        if ui
            .add_enabled(can_go_back, back_btn)
            .on_hover_text("Back (Alt+Left)")
            .clicked()
        {
            action.go_back = true;
        }

        let fwd_btn = Button::new(RichText::new("▶").size(13.0).color(if can_go_forward {
            ThemeColors::TEXT_PRIMARY
        } else {
            ThemeColors::TEXT_FAINT
        }));
        if ui
            .add_enabled(can_go_forward, fwd_btn)
            .on_hover_text("Forward (Alt+Right)")
            .clicked()
        {
            action.go_forward = true;
        }

        let has_parent = current_path.parent().is_some();
        let up_btn = Button::new(RichText::new("▲").size(13.0).color(if has_parent {
            ThemeColors::TEXT_PRIMARY
        } else {
            ThemeColors::TEXT_FAINT
        }));
        if ui
            .add_enabled(has_parent, up_btn)
            .on_hover_text("Up to Parent Folder (Alt+Up)")
            .clicked()
        {
            action.go_up = true;
        }

        let refresh_btn = Button::new(RichText::new("⟳").size(15.0).color(ThemeColors::CYAN));
        if ui
            .add(refresh_btn)
            .on_hover_text("Rescan Directory (Cmd+R)")
            .clicked()
        {
            action.refresh = true;
        }

        ui.separator();

        // Breadcrumb or Path TextEdit
        if *is_editing_path {
            let response = ui.add(
                TextEdit::singleline(path_edit_buffer)
                    .desired_width(ui.available_width() - 320.0)
                    .hint_text("Enter directory path..."),
            );
            if response.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                let target = PathBuf::from(path_edit_buffer.trim());
                if target.is_dir() {
                    action.navigate_to = Some(target);
                }
                *is_editing_path = false;
            } else if ui.input(|i| i.key_pressed(Key::Escape)) {
                *is_editing_path = false;
            }
        } else {
            // Render interactive breadcrumbs
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;

                // Root button
                if ui
                    .button(RichText::new("/").size(12.0).color(ThemeColors::TEXT_MUTED))
                    .clicked()
                {
                    action.navigate_to = Some(PathBuf::from("/"));
                }

                let mut accum = PathBuf::new();
                let components: Vec<_> = current_path.components().collect();

                for (idx, comp) in components.iter().enumerate() {
                    match comp {
                        std::path::Component::RootDir => {
                            accum.push("/");
                        }
                        std::path::Component::Normal(name) => {
                            accum.push(name);
                            let name_str = name.to_string_lossy();

                            ui.label(RichText::new("›").size(11.0).color(ThemeColors::TEXT_FAINT));

                            let is_last = idx == components.len() - 1;
                            let btn_text = if is_last {
                                RichText::new(name_str.as_ref())
                                    .size(12.0)
                                    .strong()
                                    .color(ThemeColors::CYAN)
                            } else {
                                RichText::new(name_str.as_ref())
                                    .size(12.0)
                                    .color(ThemeColors::TEXT_PRIMARY)
                            };

                            let current_accum = accum.clone();
                            if ui.button(btn_text).clicked() {
                                action.navigate_to = Some(current_accum);
                            }
                        }
                        _ => {}
                    }
                }

                // Edit path button
                if ui
                    .button(
                        RichText::new("[edit]")
                            .size(11.0)
                            .color(ThemeColors::TEXT_MUTED),
                    )
                    .on_hover_text("Edit path manually")
                    .clicked()
                {
                    *path_edit_buffer = current_path.to_string_lossy().to_string();
                    *is_editing_path = true;
                }
            });
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Hidden files toggle
            let hidden_label = if *show_hidden {
                "Hidden: on"
            } else {
                "Hidden: off"
            };
            if ui
                .selectable_label(*show_hidden, RichText::new(hidden_label).size(11.0))
                .clicked()
            {
                *show_hidden = !*show_hidden;
                action.toggle_hidden = true;
            }

            ui.separator();

            // Display Mode Selector
            ui.label(
                RichText::new("Gauge:")
                    .size(11.0)
                    .color(ThemeColors::TEXT_MUTED),
            );
            if ui
                .selectable_label(*display_mode == PercentageDisplayMode::Dual, "Dual")
                .on_hover_text("Show both Disk % and Folder %")
                .clicked()
            {
                *display_mode = PercentageDisplayMode::Dual;
            }
            if ui
                .selectable_label(
                    *display_mode == PercentageDisplayMode::FolderPercent,
                    "Folder %",
                )
                .on_hover_text("Bar shows % of current parent folder")
                .clicked()
            {
                *display_mode = PercentageDisplayMode::FolderPercent;
            }
            if ui
                .selectable_label(
                    *display_mode == PercentageDisplayMode::DiskPercent,
                    "Disk %",
                )
                .on_hover_text("Bar shows % of total hard drive capacity")
                .clicked()
            {
                *display_mode = PercentageDisplayMode::DiskPercent;
            }

            ui.separator();

            // Search Filter
            let clear_clicked = ui.button("x").clicked();
            if clear_clicked {
                search_query.clear();
            }
            ui.add(
                TextEdit::singleline(search_query)
                    .desired_width(140.0)
                    .hint_text("Filter files..."),
            );
        });
    });

    action
}
