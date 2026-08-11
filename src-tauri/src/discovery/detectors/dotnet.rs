use crate::discovery::detector::{tech, DetectContext, ProjectDetector};
use crate::models::{Tech, TechKind};
use std::path::Path;

/// Detects .NET from `*.sln` / `*.csproj` / `*.fsproj` markers.
pub struct DotNetDetector;

impl ProjectDetector for DotNetDetector {
    fn id(&self) -> &'static str {
        "dotnet"
    }
    fn priority(&self) -> u8 {
        15
    }

    fn detect(&self, dir: &Path, _ctx: &DetectContext) -> Option<Vec<Tech>> {
        let has_sln = has_extension(dir, "sln");
        let has_csproj = has_extension(dir, "csproj");
        let has_fsproj = has_extension(dir, "fsproj");
        if has_sln || has_csproj || has_fsproj {
            Some(vec![tech(".NET", TechKind::Language)])
        } else {
            None
        }
    }
}

fn has_extension(dir: &Path, ext: &str) -> bool {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries.flatten().any(|e| {
                e.path().extension().map(|x| x == ext).unwrap_or(false)
            })
        })
        .unwrap_or(false)
}
