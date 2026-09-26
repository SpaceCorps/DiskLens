# AGENTS.md — DiskLens Architecture & Agent Reference

## Overview

`DiskLens` is a native Rust desktop file explorer and storage visualizer designed under the SpaceCorps organization. Its primary purpose is pinpointing high-volume directories and caches by computing exact percentage shares of both the host storage disk (via `statvfs`/`sysinfo`) and the parent directory.

## Core Architectural Invariants

1. **Non-blocking UI:**
   - The UI runs on `egui` at 60 FPS.
   - `std::fs::read_dir` is used for shallow enumeration of immediate entries in the main thread (< 1ms).
   - Deep recursive sizing is strictly offloaded to `DirectoryScanner` which utilizes `rayon` parallel iterators and `walkdir`.
   - Results are passed via `crossbeam_channel::unbounded`.

2. **Generation Cancellation:**
   - Navigating to a new directory immediately increments `current_generation` and flips `cancel_flag` to true.
   - Background recursive scanners check `cancel.load(Ordering::Relaxed)` every 256 entries to quickly abort stale traversal jobs when the user browses rapidly.

3. **Disk Calculation Correctness:**
   - Total disk capacity is acquired using `statvfs` blocks * fragment size and matched with `sysinfo::Disks`.
   - Percentage formulas:
     - `disk_percent = (entry.size_bytes / volume.total_bytes) * 100.0`
     - `folder_percent = (entry.size_bytes / current_folder_total_size) * 100.0`
   - Both formulas guard against division-by-zero (`total == 0 => 0.0`).

4. **Safe Deletion:**
   - File removal is performed via `trash::delete` to ensure items are moved to the OS Recycle Bin / Trash rather than permanently unlinked.

## File Organization

- `src/model.rs`: Core types (`FileEntry`, `VolumeInfo`, `PercentageDisplayMode`, `FileCategory`, `format_bytes`).
- `src/volume.rs`: Mount point detection and volume metrics via `sysinfo` and `statvfs`.
- `src/scanner.rs`: Background multi-threaded recursive size scanner and cache.
- `src/ui/`: UI modules (`top_bar`, `disk_banner`, `sidebar`, `explorer`, `bottom_bar`, `trash_modal`, `theme`).
- `src/app.rs`: State machine implementing `eframe::App`.
- `src/main.rs`: Entry point and CLI flag parsing (`--help`, `--version`, directory path).
