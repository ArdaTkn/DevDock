use crate::error::Result;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};

/// Wraps the SQLite connection plus a known data directory.
///
/// `Connection` is `Send` but not `Sync`, so we hold it behind an `Arc<Mutex>` to make
/// `AppDb` `Send + Sync + Clone` (required by Tauri's `AppState` & async tasks).
#[derive(Clone)]
pub struct AppDb {
    conn: Arc<Mutex<Connection>>,
    pub data_dir: PathBuf,
}

/// Opens (creating if needed) the DevDock database and applies migrations.
pub fn init_db(data_dir: &PathBuf) -> Result<AppDb> {
    std::fs::create_dir_all(data_dir)?;
    let db_path = data_dir.join("devdock.db");

    let conn = match Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("SQLite open failed ({e}), attempting WAL recovery...");
            let _ = std::fs::remove_file(data_dir.join("devdock.db-shm"));
            let _ = std::fs::remove_file(data_dir.join("devdock.db-wal"));
            Connection::open(&db_path)?
        }
    };

    // Avoid "database is locked" hangs when concurrent reads/writes occur
    conn.busy_timeout(std::time::Duration::from_secs(10))?;
    let _ = conn.pragma_update(None, "journal_mode", "WAL");
    let _ = conn.pragma_update(None, "synchronous", "NORMAL");
    let _ = conn.pragma_update(None, "foreign_keys", "ON");
    migrate(&conn)?;
    Ok(AppDb {
        conn: Arc::new(Mutex::new(conn)),
        data_dir: data_dir.clone(),
    })
}

fn migrate(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS scan_locations (
            id INTEGER PRIMARY KEY,
            path TEXT NOT NULL UNIQUE,
            name TEXT,
            added_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS projects (
            id INTEGER PRIMARY KEY,
            path TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL,
            relative_path TEXT,
            scan_location_id INTEGER REFERENCES scan_locations(id) ON DELETE CASCADE,
            size_bytes INTEGER DEFAULT 0,
            last_modified INTEGER,
            first_seen INTEGER,
            last_scanned INTEGER,
            is_favorite INTEGER DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS project_techs (
            project_id INTEGER REFERENCES projects(id) ON DELETE CASCADE,
            tech TEXT NOT NULL,
            kind TEXT NOT NULL,
            PRIMARY KEY (project_id, tech)
        );

        CREATE TABLE IF NOT EXISTS git_metadata (
            project_id INTEGER PRIMARY KEY REFERENCES projects(id) ON DELETE CASCADE,
            is_git INTEGER DEFAULT 0,
            branch TEXT,
            remote_url TEXT,
            repo_name TEXT,
            staged_count INTEGER DEFAULT 0,
            modified_count INTEGER DEFAULT 0,
            untracked_count INTEGER DEFAULT 0,
            last_commit_message TEXT,
            last_commit_date INTEGER,
            latest_short_hash TEXT,
            refreshed_at INTEGER
        );

        CREATE TABLE IF NOT EXISTS recent_projects (
            project_id INTEGER PRIMARY KEY REFERENCES projects(id) ON DELETE CASCADE,
            opened_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS project_tags (
            project_id INTEGER REFERENCES projects(id) ON DELETE CASCADE,
            tag TEXT NOT NULL,
            PRIMARY KEY (project_id, tag)
        );

        CREATE TABLE IF NOT EXISTS project_notes (
            project_id INTEGER PRIMARY KEY REFERENCES projects(id) ON DELETE CASCADE,
            content TEXT NOT NULL,
            updated_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS project_custom_commands (
            id INTEGER PRIMARY KEY,
            project_id INTEGER REFERENCES projects(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            command TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_projects_path ON projects(path);
        CREATE INDEX IF NOT EXISTS idx_projects_location ON projects(scan_location_id);
        "#,
    )?;
    Ok(())
}

impl AppDb {
    /// Locks and returns the underlying connection. Keep the guard alive for the
    /// duration of a single statement batch; never hold two guards at once.
    pub fn conn(&self) -> MutexGuard<'_, Connection> {
        self.conn.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO settings(key,value) VALUES(?1,?2)
             ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            rusqlite::params![key, value],
        )?;
        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn();
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key=?1")?;
        let mut rows = stmt.query(rusqlite::params![key])?;
        match rows.next()? {
            Some(row) => Ok(Some(row.get(0)?)),
            None => Ok(None),
        }
    }

    // --- Tags ---
    pub fn get_tags(&self, project_id: i64) -> Result<Vec<String>> {
        let conn = self.conn();
        let mut stmt =
            conn.prepare("SELECT tag FROM project_tags WHERE project_id=?1 ORDER BY tag")?;
        let rows = stmt.query_map(rusqlite::params![project_id], |r| r.get(0))?;
        let mut tags = Vec::new();
        for t in rows {
            tags.push(t?);
        }
        Ok(tags)
    }

    pub fn add_tag(&self, project_id: i64, tag: &str) -> Result<()> {
        let conn = self.conn();
        conn.execute(
            "INSERT OR IGNORE INTO project_tags(project_id, tag) VALUES(?1, ?2)",
            rusqlite::params![project_id, tag],
        )?;
        Ok(())
    }

    pub fn remove_tag(&self, project_id: i64, tag: &str) -> Result<()> {
        let conn = self.conn();
        conn.execute(
            "DELETE FROM project_tags WHERE project_id=?1 AND tag=?2",
            rusqlite::params![project_id, tag],
        )?;
        Ok(())
    }

    // --- Notes ---
    pub fn get_notes(&self, project_id: i64) -> Result<Option<String>> {
        let conn = self.conn();
        let mut stmt = conn.prepare("SELECT content FROM project_notes WHERE project_id=?1")?;
        let mut rows = stmt.query(rusqlite::params![project_id])?;
        match rows.next()? {
            Some(r) => Ok(Some(r.get(0)?)),
            None => Ok(None),
        }
    }

    pub fn set_notes(&self, project_id: i64, content: &str) -> Result<()> {
        let conn = self.conn();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        conn.execute(
            "INSERT INTO project_notes(project_id, content, updated_at) VALUES(?1, ?2, ?3)
             ON CONFLICT(project_id) DO UPDATE SET content=excluded.content, updated_at=excluded.updated_at",
            rusqlite::params![project_id, content, now],
        )?;
        Ok(())
    }

    // --- Custom Commands ---
    pub fn list_custom_commands(&self, project_id: i64) -> Result<Vec<(i64, String, String)>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT id, name, command FROM project_custom_commands WHERE project_id=?1 ORDER BY id",
        )?;
        let rows = stmt.query_map(rusqlite::params![project_id], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn add_custom_command(&self, project_id: i64, name: &str, command: &str) -> Result<()> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO project_custom_commands(project_id, name, command) VALUES(?1, ?2, ?3)",
            rusqlite::params![project_id, name, command],
        )?;
        Ok(())
    }

    pub fn remove_custom_command(&self, id: i64) -> Result<()> {
        let conn = self.conn();
        conn.execute(
            "DELETE FROM project_custom_commands WHERE id=?1",
            rusqlite::params![id],
        )?;
        Ok(())
    }
}
