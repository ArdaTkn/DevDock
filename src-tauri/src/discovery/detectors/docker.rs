use crate::discovery::detector::{tech, DetectContext, ProjectDetector};
use crate::models::{Tech, TechKind};
use std::path::Path;

/// Detects Docker from `Dockerfile` / compose files. Adds a Docker tech tag only;
/// projecthood still needs another detector or we let the scanner accept it.
pub struct DockerDetector;

impl ProjectDetector for DockerDetector {
    fn id(&self) -> &'static str {
        "docker"
    }
    fn priority(&self) -> u8 {
        20
    }

    fn detect(&self, dir: &Path, _ctx: &DetectContext) -> Option<Vec<Tech>> {
        let has_dockerfile =
            dir.join("Dockerfile").is_file() || dir.join("Dockerfile.dev").is_file();
        let has_compose = ["docker-compose.yml", "docker-compose.yaml", "compose.yml", "compose.yaml"]
            .iter()
            .any(|f| dir.join(f).is_file());
        if has_dockerfile || has_compose {
            Some(vec![tech("Docker", TechKind::Tool)])
        } else {
            None
        }
    }
}
