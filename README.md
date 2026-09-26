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
  - **Reveal in Finder / File Manager** (`Cmd`-click or Open button)
  - **Open in Terminal** directly at the target path
  - **Copy Path** to clipboard
  - **Safe Move to Trash** with reclaimable space confirmation preview

---

## Download & Installation

DiskLens is distributed as a single unified executable that powers both the **Terminal CLI** and the **Native Desktop Application**.

### Option 1: Pre-Built Binaries (GitHub Releases)

Standalone pre-compiled binaries are published on the [GitHub Releases Page](https://github.com/SpaceCorps/DiskLens/releases).

| Platform | Architecture | Binary Package | Mode Included |
| :--- | :--- | :--- | :--- |
| **macOS** | Apple Silicon (M1/M2/M3/M4) | [`disklens-v0.1.0-aarch64-apple-darwin.tar.gz`](https://github.com/SpaceCorps/DiskLens/releases/latest) | CLI + Desktop GUI |
| **macOS** | Intel (x86_64) | [`disklens-v0.1.0-x86_64-apple-darwin.tar.gz`](https://github.com/SpaceCorps/DiskLens/releases/latest) | CLI + Desktop GUI |
| **Linux** | x86_64 (glibc) | [`disklens-v0.1.0-x86_64-unknown-linux-gnu.tar.gz`](https://github.com/SpaceCorps/DiskLens/releases/latest) | CLI + Desktop GUI |
| **Windows** | x86_64 | [`disklens-v0.1.0-x86_64-pc-windows-msvc.zip`](https://github.com/SpaceCorps/DiskLens/releases/latest) | CLI + Desktop GUI |

#### Quick Install to Terminal (macOS & Linux)

Download and extract directly to your local PATH (`~/.local/bin` or `/usr/local/bin`):

```bash
# macOS (Apple Silicon)
curl -fsSL https://github.com/SpaceCorps/DiskLens/releases/latest/download/disklens-v0.1.0-aarch64-apple-darwin.tar.gz | tar -xz
mv disklens ~/.local/bin/

# Linux (x86_64)
curl -fsSL https://github.com/SpaceCorps/DiskLens/releases/latest/download/disklens-v0.1.0-x86_64-unknown-linux-gnu.tar.gz | tar -xz
sudo mv disklens /usr/local/bin/
```

### Option 2: Install via Cargo

```bash
# Install directly from the GitHub repository
cargo install --git https://github.com/SpaceCorps/DiskLens

# Or clone and install locally
git clone https://github.com/SpaceCorps/DiskLens.git
cd DiskLens
cargo install --path .
```

---

## How to Use the CLI

When called in a terminal, `disklens` runs the fast parallel scanner with zero GUI overhead:

```bash
# Scan current directory
disklens

# Scan target directory (e.g. ~/git)
disklens ~/git

# Display top 10 heaviest entries
disklens ~/git -n 10

# Include hidden files & dotfolders (.git, .cache, etc.)
disklens -a

# Output structured JSON for automation or LLM tools
disklens ~/git --json
```

Example CLI Output:

```
DiskLens v0.1.0 · SpaceCorps
Volume: Macintosh HD [apfs] · 926.35 GB Total · 564.03 GB Free (39.1% used)
Target: /Users/rorychatt/git/spacecorps (178.04 GB · 19.22% of disk)

FILL GAUGE               DISK %    DIR %        SIZE     ITEMS  NAME
────────────────────────────────────────────────────────────────────────────────────
[████████████████████]     5.43%    28.2%    50.29 GB  151.4k items  Space3d/
[██████████░░░░░░░░░░]     2.77%    14.4%    25.70 GB   43.4k items  SpaceCorps2027-wt/
[██████████░░░░░░░░░░]     2.73%    14.2%    25.33 GB   73.6k items  Space3d-graphics/
[████████░░░░░░░░░░░░]     2.06%    10.7%    19.12 GB   49.6k items  Space3d-gfx-particles/
[███████░░░░░░░░░░░░░]     1.81%     9.4%    16.74 GB   68.7k items  SpaceCorps2027/
────────────────────────────────────────────────────────────────────────────────────
Total: 44 items · 178.04 GB · 19.22% of entire disk · Scanned in 5734ms
```

---

## How to Use the Desktop App

The same binary can be launched in graphical mode at any time.

### Launching the Desktop Window

```bash
# Launch desktop GUI at current directory
disklens --gui

# Launch desktop GUI focused on a specific directory
disklens ~/git/spacecorps --gui
```

### Adding to macOS Applications / Dock

To launch DiskLens from Spotlight, Finder, or your Dock without needing a terminal:

```bash
# Create a native macOS Application launcher
osacompile -e 'do shell script "disklens --gui >/dev/null 2>&1 &"' -o /Applications/DiskLens.app
```

Now you can press `Cmd + Space`, search for **DiskLens**, and launch it directly from macOS.

### Adding to Linux Applications Menu

Create a desktop entry at `~/.local/share/applications/disklens.desktop`:

```ini
[Desktop Entry]
Name=DiskLens
Comment=Disk Space Visualizer & File Explorer
Exec=disklens --gui
Icon=drive-harddisk
Terminal=false
Type=Application
Categories=Utility;System;FileManager;
```

### Windows Desktop Shortcut

1. Right-click `disklens.exe` and select **Create Shortcut**.
2. Right-click the shortcut and open **Properties**.
3. In the **Target** field, add `--gui` at the end:
   ```text
   "C:\path\to\disklens.exe" --gui
   ```
4. Double-click the shortcut to open the desktop application directly.

---

## Keyboard Shortcuts (Desktop GUI)

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
