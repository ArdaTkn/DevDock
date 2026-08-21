use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheFolderInfo {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub human_size: String,
    pub category: String,
    pub is_safe: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectCacheReport {
    pub total_size_bytes: u64,
    pub total_human_size: String,
    pub reclaimable_bytes: u64,
    pub reclaimable_human_size: String,
    pub cache_folders: Vec<CacheFolderInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskHogItem {
    pub project_path: String,
    pub project_name: String,
    pub total_size_bytes: u64,
    pub reclaimable_bytes: u64,
    pub reclaimable_human_size: String,
    pub last_modified: u64,
    pub is_stale: bool, // > 90 days without modification
    pub cache_folders: Vec<CacheFolderInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskHogReport {
    pub total_reclaimable_bytes: u64,
    pub total_reclaimable_human_size: String,
    pub stale_projects_count: usize,
    pub items: Vec<DiskHogItem>,
}

const SAFE_CACHE_NAMES: &[(&str, &str)] = &[
    ("node_modules", "Node.js Dependencies"),
    ("target", "Rust Cargo Build Artifacts"),
    (".dart_tool", "Flutter/Dart Cache"),
    ("build", "Build Output / Artifacts"),
    (".venv", "Python Virtual Environment"),
    ("venv", "Python Virtual Environment"),
    ("__pycache__", "Python Bytecode Cache"),
    (".gradle", "Gradle Build Cache"),
    ("dist", "Frontend Distribution Build"),
    (".next", "Next.js Build Cache"),
    (".nuxt", "Nuxt.js Build Cache"),
    (".turbo", "TurboRepo Cache"),
    ("Pods", "CocoaPods Dependencies"),
    ("DerivedData", "Xcode Derived Data"),
    (".cache", "Generic App Cache"),
];

pub struct CacheJanitor;

impl CacheJanitor {
    /// Format bytes to human-readable string (KB, MB, GB).
    pub fn format_size(bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = 1024 * KB;
        const GB: u64 = 1024 * MB;

        if bytes >= GB {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.1} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.1} KB", bytes as f64 / KB as f64)
        } else {
            format!("{bytes} B")
        }
    }

    /// Recursively calculate directory size in bytes without following symlinks.
    pub fn calculate_dir_size(path: &Path) -> u64 {
        let mut total = 0;
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Ok(meta) = entry.metadata() {
                    if meta.is_dir() {
                        total += Self::calculate_dir_size(&entry.path());
                    } else {
                        total += meta.len();
                    }
                }
            }
        }
        total
    }

    /// Scans a project for heavy cache and build artifact folders.
    pub fn scan_project_cache(project_path: &Path) -> ProjectCacheReport {
        let mut cache_folders = Vec::new();
        let mut reclaimable_bytes = 0;

        for (name, category) in SAFE_CACHE_NAMES {
            let folder_path = project_path.join(name);
            if folder_path.is_dir() {
                let size = Self::calculate_dir_size(&folder_path);
                if size > 0 {
                    reclaimable_bytes += size;
                    cache_folders.push(CacheFolderInfo {
                        name: name.to_string(),
                        path: folder_path.to_string_lossy().to_string(),
                        size_bytes: size,
                        human_size: Self::format_size(size),
                        category: category.to_string(),
                        is_safe: true,
                    });
                }
            }
        }

        // Sort cache folders by size descending
        cache_folders.sort_by_key(|b| std::cmp::Reverse(b.size_bytes));

        let total_size_bytes = Self::calculate_dir_size(project_path);

        ProjectCacheReport {
            total_size_bytes,
            total_human_size: Self::format_size(total_size_bytes),
            reclaimable_bytes,
            reclaimable_human_size: Self::format_size(reclaimable_bytes),
            cache_folders,
        }
    }

    /// Safely deletes a cache folder within a project directory.
    pub fn clean_cache_folder(project_path: &Path, folder_name: &str) -> std::io::Result<u64> {
        let is_known_safe = SAFE_CACHE_NAMES
            .iter()
            .any(|(name, _)| *name == folder_name);
        if !is_known_safe {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                format!("Folder '{folder_name}' is not in the safe-to-clean list."),
            ));
        }

        let target = project_path.join(folder_name);
        if !target.starts_with(project_path) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "Target path escapes project directory.",
            ));
        }

        if target.is_dir() {
            let size = Self::calculate_dir_size(&target);
            fs::remove_dir_all(&target)?;
            Ok(size)
        } else {
            Ok(0)
        }
    }

    /// Scans multiple projects and returns a ranked report of disk space hogs and stale projects.
    pub fn scan_all_hogs(project_paths: &[PathBuf]) -> DiskHogReport {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let ninety_days_secs = 90 * 24 * 60 * 60;

        let mut items = Vec::new();
        let mut total_reclaimable_bytes = 0;
        let mut stale_projects_count = 0;

        for path in project_paths {
            if !path.is_dir() {
                continue;
            }
            let cache_report = Self::scan_project_cache(path);
            if cache_report.reclaimable_bytes == 0 {
                continue;
            }

            let project_name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.to_string_lossy().to_string());

            let modified = fs::metadata(path)
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);

            let is_stale = (now.saturating_sub(modified)) > ninety_days_secs;
            if is_stale {
                stale_projects_count += 1;
            }

            total_reclaimable_bytes += cache_report.reclaimable_bytes;

            items.push(DiskHogItem {
                project_path: path.to_string_lossy().to_string(),
                project_name,
                total_size_bytes: cache_report.total_size_bytes,
                reclaimable_bytes: cache_report.reclaimable_bytes,
                reclaimable_human_size: cache_report.reclaimable_human_size,
                last_modified: modified,
                is_stale,
                cache_folders: cache_report.cache_folders,
            });
        }

        // Sort items by reclaimable bytes descending
        items.sort_by_key(|b| std::cmp::Reverse(b.reclaimable_bytes));

        DiskHogReport {
            total_reclaimable_bytes,
            total_reclaimable_human_size: Self::format_size(total_reclaimable_bytes),
            stale_projects_count,
            items,
        }
    }
}
