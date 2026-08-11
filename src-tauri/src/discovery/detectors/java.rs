use crate::discovery::detector::{tech, DetectContext, ProjectDetector};
use crate::models::{Tech, TechKind};
use std::path::Path;

/// Detects Java from Maven/Gradle markers.
pub struct JavaDetector;

impl ProjectDetector for JavaDetector {
    fn id(&self) -> &'static str {
        "java"
    }
    fn priority(&self) -> u8 {
        22
    }

    fn detect(&self, dir: &Path, _ctx: &DetectContext) -> Option<Vec<Tech>> {
        let markers = [
            "pom.xml",
            "build.gradle",
            "build.gradle.kts",
            "settings.gradle",
        ];
        if markers.iter().any(|m| dir.join(m).is_file()) {
            let mut techs = vec![tech("Java", TechKind::Language)];
            if dir.join("pom.xml").exists() {
                techs.push(tech("Maven", TechKind::Tool));
            } else {
                techs.push(tech("Gradle", TechKind::Tool));
            }
            Some(techs)
        } else {
            None
        }
    }
}
