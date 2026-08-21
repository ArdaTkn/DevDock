//! Integration tests: project discovery across the fixture tree, plus Git
//! status detection on real (temp) repositories.

use devdock_lib::discovery::detector::{DetectContext, DetectorRegistry};
use devdock_lib::discovery::scanner::{ScanHandle, Scanner};
use devdock_lib::git::git_core::GitCommand;
use devdock_lib::models::TechKind;
use devdock_lib::storage::project_repo::ProjectRepo;
use std::path::PathBuf;
use std::process::Command;

fn registry() -> DetectorRegistry {
    DetectorRegistry::default_registry()
}

/// Absolute path to a fixture dir, independent of the CWD (CI runs tests with
/// `--manifest-path`, so the working dir is the repo root, not src-tauri).
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn ctx(is_git: bool) -> DetectContext {
    DetectContext {
        is_git_repo: is_git,
    }
}

#[test]
fn node_detector_recognises_node_project() {
    let dir = fixture("node-project");
    let techs = registry()
        .detect_all(&dir, &ctx(false))
        .expect("should detect");
    assert!(techs.iter().any(|t| t.name == "Node.js"));
    assert!(techs.iter().any(|t| t.name == "Vite"));
    assert!(techs.iter().any(|t| t.name == "React"));
}

#[test]
fn python_detector_recognises_python_project() {
    let dir = fixture("python-project");
    let techs = registry()
        .detect_all(&dir, &ctx(false))
        .expect("should detect");
    assert!(techs
        .iter()
        .any(|t| t.name == "Python" && t.kind == TechKind::Language));
}

#[test]
fn rust_detector_recognises_rust_project() {
    let dir = fixture("rust-project");
    let techs = registry()
        .detect_all(&dir, &ctx(false))
        .expect("should detect");
    assert!(techs.iter().any(|t| t.name == "Rust"));
    assert!(techs.iter().any(|t| t.name == "Cargo"));
}

#[test]
fn flutter_detector_recognises_flutter_project() {
    let dir = fixture("flutter-project");
    let techs = registry()
        .detect_all(&dir, &ctx(false))
        .expect("should detect");
    assert!(techs.iter().any(|t| t.name == "Flutter"));
    assert!(techs.iter().any(|t| t.name == "Dart"));
}

#[test]
fn docker_detector_recognises_docker_project() {
    let dir = fixture("docker-project");
    let techs = registry()
        .detect_all(&dir, &ctx(false))
        .expect("should detect");
    assert!(techs.iter().any(|t| t.name == "Docker"));
}

#[test]
fn mixed_project_detects_both_node_and_docker() {
    let dir = fixture("mixed-project");
    let techs = registry()
        .detect_all(&dir, &ctx(false))
        .expect("should detect");
    assert!(techs.iter().any(|t| t.name == "Node.js"));
    assert!(techs.iter().any(|t| t.name == "Docker"));
}

#[test]
fn empty_dir_is_not_a_project() {
    let dir = fixtures_root(); // no marker at this level
    assert!(registry().detect_all(&dir, &ctx(false)).is_none());
}

// ── Git integration ───────────────────────────────────────────────

#[test]
fn git_clean_vs_dirty_detection() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let repo = tmp.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();

    run(&repo, &["init", "-q"]);
    run(&repo, &["config", "user.email", "test@test.dev"]);
    run(&repo, &["config", "user.name", "Test"]);
    std::fs::write(repo.join("f.txt"), "one").unwrap();
    run(&repo, &["add", "."]);
    run(&repo, &["commit", "-qm", "initial"]);

    // Clean state
    let clean = GitCommand::inspect(&repo)
        .expect("git inspect")
        .expect("should be a git repo");
    assert!(clean.is_git);
    assert_eq!(clean.modified_count, 0);
    assert_eq!(clean.untracked_count, 0);
    assert!(clean.clean());

    // Introduce a modification + untracked file → dirty
    std::fs::write(repo.join("f.txt"), "two").unwrap();
    std::fs::write(repo.join("new.txt"), "hello").unwrap();
    let dirty = GitCommand::inspect(&repo)
        .expect("git inspect")
        .expect("should be a git repo");
    assert_eq!(dirty.modified_count, 1);
    assert_eq!(dirty.untracked_count, 1);
    assert!(!dirty.clean());
}

fn run(dir: &PathBuf, args: &[&str]) {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
}

// ── End-to-end scan over fixtures ─────────────────────────────────

#[test]
fn scan_fixtures_discovers_projects() {
    let root = fixtures_root();
    let tmp = tempfile::tempdir().expect("tempdir");
    let data_dir = tmp.path().join("data");
    let db = devdock_lib::storage::init_db(&data_dir).expect("db");
    ProjectRepo::add_scan_location(&db, &root.to_string_lossy(), "fixtures").unwrap();

    let scanner = Scanner::new(db, true);
    let handle = ScanHandle::new();
    let summary = scanner.scan_all(&handle, &|_| {}).expect("scan");

    // node/python/rust/flutter/docker/mixed are discovered; git-clean/dirty are
    // empty dirs with no markers (they become git repos only in the git test).
    assert!(
        summary.total >= 6,
        "expected >=6 projects, got {}",
        summary.total
    );
}

