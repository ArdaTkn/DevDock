use crate::error::{Error, Result};
use std::path::Path;
use std::process::Command;

/// Detected editor applications, from the strongest preference downward.
/// Detection is best-effort and platform-aware.
pub struct SystemActions;

const EDITORS: &[(&str, &[&str])] = &[
    // (editor name, candidate binaries on PATH)
    ("VS Code", &["code"]),
    ("Cursor", &["cursor"]),
    ("Zed", &["zed"]),
    ("Windsurf", &["windsurf"]),
    ("JetBrains Toolbox", &["jetbrains-toolbox"]),
];

impl SystemActions {
    /// Opens the project folder in the system file manager.
    pub fn open_folder(path: &Path) -> Result<()> {
        if !path.is_dir() {
            return Err(Error::OpenFolder(format!(
                "The folder does not exist or is not accessible: {}",
                path.display()
            )));
        }
        let status = Self::folder_command(path).status();
        status
            .map(|_| ())
            .map_err(|e| Error::OpenFolder(e.to_string()))
    }

    #[cfg(target_os = "macos")]
    fn folder_command(path: &Path) -> Command {
        let mut c = Command::new("open");
        c.arg(path);
        c
    }

    #[cfg(target_os = "linux")]
    fn folder_command(path: &Path) -> Command {
        let mut c = Command::new("xdg-open");
        c.arg(path);
        c
    }

    #[cfg(target_os = "windows")]
    fn folder_command(path: &Path) -> Command {
        let mut c = Command::new("cmd");
        c.args(["/c", "explorer", &path.to_string_lossy()]);
        c
    }

    /// Opens the preferred detected editor in the project directory.
    pub fn open_editor(path: &Path) -> Result<()> {
        let editor = Self::detect_editor().ok_or(Error::EditorNotFound)?;
        let status = Command::new(editor.bin).arg(path).status();
        status.map(|_| ()).map_err(|_| Error::EditorNotFound)
    }

    /// Returns the preferred detected editor, or None.
    pub fn detect_editor() -> Option<Editor> {
        for (name, bins) in EDITORS {
            for bin in *bins {
                if Self::binary_exists(bin) {
                    return Some(Editor { name, bin });
                }
            }
        }
        None
    }

    /// Opens a new terminal window in `path`.
    pub fn open_terminal(path: &Path) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            let clean = path.display().to_string().replace('\'', "'\\''");
            let script = format!(
                "tell application \"Terminal\" to do script \"cd '{}'\"",
                clean
            );
            Command::new("osascript")
                .arg("-e")
                .arg(&script)
                .status()
                .map(|_| ())
                .map_err(|_| Error::TerminalNotFound)
        }

        #[cfg(target_os = "linux")]
        {
            let terminal = ["gnome-terminal", "konsole", "x-terminal-emulator"]
                .iter()
                .find(|t| Self::binary_exists(t))
                .ok_or(Error::TerminalNotFound)?;
            let dir = format!("--working-directory={}", path.display());
            Command::new(terminal)
                .arg(&dir)
                .status()
                .map(|_| ())
                .map_err(|_| Error::TerminalNotFound)
        }

        #[cfg(target_os = "windows")]
        {
            Command::new("cmd")
                .args(["/c", "start", "cmd", "/K"])
                .arg(format!("cd /d \"{}\"", path.display()))
                .status()
                .map(|_| ())
                .map_err(|_| Error::TerminalNotFound)
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn binary_exists(bin: &str) -> bool {
        Command::new("which")
            .arg(bin)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    #[cfg(target_os = "windows")]
    fn binary_exists(bin: &str) -> bool {
        Command::new("where")
            .arg(bin)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

/// A detected editor with its display name and launcher binary.
#[derive(Clone, Copy)]
pub struct Editor {
    pub name: &'static str,
    pub bin: &'static str,
}
