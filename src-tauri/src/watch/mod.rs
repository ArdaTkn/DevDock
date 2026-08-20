use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

pub struct ProjectWatcher {
    _watcher: RecommendedWatcher,
}

impl ProjectWatcher {
    pub fn new(app: AppHandle, scan_paths: Vec<PathBuf>) -> Result<Self, crate::error::Error> {
        let (tx, rx) = channel::<PathBuf>();

        // Spawn background debouncer thread
        let app_handle = app.clone();
        thread::spawn(move || {
            debounce_loop(rx, app_handle);
        });

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                if is_relevant_event(&event) {
                    for path in event.paths {
                        let _ = tx.send(path);
                    }
                }
            }
        })
        .map_err(|e| crate::error::Error::Other(format!("Failed to create watcher: {e}")))?;

        let home_dir = std::env::home_dir();

        for p in scan_paths {
            if !p.is_dir() {
                continue;
            }

            // Safety guard: Never recursively watch the root home directory ~ directly,
            // as watching ~ recursively exhausts OS FSEvents/file descriptors and hangs the OS.
            let is_home = home_dir.as_ref().map(|h| h == &p).unwrap_or(false);
            let mode = if is_home {
                RecursiveMode::NonRecursive
            } else {
                RecursiveMode::Recursive
            };

            if let Err(err) = watcher.watch(&p, mode) {
                tracing::warn!("Failed to watch directory {}: {err}", p.display());
            }
        }

        Ok(Self { _watcher: watcher })
    }
}

fn is_relevant_event(event: &Event) -> bool {
    // Only care about data modifications, file creation, or deletion
    match event.kind {
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {}
        _ => return false,
    }

    for p in &event.paths {
        let s = p.to_string_lossy();
        // Ignore noise in build/node_modules/git objects
        if s.contains("node_modules")
            || s.contains("target")
            || s.contains(".git/objects")
            || s.contains(".git/logs")
            || s.contains("dist")
            || s.contains(".next")
            || s.contains("build")
        {
            continue;
        }

        // Relevant markers: HEAD, index, package.json, Cargo.toml, pubspec.yaml, etc.
        if s.ends_with(".git/HEAD")
            || s.ends_with(".git/index")
            || s.ends_with("package.json")
            || s.ends_with("Cargo.toml")
            || s.ends_with("pubspec.yaml")
            || s.ends_with("pyproject.toml")
            || s.ends_with("requirements.txt")
            || s.ends_with("go.mod")
            || s.ends_with("pom.xml")
        {
            return true;
        }
    }

    false
}

fn debounce_loop(rx: Receiver<PathBuf>, app: AppHandle) {
    let mut last_emit = Instant::now() - Duration::from_secs(10);
    while let Ok(_path) = rx.recv() {
        // Drain any pending paths in channel
        while rx.try_recv().is_ok() {}

        // 500ms debounce
        if last_emit.elapsed() >= Duration::from_millis(500) {
            last_emit = Instant::now();
            let _ = app.emit("fs-change", ());
        }
    }
}