#[test]
fn test_env_sentinel_diff_detection() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let p = tmp.path();

    // Create .env.example with 3 keys
    std::fs::write(
        p.join(".env.example"),
        "DATABASE_URL=postgres://...\nPORT=3000\nAPI_SECRET=\n# Comment\n",
    )
    .unwrap();

    // Create .env with 2 keys (missing API_SECRET)
    std::fs::write(p.join(".env"), "DATABASE_URL=postgres://...\nPORT=3000\n").unwrap();

    let report = devdock_lib::security::EnvSentinel::check_env_diff(p);
    assert!(report.has_template);
    assert_eq!(report.template_file.as_deref(), Some(".env.example"));
    assert!(report.has_local_env);
    assert_eq!(report.missing_keys, vec!["API_SECRET".to_string()]);
}

#[test]
fn test_env_sentinel_gitignore_audit() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let p = tmp.path();

    // Create sensitive file
    std::fs::write(p.join(".env"), "SECRET=123").unwrap();
    std::fs::write(p.join("id_rsa"), "KEY").unwrap();

    // Create .gitignore with only .env
    std::fs::write(p.join(".gitignore"), ".env\n").unwrap();

    let audit = devdock_lib::security::EnvSentinel::audit_gitignore(p);
    assert!(audit.has_gitignore);
    assert_eq!(audit.unignored_sensitive_files, vec!["id_rsa".to_string()]);

    // Add id_rsa to .gitignore
    devdock_lib::security::EnvSentinel::add_to_gitignore(p, "id_rsa").unwrap();

    let audit2 = devdock_lib::security::EnvSentinel::audit_gitignore(p);
    assert!(audit2.unignored_sensitive_files.is_empty());
}

#[test]
fn test_cache_janitor_scan_and_clean() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let p = tmp.path();

    // Create node_modules and target with dummy files
    let nm = p.join("node_modules");
    std::fs::create_dir_all(&nm).unwrap();
    std::fs::write(nm.join("package.json"), "{\"name\":\"dummy\"}").unwrap();

    let target = p.join("target");
    std::fs::create_dir_all(&target).unwrap();
    std::fs::write(target.join("output.bin"), "binary data 123456").unwrap();

    let rep = devdock_lib::system::CacheJanitor::scan_project_cache(p);
    assert_eq!(rep.cache_folders.len(), 2);
    assert!(rep.reclaimable_bytes > 0);

    // Clean node_modules
    let freed = devdock_lib::system::CacheJanitor::clean_cache_folder(p, "node_modules").unwrap();
    assert!(freed > 0);
    assert!(!nm.exists());

    // Security check: trying to clean an unsafe folder must fail
    let err = devdock_lib::system::CacheJanitor::clean_cache_folder(p, "src");
    assert!(err.is_err());

    let rep2 = devdock_lib::system::CacheJanitor::scan_project_cache(p);
    assert_eq!(rep2.cache_folders.len(), 1);
    assert_eq!(rep2.cache_folders[0].name, "target");
}

#[test]
fn test_knowledge_graph_building() {
    use devdock_lib::models::{Project, Tech, TechKind};
    use std::collections::HashMap;

    let proj = Project {
        id: 1,
        name: "test-app".into(),
        path: "/path/to/test-app".into(),
        relative_path: None,
        size_bytes: 1024,
        last_modified: 0,
        techs: vec![Tech {
            name: "Rust".into(),
            kind: TechKind::Language,
        }],
        git: None,
        is_favorite: false,
    };

    let ws = (10i64, "Backend".to_string(), "#10b981".to_string());

    let mut ws_map = HashMap::new();
    ws_map.insert(1, vec![10]);

    let graph =
        devdock_lib::graph::GraphEngine::build_knowledge_graph(&[proj], &[ws], &ws_map, &[]);

    assert_eq!(graph.total_projects, 1);
    assert_eq!(graph.total_workspaces, 1);
    assert_eq!(graph.total_techs, 1);
    assert!(graph.links.len() >= 2); // 1 to workspace, 1 to tech
}

#[test]
fn test_local_ai_architecture_analysis() {
    use devdock_lib::health::ProjectHealth;
    use devdock_lib::models::{Project, Tech, TechKind};
    use devdock_lib::system::ProjectCacheReport;

    let proj = Project {
        id: 2,
        name: "my-flutter-app".into(),
        path: "/path/to/my-flutter-app".into(),
        relative_path: None,
        size_bytes: 2048,
        last_modified: 0,
        techs: vec![Tech {
            name: "Flutter".into(),
            kind: TechKind::Framework,
        }],
        git: None,
        is_favorite: false,
    };

    let health = ProjectHealth {
        score: 95,
        status: "Healthy".into(),
        deps_installed: true,
        has_readme: true,
        is_git_clean: true,
        issues: vec![],
    };

    let cache = ProjectCacheReport {
        total_size_bytes: 1000,
        total_human_size: "1 KB".into(),
        reclaimable_bytes: 0,
        reclaimable_human_size: "0 B".into(),
        cache_folders: vec![],
    };

    let ai = devdock_lib::ai::LocalAiEngine::analyze_project(&proj, &health, &cache);
    assert_eq!(ai.suggested_run_command, "flutter run");
    assert!(ai.architecture_pattern.contains("Flutter"));
    assert!(ai.is_ai_generated_offline);
}
