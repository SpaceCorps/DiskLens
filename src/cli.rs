use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::time::Instant;
use walkdir::WalkDir;

use crate::model::{FileCategory, VolumeInfo, format_bytes, format_count};
use crate::volume::VolumeManager;

#[derive(Debug, Clone)]
pub struct CliArgs {
    pub path: PathBuf,
    pub top_n: usize,
    pub show_all: bool,
    pub json: bool,
    pub gui: bool,
    pub depth: usize,
    pub sort_by_name: bool,
}

impl Default for CliArgs {
    fn default() -> Self {
        Self {
            path: PathBuf::from("."),
            top_n: 20,
            show_all: false,
            json: false,
            gui: false,
            depth: 1,
            sort_by_name: false,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CliEntryOutput {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub category: String,
    pub size_bytes: u64,
    pub size_formatted: String,
    pub item_count: u64,
    pub disk_percent: f64,
    pub dir_percent: f64,
}

#[derive(Debug, serde::Serialize)]
pub struct CliVolumeOutput {
    pub name: String,
    pub mount: String,
    pub fs_type: String,
    pub total_bytes: u64,
    pub total_formatted: String,
    pub used_bytes: u64,
    pub used_formatted: String,
    pub available_bytes: u64,
    pub available_formatted: String,
    pub used_percent: f64,
}

#[derive(Debug, serde::Serialize)]
pub struct CliJsonOutput {
    pub volume: CliVolumeOutput,
    pub target_path: String,
    pub total_dir_bytes: u64,
    pub total_dir_formatted: String,
    pub total_items: usize,
    pub dir_percent_of_disk: f64,
    pub scan_duration_ms: u128,
    pub entries: Vec<CliEntryOutput>,
}

pub fn run_cli(args: &CliArgs) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();

    let target_path = args
        .path
        .canonicalize()
        .unwrap_or_else(|_| args.path.clone());

    if !target_path.exists() {
        eprintln!(
            "Error: target path '{}' does not exist",
            target_path.display()
        );
        std::process::exit(1);
    }

    if !target_path.is_dir() {
        eprintln!(
            "Error: target path '{}' is not a directory",
            target_path.display()
        );
        std::process::exit(1);
    }

    let volume_mgr = VolumeManager::new();
    let volume: VolumeInfo = volume_mgr.get_volume_for_path(&target_path);

    // Read immediate entries
    let raw_entries = match std::fs::read_dir(&target_path) {
        Ok(read_dir) => {
            let mut list = Vec::new();
            for entry in read_dir.flatten() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                let is_hidden = name.starts_with('.');

                if !args.show_all && is_hidden {
                    continue;
                }

                let metadata = match entry.metadata() {
                    Ok(m) => m,
                    Err(_) => continue,
                };

                let is_dir = metadata.is_dir();
                list.push((path, name, is_dir, metadata.len()));
            }
            list
        }
        Err(e) => {
            eprintln!("Error reading directory '{}': {}", target_path.display(), e);
            std::process::exit(1);
        }
    };

    // Scan directories in parallel across all CPU cores with Rayon
    let mut scanned_entries: Vec<CliEntryOutput> = raw_entries
        .into_par_iter()
        .map(|(path, name, is_dir, file_len)| {
            let category = FileCategory::from_path(&path, is_dir);
            let category_str = format!("{:?}", category);

            if is_dir {
                let (dir_size, item_count) = scan_subtree_fast(&path);
                CliEntryOutput {
                    name,
                    path: path.to_string_lossy().to_string(),
                    is_dir: true,
                    category: category_str,
                    size_bytes: dir_size,
                    size_formatted: format_bytes(dir_size),
                    item_count,
                    disk_percent: 0.0,
                    dir_percent: 0.0,
                }
            } else {
                CliEntryOutput {
                    name,
                    path: path.to_string_lossy().to_string(),
                    is_dir: false,
                    category: category_str,
                    size_bytes: file_len,
                    size_formatted: format_bytes(file_len),
                    item_count: 1,
                    disk_percent: 0.0,
                    dir_percent: 0.0,
                }
            }
        })
        .collect();

    let total_dir_bytes: u64 = scanned_entries.iter().map(|e| e.size_bytes).sum();
    let total_items = scanned_entries.len();

    // Compute percentages
    for entry in &mut scanned_entries {
        if volume.total_bytes > 0 {
            entry.disk_percent = (entry.size_bytes as f64 / volume.total_bytes as f64) * 100.0;
        }
        if total_dir_bytes > 0 {
            entry.dir_percent = (entry.size_bytes as f64 / total_dir_bytes as f64) * 100.0;
        }
    }

    // Sort entries
    if args.sort_by_name {
        scanned_entries.sort_by_key(|a| a.name.to_lowercase());
    } else {
        // Descending by size (heaviest folders first)
        scanned_entries.sort_by_key(|a| std::cmp::Reverse(a.size_bytes));
    }

    let elapsed = start_time.elapsed();
    let dir_percent_of_disk = if volume.total_bytes > 0 {
        (total_dir_bytes as f64 / volume.total_bytes as f64) * 100.0
    } else {
        0.0
    };

    // Output JSON mode
    if args.json {
        let json_output = CliJsonOutput {
            volume: CliVolumeOutput {
                name: volume.name.clone(),
                mount: volume.mount_point.to_string_lossy().to_string(),
                fs_type: volume.fs_type.clone(),
                total_bytes: volume.total_bytes,
                total_formatted: format_bytes(volume.total_bytes),
                used_bytes: volume.used_bytes,
                used_formatted: format_bytes(volume.used_bytes),
                available_bytes: volume.available_bytes,
                available_formatted: format_bytes(volume.available_bytes),
                used_percent: volume.used_percent(),
            },
            target_path: target_path.to_string_lossy().to_string(),
            total_dir_bytes,
            total_dir_formatted: format_bytes(total_dir_bytes),
            total_items,
            dir_percent_of_disk,
            scan_duration_ms: elapsed.as_millis(),
            entries: scanned_entries,
        };

        println!("{}", serde_json::to_string_pretty(&json_output)?);
        return Ok(());
    }

    // Human terminal formatting
    let max_size = scanned_entries
        .iter()
        .map(|e| e.size_bytes)
        .max()
        .unwrap_or(1)
        .max(1);

    let display_limit = if args.top_n == 0 {
        scanned_entries.len()
    } else {
        scanned_entries.len().min(args.top_n)
    };

    println!("\x1b[1;36mDiskLens\x1b[0m \x1b[2mv0.1.0 · SpaceCorps\x1b[0m");
    println!(
        "\x1b[1mVolume:\x1b[0m {} [{}] · {} Total · {} Free (\x1b[1;33m{:.1}% used\x1b[0m)",
        volume.name,
        volume.fs_type,
        format_bytes(volume.total_bytes),
        format_bytes(volume.available_bytes),
        volume.used_percent()
    );
    println!(
        "\x1b[1mTarget:\x1b[0m {} (\x1b[1;36m{}\x1b[0m · \x1b[1m{:.2}% of disk\x1b[0m)",
        target_path.display(),
        format_bytes(total_dir_bytes),
        dir_percent_of_disk
    );
    println!();

    // Table header
    println!(
        "\x1b[2m{:<22} {:>8}  {:>7}  {:>10}  {:>8}  NAME\x1b[0m",
        "FILL GAUGE", "DISK %", "DIR %", "SIZE", "ITEMS"
    );
    println!("\x1b[2m{}\x1b[0m", "─".repeat(84));

    for entry in scanned_entries.iter().take(display_limit) {
        let fill_frac = (entry.size_bytes as f64 / max_size as f64).clamp(0.0, 1.0);
        let bar = render_ascii_bar(fill_frac, 20);

        let color_code = color_code_for_disk_pct(entry.disk_percent);
        let name_suffix = if entry.is_dir { "/" } else { "" };
        let count_str = if entry.is_dir {
            format_count(entry.item_count)
        } else {
            "1".to_string()
        };

        println!(
            "{color_code}[{bar}]\x1b[0m  {:>7.2}%  {:>6.1}%  {:>10}  {:>8}  {color_code}{}{name_suffix}\x1b[0m",
            entry.disk_percent, entry.dir_percent, entry.size_formatted, count_str, entry.name
        );
    }

    if scanned_entries.len() > display_limit {
        println!(
            "\x1b[2m... and {} more entries (use -a / --all to display all)\x1b[0m",
            scanned_entries.len() - display_limit
        );
    }

    println!("\x1b[2m{}\x1b[0m", "─".repeat(84));
    println!(
        "\x1b[1mTotal:\x1b[0m {} items · {} · \x1b[1m{:.2}% of entire disk\x1b[0m · Scanned in \x1b[1;32m{}ms\x1b[0m",
        total_items,
        format_bytes(total_dir_bytes),
        dir_percent_of_disk,
        elapsed.as_millis()
    );

    Ok(())
}

fn scan_subtree_fast(dir: &Path) -> (u64, u64) {
    let mut total_size = 0u64;
    let mut total_count = 0u64;

    for entry in WalkDir::new(dir)
        .min_depth(1)
        .into_iter()
        .filter_entry(|e| !e.path_is_symlink())
        .flatten()
    {
        total_count += 1;
        if let Ok(meta) = entry.metadata()
            && meta.is_file()
        {
            total_size += meta.len();
        }
    }

    (total_size, total_count)
}

fn render_ascii_bar(fraction: f64, width: usize) -> String {
    let filled_chars = (fraction * width as f64).round() as usize;
    let filled = filled_chars.min(width);
    let empty = width.saturating_sub(filled);

    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

fn color_code_for_disk_pct(percent: f64) -> &'static str {
    if percent >= 5.0 {
        "\x1b[1;31m" // Bold Red / Crimson
    } else if percent >= 1.0 {
        "\x1b[1;33m" // Bold Yellow / Amber
    } else if percent >= 0.2 {
        "\x1b[1;36m" // Bold Cyan
    } else {
        "\x1b[32m" // Green
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_ascii_bar() {
        assert_eq!(render_ascii_bar(0.0, 10), "░░░░░░░░░░");
        assert_eq!(render_ascii_bar(0.5, 10), "█████░░░░░");
        assert_eq!(render_ascii_bar(1.0, 10), "██████████");
    }

    #[test]
    fn test_color_code_for_disk_pct() {
        assert_eq!(color_code_for_disk_pct(5.5), "\x1b[1;31m");
        assert_eq!(color_code_for_disk_pct(2.0), "\x1b[1;33m");
        assert_eq!(color_code_for_disk_pct(0.5), "\x1b[1;36m");
        assert_eq!(color_code_for_disk_pct(0.1), "\x1b[32m");
    }
}
