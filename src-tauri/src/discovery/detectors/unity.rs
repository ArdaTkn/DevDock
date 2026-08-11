use crate::discovery::detector::{tech, DetectContext, ProjectDetector};
use crate::models::{Tech, TechKind};
use std::path::Path;

/// Recognises common Unity project structures (without touching big Assets).
pub struct UnityDetector;

impl ProjectDetector for UnityDetector {
    fn id(&self) -> &'static str {
        "unity"
    }
    fn priority(&self) -> u8 {
        21
    }

    fn detect(&self, dir: &Path, _ctx: &DetectContext) -> Option<Vec<Tech>> {
        let has_project_version = dir
            .join("ProjectSettings")
            .join("ProjectVersion.txt")
            .is_file();
        let has_assets = dir.join("Assets").is_dir();
        let has_packages = dir.join("Packages").is_dir();
        if has_project_version || (has_assets && has_packages) {
            Some(vec![tech("Unity", TechKind::Framework)])
        } else {
            None
        }
    }
}
