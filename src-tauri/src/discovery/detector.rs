use crate::models::{Tech, TechKind};
use std::path::Path;

/// Context passed to every detector during a scan.
pub struct DetectContext {
    /// True when the caller has confirmed `dir` is a git repo root.
    pub is_git_repo: bool,
}

/// A detector recognises one class of software project by inspecting ONLY the
/// marker files it needs (an entry-point regular file / directory). Each
/// detector is independently unit-testable against `tests/fixtures/`.
pub trait ProjectDetector: Send + Sync {
    /// Stable identifier, e.g. "node", "python", "git".
    fn id(&self) -> &'static str;

    /// Lower runs first; detectors that also *define* a project (e.g. git)
    /// get a high priority so they contribute even when alone.
    fn priority(&self) -> u8;

    /// Returns the technologies this project uses, if `dir` matches this detector.
    /// Returns `Some(vec![])` is NOT used — detectors return `None` when they do
    /// not match, and the caller decides projecthood from the union of matches.
    fn detect(&self, dir: &Path, ctx: &DetectContext) -> Option<Vec<Tech>>;
}

/// Registry holds all detectors in a stable, ordered list.
#[derive(Default)]
pub struct DetectorRegistry {
    detectors: Vec<Box<dyn ProjectDetector>>,
}

impl DetectorRegistry {
    pub fn default_registry() -> Self {
        let mut reg = Self::default();
        reg.register(Box::new(crate::discovery::detectors::GitDetector));
        reg.register(Box::new(crate::discovery::detectors::NodeDetector));
        reg.register(Box::new(crate::discovery::detectors::PythonDetector));
        reg.register(Box::new(crate::discovery::detectors::RustDetector));
        reg.register(Box::new(crate::discovery::detectors::GoDetector));
        reg.register(Box::new(crate::discovery::detectors::FlutterDetector));
        reg.register(Box::new(crate::discovery::detectors::DotNetDetector));
        reg.register(Box::new(crate::discovery::detectors::DockerDetector));
        reg.register(Box::new(crate::discovery::detectors::UnityDetector));
        reg.register(Box::new(crate::discovery::detectors::JavaDetector));
        reg
    }

    pub fn register(&mut self, d: Box<dyn ProjectDetector>) {
        self.detectors.push(d);
    }

    /// Runs every detector against `dir` and unions all detected techs.
    /// Returns None if no detector matched (not a project).
    pub fn detect_all(&self, dir: &Path, ctx: &DetectContext) -> Option<Vec<Tech>> {
        let mut techs = Vec::new();
        for d in &self.detectors {
            if let Some(mut t) = d.detect(dir, ctx) {
                techs.append(&mut t);
            }
        }
        if techs.is_empty() {
            None
        } else {
            Some(techs)
        }
    }

    /// True if the given entry-point file path is one we care about, used to
    /// decide whether a directory is worth descending into or is a project root.
    pub fn is_project_marker(dir: &Path) -> bool {
        PROJECT_MARKERS.iter().any(|m| dir.join(m).exists())
    }
}

/// Well-known files that mark a directory as a project root.
pub const PROJECT_MARKERS: &[&str] = &[
    ".git",
    "package.json",
    "pyproject.toml",
    "requirements.txt",
    "Pipfile",
    "setup.py",
    "Cargo.toml",
    "go.mod",
    "pubspec.yaml",
    "pom.xml",
    "build.gradle",
    "build.gradle.kts",
    "Dockerfile",
    "docker-compose.yml",
    "compose.yml",
    "Gemfile",
    "composer.json",
];

// Small helper reused by several detectors.
pub fn has_any(dir: &Path, names: &[&str]) -> bool {
    names.iter().any(|n| dir.join(n).exists())
}

pub fn tech(name: &str, kind: TechKind) -> Tech {
    Tech {
        name: name.to_string(),
        kind,
    }
}
