use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalAiSummaryDto {
    pub project_name: String,
    pub architecture_pattern: String,
    pub suggested_run_command: String,
    pub key_highlights: Vec<String>,
    pub maintenance_tips: Vec<String>,
    pub is_ai_generated_offline: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalAiQueryResponseDto {
    pub answer: String,
    pub model_used: String,
    pub is_offline: bool,
}

pub struct LocalAiEngine;

impl LocalAiEngine {
    /// Generates an intelligent architecture breakdown and summary 100% offline.
    pub fn analyze_project(
        project: &crate::models::Project,
        health: &crate::health::ProjectHealth,
        cache_report: &crate::system::ProjectCacheReport,
    ) -> LocalAiSummaryDto {
        let tech_names: Vec<String> = project.techs.iter().map(|t| t.name.clone()).collect();
        let has_rust = tech_names.iter().any(|t| t == "Rust");
        let has_node = tech_names
            .iter()
            .any(|t| t == "Node.js" || t == "TypeScript" || t == "JavaScript");
        let has_flutter = tech_names.iter().any(|t| t == "Flutter" || t == "Dart");
        let has_python = tech_names.iter().any(|t| t == "Python");
        let has_docker = tech_names.iter().any(|t| t == "Docker");
        let has_tauri = Path::new(&project.path).join("src-tauri").exists();

        // 1. Infer Architecture Pattern
        let architecture_pattern = if has_tauri && has_node {
            "Desktop Native App (Tauri 2 Rust Core + React/Vite Frontend)".to_string()
        } else if has_rust && has_docker {
            "High-Performance Backend Service (Rust async + Dockerized)".to_string()
        } else if has_flutter {
            "Cross-Platform Mobile/Desktop Application (Flutter & Dart)".to_string()
        } else if has_python && has_docker {
            "Python Data / ML / Microservice Pipeline (Docker containerized)".to_string()
        } else if has_node {
            "Modern JavaScript / TypeScript Web Application".to_string()
        } else if has_rust {
            "Native Systems / CLI Utility (Rust)".to_string()
        } else {
            "Modular Software Repository".to_string()
        };

        // 2. Determine Recommended Run Command
        let suggested_run_command = if has_tauri {
            "npm run tauri dev".to_string()
        } else if has_flutter {
            "flutter run".to_string()
        } else if has_rust {
            "cargo run".to_string()
        } else if has_python {
            "python main.py".to_string()
        } else if has_node {
            "npm run dev".to_string()
        } else {
            "devdock open".to_string()
        };

        // 3. Key Highlights
        let mut key_highlights = Vec::new();
        if let Some(ref git) = project.git {
            let branch = git.branch.as_deref().unwrap_or("main");
            if !git.clean() {
                key_highlights.push(format!(
                    "Active working branch: {} (has uncommitted changes)",
                    branch
                ));
            } else {
                key_highlights.push(format!(
                    "Git working directory is clean on branch '{}'",
                    branch
                ));
            }
        }
        if health.score >= 90 {
            key_highlights.push("🏆 Excellent project health score (>=90%)".to_string());
        } else if health.score < 60 {
            key_highlights.push(
                "⚠️ Needs attention: missing dependencies or build outputs detected".to_string(),
            );
        }
        if has_docker {
            key_highlights.push("🐳 Docker orchestration files detected".to_string());
        }

        // 4. Maintenance & Optimization Tips
        let mut maintenance_tips = Vec::new();
        if cache_report.reclaimable_bytes > 500_000_000 {
            maintenance_tips.push(format!(
                "Reclaim {} of disposable build cache via Disk Janitor.",
                cache_report.reclaimable_human_size
            ));
        }
        for issue in &health.issues {
            maintenance_tips.push(issue.clone());
        }
        if maintenance_tips.is_empty() {
            maintenance_tips.push("Project is well-maintained and fully optimized.".to_string());
        }

        LocalAiSummaryDto {
            project_name: project.name.clone(),
            architecture_pattern,
            suggested_run_command,
            key_highlights,
            maintenance_tips,
            is_ai_generated_offline: true,
        }
    }
}
