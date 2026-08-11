use crate::error::Result;
use crate::models::{GitInfo, Project, ScanLocation, Tech, TechKind};
use crate::storage::AppDb;

/// All SQLite reads/writes for projects, techs, and scan locations.
/// Each function first locks the connection once via `let conn = db.conn();`.
pub struct ProjectRepo;

impl ProjectRepo {
    pub fn add_scan_location(db: &AppDb, path: &str, name: &str) -> Result<i64> {
        let conn = db.conn();
        conn.execute(
            "INSERT OR IGNORE INTO scan_locations(path,name,added_at)
             VALUES(?1,?2,strftime('%s','now'))",
            rusqlite::params![path, name],
        )?;
        Ok(conn.query_row(
            "SELECT id FROM scan_locations WHERE path=?1",
            rusqlite::params![path],
            |r| r.get(0),
        )?)
    }

    pub fn remove_scan_location(db: &AppDb, id: i64) -> Result<()> {
        let conn = db.conn();
        conn.execute(
            "DELETE FROM scan_locations WHERE id=?1",
            rusqlite::params![id],
        )?;
        Ok(())
    }

    pub fn list_scan_locations(db: &AppDb) -> Result<Vec<ScanLocation>> {
        let conn = db.conn();
        let mut stmt = conn.prepare("SELECT id,path,name FROM scan_locations ORDER BY path")?;
        let rows = stmt.query_map([], |r| {
            Ok(ScanLocation {
                id: r.get(0)?,
                path: r.get(1)?,
                name: r.get(2).unwrap_or_default(),
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    /// Idempotent upsert of a discovered project root. Returns its id.
    pub fn upsert_project(
        db: &AppDb,
        path: &str,
        name: &str,
        relative_path: Option<&str>,
        scan_location_id: Option<i64>,
        size_bytes: i64,
        last_modified: i64,
    ) -> Result<i64> {
        let conn = db.conn();
        conn.execute(
            "INSERT INTO projects(path,name,relative_path,scan_location_id,size_bytes,last_modified,first_seen,last_scanned)
             VALUES(?1,?2,?3,?4,?5,?6,strftime('%s','now'),strftime('%s','now'))
             ON CONFLICT(path) DO UPDATE SET
               name=excluded.name,
               size_bytes=excluded.size_bytes,
               last_modified=excluded.last_modified,
               last_scanned=strftime('%s','now')",
            rusqlite::params![path, name, relative_path, scan_location_id, size_bytes, last_modified],
        )?;
        Ok(conn.query_row(
            "SELECT id FROM projects WHERE path=?1",
            rusqlite::params![path],
            |r| r.get(0),
        )?)
    }

    /// Empties the projects table (cascades to techs/git/recents). Called at the
    /// start of every full scan so removed/vanished projects don't linger and
    /// noise from a scope change is dropped.
    pub fn clear_projects(db: &AppDb) -> Result<()> {
        let conn = db.conn();
        conn.execute("DELETE FROM projects", [])?;
        Ok(())
    }

    pub fn set_techs(db: &AppDb, project_id: i64, techs: &[Tech]) -> Result<()> {
        let conn = db.conn();
        conn.execute(
            "DELETE FROM project_techs WHERE project_id=?1",
            rusqlite::params![project_id],
        )?;
        for t in techs {
            let kind = match t.kind {
                TechKind::Language => "language",
                TechKind::Framework => "framework",
                TechKind::Tool => "tool",
                TechKind::Runtime => "runtime",
            };
            conn.execute(
                "INSERT OR IGNORE INTO project_techs(project_id,tech,kind) VALUES(?1,?2,?3)",
                rusqlite::params![project_id, t.name, kind],
            )?;
        }
        Ok(())
    }

    pub fn set_git(db: &AppDb, project_id: i64, git: &GitInfo) -> Result<()> {
        let conn = db.conn();
        conn.execute(
            "INSERT INTO git_metadata(project_id,is_git,branch,remote_url,repo_name,
                 staged_count,modified_count,untracked_count,last_commit_message,
                 last_commit_date,latest_short_hash,refreshed_at)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,strftime('%s','now'))
             ON CONFLICT(project_id) DO UPDATE SET
               is_git=excluded.is_git, branch=excluded.branch, remote_url=excluded.remote_url,
               repo_name=excluded.repo_name, staged_count=excluded.staged_count,
               modified_count=excluded.modified_count, untracked_count=excluded.untracked_count,
               last_commit_message=excluded.last_commit_message, last_commit_date=excluded.last_commit_date,
               latest_short_hash=excluded.latest_short_hash, refreshed_at=strftime('%s','now')",
            rusqlite::params![
                project_id, git.is_git, git.branch, git.remote_url, git.repo_name,
                git.staged_count, git.modified_count, git.untracked_count,
                git.last_commit_message, git.last_commit_date, git.latest_short_hash
            ],
        )?;
        Ok(())
    }

    /// Ordered by favorites first, then by last_modified desc.
    pub fn list_projects(db: &AppDb, limit: Option<u64>) -> Result<Vec<Project>> {
        let conn = db.conn();
        let mut stmt = conn.prepare(
            "SELECT p.id,p.path,p.name,p.relative_path,p.size_bytes,p.last_modified,p.is_favorite
             FROM projects p
             ORDER BY p.is_favorite DESC, p.last_modified DESC",
        )?;
        let mut rows = stmt.query([])?;
        let mut out = Vec::new();
        let mut count = 0u64;
        while let Some(r) = rows.next()? {
            if let Some(l) = limit {
                if count >= l {
                    break;
                }
            }
            let id: i64 = r.get(0)?;
            let path: String = r.get(1)?;
            let name: String = r.get(2)?;
            let rel: Option<String> = r.get(3)?;
            let size: i64 = r.get(4)?;
            let mtime: i64 = r.get(5)?;
            let fav: i64 = r.get(6)?;
            let techs = Self::techs_for(&conn, id)?;
            let git = Self::git_for(&conn, id)?;
            out.push(Project {
                id,
                path,
                name,
                relative_path: rel,
                size_bytes: size,
                last_modified: mtime,
                is_favorite: fav != 0,
                techs,
                git,
            });
            count += 1;
        }
        Ok(out)
    }

    fn techs_for(conn: &rusqlite::Connection, project_id: i64) -> Result<Vec<Tech>> {
        let mut stmt =
            conn.prepare("SELECT tech,kind FROM project_techs WHERE project_id=?1 ORDER BY tech")?;
        let rows = stmt.query_map(rusqlite::params![project_id], |r| {
            let kind_str: String = r.get(1)?;
            let kind = match kind_str.as_str() {
                "language" => TechKind::Language,
                "framework" => TechKind::Framework,
                "tool" => TechKind::Tool,
                _ => TechKind::Runtime,
            };
            Ok(Tech {
                name: r.get(0)?,
                kind,
            })
        })?;
        let mut v = Vec::new();
        for row in rows {
            v.push(row?);
        }
        Ok(v)
    }

    fn git_for(conn: &rusqlite::Connection, project_id: i64) -> Result<Option<GitInfo>> {
        let mut stmt = conn.prepare(
            "SELECT is_git,branch,remote_url,repo_name,staged_count,modified_count,
                    untracked_count,last_commit_message,last_commit_date,latest_short_hash
             FROM git_metadata WHERE project_id=?1",
        )?;
        let mut rows = stmt.query(rusqlite::params![project_id])?;
        match rows.next()? {
            Some(r) => Ok(Some(GitInfo {
                is_git: r.get::<_, i64>(0)? != 0,
                branch: r.get(1)?,
                remote_url: r.get(2)?,
                repo_name: r.get(3)?,
                staged_count: r.get::<_, i64>(4)? as u32,
                modified_count: r.get::<_, i64>(5)? as u32,
                untracked_count: r.get::<_, i64>(6)? as u32,
                last_commit_message: r.get(7)?,
                last_commit_date: r.get(8)?,
                latest_short_hash: r.get(9)?,
            })),
            None => Ok(None),
        }
    }

    pub fn get_project(db: &AppDb, id: i64) -> Result<Option<Project>> {
        let conn = db.conn();
        let mut stmt = conn.prepare(
            "SELECT id,path,name,relative_path,size_bytes,last_modified,is_favorite
             FROM projects WHERE id=?1",
        )?;
        let mut rows = stmt.query(rusqlite::params![id])?;
        if let Some(r) = rows.next()? {
            let pid: i64 = r.get(0)?;
            Ok(Some(Project {
                id: pid,
                path: r.get(1)?,
                name: r.get(2)?,
                relative_path: r.get(3)?,
                size_bytes: r.get(4)?,
                last_modified: r.get(5)?,
                is_favorite: r.get::<_, i64>(6)? != 0,
                techs: Self::techs_for(&conn, pid)?,
                git: Self::git_for(&conn, pid)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn set_favorite(db: &AppDb, id: i64, fav: bool) -> Result<()> {
        let conn = db.conn();
        conn.execute(
            "UPDATE projects SET is_favorite=?2 WHERE id=?1",
            rusqlite::params![id, fav as i64],
        )?;
        Ok(())
    }

    pub fn touch_recent(db: &AppDb, id: i64) -> Result<()> {
        let conn = db.conn();
        conn.execute(
            "INSERT INTO recent_projects(project_id,opened_at) VALUES(?1,strftime('%s','now'))
             ON CONFLICT(project_id) DO UPDATE SET opened_at=excluded.opened_at",
            rusqlite::params![id],
        )?;
        Ok(())
    }

    pub fn list_recent(db: &AppDb, limit: u64) -> Result<Vec<Project>> {
        let conn = db.conn();
        let mut stmt = conn
            .prepare("SELECT project_id FROM recent_projects ORDER BY opened_at DESC LIMIT ?1")?;
        let ids: Vec<i64> = stmt
            .query_map(rusqlite::params![limit], |r| r.get(0))?
            .collect::<std::result::Result<_, _>>()?;
        let mut out = Vec::new();
        for id in ids {
            if let Some(p) = Self::get_project(db, id)? {
                out.push(p);
            }
        }
        Ok(out)
    }
}
