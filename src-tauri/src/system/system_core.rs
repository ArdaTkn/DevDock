use crate::error::{Error, Result};
use std::path::Path;
use std::process::{Child, Command};

/// Launches editors / terminals / file managers for a project.
///
/// All launchers use `.spawn()` (fire-and-forget) rather than `.status()`:
/// waiting on a GUI app's process can hang the command thread and make the
/// app feel frozen/crashed. Returns as soon as the child is forked.
pub struct SystemActions;

/// Recognised editors on macOS.
#[cfg(target_os = "macos")]
const EDITORS: &[&str] = &[
    "Visual Studio Code",
    "Cursor",
    "Zed",
    "Windsurf",
    "Sublime Text",
    "WebStorm",
    "Android Studio",
    "Xcode",
];

/// Recognised terminals on macOS.
#[cfg(target_os = "macos")]
const TERMINALS: &[&str] = &["iTerm", "Warp", "Ghostty", "Hyper", "Alacritty", "Kitty"];

impl SystemActions {
    fn spawn(cmd: &mut Command) -> Result<Child> {
        cmd.spawn()
            .map_err(|e| Error::Other(format!("failed to launch: {e}")))
    }

    #[cfg(target_os = "macos")]
    fn bundle_exists(app: &str) -> bool {
        std::path::Path::new("/Applications")
            .join(format!("{app}.app"))
            .is_dir()
            || std::path::Path::new("/System/Applications")
                .join(format!("{app}.app"))
                .is_dir()
            || std::env::var("HOME")
                .map(|h| std::path::Path::new(&h).join("Applications").join(format!("{app}.app")).is_dir())
                .unwrap_or(false)
    }

    /// Opens the project folder in the system file manager.
    pub fn open_folder(path: &Path) -> Result<()> {
        if !path.is_dir() {
            return Err(Error::OpenFolder(format!(
                "The folder does not exist or is not accessible: {}",
                path.display()
            )));
        }
        let mut cmd = Self::folder_command(path);
        Self::spawn(&mut cmd)
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

    /// Opens the project in preferred or detected editor.
    pub fn open_editor(path: &Path, preferred: Option<&str>) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            let app_name = match preferred {
                Some(p) if Self::bundle_exists(p) => Some(p),
                _ => EDITORS.iter().copied().find(|e| Self::bundle_exists(e)),
            };

            if let Some(editor) = app_name {
                let mut cmd = Command::new("open");
                cmd.args(["-a", editor]).arg(path);
                Self::spawn(&mut cmd)
                    .map(|_| ())
                    .map_err(|_| Error::EditorNotFound)
            } else {
                // Fallback to default system handler for folder
                let mut cmd = Command::new("open");
                cmd.arg(path);
                Self::spawn(&mut cmd)
                    .map(|_| ())
                    .map_err(|_| Error::EditorNotFound)
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = preferred;
            let mut cmd = Command::new("code");
            cmd.arg(path);
            Self::spawn(&mut cmd)
                .map(|_| ())
                .map_err(|_| Error::EditorNotFound)
        }
    }

    /// Returns the display names of detected installed editors.
    pub fn detect_editors() -> Vec<String> {
        #[cfg(target_os = "macos")]
        {
            EDITORS
                .iter()
                .filter(|n| Self::bundle_exists(n))
                .map(|s| s.to_string())
                .collect()
        }
        #[cfg(not(target_os = "macos"))]
        {
            if Self::binary_exists("code") {
                vec!["VS Code".to_string()]
            } else {
                vec![]
            }
        }
    }

    /// Returns the terminal bundles/CLIs that are actually installed.
    pub fn detect_terminals() -> Vec<String> {
        #[cfg(target_os = "macos")]
        {
            TERMINALS
                .iter()
                .filter(|n| Self::bundle_exists(n))
                .map(|s| s.to_string())
                .collect()
        }
        #[cfg(target_os = "linux")]
        {
            ["gnome-terminal", "konsole", "x-terminal-emulator"]
                .iter()
                .filter(|t| Self::binary_exists(t))
                .map(|s| s.to_string())
                .collect()
        }
        #[cfg(target_os = "windows")]
        {
            ["Windows Terminal", "cmd"]
                .iter()
                .map(|s| s.to_string())
                .collect()
        }
    }

    /// Opens a terminal in `path`. `preferred` is an optional user-chosen
    /// terminal (ignored if not installed); otherwise the installed terminal is
    /// detected, falling back to the OS default (Terminal.app on macOS).
    pub fn open_terminal(path: &Path, preferred: Option<&str>) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            Self::open_terminal_macos(path, preferred)
        }
        #[cfg(target_os = "linux")]
        {
            let _ = preferred; // per-terminal choice is macOS-only for now
            let terminal = ["gnome-terminal", "konsole", "x-terminal-emulator"]
                .iter()
                .find(|t| Self::binary_exists(t))
                .ok_or(Error::TerminalNotFound)?;
            let mut cmd = Command::new(terminal);
            cmd.arg(format!("--working-directory={}", path.display()));
            Self::spawn(&mut cmd)
                .map(|_| ())
                .map_err(|_| Error::TerminalNotFound)
        }
        #[cfg(target_os = "windows")]
        {
            let _ = preferred;
            let mut cmd = Command::new("cmd");
            cmd.args(["/c", "start", "cmd", "/K"])
                .arg(format!("cd /d \"{}\"", path.display()));
            Self::spawn(&mut cmd)
                .map(|_| ())
                .map_err(|_| Error::TerminalNotFound)
        }
    }

    #[cfg(target_os = "macos")]
    fn detect_terminal() -> Option<&'static str> {
        TERMINALS
            .iter()
            .copied()
            .find(|name| Self::bundle_exists(name))
    }

    #[cfg(target_os = "macos")]
    fn open_terminal_macos(path: &Path, preferred: Option<&str>) -> Result<()> {
        // bash-safe `cd '<dir>'`.
        let cd_cmd = format!("cd '{}'", path.display().to_string().replace('\'', "'\\''"));
        let embed = cd_cmd.replace('\\', "\\\\").replace('"', "\\\"");

        // Resolve which terminal to use: the user's chosen one if it's actually
        // installed, otherwise whichever is installed, otherwise the OS default.
        let target = match preferred {
            Some(p) if Self::bundle_exists(p) => Some(p),
            _ => Self::detect_terminal(),
        };

        match target {
            // iTerm exposes a rich AppleScript: new window + write the cd command.
            Some("iTerm") => {
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
                let mut cmd = Command::new("osascript");
                cmd.args(["-e", &script]);
                let _ = Self::spawn(&mut cmd);
                Ok(())
            }
            // Any other recognised terminal: launch it pointed at the directory.
            Some(name) => {
                let mut cmd = Command::new("open");
                cmd.args(["-a", name]).arg(path);
                Self::spawn(&mut cmd)
                    .map(|_| ())
                    .map_err(|_| Error::TerminalNotFound)
            }
            // No third-party terminal → the macOS default, Terminal.app.
            None => {
                let script = format!("tell application \"Terminal\" to do script \"{embed}\"");
                let mut cmd = Command::new("osascript");
                cmd.args(["-e", &script]);
                Self::spawn(&mut cmd)
                    .map(|_| ())
                    .map_err(|_| Error::TerminalNotFound)
            }
        }
    }

    #[cfg(target_os = "linux")]
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
