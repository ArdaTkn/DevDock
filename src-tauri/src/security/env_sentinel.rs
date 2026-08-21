use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvDiffReport {
    pub has_template: bool,
    pub template_file: Option<String>,
    pub has_local_env: bool,
    pub local_env_file: Option<String>,
    pub template_keys: Vec<String>,
    pub local_keys: Vec<String>,
    pub missing_keys: Vec<String>,
    pub extra_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitIgnoreAuditReport {
    pub has_gitignore: bool,
    pub sensitive_files_found: Vec<String>,
    pub unignored_sensitive_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeVersionInfo {
    pub toolchain: String,                // e.g. "Node.js", "Python", "Rust"
    pub required_version: String,         // e.g. "20.10.0" from .nvmrc
    pub detected_version: Option<String>, // e.g. "v20.18.0" from node -v
    pub source_file: String,              // e.g. ".nvmrc", "pyproject.toml"
    pub is_matched: bool,
}

pub struct EnvSentinel;

impl EnvSentinel {
    /// Extracts key names only from an env file without storing any values.
    fn extract_keys_from_file(path: &Path) -> Vec<String> {
        if !path.is_file() {
            return Vec::new();
        }
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };

        let mut keys = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if let Some(key) = trimmed.split('=').next() {
                let clean_key = key.trim();
                if !clean_key.is_empty() && !keys.contains(&clean_key.to_string()) {
                    keys.push(clean_key.to_string());
                }
            }
        }
        keys
    }

    /// Compares `.env.example` vs `.env` (keys only).
    pub fn check_env_diff(project_path: &Path) -> EnvDiffReport {
        let template_names = [
            ".env.example",
            ".env.template",
            ".env.sample",
            ".env.dist",
            "example.env",
        ];
        let local_names = [".env", ".env.local", ".env.development"];

        let mut template_file = None;
        let mut template_keys = Vec::new();

        for name in template_names {
            let p = project_path.join(name);
            if p.is_file() {
                template_keys = Self::extract_keys_from_file(&p);
                template_file = Some(name.to_string());
                break;
            }
        }

        let mut local_env_file = None;
        let mut local_keys = Vec::new();

        for name in local_names {
            let p = project_path.join(name);
            if p.is_file() {
                local_keys = Self::extract_keys_from_file(&p);
                local_env_file = Some(name.to_string());
                break;
            }
        }

        let has_template = template_file.is_some();
        let has_local_env = local_env_file.is_some();

        let mut missing_keys = Vec::new();
        let mut extra_keys = Vec::new();

        if has_template && has_local_env {
            for tk in &template_keys {
                if !local_keys.contains(tk) {
                    missing_keys.push(tk.clone());
                }
            }
            for lk in &local_keys {
                if !template_keys.contains(lk) {
                    extra_keys.push(lk.clone());
                }
            }
        } else if has_template && !has_local_env {
            missing_keys = template_keys.clone();
        }

        EnvDiffReport {
            has_template,
            template_file,
            has_local_env,
            local_env_file,
            template_keys,
            local_keys,
            missing_keys,
            extra_keys,
        }
    }

    /// Checks if sensitive files exist and whether they are excluded by .gitignore.
    pub fn audit_gitignore(project_path: &Path) -> GitIgnoreAuditReport {
        let sensitive_names = [
            ".env",
            ".env.local",
            ".env.development",
            ".env.production",
            "id_rsa",
            "id_ed25519",
            "credentials.json",
            "service-account.json",
            "secret.key",
            "private.key",
        ];

        let gitignore_path = project_path.join(".gitignore");
        let has_gitignore = gitignore_path.is_file();

        let gitignore_content = if has_gitignore {
            fs::read_to_string(&gitignore_path).unwrap_or_default()
        } else {
            String::new()
        };

        let gitignore_lines: Vec<&str> = gitignore_content
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .collect();

        let mut sensitive_files_found = Vec::new();
        let mut unignored_sensitive_files = Vec::new();

        for name in sensitive_names {
            let p = project_path.join(name);
            if p.exists() {
                sensitive_files_found.push(name.to_string());

                let is_ignored = gitignore_lines.iter().any(|pattern| {
                    if pattern == &name || *pattern == format!("/{name}") {
                        return true;
                    }
                    if pattern.starts_with('*') && name.ends_with(pattern.trim_start_matches('*')) {
                        return true;
                    }
                    if pattern.contains(".env*") && name.starts_with(".env") {
                        return true;
                    }
                    if pattern.contains("*.key") && name.ends_with(".key") {
                        return true;
                    }
                    if pattern.contains("*.json") && name.ends_with(".json") {
                        return true;
                    }
                    false
                });

                if !is_ignored {
                    unignored_sensitive_files.push(name.to_string());
                }
            }
        }

        GitIgnoreAuditReport {
            has_gitignore,
            sensitive_files_found,
            unignored_sensitive_files,
        }
    }

    /// Adds an entry to .gitignore if not already present.
    pub fn add_to_gitignore(project_path: &Path, entry: &str) -> std::io::Result<()> {
        let gitignore_path = project_path.join(".gitignore");
        let mut content = if gitignore_path.is_file() {
            fs::read_to_string(&gitignore_path).unwrap_or_default()
        } else {
            String::new()
        };

        if !content.lines().any(|l| l.trim() == entry.trim()) {
            if !content.is_empty() && !content.ends_with('\n') {
                content.push('\n');
            }
            content.push_str(entry.trim());
            content.push('\n');
            fs::write(gitignore_path, content)?;
        }
        Ok(())
    }

    /// Checks runtime versions specified in configuration files vs installed versions.
    pub fn check_runtime_versions(project_path: &Path) -> Vec<RuntimeVersionInfo> {
        let mut results = Vec::new();

        // 1. Node.js (.nvmrc or .node-version)
        let nvmrc = project_path.join(".nvmrc");
        let node_version_file = project_path.join(".node-version");

        let (node_req, node_src) = if nvmrc.is_file() {
            (fs::read_to_string(&nvmrc).ok(), ".nvmrc")
        } else if node_version_file.is_file() {
            (fs::read_to_string(&node_version_file).ok(), ".node-version")
        } else {
            (None, "")
        };

        if let Some(req) = node_req {
            let req_clean = req.trim().to_string();
            if !req_clean.is_empty() {
                let detected = Command::new("node")
                    .arg("-v")
                    .output()
                    .ok()
                    .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

                let is_matched = if let Some(ref det) = detected {
                    det.contains(&req_clean) || det.trim_start_matches('v').starts_with(&req_clean)
                } else {
                    false
                };

                results.push(RuntimeVersionInfo {
                    toolchain: "Node.js".to_string(),
                    required_version: req_clean,
                    detected_version: detected,
                    source_file: node_src.to_string(),
                    is_matched,
                });
            }
        }

        // 2. Python (.python-version)
        let py_version_file = project_path.join(".python-version");
        if py_version_file.is_file() {
            if let Ok(req) = fs::read_to_string(&py_version_file) {
                let req_clean = req.trim().to_string();
                if !req_clean.is_empty() {
                    let detected = Command::new("python3")
                        .arg("--version")
                        .output()
                        .ok()
                        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

                    let is_matched = if let Some(ref det) = detected {
                        det.contains(&req_clean)
                    } else {
                        false
                    };

                    results.push(RuntimeVersionInfo {
                        toolchain: "Python".to_string(),
                        required_version: req_clean,
                        detected_version: detected,
                        source_file: ".python-version".to_string(),
                        is_matched,
                    });
                }
            }
        }

        // 3. Rust (rust-toolchain.toml or rust-toolchain)
        let rust_toolchain = project_path.join("rust-toolchain.toml");
        let rust_toolchain_legacy = project_path.join("rust-toolchain");

        let (rust_req, rust_src) = if rust_toolchain.is_file() {
            (
                fs::read_to_string(&rust_toolchain).ok(),
                "rust-toolchain.toml",
            )
        } else if rust_toolchain_legacy.is_file() {
            (
                fs::read_to_string(&rust_toolchain_legacy).ok(),
                "rust-toolchain",
            )
        } else {
            (None, "")
        };

        if let Some(req) = rust_req {
            let req_clean = req
                .lines()
                .find(|l| l.contains("channel ="))
                .and_then(|l| l.split('=').nth(1))
                .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
                .unwrap_or_else(|| req.trim().to_string());

            if !req_clean.is_empty() {
                let detected = Command::new("rustc")
                    .arg("--version")
                    .output()
                    .ok()
                    .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

                let is_matched = if let Some(ref det) = detected {
                    det.contains(&req_clean)
                } else {
                    false
                };

                results.push(RuntimeVersionInfo {
                    toolchain: "Rust".to_string(),
                    required_version: req_clean,
                    detected_version: detected,
                    source_file: rust_src.to_string(),
                    is_matched,
                });
            }
        }

        results
    }
}
