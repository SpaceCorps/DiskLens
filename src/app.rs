use egui::{CentralPanel, Context, Panel, Ui};
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::model::{
    FileEntry, FilterPreset, PercentageDisplayMode, SortColumn, SortDirection, VolumeInfo,
};
use crate::scanner::DirectoryScanner;
use crate::ui::bottom_bar::render_bottom_bar;
use crate::ui::disk_banner::render_disk_banner;
use crate::ui::explorer::render_explorer;
use crate::ui::sidebar::render_sidebar;
use crate::ui::top_bar::render_top_bar;
use crate::ui::trash_modal::{TrashModalResult, render_trash_modal};
use crate::volume::VolumeManager;

pub struct DiskLensApp {
    current_path: PathBuf,
    history: Vec<PathBuf>,
    history_index: usize,
    entries: Vec<FileEntry>,
    current_generation: usize,
    scanner: DirectoryScanner,
    volume_mgr: VolumeManager,
    current_volume: VolumeInfo,
    all_volumes: Vec<VolumeInfo>,
    bookmarks: Vec<PathBuf>,
    display_mode: PercentageDisplayMode,
    active_filter: FilterPreset,
    search_query: String,
    show_hidden: bool,
    sort_column: SortColumn,
    sort_direction: SortDirection,
    is_editing_path: bool,
    path_edit_buffer: String,
    selected_entry: Option<FileEntry>,
    pending_trash: Option<FileEntry>,
    pub status_notice: Option<(String, Instant)>,
}

impl DiskLensApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, initial_path_override: Option<PathBuf>) -> Self {
        let initial_path = initial_path_override
            .filter(|p| p.is_dir())
            .or_else(|| {
                dirs::home_dir().map(|h| {
                    let git_sc = h.join("git").join("spacecorps");
                    if git_sc.exists() {
                        git_sc
                    } else {
                        let git = h.join("git");
                        if git.exists() { git } else { h }
                    }
                })
            })
            .unwrap_or_else(|| PathBuf::from("/"));

        let volume_mgr = VolumeManager::new();
        let all_volumes = volume_mgr.get_all_volumes();
        let current_volume = volume_mgr.get_volume_for_path(&initial_path);

        let mut scanner = DirectoryScanner::new();
        let (entries, gen_id) = scanner.load_directory(&initial_path);

        // Default bookmarks
        let mut bookmarks = Vec::new();
        if let Some(home) = dirs::home_dir() {
            let sc = home.join("git").join("spacecorps");
            if sc.exists() {
                bookmarks.push(sc);
            }
            let git = home.join("git");
            if git.exists() {
                bookmarks.push(git);
            }
            bookmarks.push(home);
        }

        Self {
            current_path: initial_path.clone(),
            history: vec![initial_path],
            history_index: 0,
            entries,
            current_generation: gen_id,
            scanner,
            volume_mgr,
            current_volume,
            all_volumes,
            bookmarks,
            display_mode: PercentageDisplayMode::DiskPercent,
            active_filter: FilterPreset::All,
            search_query: String::new(),
            show_hidden: false,
            sort_column: SortColumn::Size,
            sort_direction: SortDirection::Descending,
            is_editing_path: false,
            path_edit_buffer: String::new(),
            selected_entry: None,
            pending_trash: None,
            status_notice: None,
        }
    }

    pub fn navigate_to(&mut self, path: PathBuf) {
        let canonical = path.canonicalize().unwrap_or(path);
        if !canonical.is_dir() {
            return;
        }

        if canonical == self.current_path {
            return;
        }

        // Truncate forward history if we are in the middle
        if self.history_index + 1 < self.history.len() {
            self.history.truncate(self.history_index + 1);
        }
        self.history.push(canonical.clone());
        self.history_index = self.history.len() - 1;

        self.load_path(canonical);
    }

    fn load_path(&mut self, path: PathBuf) {
        self.current_path = path;
        self.selected_entry = None;
        self.is_editing_path = false;
        self.current_volume = self.volume_mgr.get_volume_for_path(&self.current_path);

        let (entries, gen_id) = self.scanner.load_directory(&self.current_path);
        self.entries = entries;
        self.current_generation = gen_id;
    }

    pub fn refresh(&mut self) {
        self.volume_mgr.refresh();
        self.all_volumes = self.volume_mgr.get_all_volumes();
        self.current_volume = self.volume_mgr.get_volume_for_path(&self.current_path);

        let (entries, gen_id) = self.scanner.load_directory(&self.current_path);
        self.entries = entries;
        self.current_generation = gen_id;
    }

    pub fn go_back(&mut self) {
        if self.history_index > 0 {
            self.history_index -= 1;
            let target = self.history[self.history_index].clone();
            self.load_path(target);
        }
    }

    pub fn go_forward(&mut self) {
        if self.history_index + 1 < self.history.len() {
            self.history_index += 1;
            let target = self.history[self.history_index].clone();
            self.load_path(target);
        }
    }

    pub fn go_up(&mut self) {
        if let Some(parent) = self.current_path.parent() {
            self.navigate_to(parent.to_path_buf());
        }
    }
}

