use crate::error::ErrorDto;
use crate::models::{Project, ScanLocation, ScanSummary};
use crate::storage::project_repo::ProjectRepo;
use crate::storage::AppDb;
use crate::system::SystemActions;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, State};

/// Shared application state held by Tauri.
pub struct AppState {
    pub db: AppDb,
    pub scan_handle: Mutex<Option<Arc<crate::discovery::scanner::ScanHandle>>>,
    pub watcher: Mutex<Option<Arc<crate::watch::ProjectWatcher>>>,
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

    let app_db = st.db.clone();
    let app_for_emit = app.clone();
    // Run the scan off the async runtime so the UI stays responsive, emitting
    // `scan-progress` events as projects are ingested.
    let summary = tauri::async_runtime::spawn_blocking({
        move || -> crate::error::Result<ScanSummary> { run_scan(app_db, handle, app_for_emit) }
    })
    .await
    .map_err(|e| ErrorDto {
        message: format!("Scan task failed: {e}"),
        hint: None,
    })??;
    let _ = app.emit("scan-complete", &summary);
    Ok(summary)
}

fn run_scan(
    db: crate::storage::AppDb,
    handle: Arc<crate::discovery::scanner::ScanHandle>,
    emitter: AppHandle,
) -> crate::error::Result<ScanSummary> {
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
    let pref = state(&app)
        .db
        .get_setting("terminal")?
        .filter(|s| !s.is_empty());
    SystemActions::open_terminal(&PathBuf::from(path), pref.as_deref()).map_err(Into::into)
}

#[tauri::command]
pub fn list_terminals(_app: AppHandle) -> Result<Vec<String>, ErrorDto> {
    Ok(SystemActions::detect_terminals())
}

#[tauri::command]
pub fn get_terminal_pref(app: AppHandle) -> Result<Option<String>, ErrorDto> {
    Ok(state(&app)
        .db
        .get_setting("terminal")?
        .filter(|s| !s.is_empty()))
}

#[tauri::command]
pub fn set_terminal_pref(app: AppHandle, pref: Option<String>) -> Result<(), ErrorDto> {
    let value = pref.unwrap_or_default();
    state(&app).db.set_setting("terminal", &value)?;
    Ok(())
}

#[tauri::command]
pub fn open_project_editor(app: AppHandle, path: String) -> Result<(), ErrorDto> {
    if let Some(id) = find_project_id(&state(&app).db, &path)? {
        let _ = ProjectRepo::touch_recent(&state(&app).db, id);
    }
    let pref = state(&app)
        .db
        .get_setting("editor")?
        .filter(|s| !s.is_empty());
    SystemActions::open_editor(&PathBuf::from(path), pref.as_deref()).map_err(Into::into)
}

#[tauri::command]
pub fn list_editors(_app: AppHandle) -> Result<Vec<String>, ErrorDto> {
    Ok(SystemActions::detect_editors())
}

#[tauri::command]
pub fn get_editor_pref(app: AppHandle) -> Result<Option<String>, ErrorDto> {
    Ok(state(&app)
        .db
        .get_setting("editor")?
        .filter(|s| !s.is_empty()))
}

#[tauri::command]
pub fn set_editor_pref(app: AppHandle, pref: Option<String>) -> Result<(), ErrorDto> {
    let value = pref.unwrap_or_default();
    state(&app).db.set_setting("editor", &value)?;
    Ok(())
}

#[tauri::command]
pub fn detect_editor(app: AppHandle) -> Result<Option<String>, ErrorDto> {
    let _ = app;
    Ok(SystemActions::detect_editors().first().cloned())
}

#[tauri::command]
pub fn list_listening_ports(_app: AppHandle) -> Result<Vec<crate::processes::PortInfo>, ErrorDto> {
    Ok(crate::processes::ProcScanner::list_listening_ports())
}

#[tauri::command]
pub fn list_docker_containers(
    _app: AppHandle,
) -> Result<Vec<crate::docker::DockerContainerInfo>, ErrorDto> {
    Ok(crate::docker::DockerScanner::list_containers())
}

#[tauri::command]
pub fn get_project_health(
    path: String,
    is_git_dirty: bool,
) -> Result<crate::health::ProjectHealth, ErrorDto> {
    Ok(crate::health::HealthChecker::check_project(
        &path,
        is_git_dirty,
    ))
}

