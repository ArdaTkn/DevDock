use serde::Serialize;

/// A detected technology / language tag on a project.
#[derive(Debug, Clone, Serialize)]
pub struct Tech {
    pub name: String,
    pub kind: TechKind,
}

/// Coarse category used for filter chips and pretty grouping.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TechKind {
    Language,
    Framework,
    Tool,
    Runtime,
}

/// Read-only Git metadata computed for a project.
#[derive(Debug, Clone, Serialize, Default)]
pub struct GitInfo {
    pub is_git: bool,
    pub branch: Option<String>,
    pub remote_url: Option<String>,
    pub repo_name: Option<String>,
    pub staged_count: u32,
    pub modified_count: u32,
    pub untracked_count: u32,
    pub last_commit_message: Option<String>,
    pub last_commit_date: Option<i64>,
    pub latest_short_hash: Option<String>,
}

impl GitInfo {
    pub fn clean(&self) -> bool {
        self.is_git
            && self.staged_count == 0
            && self.modified_count == 0
            && self.untracked_count == 0
    }
}

/// The full project record returned to the UI.
#[derive(Debug, Clone, Serialize)]
pub struct Project {
    pub id: i64,
    pub path: String,
    pub name: String,
    pub relative_path: Option<String>,
    pub size_bytes: i64,
    pub last_modified: i64,
    pub is_favorite: bool,
    pub techs: Vec<Tech>,
    pub git: Option<GitInfo>,
}

/// A user-configured root directory to scan.
#[derive(Debug, Clone, Serialize)]
pub struct ScanLocation {
    pub id: i64,
    pub path: String,
    pub name: String,
}

/// Progress event emitted during a scan so the UI can show progress + cancel.
#[derive(Debug, Clone, Serialize)]
pub struct ScanProgress {
    pub scanned: u64,
    pub total: u64,
    pub current_path: String,
    pub done: bool,
    pub found: u64,
    pub cancelled: bool,
}

/// Aggregate breakdown shown on the "magic moment" after a scan.
#[derive(Debug, Clone, Serialize, Default)]
pub struct ScanSummary {
    pub total: u64,
    pub tech_breakdown: Vec<(String, u64)>,
    pub dirty_count: u64,
    pub clean_count: u64,
}
