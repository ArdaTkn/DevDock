use serde::Serialize;
use thiserror::Error;

/// DevDock's single error type. Every variant maps to a human-readable
/// message surfaced to the UI; one failing project never crashes the app.
#[derive(Error, Debug)]
pub enum Error {
    #[error("git is not installed or could not be run")]
    GitUnavailable,

    #[error("could not open the terminal. The configured terminal application could not be found. Check your terminal settings.")]
    TerminalNotFound,

    #[error("could not open the editor. The detected editor is not available on this system.")]
    EditorNotFound,

    #[error("could not open the project folder: {0}")]
    OpenFolder(String),

    #[error("filesystem error: {0}")]
    Io(#[from] std::io::Error),

    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("failed to run `git` in {dir}: {detail}")]
    Git { dir: String, detail: String },

    #[error("git repository in {0} is corrupted or inaccessible")]
    CorruptRepo(String),

    #[error("invalid path: {0}")]
    InvalidPath(String),

    #[error("{0}")]
    Other(String),
}

impl Error {
    /// Serialisable shape the UI consumes.
    pub fn to_dto(&self) -> ErrorDto {
        let msg = self.to_string();
        let hint = match self {
            Error::GitUnavailable => Some("Install Git and restart DevDock to enable Git features.".into()),
            Error::TerminalNotFound => Some("Check &#39;Terminal&#39; in Settings.".into()),
            Error::EditorNotFound => Some("Install one of the supported editors (VS Code, Cursor, Zed, JetBrains) or set one in Settings.".into()),
            Error::CorruptRepo(_) => Some("This repository may be damaged. Try re-cloning it, or remove the .git folder to treat it as a plain folder.".into()),
            _ => None,
        };
        ErrorDto { message: msg, hint }
    }
}

/// Machine-readable error payload sent over IPC.
#[derive(Debug, Clone, Serialize)]
pub struct ErrorDto {
    pub message: String,
    pub hint: Option<String>,
}

impl From<Error> for ErrorDto {
    fn from(e: Error) -> Self {
        e.to_dto()
    }
}

pub type Result<T> = std::result::Result<T, Error>;
