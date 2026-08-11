use crate::error::{Error, Result};
use crate::models::GitInfo;
use std::path::Path;
use std::process::Command;

/// Thin, read-only wrapper around the `git` CLI. Always runs `git -C <dir> …`
/// with explicit args (no shell), so there is no injection surface.
pub struct GitCommand;

const GIT_BIN: &str = "git";

/// A parsed, untracked-file count and dirty state.
#[derive(Debug, Clone, Default)]
pub struct GitStatus {
    pub is_git: bool,
    pub branch: String,
    pub staged: u32,
    pub modified: u32,
    pub untracked: u32,
}

impl GitStatus {
    pub fn clean(&self) -> bool {
        self.staged == 0 && self.modified == 0 && self.untracked == 0
    }
}

impl GitCommand {
    /// Returns true if the `git` binary is available on PATH.
    pub fn available() -> bool {
        Command::new(GIT_BIN)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn run(dir: &Path, args: &[&str]) -> Result<String> {
        let out = Command::new(GIT_BIN)
            .arg("-C")
            .arg(dir)
            .args(args)
            .output()
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => Error::GitUnavailable,
                _ => Error::Git {
                    dir: dir.display().to_string(),
                    detail: e.to_string(),
                },
            })?;
        if out.status.success() {
            Ok(String::from_utf8_lossy(&out.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            Err(Error::Git {
                dir: dir.display().to_string(),
                detail: stderr.trim().to_string(),
            })
        }
    }

    /// Parses `git status --porcelain -b`. Returns None if not a git repo.
    fn parse_status(dir: &Path) -> Result<Option<GitStatus>> {
        let raw = match Self::run(dir, &["status", "--porcelain", "-b"]) {
            Ok(r) => r,
            Err(Error::Git { .. }) => return Ok(None), // not a repo
            Err(e) => return Err(e),
        };
        let mut status = GitStatus::default();
        let mut head = None;
        for line in raw.lines() {
            if let Some(branch_part) = line.strip_prefix("## ") {
                // "## main...origin/main [ahead 1]" → "main"
                let branch = branch_part
                    .split("...")
                    .next()
                    .unwrap_or("HEAD")
                    .to_string();
                head = Some(branch);
                continue;
            }
            let code: Vec<char> = line.chars().take(2).collect();
            if code.len() < 2 {
                continue;
            }
            let (x, y) = (code[0], code[1]);
            if x != ' ' && x != '?' {
                status.staged += 1;
            }
            if y != ' ' {
                // '??' untracked handled below; 'M'/' ' etc.
                if !(x == '?' && y == '?') {
                    status.modified += 1;
                }
            }
        }
        status.untracked = raw.lines().filter(|l| l.starts_with("??")).count() as u32;
        status.is_git = true;
        status.branch = head.unwrap_or_else(|| "HEAD".into());
        Ok(Some(status))
    }

    fn last_commit(dir: &Path) -> Result<(Option<String>, Option<i64>, Option<String>)> {
        let raw = match Self::run(dir, &["log", "-1", "--format=%h%x09%s%x09%ct"]) {
            Ok(r) => r,
            Err(_) => return Ok((None, None, None)), // empty repo
        };
        let line = raw.trim();
        let mut parts = line.splitn(3, '\t');
        let hash = parts.next().map(|s| s.to_string());
        let msg = parts.next().map(|s| s.to_string());
        let date = parts.next().and_then(|s| s.trim().parse::<i64>().ok());
        Ok((msg, date, hash))
    }

    fn remote_url(dir: &Path) -> Option<String> {
        Self::run(dir, &["config", "--get", "remote.origin.url"])
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    fn repo_name(remote: &Option<String>) -> Option<String> {
        remote.as_ref().and_then(|u| {
            let trimmed = u.trim_end_matches(".git");
            trimmed.rsplit(['/', ':']).next().map(|s| s.to_string())
        })
    }

    /// Computes full read-only Git metadata for a directory.
    /// Never panics; broken/missing repos return `Ok(None)` or a typed error.
    pub fn inspect(dir: &Path) -> Result<Option<GitInfo>> {
        if !Self::available() {
            return Err(Error::GitUnavailable);
        }
        match Self::parse_status(dir)? {
            None => Ok(None),
            Some(status) => {
                let (msg, date, hash) = Self::last_commit(dir)?;
                let remote = Self::remote_url(dir);
                Ok(Some(GitInfo {
                    is_git: true,
                    branch: if status.branch == "HEAD" {
                        None
                    } else {
                        Some(status.branch.clone())
                    },
                    remote_url: remote.clone(),
                    repo_name: Self::repo_name(&remote),
                    staged_count: status.staged,
                    modified_count: status.modified,
                    untracked_count: status.untracked,
                    last_commit_message: msg,
                    last_commit_date: date,
                    latest_short_hash: hash,
                }))
            }
        }
    }
}
