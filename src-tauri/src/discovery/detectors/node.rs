use crate::discovery::detector::{tech, DetectContext, ProjectDetector};
use crate::models::{Tech, TechKind};
use std::path::Path;

/// Detects Node.js projects from `package.json`, reading scripts + deps keys.
pub struct NodeDetector;

impl ProjectDetector for NodeDetector {
    fn id(&self) -> &'static str {
        "node"
    }
    fn priority(&self) -> u8 {
        10
    }

    fn detect(&self, dir: &Path, _ctx: &DetectContext) -> Option<Vec<Tech>> {
        let pkg = dir.join("package.json");
        if !pkg.is_file() {
            return None;
        }
        let mut techs = vec![
            tech("Node.js", TechKind::Runtime),
            tech("Node", TechKind::Language),
        ];

        // Read scripts/deps minimally (best-effort, never fatal).
        if let Ok(raw) = std::fs::read_to_string(&pkg) {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&raw) {
                if let Some(s) = parsed.get("scripts") {
                    let named = s.as_object().is_some_and(|o| !o.is_empty());
                    if named {
                        techs.push(tech("npm-scripts", TechKind::Tool));
                    }
                }
                // package manager hints.
                if parsed.get("packageManager").is_some() {
                    techs.push(tech("pnpm", TechKind::Tool));
                }
            }
        }

        // Detect which package manager files are present.
        if dir.join("pnpm-lock.yaml").exists() {
            techs.push(tech("pnpm", TechKind::Tool));
        } else if dir.join("yarn.lock").exists() {
            techs.push(tech("yarn", TechKind::Tool));
        } else if dir.join("package-lock.json").exists() {
            techs.push(tech("npm", TechKind::Tool));
        }

        // Supabase (common backend add-on — cheap filename check).
        if dir.join("supabase").exists() {
            techs.push(tech("Supabase", TechKind::Tool));
        }

        // Vite / React / else common frontend hints from deps.
        if let Ok(raw) = std::fs::read_to_string(&pkg) {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&raw) {
                let all = json_deps(&parsed);
                if all.iter().any(|d| d == "vite") {
                    techs.push(tech("Vite", TechKind::Tool));
                }
                if all.iter().any(|d| d == "react" || d == "react-dom") {
                    techs.push(tech("React", TechKind::Framework));
                }
                if all.iter().any(|d| d == "next") {
                    techs.push(tech("Next.js", TechKind::Framework));
                }
                for d in all {
                    if d.contains("tauri") {
                        techs.push(tech("Tauri", TechKind::Framework));
                        break;
                    }
                }
            }
        }

        Some(dedup(techs))
    }
}

fn json_deps(v: &serde_json::Value) -> Vec<String> {
    let mut out = Vec::new();
    for key in ["dependencies", "devDependencies", "peerDependencies"] {
        if let Some(obj) = v.get(key).and_then(|d| d.as_object()) {
            out.extend(obj.keys().cloned());
        }
    }
    out
}

fn dedup(v: Vec<Tech>) -> Vec<Tech> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for t in v {
        if seen.insert(t.name.clone()) {
            out.push(t);
        }
    }
    out
}
