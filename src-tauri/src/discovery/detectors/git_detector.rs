use crate::discovery::detector::{tech, DetectContext, ProjectDetector};
use crate::models::{Tech, TechKind};
use std::path::Path;

/// A `.git` folder marks a directory as a project even if nothing else matches.
pub struct GitDetector;

impl ProjectDetector for GitDetector {
    fn id(&self) -> &'static str {
        "git"
    }
    // Highest priority: a git repo alone implies a project.
    fn priority(&self) -> u8 {
        0
    }

    fn detect(&self, dir: &Path, ctx: &DetectContext) -> Option<Vec<Tech>> {
        if ctx.is_git_repo || dir.join(".git").exists() {
            Some(vec![tech("Git", TechKind::Tool)])
        } else {
            None
        }
    }
}
