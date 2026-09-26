use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryType {
    Directory,
    File,
    Symlink,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileCategory {
    Folder,
    Git,
    BuildArtifact,
    Archive,
    Media,
    Code,
    Document,
    Generic,
}

impl FileCategory {
    pub fn from_path(path: &Path, is_dir: bool) -> Self {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let lower = name.to_lowercase();

        if is_dir {
            if lower == ".git" {
                return FileCategory::Git;
            }
            if matches!(
                lower.as_str(),
                "node_modules"
                    | "target"
                    | "dist"
                    | "build"
                    | ".next"
                    | ".nuxt"
                    | "vendor"
                    | "venv"
                    | ".venv"
                    | "__pycache__"
                    | "deriveddata"
            ) {
                return FileCategory::BuildArtifact;
            }
            return FileCategory::Folder;
        }

        // Extension check
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "rs" | "ts" | "js" | "jsx" | "tsx" | "py" | "c" | "cpp" | "h" | "go" | "java"
            | "swift" | "kt" | "rb" | "php" | "html" | "css" | "json" | "toml" | "yaml" | "yml"
            | "xml" | "sql" | "sh" | "zsh" | "bash" => FileCategory::Code,
            "zip" | "tar" | "gz" | "tgz" | "7z" | "rar" | "bz2" | "xz" | "dmg" | "iso" | "pkg"
            | "deb" => FileCategory::Archive,
            "mp4" | "mov" | "mkv" | "avi" | "webm" | "mp3" | "wav" | "flac" | "aac" | "png"
            | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "heic" => FileCategory::Media,
            "pdf" | "doc" | "docx" | "md" | "txt" | "rtf" | "epub" | "csv" | "xlsx" | "pptx" => {
                FileCategory::Document
            }
            _ => FileCategory::Generic,
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            FileCategory::Folder => "📁",
            FileCategory::Git => "🐙",
            FileCategory::BuildArtifact => "📦",
            FileCategory::Archive => "🗜️",
            FileCategory::Media => "🎬",
            FileCategory::Code => "💻",
            FileCategory::Document => "📄",
            FileCategory::Generic => "📄",
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub entry_type: EntryType,
    pub category: FileCategory,
    pub size_bytes: u64,
    pub file_count: u64,
    pub modified: Option<SystemTime>,
    pub is_scanning: bool,
    pub is_hidden: bool,
}

impl FileEntry {
    pub fn is_dir(&self) -> bool {
        self.entry_type == EntryType::Directory
    }

    pub fn disk_percent(&self, total_disk: u64) -> f64 {
        if total_disk == 0 {
            0.0
        } else {
            (self.size_bytes as f64 / total_disk as f64) * 100.0
        }
    }

    pub fn folder_percent(&self, total_folder: u64) -> f64 {
        if total_folder == 0 {
            0.0
        } else {
            (self.size_bytes as f64 / total_folder as f64) * 100.0
        }
    }
}

#[derive(Debug, Clone)]
pub struct VolumeInfo {
    pub mount_point: PathBuf,
    pub name: String,
    pub fs_type: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub used_bytes: u64,
}

impl VolumeInfo {
    pub fn used_percent(&self) -> f64 {
        if self.total_bytes == 0 {
            0.0
        } else {
            (self.used_bytes as f64 / self.total_bytes as f64) * 100.0
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    Name,
    Size,
    FillPercent,
    ItemCount,
    Modified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PercentageDisplayMode {
    DiskPercent,
    FolderPercent,
    Dual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterPreset {
    All,
    FoldersOnly,
    HeavyOnly,    // > 100MB
    MassiveOnly,  // > 1GB
    DevArtifacts, // .git, node_modules, target, build
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    const TB: f64 = GB * 1024.0;

    let b = bytes as f64;
    if b >= TB {
        format!("{:.2} TB", b / TB)
    } else if b >= GB {
        format!("{:.2} GB", b / GB)
    } else if b >= MB {
        format!("{:.1} MB", b / MB)
    } else if b >= KB {
        format!("{:.1} KB", b / KB)
    } else {
        format!("{} B", bytes)
    }
}

pub fn format_count(count: u64) -> String {
    if count >= 1_000_000 {
        format!("{:.1}M items", count as f64 / 1_000_000.0)
    } else if count >= 1_000 {
        format!("{:.1}k items", count as f64 / 1_000.0)
    } else if count == 1 {
        "1 item".to_string()
    } else {
        format!("{} items", count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.0 MB");
        assert_eq!(format_bytes(50 * 1024 * 1024), "50.0 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
        assert_eq!(format_bytes(35 * 1024 * 1024 * 1024), "35.00 GB");
        assert_eq!(format_bytes(2 * 1024 * 1024 * 1024 * 1024), "2.00 TB");
    }

    #[test]
    fn test_format_count() {
        assert_eq!(format_count(0), "0 items");
        assert_eq!(format_count(1), "1 item");
        assert_eq!(format_count(42), "42 items");
        assert_eq!(format_count(1500), "1.5k items");
        assert_eq!(format_count(2_500_000), "2.5M items");
    }

    #[test]
    fn test_percentage_calculations() {
        let entry = FileEntry {
            path: PathBuf::from("/Users/rorychatt/git/spacecorps/test"),
            name: "test".to_string(),
            entry_type: EntryType::Directory,
            category: FileCategory::Folder,
            size_bytes: 35_000_000_000, // 35 GB
            file_count: 5000,
            modified: None,
            is_scanning: false,
            is_hidden: false,
        };

        // 35 GB on 1000 GB disk = 3.5%
        let disk_total = 1_000_000_000_000;
        let disk_pct = entry.disk_percent(disk_total);
        assert!((disk_pct - 3.5).abs() < 0.001);

        // 35 GB inside 70 GB parent folder = 50%
        let folder_total = 70_000_000_000;
        let folder_pct = entry.folder_percent(folder_total);
        assert!((folder_pct - 50.0).abs() < 0.001);

        // Zero total safety
        assert_eq!(entry.disk_percent(0), 0.0);
        assert_eq!(entry.folder_percent(0), 0.0);
    }

    #[test]
    fn test_category_detection() {
        assert_eq!(
            FileCategory::from_path(Path::new(".git"), true),
            FileCategory::Git
        );
        assert_eq!(
            FileCategory::from_path(Path::new("node_modules"), true),
            FileCategory::BuildArtifact
        );
        assert_eq!(
            FileCategory::from_path(Path::new("target"), true),
            FileCategory::BuildArtifact
        );
        assert_eq!(
            FileCategory::from_path(Path::new("main.rs"), false),
            FileCategory::Code
        );
        assert_eq!(
            FileCategory::from_path(Path::new("archive.zip"), false),
            FileCategory::Archive
        );
        assert_eq!(
            FileCategory::from_path(Path::new("video.mp4"), false),
            FileCategory::Media
        );
        assert_eq!(
            FileCategory::from_path(Path::new("README.md"), false),
            FileCategory::Document
        );
    }
}
