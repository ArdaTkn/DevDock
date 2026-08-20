use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct ProjectScript {
    pub name: String,
    pub command: String,
    pub source: String,
}

pub struct ScriptLauncher;

impl ScriptLauncher {
    /// Detects available scripts for a given project directory.
    pub fn list_scripts(path: &str) -> Vec<ProjectScript> {
        let mut scripts = Vec::new();
        let p = Path::new(path);

        // 1. package.json scripts
        let pkg_json = p.join("package.json");
        if pkg_json.exists() {
            if let Ok(content) = fs::read_to_string(&pkg_json) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(scr_obj) = val.get("scripts").and_then(|s| s.as_object()) {
                        for (k, v) in scr_obj {
                            if let Some(cmd_str) = v.as_str() {
                                scripts.push(ProjectScript {
                                    name: k.clone(),
                                    command: cmd_str.to_string(),
                                    source: "package.json".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }

        // 2. Cargo.toml commands
        if p.join("Cargo.toml").exists() {
            scripts.push(ProjectScript {
                name: "run".to_string(),
                command: "cargo run".to_string(),
                source: "Cargo.toml".to_string(),
            });
            scripts.push(ProjectScript {
                name: "test".to_string(),
                command: "cargo test".to_string(),
                source: "Cargo.toml".to_string(),
            });
            scripts.push(ProjectScript {
                name: "check".to_string(),
                command: "cargo check".to_string(),
                source: "Cargo.toml".to_string(),
            });
        }

        // 3. Makefile targets
        let makefile = p.join("Makefile");
        if makefile.exists() {
            if let Ok(content) = fs::read_to_string(&makefile) {
                for line in content.lines() {
                    if !line.starts_with('\t') && !line.starts_with('#') && line.contains(':') {
                        if let Some(target) = line.split(':').next() {
                            let trimmed = target.trim();
                            if !trimmed.is_empty()
                                && !trimmed.contains('=')
                                && !trimmed.starts_with('.')
                            {
                                scripts.push(ProjectScript {
                                    name: trimmed.to_string(),
                                    command: format!("make {trimmed}"),
                                    source: "Makefile".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }

        scripts
    }
}