impl eframe::App for DiskLensApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx: Context = ui.ctx().clone();

        // Drain incoming background scanner messages
        let mut received_any = false;
        while let Ok(msg) = self.scanner.receiver().try_recv() {
            if msg.generation_id == self.current_generation {
                if let Some(entry) = self.entries.iter_mut().find(|e| e.path == msg.path) {
                    entry.size_bytes = msg.size_bytes;
                    entry.file_count = msg.file_count;
                    entry.is_scanning = false;
                }
                self.scanner
                    .cache_result(msg.path, msg.size_bytes, msg.file_count);
                received_any = true;
            }
        }

        let is_still_scanning = self.entries.iter().any(|e| e.is_scanning);
        if received_any || is_still_scanning {
            ctx.request_repaint();
        }

        // Keyboard shortcuts
        if !self.is_editing_path {
            if ctx.input(|i| i.modifiers.alt && i.key_pressed(egui::Key::ArrowLeft)) {
                self.go_back();
            } else if ctx.input(|i| i.modifiers.alt && i.key_pressed(egui::Key::ArrowRight)) {
                self.go_forward();
            } else if ctx.input(|i| i.modifiers.alt && i.key_pressed(egui::Key::ArrowUp)) {
                self.go_up();
            } else if ctx
                .input(|i| (i.modifiers.command || i.modifiers.ctrl) && i.key_pressed(egui::Key::R))
            {
                self.refresh();
            } else if ctx
                .input(|i| (i.modifiers.command || i.modifiers.ctrl) && i.key_pressed(egui::Key::L))
            {
                self.path_edit_buffer = self.current_path.to_string_lossy().to_string();
                self.is_editing_path = true;
            }
        }

        let current_folder_total_size: u64 = self.entries.iter().map(|e| e.size_bytes).sum();
        let scanning_count = self.entries.iter().filter(|e| e.is_scanning).count();

        // Top Panel: Navigation and Disk Banner
        Panel::top("top_panel").show(ui, |ui| {
            ui.add_space(4.0);
            let can_back = self.history_index > 0;
            let can_forward = self.history_index + 1 < self.history.len();

            let top_action = render_top_bar(
                ui,
                &self.current_path,
                can_back,
                can_forward,
                &mut self.is_editing_path,
                &mut self.path_edit_buffer,
                &mut self.search_query,
                &mut self.display_mode,
                &mut self.show_hidden,
            );

            if let Some(target) = top_action.navigate_to {
                self.navigate_to(target);
            } else if top_action.go_back {
                self.go_back();
            } else if top_action.go_forward {
                self.go_forward();
            } else if top_action.go_up {
                self.go_up();
            } else if top_action.refresh {
                self.refresh();
            }

            ui.add_space(4.0);
            render_disk_banner(ui, &self.current_volume, current_folder_total_size);
            ui.add_space(2.0);
        });

        // Bottom Panel: Status and Quick Actions
        let active_notice = self.status_notice.as_ref().and_then(|(msg, time)| {
            if time.elapsed().as_secs() < 4 {
                Some(msg.as_str())
            } else {
                None
            }
        });

        Panel::bottom("bottom_panel").show(ui, |ui| {
            ui.add_space(4.0);
            let bottom_action = render_bottom_bar(
                ui,
                self.entries.len(),
                current_folder_total_size,
                scanning_count,
                &self.current_volume,
                self.selected_entry.as_ref(),
                active_notice,
            );

            if bottom_action.reveal_selected
                && let Some(ref sel) = self.selected_entry
            {
                reveal_in_finder(&sel.path);
            } else if bottom_action.trash_selected
                && let Some(ref sel) = self.selected_entry
            {
                self.pending_trash = Some(sel.clone());
            }
            ui.add_space(4.0);
        });

        // Left Sidebar: Navigation Shortcuts & Disks
        Panel::left("left_sidebar")
            .resizable(true)
            .default_size(210.0)
            .show(ui, |ui| {
                ui.add_space(6.0);
                let sidebar_action = render_sidebar(
                    ui,
                    &self.current_path,
                    &self.all_volumes,
                    &self.bookmarks,
                    &mut self.active_filter,
                );

                if let Some(target) = sidebar_action.navigate_to {
                    self.navigate_to(target);
                }
                if sidebar_action.pin_current && !self.bookmarks.contains(&self.current_path) {
                    self.bookmarks.push(self.current_path.clone());
                }
                if let Some(idx) = sidebar_action.unpin_index
                    && idx < self.bookmarks.len()
                {
                    self.bookmarks.remove(idx);
                }
                if sidebar_action.clear_cache {
                    self.scanner.clear_cache();
                    self.status_notice =
                        Some(("Folder size cache cleared".to_string(), Instant::now()));
                    self.refresh();
                }
            });

        // Central Panel: The File Explorer Table
        CentralPanel::default().show(ui, |ui| {
            let explorer_action = render_explorer(
                ui,
                &self.entries,
                &self.current_volume,
                current_folder_total_size,
                self.display_mode,
                self.active_filter,
                &self.search_query,
                self.show_hidden,
                &mut self.sort_column,
                &mut self.sort_direction,
                self.selected_entry.as_ref().map(|e| e.path.as_path()),
            );

            if let Some(target) = explorer_action.navigate_to {
                self.navigate_to(target);
            }
            if let Some(file_path) = explorer_action.open_file {
                let _ = open::that(file_path);
            }
            if let Some(target) = explorer_action.reveal_in_finder {
                reveal_in_finder(&target);
            }
            if let Some(target) = explorer_action.open_terminal {
                open_terminal_at(&target);
            }
            if let Some(entry) = explorer_action.prompt_trash {
                self.pending_trash = Some(entry);
            }
            if let Some(entry) = explorer_action.select_entry {
                self.selected_entry = Some(entry);
            }
        });

        // Trash Confirmation Modal
        if let Some(ref entry) = self.pending_trash
            && let Some(modal_res) = render_trash_modal(&ctx, entry)
        {
            match modal_res {
                TrashModalResult::Confirmed(item) => {
                    let _ = trash::delete(&item.path);
                    self.status_notice =
                        Some((format!("Moved '{}' to Trash", item.name), Instant::now()));
                    self.pending_trash = None;
                    self.refresh();
                }
                TrashModalResult::Cancelled => {
                    self.pending_trash = None;
                }
            }
        }
    }
}

fn reveal_in_finder(path: &Path) {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg("-R")
            .arg(path)
            .spawn();
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = open::that(path);
    }
}

fn open_terminal_at(path: &Path) {
    let dir = if path.is_dir() {
        path
    } else {
        path.parent().unwrap_or(path)
    };

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg("-a")
            .arg("Terminal")
            .arg(dir)
            .spawn();
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = open::that(dir);
    }
}
