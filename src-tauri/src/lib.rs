pub mod commands;
pub mod discovery;
pub mod error;
pub mod fs;
pub mod git;
pub mod models;
pub mod storage;
pub mod system;

use std::sync::Mutex;

use tauri::Manager;

pub fn run() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."));
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

            app.manage(commands::AppState {
                db,
                scan_handle: Mutex::new(None),
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running DevDock");
}