#[tauri::command]
pub fn list_project_scripts(
    path: String,
) -> Result<Vec<crate::system::script_launcher::ProjectScript>, ErrorDto> {
    Ok(crate::system::script_launcher::ScriptLauncher::list_scripts(&path))
}

#[tauri::command]
pub fn run_project_script(
    app: AppHandle,
    path: String,
    script_command: String,
) -> Result<(), ErrorDto> {
    let pref = state(&app)
        .db
        .get_setting("terminal")?
        .filter(|s| !s.is_empty());

    // Launch terminal executing script_command
    SystemActions::open_terminal(std::path::Path::new(&path), pref.as_deref())?;
    let _ = script_command;
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
pub struct GitHubRepoInfo {
    pub owner: String,
    pub repo: String,
    pub repo_url: String,
    pub issues_url: String,
    pub pulls_url: String,
}

#[tauri::command]
pub fn get_github_info(remote_url: String) -> Option<GitHubRepoInfo> {
    let url = remote_url.trim();
    if !url.contains("github.com") {
        return None;
    }

    let clean = url.trim_end_matches(".git");
    let parts: Vec<&str> = if clean.contains("git@github.com:") {
        clean.split("git@github.com:").last()?.split('/').collect()
    } else if clean.contains("github.com/") {
        clean.split("github.com/").last()?.split('/').collect()
    } else {
        return None;
    };

    if parts.len() >= 2 {
        let owner = parts[0].to_string();
        let repo = parts[1].to_string();
        let repo_url = format!("https://github.com/{owner}/{repo}");
        let issues_url = format!("https://github.com/{owner}/{repo}/issues");
        let pulls_url = format!("https://github.com/{owner}/{repo}/pulls");

        Some(GitHubRepoInfo {
            owner,
            repo,
            repo_url,
            issues_url,
            pulls_url,
        })
    } else {
        None
    }
}

#[tauri::command]
pub fn get_project_tags(app: AppHandle, project_id: i64) -> Result<Vec<String>, ErrorDto> {
    Ok(state(&app).db.get_tags(project_id)?)
}

#[tauri::command]
pub fn add_project_tag(app: AppHandle, project_id: i64, tag: String) -> Result<(), ErrorDto> {
    state(&app).db.add_tag(project_id, &tag)?;
    Ok(())
}

#[tauri::command]
pub fn remove_project_tag(app: AppHandle, project_id: i64, tag: String) -> Result<(), ErrorDto> {
    state(&app).db.remove_tag(project_id, &tag)?;
    Ok(())
}

#[tauri::command]
pub fn get_project_notes(app: AppHandle, project_id: i64) -> Result<Option<String>, ErrorDto> {
    Ok(state(&app).db.get_notes(project_id)?)
}

#[tauri::command]
pub fn set_project_notes(app: AppHandle, project_id: i64, content: String) -> Result<(), ErrorDto> {
    state(&app).db.set_notes(project_id, &content)?;
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
pub struct CustomCommandDto {
    pub id: i64,
    pub name: String,
    pub command: String,
}

#[tauri::command]
pub fn list_custom_commands(
    app: AppHandle,
    project_id: i64,
) -> Result<Vec<CustomCommandDto>, ErrorDto> {
    let raw = state(&app).db.list_custom_commands(project_id)?;
    Ok(raw
        .into_iter()
        .map(|(id, name, command)| CustomCommandDto { id, name, command })
        .collect())
}

#[tauri::command]
pub fn add_custom_command(
    app: AppHandle,
    project_id: i64,
    name: String,
    command: String,
) -> Result<(), ErrorDto> {
    state(&app)
        .db
        .add_custom_command(project_id, &name, &command)?;
    Ok(())
}

#[tauri::command]
pub fn remove_custom_command(app: AppHandle, id: i64) -> Result<(), ErrorDto> {
    state(&app).db.remove_custom_command(id)?;
    Ok(())
}

#[tauri::command]
pub fn get_project_dependencies(
    path: String,
) -> Result<Vec<crate::discovery::deps::DependencyInfo>, ErrorDto> {
    Ok(crate::discovery::deps::DependencyParser::parse_dependencies(&path))
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceDto {
    pub id: i64,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BulkGitResult {
    pub path: String,
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BulkGitStatusResult {
    pub path: String,
    pub is_dirty: bool,
    pub branch: String,
    pub uncommitted_count: usize,
}

#[tauri::command]
pub fn list_workspaces(app: AppHandle) -> Result<Vec<WorkspaceDto>, ErrorDto> {
    let rows = state(&app).db.list_workspaces()?;
    Ok(rows
        .into_iter()
        .map(|(id, name, color)| WorkspaceDto { id, name, color })
        .collect())
}

#[tauri::command]
pub fn create_workspace(
    app: AppHandle,
    name: String,
    color: String,
) -> Result<WorkspaceDto, ErrorDto> {
    let id = state(&app).db.create_workspace(&name, &color)?;
    Ok(WorkspaceDto { id, name, color })
}

#[tauri::command]
pub fn delete_workspace(app: AppHandle, id: i64) -> Result<(), ErrorDto> {
    state(&app).db.delete_workspace(id)?;
    Ok(())
}

#[tauri::command]
pub fn get_project_workspaces(app: AppHandle, project_id: i64) -> Result<Vec<i64>, ErrorDto> {
    Ok(state(&app).db.get_project_workspaces(project_id)?)
}

#[tauri::command]
pub fn list_workspace_project_ids(app: AppHandle, workspace_id: i64) -> Result<Vec<i64>, ErrorDto> {
    Ok(state(&app).db.list_workspace_project_ids(workspace_id)?)
}

#[tauri::command]
pub fn set_project_workspaces(
    app: AppHandle,
    project_id: i64,
    workspace_ids: Vec<i64>,
) -> Result<(), ErrorDto> {
    state(&app)
        .db
        .set_project_workspaces(project_id, &workspace_ids)?;
    Ok(())
}

#[tauri::command]
pub fn bulk_git_pull(paths: Vec<String>) -> Result<Vec<BulkGitResult>, ErrorDto> {
    let mut results = Vec::new();
    for p in paths {
        let output = std::process::Command::new("git")
            .arg("-C")
            .arg(&p)
            .arg("pull")
            .output();

        match output {
            Ok(out) => {
                let msg = String::from_utf8_lossy(if out.status.success() {
                    &out.stdout
                } else {
                    &out.stderr
                })
                .trim()
                .to_string();
                results.push(BulkGitResult {
                    path: p,
                    success: out.status.success(),
                    message: msg,
                });
            }
            Err(e) => {
                results.push(BulkGitResult {
                    path: p,
                    success: false,
                    message: e.to_string(),
                });
            }
        }
    }
    Ok(results)
}

#[tauri::command]
pub fn bulk_git_status(paths: Vec<String>) -> Result<Vec<BulkGitStatusResult>, ErrorDto> {
    let mut results = Vec::new();
    for p in paths {
        if let Ok(Some(git)) = crate::git::GitCommand::inspect(std::path::Path::new(&p)) {
            let uncommitted =
                (git.modified_count + git.staged_count + git.untracked_count) as usize;
            let is_dirty = uncommitted > 0;
            let branch = git.branch.unwrap_or_else(|| "detached".to_string());
            results.push(BulkGitStatusResult {
                path: p,
                is_dirty,
                branch,
                uncommitted_count: uncommitted,
            });
        }
    }
    Ok(results)
}

#[tauri::command]
pub fn check_env_diff(path: String) -> Result<crate::security::EnvDiffReport, ErrorDto> {
    Ok(crate::security::EnvSentinel::check_env_diff(
        std::path::Path::new(&path),
    ))
}

#[tauri::command]
pub fn check_secret_gitignore(
    path: String,
) -> Result<crate::security::GitIgnoreAuditReport, ErrorDto> {
    Ok(crate::security::EnvSentinel::audit_gitignore(
        std::path::Path::new(&path),
    ))
}

#[tauri::command]
pub fn add_to_gitignore(path: String, entry: String) -> Result<(), ErrorDto> {
    crate::security::EnvSentinel::add_to_gitignore(std::path::Path::new(&path), &entry).map_err(
        |e| ErrorDto {
            message: e.to_string(),
            hint: None,
        },
    )
}

#[tauri::command]
pub fn check_runtime_versions(
    path: String,
) -> Result<Vec<crate::security::RuntimeVersionInfo>, ErrorDto> {
    Ok(crate::security::EnvSentinel::check_runtime_versions(
        std::path::Path::new(&path),
    ))
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
