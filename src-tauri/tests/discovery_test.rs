//! Integration tests: project discovery across the fixture tree, plus Git
//! status detection on real (temp) repositories.

use devdock_lib::discovery::detector::{DetectContext, DetectorRegistry};
use devdock_lib::discovery::scanner::{ScanHandle, Scanner};
use devdock_lib::git::git::GitCommand;
use devdock_lib::models::TechKind;
use devdock_lib::storage::project_repo::ProjectRepo;
use std::path::PathBuf;
use std::process::Command;

fn registry() -> DetectorRegistry {
    DetectorRegistry::default_registry()
}

fn ctx(is_git: bool) -> DetectContext {
    DetectContext { is_git_repo: is_git }
}

#[test]
fn node_detector_recognises_node_project() {
    let dir = PathBuf::from("tests/fixtures/node-project");
    let techs = registry().detect_all(&dir, &ctx(false)).expect("should detect");
    assert!(techs.iter().any(|t| t.name == "Node.js"));
    assert!(techs.iter().any(|t| t.name == "Vite"));
    assert!(techs.iter().any(|t| t.name == "React"));
}

#[test]
fn python_detector_recognises_python_project() {
    let dir = PathBuf::from("tests/fixtures/python-project");
    let techs = registry().detect_all(&dir, &ctx(false)).expect("should detect");
    assert!(techs.iter().any(|t| t.name == "Python" && t.kind == TechKind::Language));
}

#[test]
fn rust_detector_recognises_rust_project() {
    let dir = PathBuf::from("tests/fixtures/rust-project");
    let techs = registry().detect_all(&dir, &ctx(false)).expect("should detect");
    assert!(techs.iter().any(|t| t.name == "Rust"));
    assert!(techs.iter().any(|t| t.name == "Cargo"));
}

#[test]
fn flutter_detector_recognises_flutter_project() {
    let dir = PathBuf::from("tests/fixtures/flutter-project");
    let techs = registry().detect_all(&dir, &ctx(false)).expect("should detect");
    assert!(techs.iter().any(|t| t.name == "Flutter"));
    assert!(techs.iter().any(|t| t.name == "Dart"));
}

#[test]
fn docker_detector_recognises_docker_project() {
    let dir = PathBuf::from("tests/fixtures/docker-project");
    let techs = registry().detect_all(&dir, &ctx(false)).expect("should detect");
    assert!(techs.iter().any(|t| t.name == "Docker"));
}

#[test]
fn mixed_project_detects_both_node_and_docker() {
    let dir = PathBuf::from("tests/fixtures/mixed-project");
    let techs = registry().detect_all(&dir, &ctx(false)).expect("should detect");
    assert!(techs.iter().any(|t| t.name == "Node.js"));
    assert!(techs.iter().any(|t| t.name == "Docker"));
}

#[test]
fn empty_dir_is_not_a_project() {
    let dir = PathBuf::from("tests/fixtures"); // no marker at this level
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
    let tmp = tempfile::tempdir().expect("tempdir");
    let data_dir = tmp.path().join("data");
    let db = devdock_lib::storage::init_db(&data_dir).expect("db");
    ProjectRepo::add_scan_location(&db, "tests/fixtures", "fixtures").unwrap();

    let scanner = Scanner::new(db, true);
    let handle = ScanHandle::new();
    let summary = scanner.scan_all(&handle).expect("scan");

    // git-clean / git-dirty have no marker files + empty → not counted here,
    // but node/python/rust/flutter/docker/mixed should be.
    assert!(summary.total >= 6, "expected >=6 projects, got {}", summary.total);
}
