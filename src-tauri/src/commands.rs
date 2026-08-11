use crate::error::ErrorDto;
use crate::models::{Project, ScanLocation, ScanSummary};
use crate::storage::project_repo::ProjectRepo;
use crate::storage::AppDb;
use crate::system::SystemActions;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, State};

/// Shared application state held by Tauri.
pub struct AppState {
    pub db: AppDb,
    pub scan_handle: Mutex<Option<Arc<crate::discovery::scanner::ScanHandle>>>,
}

fn state(app: &AppHandle) -> State<'_, AppState> {
    app.state::<AppState>()
}

// ── Scan locations ────────────────────────────────────────────────

#[tauri::command]
pub fn list_scan_locations(app: AppHandle) -> Result<Vec<ScanLocation>, ErrorDto> {
    ProjectRepo::list_scan_locations(&state(&app).db).map_err(Into::into)
}

#[tauri::command]
pub fn add_scan_location(app: AppHandle, path: String) -> Result<i64, ErrorDto> {
    let p = PathBuf::from(&path);
    if !p.is_dir() {
        return Err(crate::error::Error::Other(format!(
            "The selected path is not a directory: {path}"
        ))
        .into());
    }
    let canonical = crate::fs::canonicalize(&p)?;
    let name = crate::fs::path_name(&canonical);
    let db = &state(&app).db;
    ProjectRepo::add_scan_location(db, &canonical.to_string_lossy(), &name).map_err(Into::into)
}

#[tauri::command]
pub fn remove_scan_location(app: AppHandle, id: i64) -> Result<(), ErrorDto> {
    ProjectRepo::remove_scan_location(&state(&app).db, id).map_err(Into::into)
}

// ── Scan / projects ───────────────────────────────────────────────

#[tauri::command]
pub async fn scan_projects(app: AppHandle) -> Result<ScanSummary, ErrorDto> {
    let st = state(&app);
    let handle = st
        .scan_handle
        .lock()
        .unwrap()
        .get_or_insert_with(|| Arc::new(crate::discovery::scanner::ScanHandle::new()))
        .clone();

    let db_dir = st.db.data_dir.clone();
    let app_for_emit = app.clone();
    // Run the scan off the async runtime so the UI stays responsive, emitting
    // `scan-progress` events as projects are ingested.
    let db = tauri::async_runtime::spawn_blocking({
        move || -> crate::error::Result<ScanSummary> {
            run_scan(db_dir, handle, app_for_emit)
        }
    })
    .await
    .map_err(|e| ErrorDto {
        message: format!("Scan task failed: {e}"),
        hint: None,
    })??;
    let _ = app.emit("scan-complete", &db);
    Ok(db)
}

fn run_scan(
    db_dir: PathBuf,
    handle: Arc<crate::discovery::scanner::ScanHandle>,
    emitter: AppHandle,
) -> crate::error::Result<ScanSummary> {
    let db = crate::storage::init_db(&db_dir)?;
    // Full-scan semantics: drop whatever was found before and rebuild, so stale
    // projects or noise from a previous scope never linger.
    crate::storage::project_repo::ProjectRepo::clear_projects(&db)?;
    let scanner = crate::discovery::scanner::Scanner::new(db, true);
    scanner.scan_all(&handle, &|p| {
        let _ = emitter.emit("scan-progress", &p);
    })
}

#[tauri::command]
pub fn cancel_scan(app: AppHandle) -> Result<(), ErrorDto> {
    if let Some(h) = state(&app).scan_handle.lock().unwrap().as_ref() {
        h.cancel();
    }
    Ok(())
}

#[tauri::command]
pub fn list_projects(app: AppHandle, limit: Option<u64>) -> Result<Vec<Project>, ErrorDto> {
    ProjectRepo::list_projects(&state(&app).db, limit).map_err(Into::into)
}

#[tauri::command]
pub fn get_project(app: AppHandle, id: i64) -> Result<Option<Project>, ErrorDto> {
    ProjectRepo::get_project(&state(&app).db, id).map_err(Into::into)
}

#[tauri::command]
pub fn set_favorite(app: AppHandle, id: i64, favorite: bool) -> Result<(), ErrorDto> {
    ProjectRepo::set_favorite(&state(&app).db, id, favorite).map_err(Into::into)
}

#[tauri::command]
pub fn list_recent(app: AppHandle, limit: Option<u64>) -> Result<Vec<Project>, ErrorDto> {
    ProjectRepo::list_recent(&state(&app).db, limit.unwrap_or(10)).map_err(Into::into)
}

// ── System actions ────────────────────────────────────────────────

#[tauri::command]
pub fn open_project_folder(app: AppHandle, path: String) -> Result<(), ErrorDto> {
    if let Some(id) = find_project_id(&state(&app).db, &path)? {
        let _ = ProjectRepo::touch_recent(&state(&app).db, id);
    }
    SystemActions::open_folder(&PathBuf::from(path)).map_err(Into::into)
}

#[tauri::command]
pub fn open_project_terminal(app: AppHandle, path: String) -> Result<(), ErrorDto> {
    if let Some(id) = find_project_id(&state(&app).db, &path)? {
        let _ = ProjectRepo::touch_recent(&state(&app).db, id);
    }
    SystemActions::open_terminal(&PathBuf::from(path)).map_err(Into::into)
}

#[tauri::command]
pub fn open_project_editor(app: AppHandle, path: String) -> Result<(), ErrorDto> {
    if let Some(id) = find_project_id(&state(&app).db, &path)? {
        let _ = ProjectRepo::touch_recent(&state(&app).db, id);
    }
    SystemActions::open_editor(&PathBuf::from(path)).map_err(Into::into)
}

#[tauri::command]
pub fn detect_editor(app: AppHandle) -> Result<Option<String>, ErrorDto> {
    let _ = app;
    Ok(SystemActions::detect_editor().map(|s| s.to_string()))
}

fn find_project_id(db: &AppDb, path: &str) -> crate::error::Result<Option<i64>> {
    let conn = db.conn();
    let mut stmt = conn.prepare("SELECT id FROM projects WHERE path=?1")?;
    let mut rows = stmt.query(rusqlite::params![path])?;
    match rows.next()? {
        Some(r) => Ok(Some(r.get(0)?)),
        None => Ok(None),
    }
}