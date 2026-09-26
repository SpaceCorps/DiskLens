use crossbeam_channel::{Receiver, Sender};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::SystemTime;
use walkdir::WalkDir;

use crate::model::{EntryType, FileCategory, FileEntry};

#[derive(Debug, Clone)]
pub struct FolderScanResult {
    pub generation_id: usize,
    pub path: PathBuf,
    pub size_bytes: u64,
    pub file_count: u64,
}

pub struct DirectoryScanner {
    sender: Sender<FolderScanResult>,
    receiver: Receiver<FolderScanResult>,
    current_generation: Arc<AtomicUsize>,
    cancel_flag: Arc<AtomicBool>,
    cache: HashMap<PathBuf, (u64, u64, SystemTime)>,
}

impl DirectoryScanner {
    pub fn new() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        Self {
            sender,
            receiver,
            current_generation: Arc::new(AtomicUsize::new(0)),
            cancel_flag: Arc::new(AtomicBool::new(false)),
            cache: HashMap::new(),
        }
    }

    pub fn receiver(&self) -> &Receiver<FolderScanResult> {
        &self.receiver
    }

    #[allow(dead_code)]
    pub fn current_generation(&self) -> usize {
        self.current_generation.load(Ordering::SeqCst)
    }

    #[allow(dead_code)]
    pub fn get_cached(&self, path: &Path) -> Option<(u64, u64)> {
        self.cache.get(path).map(|(size, count, _)| (*size, *count))
    }

    pub fn cache_result(&mut self, path: PathBuf, size: u64, count: u64) {
        self.cache.insert(path, (size, count, SystemTime::now()));
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Read the immediate contents of a directory and start background sizing for subdirectories.
    pub fn load_directory(&mut self, dir_path: &Path) -> (Vec<FileEntry>, usize) {
        // Cancel previous scans
        self.cancel_flag.store(true, Ordering::SeqCst);

        // Advance generation
        let next_gen = self.current_generation.fetch_add(1, Ordering::SeqCst) + 1;
        let cancel_flag = Arc::new(AtomicBool::new(false));
        self.cancel_flag = cancel_flag.clone();

        let mut entries = Vec::new();
        let mut dirs_to_scan = Vec::new();

        if let Ok(read_dir) = std::fs::read_dir(dir_path) {
            for entry_res in read_dir {
                let entry = match entry_res {
                    Ok(e) => e,
                    Err(_) => continue,
                };

                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                let is_hidden = name.starts_with('.');

                let metadata = match entry.metadata() {
                    Ok(m) => m,
                    Err(_) => continue,
                };

                let is_dir = metadata.is_dir();
                let is_symlink = metadata.file_type().is_symlink();

                let entry_type = if is_symlink {
                    EntryType::Symlink
                } else if is_dir {
                    EntryType::Directory
                } else {
                    EntryType::File
                };

                let category = FileCategory::from_path(&path, is_dir);
                let modified = metadata.modified().ok();

                if is_dir {
                    // Check if cached
                    let (cached_size, cached_count, is_cached) =
                        if let Some(&(size, count, _)) = self.cache.get(&path) {
                            (size, count, true)
                        } else {
                            (0, 0, false)
                        };

                    entries.push(FileEntry {
                        path: path.clone(),
                        name,
                        entry_type,
                        category,
                        size_bytes: cached_size,
                        file_count: cached_count,
                        modified,
                        is_scanning: !is_cached,
                        is_hidden,
                    });

                    // Even if cached, we can queue a background verify or scan if not cached
                    if !is_cached {
                        dirs_to_scan.push(path);
                    }
                } else {
                    let size = metadata.len();
                    entries.push(FileEntry {
                        path,
                        name,
                        entry_type,
                        category,
                        size_bytes: size,
                        file_count: 1,
                        modified,
                        is_scanning: false,
                        is_hidden,
                    });
                }
            }
        }

        // Spawn parallel background scans using rayon
        if !dirs_to_scan.is_empty() {
            let sender = self.sender.clone();
            let gen_id = next_gen;
            let cancel = cancel_flag;

            std::thread::spawn(move || {
                use rayon::prelude::*;

                dirs_to_scan.into_par_iter().for_each(|dir| {
                    if cancel.load(Ordering::Relaxed) {
                        return;
                    }

                    let (size, count) = scan_folder_recursive(&dir, &cancel);

                    if !cancel.load(Ordering::Relaxed) {
                        let _ = sender.send(FolderScanResult {
                            generation_id: gen_id,
                            path: dir,
                            size_bytes: size,
                            file_count: count,
                        });
                    }
                });
            });
        }

        (entries, next_gen)
    }
}

/// Recursively calculate folder size and file count with cancellation support.
fn scan_folder_recursive(dir: &Path, cancel: &AtomicBool) -> (u64, u64) {
    let mut total_size = 0u64;
    let mut total_count = 0u64;

    // Fast WalkDir traversal
    for (step_idx, entry) in WalkDir::new(dir)
        .min_depth(1)
        .into_iter()
        .filter_entry(|e| {
            // We can skip recursion into broken symlinks or cyclical dirs
            !e.path_is_symlink()
        })
        .enumerate()
    {
        // Check cancellation every 256 items to minimize atomic overhead
        if step_idx % 256 == 0 && cancel.load(Ordering::Relaxed) {
            return (total_size, total_count);
        }

        match entry {
            Ok(e) => {
                total_count += 1;
                if let Ok(meta) = e.metadata()
                    && meta.is_file()
                {
                    total_size += meta.len();
                }
            }
            Err(_) => {
                // Ignore permission denied or inaccessible entries
            }
        }
    }

    (total_size, total_count)
}
