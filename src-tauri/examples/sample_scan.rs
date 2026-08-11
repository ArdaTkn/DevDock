//! Verify DevDock against real directories: init a temp DB, add the given
//! scan locations (from argv), scan, and print discovered projects with
//! their techs + git status.
//!
//! Usage: cargo run --example scan -- /path/projects /path/other ...

use devdock_lib::discovery::scanner::{ScanHandle, Scanner};
use devdock_lib::storage::project_repo::ProjectRepo;
use devdock_lib::storage::AppDb;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("usage: cargo run --example scan -- <dir> [<dir> ...]");
        std::process::exit(2);
    }

    let tmp = std::env::temp_dir().join("devdock_sample_db");
    let _ = std::fs::remove_dir_all(&tmp);
    let db: AppDb = devdock_lib::storage::init_db(&tmp).expect("db");

    for dir in &args {
        ProjectRepo::add_scan_location(&db, dir, dir).expect("add location");
    }

    let scanner = Scanner::new(db, true);
    let handle = ScanHandle::new();
    let summary = scanner.scan_all(&handle, &|_| {}).expect("scan");

    println!("\n=== SCAN RESULT: {} projects ===", summary.total);
    let db2: AppDb = devdock_lib::storage::init_db(&tmp).expect("reopen");
    let projects = ProjectRepo::list_projects(&db2, None).expect("list");

    let mut breakdown = summary.tech_breakdown.clone();
    breakdown.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    println!("Tech breakdown:");
    for (name, count) in breakdown.iter().take(15) {
        println!("  {name:15} {count}");
    }

    println!("\nProjects:");
    for p in projects {
        let techs: Vec<&str> = p.techs.iter().map(|t| t.name.as_str()).take(5).collect();
        let git = match &p.git {
            Some(g) if g.is_git => {
                let state = if g.clean() {
                    format!("clean on {}", g.branch.as_deref().unwrap_or("?"))
                } else {
                    format!(
                        "DIRTY (+{} ~{} on {})",
                        g.untracked_count,
                        g.modified_count,
                        g.branch.as_deref().unwrap_or("?")
                    )
                };
                state
            }
            _ => "no git".to_string(),
        };
        println!("  • {}  [{}]  {git}", p.name, techs.join(", "));
        println!("      {}", p.path);
    }

    println!(
        "\ndone. ({} clean, {} dirty)",
        summary.clean_count, summary.dirty_count
    );
}
