pub mod commands;
pub mod discovery;
pub mod docker;
pub mod error;
pub mod fs;
pub mod git;
pub mod health;
pub mod models;
pub mod processes;
pub mod security;
pub mod storage;
pub mod system;
pub mod watch;

use std::sync::{Arc, Mutex};

use tauri::Manager;

pub fn run() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = std::env::home_dir()
                .map(|h| h.join(".devdock"))
                .unwrap_or_else(|| {
                    app.path()
                        .app_data_dir()
                        .unwrap_or_else(|_| std::path::PathBuf::from("."))
                });
            let db = storage::init_db(&data_dir)?;

            // Default: scan the whole home directory so the user can hit "Scan"
            // and immediately see every project without adding folders by hand.
            // (Only added if the user hasn't configured locations yet.)
            if storage::project_repo::ProjectRepo::list_scan_locations(&db)?.is_empty() {
                if let Some(home) = std::env::home_dir() {
                    let home_str = home.to_string_lossy().to_string();
                    let name = crate::fs::path_name(&home);
                    let _ = storage::project_repo::ProjectRepo::add_scan_location(
                        &db, &home_str, &name,
                    );
                }
            }

            // Start watcher on discovered project directories + scan locations
            let mut scan_paths: Vec<std::path::PathBuf> = Vec::new();
            if let Ok(projects) = storage::project_repo::ProjectRepo::list_projects(&db, None) {
                for p in projects {
                    scan_paths.push(std::path::PathBuf::from(p.path));
                }
            }

            let locs =
                storage::project_repo::ProjectRepo::list_scan_locations(&db).unwrap_or_default();
            for l in locs {
                let p = std::path::PathBuf::from(l.path);
                if !scan_paths.contains(&p) {
                    scan_paths.push(p);
                }
            }

            let watcher = crate::watch::ProjectWatcher::new(app.handle().clone(), scan_paths).ok();

            app.manage(commands::AppState {
                db,
                scan_handle: Mutex::new(None),
                watcher: Mutex::new(watcher.map(Arc::new)),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_scan_locations,
            commands::add_scan_location,
            commands::remove_scan_location,
            commands::scan_projects,
            commands::cancel_scan,
            commands::list_projects,
            commands::get_project,
            commands::set_favorite,
            commands::list_recent,
            commands::open_project_folder,
            commands::open_project_terminal,
            commands::open_project_editor,
            commands::detect_editor,
            commands::list_editors,
            commands::get_editor_pref,
            commands::set_editor_pref,
            commands::list_terminals,
            commands::get_terminal_pref,
            commands::set_terminal_pref,
            commands::list_listening_ports,
            commands::list_docker_containers,
            commands::get_project_health,
            commands::list_project_scripts,
            commands::run_project_script,
            commands::get_github_info,
            commands::get_project_tags,
            commands::add_project_tag,
            commands::remove_project_tag,
            commands::get_project_notes,
            commands::set_project_notes,
            commands::list_custom_commands,
            commands::add_custom_command,
            commands::remove_custom_command,
            commands::get_project_dependencies,
            commands::list_workspaces,
            commands::create_workspace,
            commands::delete_workspace,
            commands::get_project_workspaces,
            commands::list_workspace_project_ids,
            commands::set_project_workspaces,
            commands::bulk_git_pull,
            commands::bulk_git_status,
            commands::check_env_diff,
            commands::check_secret_gitignore,
            commands::add_to_gitignore,
            commands::check_runtime_versions,
            commands::get_project_cache_info,
            commands::clean_cache_folder,
            commands::get_disk_hogs_report,
        ])
        .run(tauri::generate_context!())
        .expect("error while running DevDock");
}
