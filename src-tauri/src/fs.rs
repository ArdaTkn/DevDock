use crate::error::{Error, Result};
use std::path::{Path, PathBuf};

/// Returns the canonical (symlink-resolved) absolute path for `p`.
/// Fails with a human-readable error if the path cannot be canonicalised.
pub fn canonicalize(p: &Path) -> Result<PathBuf> {
    p.canonicalize().map_err(|e| {
        Error::Other(format!(
            "Path `{}` is not accessible: {e}",
            p.display()
        ))
    })
}

/// True if `child` is `base` or lies underneath `base` (canonicalised).
/// Used to ensure we only open/scan things the user asked us to.
pub fn is_under(base: &Path, child: &Path) -> bool {
    child.starts_with(base)
}

/// Best-effort display name derived from a path's final component.
pub fn path_name(p: &Path) -> String {
    p.file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| p.display().to_string())
}

/// Directories we never descend into during a scan.
pub fn is_ignored_dir(name: &str) -> bool {
    // Hidden dirs are almost always config/cache/module junk (e.g. ~/.cursor,
    // ~/.npm, ~/.config, editor extensions) — not real user projects.
    if name.starts_with('.') {
        return true;
    }
    matches!(
        name,
        "node_modules"
            | "target"
            | "build"
            | "dist"
            | "dart_tool"
            | "gradle"
            | "__pycache__"
            | "venv"
            | "Pods"
            | "idea"
            | "vscode"
            | "next"
            | "coverage"
            // macOS system/mechanism folders that contain no user projects.
            | "Library"
            | "Applications"
            | "AppData"
            | "System"
    )
}
