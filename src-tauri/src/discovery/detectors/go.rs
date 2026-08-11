use crate::discovery::detector::{tech, DetectContext, ProjectDetector};
use crate::models::{Tech, TechKind};
use std::path::Path;

/// Detects Go from `go.mod`.
pub struct GoDetector;

impl ProjectDetector for GoDetector {
    fn id(&self) -> &'static str {
        "go"
    }
    fn priority(&self) -> u8 {
        13
    }

    fn detect(&self, dir: &Path, _ctx: &DetectContext) -> Option<Vec<Tech>> {
        if dir.join("go.mod").is_file() {
            Some(vec![tech("Go", TechKind::Language)])
        } else {
            None
        }
    }
}
