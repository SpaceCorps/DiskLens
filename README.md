# DiskLens 🔍💾

[![CI](https://github.com/SpaceCorps/DiskLens/actions/workflows/ci.yml/badge.svg)](https://github.com/SpaceCorps/DiskLens/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-edition%202024-orange.svg)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey.svg)]()

> High-performance native desktop file explorer and disk space visualizer built in Rust with real-time percentage fill gauges.

Easily find the culprit directories silently swallowing your hard drive. Instead of guessing which repository or cache folder is eating your storage, **DiskLens** displays real-time, color-coded fill bars showing exactly what percentage of your whole disk each folder consumes.

---

## ✨ Features

- **📊 Real-Time Disk & Folder Percentage Gauges:**
  - **`💾 Disk % Mode`**: See exactly what percentage of your entire storage drive (e.g. 1 TB Macintosh HD) each folder occupies.
  - **`📁 Folder % Mode`**: See the proportion an item occupies relative to its siblings in the current directory.
  - **`⚡ Dual Mode`**: Displays directory percentage and global disk percentage side-by-side.
- **🎨 Severity-Aware Color Feedback:**
  - 🚨 **Massive (> 5.0% of disk)**: Vibrant Crimson (`#EF4444`)
  - ⚠️ **Heavy (1.0% - 5.0% of disk)**: Warm Amber (`#F59E0B`)
  - 🔷 **Moderate (0.2% - 1.0% of disk)**: Sky Blue (`#38BDF8`)
  - 🟢 **Light (< 0.2% of disk)**: Mint Emerald (`#10B981`)
- **⚡ Asynchronous Multi-Threaded Engine:**
  - Built with `rayon` and `walkdir` on a background worker pool.
  - Immediate child item enumeration (< 1 ms); subdirectory sizes stream back progressively without UI stutter.
  - In-memory cache for instant navigation back and forth.
  - Automatic task cancellation when changing directories to conserve CPU.
- **🗺️ Complete Desktop File Explorer:**
  - Interactive breadcrumbs with 1-click jump to any parent level.
  - Full history traversal (`◀ Back`, `▶ Forward`, `▲ Up`).
  - Direct path input (`Cmd+L` or edit button).
  - Quick access sidebar: Home, Git Repos, SpaceCorps directory, Downloads, Documents, Desktop, Root.
  - Storage drive selector: Live status and mount switcher for all connected drives.
  - Pinned bookmarks: Star any directory to pin it to your favorites.
- **📦 Developer Cache Isolation:**
  - Quick filter presets: All Items, Folders Only, Heavy (>100MB), Massive (>1GB), and Dev Artifacts (`.git`, `node_modules`, `target`, `build`).
- **🛠️ Direct Action Suite:**
  - 🔍 **Reveal in Finder** (`Cmd`-click or action icon)
  - 💻 **Open in Terminal** directly at the target path
  - 📋 **Copy Path** to clipboard
  - 🗑️ **Safe Move to Trash** with reclaimable space confirmation preview

---

## 🚀 Quick Start

### Installation from Source

Ensure you have the Rust toolchain installed:

```bash
# Clone the repository
git clone https://github.com/SpaceCorps/DiskLens.git
cd DiskLens

# Run tests
cargo test

# Run DiskLens
cargo run --release
```

### Launch at a Specific Directory

```bash
# Launch directly in your git folder
cargo run --release -- ~/git/spacecorps

# Or when installed via cargo install --path .
disklens /path/to/folder
```

---

## ⌨️ Keyboard Shortcuts

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

## 🏗️ Architecture

```
src/
├── main.rs            # Native viewport configuration, dark theme setup, and CLI args
├── app.rs             # DiskLensApp state machine and panel event dispatch
├── model.rs           # Core data structures (FileEntry, VolumeInfo, formatting)
├── scanner.rs         # Rayon background worker pool, caching, and WalkDir traversal
├── volume.rs          # Real-time disk capacity via statvfs and sysinfo
└── ui/
    ├── mod.rs         # UI component exports
    ├── top_bar.rs     # Navigation buttons, interactive breadcrumbs, search, gauge toggles
    ├── disk_banner.rs # Multi-segmented disk capacity breakdown visualizer
    ├── sidebar.rs     # Storage drives, quick access folders, bookmarks, preset filters
    ├── explorer.rs    # Central table with sortable columns, custom % bars, and action icons
    ├── bottom_bar.rs  # Scan progress indicator, item statistics, and selected item inspect
    ├── trash_modal.rs # Safe deletion confirmation modal
    └── theme.rs       # SpaceCorps aesthetic palette and custom painter widgets
```

---

## 🛡️ License

MIT License. See [LICENSE](LICENSE) for details.
Built with ❤️ by **SpaceCorps**.
