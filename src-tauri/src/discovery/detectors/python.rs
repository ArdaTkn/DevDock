use crate::discovery::detector::{tech, DetectContext, ProjectDetector};
use crate::models::{Tech, TechKind};
use std::path::Path;

/// Detects Python projects from the standard marker files.
pub struct PythonDetector;

impl ProjectDetector for PythonDetector {
    fn id(&self) -> &'static str {
        "python"
    }
    fn priority(&self) -> u8 {
        11
    }

    fn detect(&self, dir: &Path, _ctx: &DetectContext) -> Option<Vec<Tech>> {
        let markers = [
            "pyproject.toml",
            "requirements.txt",
            "Pipfile",
            "setup.py",
            "setup.cfg",
            "poetry.lock",
        ];
        if !markers.iter().any(|m| dir.join(m).is_file()) {
            return None;
        }
        let mut techs = vec![
            tech("Python", TechKind::Language),
            tech("Python", TechKind::Runtime),
        ];
        if dir.join("pyproject.toml").exists() {
            techs.push(tech("Poetry", TechKind::Tool));
        }
        Some(techs)
    }
}
