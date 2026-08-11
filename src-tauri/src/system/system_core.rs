use crate::error::{Error, Result};
use std::path::Path;
use std::process::Command;

/// Detected editor applications, from the strongest preference downward.
/// Detection is best-effort and platform-aware.
pub struct SystemActions;

const EDITORS: &[(&str, &[&str])] = &[
    // (app bundle/name, candidate binaries on PATH)
    ("code", &["code"]),     // VS Code
    ("cursor", &["cursor"]), // Cursor
    ("zed", &["zed"]),       // Zed
    ("windsurf", &["windsurf"]),
    ("jb-toolbox", &["jetbrains-toolbox"]),
];

#[cfg(target_os = "macos")]
fn open_cmd() -> Command {
    Command::new("open")
}

#[cfg(not(target_os = "macos"))]
fn open_cmd() -> Command {
    // `xdg-open` on Linux, `start` handled in a platform branch.
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
    }
    #[cfg(target_os = "windows")]
    {
        unreachable!("windows open handled separately");
    }
}

impl SystemActions {
    /// Opens the project folder in the system file manager.
    pub fn open_folder(path: &Path) -> Result<()> {
        if !path.is_dir() {
            return Err(Error::OpenFolder(format!(
                "The folder does not exist or is not accessible: {}",
                path.display()
            )));
        }
        Self::launch_opener(path)
    }

    #[cfg(target_os = "macos")]
    fn launch_opener(path: &Path) -> Result<()> {
        open_cmd()
            .arg(path)
            .status()
            .map_err(|e| Error::OpenFolder(e.to_string()))?;
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn launch_opener(path: &Path) -> Result<()> {
        open_cmd()
            .arg(path)
            .status()
            .map_err(|e| Error::OpenFolder(e.to_string()))?;
        Ok(())
    }

    /// Opens an editor binary if any supported one is installed.
    pub fn open_editor(path: &Path) -> Result<()> {
        let bin = Self::detect_editor().ok_or(Error::EditorNotFound)?;
        Command::new(bin)
            .arg(path)
            .status()
            .map_err(|_| Error::EditorNotFound)?;
        Ok(())
    }

    /// Returns the preferred detected editor binary name, or None.
    pub fn detect_editor() -> Option<&'static str> {
        for (_, bins) in EDITORS {
            for b in *bins {
                if Self::binary_exists(b) {
                    return Some(b);
                }
            }
        }
        None
    }

    /// Opens a new terminal window in `path`.
    /// Uses Terminal.app on macOS; falls back gracefully otherwise.
    pub fn open_terminal(path: &Path) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            let script = format!(
                "tell application \"Terminal\" to do script \"cd '{}'\"",
                path.display().to_string().replace('\'', "'\\''")
            );
            Command::new("osascript")
                .arg("-e")
                .arg(&script)
                .status()
                .map_err(|_| Error::TerminalNotFound)?;
            return Ok(());
        }
        #[cfg(target_os = "linux")]
        {
            let terminal = ["gnome-terminal", "konsole", "x-terminal-emulator"]
                .iter()
                .find(|t| Self::binary_exists(t))
                .ok_or(Error::TerminalNotFound)?;
            Command::new(terminal).arg(format!("--working-directory={}", path.display()));
            return Ok(());
        }
        #[cfg(target_os = "windows")]
        {
            Command::new("cmd")
                .arg("/c")
                .arg("start")
                .arg("cmd")
                .arg("/K")
                .arg(format!("cd /d \"{}\"", path.display().to_string()))
                .status()
                .map_err(|_| Error::TerminalNotFound)?;
            return Ok(());
        }
        #[allow(unreachable_code)]
        Err(Error::TerminalNotFound)
    }

    fn binary_exists(bin: &str) -> bool {
        let probe = Command::new("which").arg(bin).output();
        match probe {
            Ok(o) => o.status.success(),
            Err(_) => false,
        }
    }
}
