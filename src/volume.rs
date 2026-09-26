use std::ffi::CString;
use std::mem::MaybeUninit;
use std::path::{Path, PathBuf};
use sysinfo::Disks;

use crate::model::VolumeInfo;

pub struct VolumeManager {
    disks: Disks,
}

impl VolumeManager {
    pub fn new() -> Self {
        Self {
            disks: Disks::new_with_refreshed_list(),
        }
    }

    pub fn refresh(&mut self) {
        self.disks.refresh(true);
    }

    pub fn get_all_volumes(&self) -> Vec<VolumeInfo> {
        let mut volumes = Vec::new();
        for disk in self.disks.list() {
            let mount = disk.mount_point().to_path_buf();
            let name = disk.name().to_string_lossy().to_string();
            let fs_type = disk.file_system().to_string_lossy().to_string();
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total.saturating_sub(available);

            let display_name = if name.is_empty() {
                mount
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("/")
                    .to_string()
            } else {
                name
            };

            volumes.push(VolumeInfo {
                mount_point: mount,
                name: display_name,
                fs_type,
                total_bytes: total,
                available_bytes: available,
                used_bytes: used,
            });
        }
        volumes
    }

    pub fn get_volume_for_path(&self, path: &Path) -> VolumeInfo {
        // First try to match against known sysinfo disks by longest mount point prefix
        let mut best_match: Option<&sysinfo::Disk> = None;
        let mut best_len = 0;

        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

        for disk in self.disks.list() {
            let mount = disk.mount_point();
            if canonical.starts_with(mount) {
                let len = mount.as_os_str().len();
                if len >= best_len {
                    best_len = len;
                    best_match = Some(disk);
                }
            }
        }

        // Try statvfs for exact real-time block metrics
        let statvfs_info = get_statvfs_info(&canonical);

        if let Some(disk) = best_match {
            let total = if let Some(ref s) = statvfs_info {
                s.total_bytes
            } else {
                disk.total_space()
            };
            let available = if let Some(ref s) = statvfs_info {
                s.available_bytes
            } else {
                disk.available_space()
            };
            let used = total.saturating_sub(available);

            VolumeInfo {
                mount_point: disk.mount_point().to_path_buf(),
                name: disk.name().to_string_lossy().to_string(),
                fs_type: disk.file_system().to_string_lossy().to_string(),
                total_bytes: total,
                available_bytes: available,
                used_bytes: used,
            }
        } else if let Some(s) = statvfs_info {
            s
        } else {
            // Fallback default
            VolumeInfo {
                mount_point: PathBuf::from("/"),
                name: "Macintosh HD".to_string(),
                fs_type: "APFS".to_string(),
                total_bytes: 1_000_000_000_000,
                available_bytes: 50_000_000_000,
                used_bytes: 950_000_000_000,
            }
        }
    }
}

fn get_statvfs_info(path: &Path) -> Option<VolumeInfo> {
    let path_str = path.to_str()?;
    let c_path = CString::new(path_str).ok()?;

    let mut stat = MaybeUninit::<libc::statvfs>::uninit();
    if unsafe { libc::statvfs(c_path.as_ptr(), stat.as_mut_ptr()) } == 0 {
        let stat = unsafe { stat.assume_init() };
        let block_size = stat.f_frsize;
        let total = stat.f_blocks as u64 * block_size;
        let available = stat.f_bavail as u64 * block_size;
        let used = total.saturating_sub(available);

        Some(VolumeInfo {
            mount_point: path.to_path_buf(),
            name: "Disk Volume".to_string(),
            fs_type: "APFS".to_string(),
            total_bytes: total,
            available_bytes: available,
            used_bytes: used,
        })
    } else {
        None
    }
}
