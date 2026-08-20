use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct DependencyInfo {
    pub name: String,
    pub version: String,
    pub is_dev: bool,
}

pub struct DependencyParser;

impl DependencyParser {
    /// Parses dependencies from package.json, Cargo.toml, or requirements.txt.
    pub fn parse_dependencies(path: &str) -> Vec<DependencyInfo> {
        let mut deps = Vec::new();
        let p = Path::new(path);

        // 1. Node package.json
        let pkg_json = p.join("package.json");
        if pkg_json.exists() {
            if let Ok(content) = fs::read_to_string(&pkg_json) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(dep_obj) = val.get("dependencies").and_then(|d| d.as_object()) {
                        for (k, v) in dep_obj {
                            let ver = v.as_str().unwrap_or("*").to_string();
                            deps.push(DependencyInfo {
                                name: k.clone(),
                                version: ver,
                                is_dev: false,
                            });
                        }
                    }
                    if let Some(dev_obj) = val.get("devDependencies").and_then(|d| d.as_object()) {
                        for (k, v) in dev_obj {
                            let ver = v.as_str().unwrap_or("*").to_string();
                            deps.push(DependencyInfo {
                                name: k.clone(),
                                version: ver,
                                is_dev: true,
                            });
                        }
                    }
                }
            }
        }

        // 2. Rust Cargo.toml
        let cargo_toml = p.join("Cargo.toml");
        if cargo_toml.exists() {
            if let Ok(content) = fs::read_to_string(&cargo_toml) {
                let mut in_deps = false;
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with('[') {
                        in_deps = trimmed.contains("dependencies");
                        continue;
                    }
                    if in_deps && trimmed.contains('=') {
                        let parts: Vec<&str> = trimmed.split('=').collect();
                        if parts.len() >= 2 {
                            let name = parts[0].trim().to_string();
                            let ver = parts[1].trim().trim_matches('"').to_string();
                            if !name.is_empty() {
                                deps.push(DependencyInfo {
                                    name,
                                    version: ver,
                                    is_dev: false,
                                });
                            }
                        }
                    }
                }
            }
        }

        // 3. Python requirements.txt
        let req_txt = p.join("requirements.txt");
        if req_txt.exists() {
            if let Ok(content) = fs::read_to_string(&req_txt) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() && !trimmed.starts_with('#') {
                        let parts: Vec<&str> = trimmed.split("==").collect();
                        let name = parts[0].trim().to_string();
                        let version = if parts.len() > 1 {
                            parts[1].trim().to_string()
                        } else {
                            "latest".to_string()
                        };
                        deps.push(DependencyInfo {
                            name,
                            version,
                            is_dev: false,
                        });
                    }
                }
            }
        }

        deps
    }
}
