# DiskLens

[![CI](https://github.com/SpaceCorps/DiskLens/actions/workflows/ci.yml/badge.svg)](https://github.com/SpaceCorps/DiskLens/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-edition%202024-orange.svg)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey.svg)]()

> High-speed disk space visualizer and native file explorer built in Rust with real-time percentage fill gauges.

Easily find the culprit directories silently swallowing your hard drive. Instead of guessing which repository, build cache, or dependencies folder is eating your storage, **DiskLens** displays real-time, color-coded fill bars showing exactly what percentage of your whole disk each folder consumes.

Operates as a **blazingly fast terminal CLI** by default, with an optional **native desktop GUI**.

---

## Features

- **Real-Time Disk & Folder Percentage Gauges:**
  - **Disk % Mode**: See exactly what percentage of your entire storage drive (e.g. 1 TB drive) each folder occupies.
  - **Folder % Mode**: See the proportion an item occupies relative to its siblings in the current directory.
  - **Dual Mode**: Displays directory percentage and global disk percentage side-by-side.
- **Severity-Aware Color Feedback:**
  - **Massive (> 5.0% of disk)**: Vibrant Crimson (`#EF4444`)
  - **Heavy (1.0% - 5.0% of disk)**: Warm Amber (`#F59E0B`)
  - **Moderate (0.2% - 1.0% of disk)**: Sky Blue (`#38BDF8`)
  - **Light (< 0.2% of disk)**: Mint Emerald (`#10B981`)
- **Fast Parallel Traversal Engine:**
  - Powered by `rayon` parallel iterators across all CPU cores and `libc::statvfs` block metrics.
  - Terminal scan completes in tens of milliseconds without loading any window manager or GPU stack.
  - In GUI mode, child items enumerate immediately (< 1 ms) while subdirectory sizes stream back asynchronously.
  - In-memory cache for instant navigation back and forth.
- **Standalone Terminal CLI & Desktop GUI:**
  - Run directly in your terminal for instant disk breakdowns with ASCII fill bars.
  - Pipe structured JSON output to scripts or terminal workflows (`--json`).
  - Pass `--gui` when you want the full native desktop file explorer window.
- **Desktop File Explorer:**
  - Interactive breadcrumbs with 1-click jump to any parent level.
  - Full history traversal (`Back`, `Forward`, `Up`).
  - Direct path input (`Cmd+L` or edit button).
  - Quick access sidebar: Home, Git Repos, SpaceCorps directory, Downloads, Documents, Desktop, Root.
  - Storage drive selector: Live status and mount switcher for all connected drives.
  - Pinned bookmarks: Save favorite directories for quick access.
- **Developer Cache Isolation:**
  - Quick filter presets: All Items, Folders Only, Heavy (>100MB), Massive (>1GB), and Dev Artifacts (`.git`, `node_modules`, `target`, `build`).
- **Direct Action Suite:**
  - **Reveal in Finder** (`Cmd`-click or Open button)
  - **Open in Terminal** directly at the target path
  - **Copy Path** to clipboard
  - **Safe Move to Trash** with reclaimable space confirmation preview

---

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/SpaceCorps/DiskLens.git
cd DiskLens

# Install binary to cargo path
cargo install --path .
```

### CLI Usage (Default — Fast Terminal Mode)

No window manager or desktop app download required:

```bash
# Scan current directory
disklens

# Scan specific folder (e.g. ~/git)
disklens ~/git

# Display top 20 heaviest items
disklens ~/git -n 20

# Output machine-readable JSON
disklens ~/git --json
```

Example CLI Output:

```
DiskLens — Fast Disk Space Analyzer
Target: /Users/rorychatt/git
Volume: Macintosh HD (apfs) | Total: 926.35 GB | Free: 35.12 GB (96.2% used)

  SIZE         DISK %   DIR %   VISUAL GAUGE       NAME
--------------------------------------------------------------------------------
  45.20 GB      4.88%   42.1%   [████████░░░░]     Space3d/
  18.10 GB      1.95%   16.9%   [███░░░░░░░░░]     DiskLens/
   9.30 GB      1.00%    8.7%   [██░░░░░░░░░░]     hangar/
 820.00 MB      0.09%    0.8%   [░░░░░░░░░░░░]     installer.dmg
--------------------------------------------------------------------------------
Total: 107.40 GB in 14 items (11.59% of disk)
```

### Desktop GUI Mode

Launch the interactive desktop window with `--gui`:

```bash
# Launch GUI at current directory
disklens --gui

# Launch GUI at specific path
disklens ~/git/spacecorps --gui
```

---

## Keyboard Shortcuts (GUI)

| Shortcut | Action |
| :--- | :--- |
| `Alt + Left` / `Backspace` | Navigate Back in history |
| `Alt + Right` | Navigate Forward in history |
| `Alt + Up` | Jump to Parent directory |
| `Cmd + R` / `Ctrl + R` | Refresh / Rescan current directory |
| `Cmd + L` / `Ctrl + L` | Focus direct path text editor |
| `Escape` | Cancel path editing / close dialogs |
| `Enter` | Navigate into selected folder or submit path |

---

## Architecture

```
src/
├── main.rs            # CLI / GUI dispatcher, arg parsing, terminal runner
├── cli.rs             # Fast parallel CLI scanner, statvfs, ASCII bar visualizer, JSON output
├── app.rs             # DiskLensApp desktop state machine and panel event dispatch
├── model.rs           # Core data structures (FileEntry, VolumeInfo, formatting)
├── scanner.rs         # Rayon background worker pool, caching, and WalkDir traversal
├── volume.rs          # Real-time disk capacity via statvfs and sysinfo
└── ui/
    ├── mod.rs         # UI component exports
    ├── top_bar.rs     # Navigation buttons, interactive breadcrumbs, search, gauge toggles
    ├── disk_banner.rs # Multi-segmented disk capacity breakdown visualizer
    ├── sidebar.rs     # Storage drives, quick access folders, bookmarks, preset filters
    ├── explorer.rs    # Central table with sortable columns, custom % bars, and action buttons
    ├── bottom_bar.rs  # Scan progress indicator, item statistics, and selected item inspect
    ├── trash_modal.rs # Safe deletion confirmation modal
    └── theme.rs       # SpaceCorps aesthetic palette and custom painter widgets
```

---

## License

MIT License. See [LICENSE](LICENSE) for details.
Built by **SpaceCorps**.
