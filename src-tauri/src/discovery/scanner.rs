use crate::discovery::detector::{DetectContext, DetectorRegistry};
use crate::error::Result;
use crate::git::GitCommand;
use crate::models::{ScanProgress, ScanSummary, Tech};
use crate::storage::project_repo::ProjectRepo;
use crate::storage::AppDb;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

/// Shared cancellation token for a running scan.
#[derive(Clone, Default)]
pub struct ScanHandle {
    cancelled: Arc<AtomicBool>,
    scanned: Arc<AtomicU32>,
}

impl ScanHandle {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }
    pub fn tick(&self) {
        self.scanned.fetch_add(1, Ordering::Relaxed);
    }
    pub fn scanned(&self) -> u32 {
        self.scanned.load(Ordering::Relaxed)
    }
}

pub struct Scanner {
    db: AppDb,
    registry: DetectorRegistry,
    // Whether to compute git metadata during scans (cheap-ish but adds latency).
    include_git: bool,
}

impl Scanner {
    pub fn new(db: AppDb, include_git: bool) -> Self {
        Self {
            db,
            registry: DetectorRegistry::default_registry(),
            include_git,
        }
    }

    /// Scans every configured location, upserting into SQLite, reporting
    /// per-project progress through `progress`.
    pub fn scan_all(
        &self,
        handle: &ScanHandle,
        progress: &dyn Fn(ScanProgress),
    ) -> Result<ScanSummary> {
        let locations = ProjectRepo::list_scan_locations(&self.db)?;
        let mut summary = ScanSummary::default();
        for loc in locations {
            let root = PathBuf::from(&loc.path);
            if !root.is_dir() {
                continue; // deleted/inaccessible location — skip, don't fail.
            }
            self.scan_location(&root, &loc.id, handle, &mut summary, progress)?;
            if handle.is_cancelled() {
                break;
            }
        }
        Ok(summary)
    }

    fn scan_location(
        &self,
        root: &Path,
        loc_id: &i64,
        handle: &ScanHandle,
        summary: &mut ScanSummary,
        progress: &dyn Fn(ScanProgress),
    ) -> Result<()> {
        // First pass: find project roots (up to a bounded depth).
        let mut roots: Vec<PathBuf> = Vec::new();
        // Emit an immediate begin event so the UI shows "Scanning…" even during
        // the (possibly slow) walk phase, before the first project is ingested.
        progress(ScanProgress {
            scanned: 0,
            total: 0,
            current_path: root.display().to_string(),
            done: false,
            found: summary.total,
            cancelled: false,
        });
        self.walk(root, 0, &mut roots, handle);
        let total = roots.len();
        for (i, proj_path) in roots.iter().enumerate() {
            if handle.is_cancelled() {
                break;
            }
            handle.tick();
            let scanned = handle.scanned() as u64;
            self.ingest_project(proj_path, *loc_id, summary);
            progress(ScanProgress {
                scanned,
                total: total as u64,
                current_path: proj_path.display().to_string(),
                done: i + 1 == total,
                found: summary.total,
                cancelled: handle.is_cancelled(),
            });
        }
        Ok(())
    }

    /// Bounded recursive walk; collects directories that contain a project marker.
    fn walk(&self, dir: &Path, depth: u32, out: &mut Vec<PathBuf>, handle: &ScanHandle) {
        if depth > 5 || handle.is_cancelled() {
            return;
        }
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };
        // If this dir already looks like a project root, record and prune.
        if crate::discovery::detector::DetectorRegistry::is_project_marker(dir) {
            out.push(dir.to_path_buf());
            return;
        }
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let Some(name) = path.file_name().map(|n| n.to_string_lossy().to_string()) else {
                continue;
            };
            if crate::fs::is_ignored_dir(&name) {
                continue;
            }
            // Avoid descending into symlinked dirs that escape the root.
            if entry.file_type().map(|t| t.is_symlink()).unwrap_or(false) {
                continue;
            }
            self.walk(&path, depth + 1, out, handle);
        }
    }

    fn ingest_project(&self, path: &Path, loc_id: i64, summary: &mut ScanSummary) {
        let result = (|| -> Result<bool> {
            let meta = match std::fs::metadata(path) {
                Ok(m) => m,
                Err(_) => return Ok(false), // vanished mid-scan
            };
            let is_git = path.join(".git").exists();
            let ctx = DetectContext {
                is_git_repo: is_git,
            };
            let techs: Vec<Tech> = match self.registry.detect_all(path, &ctx) {
                Some(t) => t,
                None => return Ok(false), // not a project after all
            };

            let name = crate::fs::path_name(path);
            let relative = path.file_name().map(|_| path.to_string_lossy().to_string());
            let size = dir_size(path).unwrap_or(0);
            let mtime = meta
                .modified()
                .ok()
                .map(|t| {
                    t.duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs() as i64)
                        .unwrap_or(0)
                })
                .unwrap_or(0);

            let id = ProjectRepo::upsert_project(
                &self.db,
                &path.to_string_lossy(),
                &name,
                relative.as_deref(),
                Some(loc_id),
                size,
                mtime,
            )?;
            ProjectRepo::set_techs(&self.db, id, &techs)?;

            if self.include_git && is_git {
                match GitCommand::inspect(path) {
                    Ok(Some(info)) => {
                        ProjectRepo::set_git(&self.db, id, &info)?;
                        if info.clean() {
                            summary.clean_count += 1;
                        } else {
                            summary.dirty_count += 1;
                        }
                    }
                    Ok(None) => {}
                    Err(_) => {
                        // Leave git as absent; don't fail the project.
                        ProjectRepo::set_git(
                            &self.db,
                            id,
                            &crate::models::GitInfo {
                                is_git: false,
                                ..Default::default()
                            },
                        )?;
                    }
                }
            }

            for t in &techs {
                increment(&mut summary.tech_breakdown, &t.name);
            }
            summary.total += 1;
            Ok(true)
        })();

        // Swallow per-project errors so one bad project can't stop the scan.
        let _ = result;
    }
}

/// Cumulative directory size (bounded, best-effort).
fn dir_size(path: &Path) -> Option<i64> {
    let mut total = 0u64;
    let mut stack = vec![path.to_path_buf()];
    let mut visited = 0u32;
    while let Some(dir) = stack.pop() {
        // Avoid descending into ignored/heavy dirs.
        let name = dir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        if crate::fs::is_ignored_dir(&name) {
            continue;
        }
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for e in entries.flatten() {
            visited += 1;
            if visited > 200_000 {
                return Some(total as i64); // give up on huge dirs
            }
            if let Ok(m) = e.metadata() {
                if m.is_dir() {
                    stack.push(e.path());
                } else {
                    total += m.len();
                }
            }
        }
    }
    Some(total as i64)
}

fn increment(breakdown: &mut Vec<(String, u64)>, name: &str) {
    if let Some(slot) = breakdown.iter_mut().find(|(n, _)| n == name) {
        slot.1 += 1;
    } else {
        breakdown.push((name.to_string(), 1));
    }
}
