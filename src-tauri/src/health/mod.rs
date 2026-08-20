use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct ProjectHealth {
    pub score: u8, // 0 to 100
    pub status: String,
    pub deps_installed: bool,
    pub has_readme: bool,
    pub is_git_clean: bool,
    pub issues: Vec<String>,
}

pub struct HealthChecker;

impl HealthChecker {
    /// Evaluates deterministic health checks for a project directory.
    pub fn check_project(path: &str, is_git_dirty: bool) -> ProjectHealth {
        let p = Path::new(path);
        let mut score = 100u8;
        let mut issues = Vec::new();

        // 1. Check README presence
        let has_readme = p.join("README.md").exists()
            || p.join("readme.md").exists()
            || p.join("README.txt").exists();
        if !has_readme {
            score = score.saturating_sub(15);
            issues.push("Missing README documentation".to_string());
        }

        // 2. Check Package Dependencies
        let mut deps_installed = true;
        if p.join("package.json").exists() && !p.join("node_modules").exists() {
            deps_installed = false;
            score = score.saturating_sub(35);
            issues.push("node_modules missing (run npm install)".to_string());
        }

        if (p.join("requirements.txt").exists() || p.join("pyproject.toml").exists())
            && !p.join(".venv").exists()
            && !p.join("venv").exists()
            && !p.join("__pycache__").exists()
        {
            score = score.saturating_sub(20);
            issues.push("Python virtual environment (.venv) missing".to_string());
        }

        // 3. Check Git status
        let is_git_clean = !is_git_dirty;
        if is_git_dirty {
            score = score.saturating_sub(15);
            issues.push("Uncommitted Git changes".to_string());
        }

        let status = if score >= 80 {
            "Healthy".to_string()
        } else if score >= 50 {
            "Needs Attention".to_string()
        } else {
            "Unhealthy".to_string()
        };

        ProjectHealth {
            score,
            status,
            deps_installed,
            has_readme,
            is_git_clean,
            issues,
        }
    }
}
