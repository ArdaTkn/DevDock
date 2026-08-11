use crate::error::{Error, Result};
use std::path::Path;
#[cfg(target_os = "macos")]
use std::path::PathBuf;
use std::process::Command;

/// Cross-platform "open this project in an editor / terminal / file manager".
///
/// On macOS, editors and terminals are detected from their `.app` bundles in
/// `/Applications` (not just PATH CLIs, which are often uninstalled) and opened
/// with `open -a`. PATH CLIs are preferred when present.
pub struct SystemActions;

/// Candidate editors as (display name, PATH CLI binary).
const EDITORS: &[(&str, &str)] = &[
    ("Cursor", "cursor"),
    ("VS Code", "code"),
    ("Zed", "zed"),
    ("OpenCode", "opencode"),
    ("Windsurf", "windsurf"),
];

/// macOS-only candidate `.app` bundles as (bundle name, display name).
#[cfg(target_os = "macos")]
const MACOS_EDITOR_BUNDLES: &[(&str, &str)] = &[
    ("Cursor", "Cursor"),
    ("Visual Studio Code", "VS Code"),
    ("OpenCode", "OpenCode"),
    ("Zed", "Zed"),
    ("Windsurf", "Windsurf"),
    ("IntelliJ IDEA", "IntelliJ IDEA"),
    ("PyCharm", "PyCharm"),
];

/// A detected editor.
pub struct Editor {
    pub name: &'static str,
    /// PATH CLI launcher (used on all platforms and as fallback).
    pub bin: &'static str,
    /// macOS-only `.app` bundle name to open via `open -a` (preferred on macOS).
    #[cfg(target_os = "macos")]
    pub bundle: Option<&'static str>,
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
        c.args(["/c", "explorer"]);
        c.arg(path);
        c
    }

    /// Opens the preferred detected editor in the project directory.
    pub fn open_editor(path: &Path) -> Result<()> {
        let editor = Self::detect_editor().ok_or(Error::EditorNotFound)?;

        #[cfg(target_os = "macos")]
        if let Some(bundle) = editor.bundle {
            let status = Command::new("open").args(["-a", bundle]).arg(path).status();
            return status.map(|_| ()).map_err(|_| Error::EditorNotFound);
        }

        Command::new(editor.bin)
            .arg(path)
            .status()
            .map(|_| ())
            .map_err(|_| Error::EditorNotFound)
    }

    /// Returns the preferred detected editor, or None.
    pub fn detect_editor() -> Option<Editor> {
        // Prefer a PATH CLI when it's actually installed.
        for (name, bin) in EDITORS {
            if Self::binary_exists(bin) {
                return Some(Editor {
                    name,
                    bin,
                    #[cfg(target_os = "macos")]
                    bundle: None,
                });
            }
        }

        // On macOS fall back to `.app` bundles in /Applications.
        #[cfg(target_os = "macos")]
        for (bundle, name) in MACOS_EDITOR_BUNDLES {
            if Self::macos_bundle_exists(bundle) {
                return Some(Editor {
                    name,
                    bin: "",
                    bundle: Some(bundle),
                });
            }
        }

        None
    }

    #[cfg(target_os = "macos")]
    fn macos_bundle_exists(app: &str) -> bool {
        let candidates = [
            PathBuf::from("/Applications").join(format!("{app}.app")),
            PathBuf::from("/System/Applications").join(format!("{app}.app")),
        ];
        candidates.iter().any(|p| p.is_dir())
    }

    /// Opens a new terminal window in `path`, preferring iTerm on macOS.
    pub fn open_terminal(path: &Path) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            Self::open_terminal_macos(path)
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

    #[cfg(target_os = "macos")]
    fn open_terminal_macos(path: &Path) -> Result<()> {
        // bash-safe `cd '<dir>'` (single-quote escapes included).
        let cd_cmd = format!("cd '{}'", path.display().to_string().replace('\'', "'\\''"));

        // Preferred: iTerm2.
        if Self::macos_bundle_exists("iTerm") {
            let embed = cd_cmd.replace('\\', "\\\\").replace('"', "\\\"");
            let script = format!(
                "tell application \"iTerm\"\n\
                 \tactivate\n\
                 \ttry\n\
                 \t\tcreate window with default profile\n\
                 \tend try\n\
                 \ttell current session of current window\n\
                 \t\twrite text \"{embed}\"\n\
                 \tend tell\n\
                 end tell"
            );
            let ok = Command::new("osascript")
                .arg("-e")
                .arg(&script)
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if ok {
                return Ok(());
            }
        }

        // Fallback: Terminal.app.
        let embed = cd_cmd.replace('\\', "\\\\").replace('"', "\\\"");
        let script = format!("tell application \"Terminal\" to do script \"{embed}\"");
        Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .status()
            .map(|_| ())
            .map_err(|_| Error::TerminalNotFound)
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
