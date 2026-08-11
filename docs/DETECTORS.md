# Project Detection

DevDock's job is to recognise a software project from a small set of marker
files — without reading source code. This is done by a registry of independent
**detectors**.

## The `ProjectDetector` trait

```rust
pub trait ProjectDetector: Send + Sync {
    fn id(&self) -> &'static str;      // "node", "python", "git", …
    fn priority(&self) -> u8;          // lower runs first
    fn detect(&self, dir: &Path, ctx: &DetectContext) -> Option<Vec<Tech>>;
}
```

- `detect` returns `None` when the directory does **not** match this detector,
  or `Some(techs)` when it does.
- A directory is treated as a project if the **union** of all detectors'
  matches is non-empty (`DetectorRegistry::detect_all`).
- Detectors inspect **only** the marker files they need. They never walk or read
  the project's source.

## Registered detectors

| Detector       | Marker(s)                                         | Emits tech tags          |
|----------------|---------------------------------------------------|--------------------------|
| GitDetector    | `.git/`                                           | Git                      |
| NodeDetector   | `package.json` (reads `scripts` + deps)           | Node.js, npm/pnpm/yarn, Vite, React, Next.js, Tauri, Supabase |
| PythonDetector | `pyproject.toml`, `requirements.txt`, `Pipfile`, `setup.py`, `setup.cfg`, `poetry.lock` | Python, Poetry |
| RustDetector   | `Cargo.toml`                                      | Rust, Cargo              |
| GoDetector     | `go.mod`                                          | Go                       |
| FlutterDetector| `pubspec.yaml`                                    | Flutter, Dart            |
| DotNetDetector | `*.sln`, `*.csproj`, `*.fsproj`                   | .NET                     |
| DockerDetector | `Dockerfile`, `compose.yml`, `docker-compose.yml` | Docker                   |
| UnityDetector  | `ProjectSettings/ProjectVersion.txt`, `Assets`+`Packages` | Unity             |
| JavaDetector   | `pom.xml`, `build.gradle(.kts)`, `settings.gradle`| Java, Maven/Gradle       |

## Detection rules of thumb

- **Cheap first.** Marker-file existence checks are near-free; reading
  `package.json` for scripts/deps happens only for Node and is best-effort.
- **Never fail on parse.** If a marker exists but can't be read/parsed, the
  detector still counts the tech — the project is what matters, not a perfect
  package listing.
- **Priority.** Detectors that *define* a project on their own (like Git) have
  the lowest numeric priority so they run first and always contribute.
- **Extensibility.** A future plugin system can supply additional detectors via
  the same trait. The architecture already supports it.

## Adding a detector

1. `src-tauri/src/discovery/detectors/<name>.rs` implementing the trait.
2. Register it:
   - `discovery/detectors/mod.rs` (re-export),
   - `detector.rs::DetectorRegistry::default_registry()`.
3. Add a fixture under `src-tauri/tests/fixtures/<name>-project/`.
4. Add a test in `src-tauri/tests/discovery_test.rs` (assert tech tags).
5. Update the table above and `README.md`.

## Ignored / never-descended directories

The scanner skips: `.git`, `node_modules`, `target`, `build`, `dist`,
`.dart_tool`, `.gradle`, `__pycache__`, `.venv`/`venv`, `Pods`, `.idea`,
`.vscode`, `.next`, `coverage`, plus symlinked dirs that escape the scan root.
