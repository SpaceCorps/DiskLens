#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod cli;
mod model;
mod scanner;
mod ui;
mod volume;

use app::DiskLensApp;
use cli::{CliArgs, run_cli};
use eframe::egui;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let raw_args: Vec<String> = std::env::args().collect();

    let mut cli_args = CliArgs::default();
    let mut positional_path: Option<PathBuf> = None;

    let mut i = 1;
    while i < raw_args.len() {
        match raw_args[i].as_str() {
            "-h" | "--help" => {
                print_help();
                return Ok(());
            }
            "-V" | "--version" => {
                println!("disklens 0.1.0");
                return Ok(());
            }
            "--gui" => {
                cli_args.gui = true;
            }
            "--json" => {
                cli_args.json = true;
            }
            "-a" | "--all" => {
                cli_args.show_all = true;
                cli_args.top_n = 0;
            }
            "--sort-name" => {
                cli_args.sort_by_name = true;
            }
            "-n" | "--top" => {
                if i + 1 < raw_args.len() {
                    i += 1;
                    cli_args.top_n = raw_args[i].parse().unwrap_or(20);
                }
            }
            "-d" | "--depth" => {
                if i + 1 < raw_args.len() {
                    i += 1;
                    cli_args.depth = raw_args[i].parse().unwrap_or(1);
                }
            }
            other => {
                if !other.starts_with('-') && positional_path.is_none() {
                    positional_path = Some(PathBuf::from(other));
                }
            }
        }
        i += 1;
    }

    if let Some(p) = positional_path {
        cli_args.path = p;
    }

    // If --gui flag is passed, launch the desktop app
    if cli_args.gui {
        let initial_path = if cli_args.path.as_os_str() == "." {
            None
        } else {
            Some(cli_args.path.clone())
        };

        let native_options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1120.0, 740.0])
                .with_min_inner_size([820.0, 500.0])
                .with_title("DiskLens — Disk Space File Explorer")
                .with_active(true),
            ..Default::default()
        };

        if let Err(e) = eframe::run_native(
            "DiskLens",
            native_options,
            Box::new(move |cc| {
                let mut visuals = egui::Visuals::dark();
                visuals.panel_fill = ui::theme::ThemeColors::BG_APP;
                visuals.window_fill = ui::theme::ThemeColors::BG_PANEL;
                visuals.faint_bg_color = ui::theme::ThemeColors::BG_CARD;
                visuals.extreme_bg_color = ui::theme::ThemeColors::BG_APP;
                visuals.override_text_color = Some(ui::theme::ThemeColors::TEXT_PRIMARY);

                cc.egui_ctx.set_visuals(visuals);

                Ok(Box::new(DiskLensApp::new(cc, initial_path)))
            }),
        ) {
            eprintln!("Error launching GUI: {}", e);
            std::process::exit(1);
        }

        return Ok(());
    }

    // Default fast CLI execution
    run_cli(&cli_args)
}

fn print_help() {
    println!("DiskLens v0.1.0 — Fast Disk Space Visualizer & File Explorer");
    println!();
    println!("USAGE:");
    println!("    disklens [PATH] [OPTIONS]");
    println!();
    println!("ARGUMENTS:");
    println!(
        "    [PATH]                   Target directory to inspect (defaults to current directory)"
    );
    println!();
    println!("OPTIONS:");
    println!("    -n, --top <N>            Show top N heaviest entries (default: 20)");
    println!("    -a, --all                Show all entries including hidden files and dotfolders");
    println!(
        "        --json               Output structured JSON for pipelines and agent tool calls"
    );
    println!("        --sort-name          Sort alphabetically by name instead of size");
    println!("        --gui                Launch native desktop GUI window");
    println!("    -h, --help               Print help manual");
    println!("    -V, --version            Print version");
    println!();
    println!("EXAMPLES:");
    println!("    disklens                 Scan current directory and show fill % bars");
    println!("    disklens ~/git           Inspect git directory to find heaviest repositories");
    println!("    disklens -n 5            Show the top 5 largest items in current folder");
    println!("    disklens --json          Output machine-readable breakdown");
    println!("    disklens --gui ~/git     Launch desktop GUI focused on ~/git");
}
