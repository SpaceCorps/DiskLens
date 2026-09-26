#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod model;
mod scanner;
mod ui;
mod volume;

use app::DiskLensApp;
use eframe::egui;
use std::path::PathBuf;

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let mut initial_path = None;

    if args.len() > 1 {
        let first = &args[1];
        if first == "-h" || first == "--help" {
            println!("DiskLens v0.1.0 — SpaceCorps Native Disk Space File Explorer");
            println!("\nUSAGE:");
            println!("    disklens [PATH]           Launch DiskLens starting at PATH");
            println!("    disklens -h, --help       Show this help manual");
            println!("    disklens -V, --version    Show version");
            println!("\nDESCRIPTION:");
            println!(
                "    A high-performance native desktop file explorer that visualizes folder disk space"
            );
            println!(
                "    usage percentages in real-time, helping you instantly find heavy folders on your drive."
            );
            return Ok(());
        } else if first == "-V" || first == "--version" {
            println!("disklens 0.1.0");
            return Ok(());
        } else {
            let p = PathBuf::from(first);
            if p.exists() {
                initial_path = Some(p);
            }
        }
    }

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1120.0, 740.0])
            .with_min_inner_size([820.0, 500.0])
            .with_title("DiskLens — SpaceCorps Disk Space File Explorer")
            .with_active(true),
        ..Default::default()
    };

    eframe::run_native(
        "DiskLens",
        native_options,
        Box::new(move |cc| {
            // Configure dark space theme visuals
            let mut visuals = egui::Visuals::dark();
            visuals.panel_fill = ui::theme::ThemeColors::BG_APP;
            visuals.window_fill = ui::theme::ThemeColors::BG_PANEL;
            visuals.faint_bg_color = ui::theme::ThemeColors::BG_CARD;
            visuals.extreme_bg_color = ui::theme::ThemeColors::BG_APP;
            visuals.override_text_color = Some(ui::theme::ThemeColors::TEXT_PRIMARY);

            cc.egui_ctx.set_visuals(visuals);

            Ok(Box::new(DiskLensApp::new(cc, initial_path)))
        }),
    )
}
