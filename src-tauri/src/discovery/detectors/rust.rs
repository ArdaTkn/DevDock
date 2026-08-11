use crate::discovery::detector::{tech, DetectContext, ProjectDetector};
use crate::models::{Tech, TechKind};
use std::path::Path;

/// Detects Rust from `Cargo.toml`.
pub struct RustDetector;

impl ProjectDetector for RustDetector {
    fn id(&self) -> &'static str {
        "rust"
    }
    fn priority(&self) -> u8 {
        12
    }

    fn detect(&self, dir: &Path, _ctx: &DetectContext) -> Option<Vec<Tech>> {
        if dir.join("Cargo.toml").is_file() {
            Some(vec![
                tech("Rust", TechKind::Language),
                tech("Cargo", TechKind::Tool),
            ])
        } else {
            None
        }
    }
}
