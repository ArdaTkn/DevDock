use crate::discovery::detector::{tech, DetectContext, ProjectDetector};
use crate::models::{Tech, TechKind};
use std::path::Path;

/// Detects Flutter/Dart from `pubspec.yaml`.
pub struct FlutterDetector;

impl ProjectDetector for FlutterDetector {
    fn id(&self) -> &'static str {
        "flutter"
    }
    fn priority(&self) -> u8 {
        14
    }

    fn detect(&self, dir: &Path, _ctx: &DetectContext) -> Option<Vec<Tech>> {
        if dir.join("pubspec.yaml").is_file() {
            Some(vec![
                tech("Flutter", TechKind::Framework),
                tech("Dart", TechKind::Language),
            ])
        } else {
            None
        }
    }
}
