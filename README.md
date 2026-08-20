# DevDock

**Your local development command center.**

DevDock automatically discovers the projects on your computer
and gives you one place to understand, open, and manage them.

> Developers accumulate many coding projects on their machines, but there is no
> single lightweight place that automatically **discovers**, **understands**,
> **monitors**, and **launches** those projects. DevDock is that place — it runs
> locally on your own machine, reads nothing but metadata, and never uploads
> your source code anywhere.

![Status](https://img.shields.io/badge/status-alpha-6ee7b7)
![Platform](https://img.shields.io/badge/macOS-Windows-Linux-lightgrey)

---

## The problem

You have Git repositories, Node projects, Python scripts, Flutter apps, Rust
crates, Docker compose stacks — spread across `~/Projects`, `~/Developer`,
`~/Code`, your Documents folder, sometimes forgotten in a corner of your disk.

Open DevDock and you instantly know:

- what projects exist on your machine and where
- which technology each one uses
- which Git branch each is on, and whether it's clean or dirty
- what's been modified recently
- how to open it in your editor, terminal, or file manager

## The core experience

```
Install DevDock → pick ~/Projects → DevDock scans
→ "We found 42 projects: 18 Node, 7 Python, 5 Flutter…"
→ click a project → see Git + technology + health
→ [Open Editor] [Open Terminal]
→ start working
```

No account. No cloud. No configuration-heavy setup.

---

## Features

**Core Features (v0.1, v0.2, v0.3 & v0.4 active):**

- ✅ **GitHub Integration** — automatically parses GitHub remote URLs (`owner/repo`), providing 1-click browser quick actions for Repository, Issues, and Pull Requests
- ✅ **Project Tags & Categories** — custom tag chips (`open-source`, `client`, `side-project`) stored locally in SQLite
- ✅ **Project Quick Notes & Reminders** — auto-saved markdown notes per project for tracking TODOs and reminders
- ✅ **Configurable Custom Commands** — user-defined custom shell commands (`cargo watch`, `docker compose up`, etc.) executed with 1 click in your preferred terminal
- ✅ **Smart Dev Port & Live Server Scanner** — automatically scans active listening TCP ports (`localhost:3000`, `5173`, `8000`, `5432`, `6379`, etc.), filters out non-dev background noise (Spotify, Discord, system daemons), and renders a responsive glassmorphism grid with live pulsing dots, PID badges, and 1-click browser launchers
- ✅ **Docker Container Inspector** — detects active/recent Docker containers, statuses, images, and port mappings
- ✅ **Deterministic Project Health Audit** — scores project health (0-100), checks dependency presence (`node_modules`, `.venv`), README docs, and Git cleanliness
- ✅ **Custom Project Script Launcher** — detects package/Cargo/Makefile scripts (`npm run dev`, `cargo run`, `make dev`) and runs them in your terminal with 1 click
- ✅ **Scan user-selected directories** (add/remove anytime)
- ✅ **Automatic project detection** — Git, Node, Python, Rust, Go, Flutter, .NET, Docker, Unity, Java (extensible detector architecture)
- ✅ **Read-only Git metadata** — branch, clean/dirty, staged/modified/untracked counts, last commit, remote URL
- ✅ **Command Palette (`⌘K` / `Ctrl+K`)** for fast project search & quick actions
- ✅ **Real-time FileSystem Watcher (`notify` crate)** for live background updates
- ✅ **Recently Opened projects** quick launcher strip
- ✅ **Configurable Code Editor & Terminal preferences** (VS Code, Cursor, Zed, Windsurf, etc.)
- ✅ **Technology badges + search + filter + sort** (recent, name, dirty, path)
- ✅ **Favorites** (pinned to top)
- ✅ **Open in Editor / Open in Terminal / Show in Finder**
- ✅ **SQLite storage** (metadata + paths only — never source code)
- ✅ **Local-first, privacy-first, no telemetry**
- ✅ **9 passing integration tests** (detectors + git + end-to-end scan)

**Roadmap:**

- v0.5+ — opt-in AI assistance (local / OpenAI / Anthropic / Google — never on by default)

---

## Screenshots

*Screenshots will be added when the desktop UI is in its polished state.
The scanner + dashboard are functional now; visual polish is in progress.*

---

## Supported platforms

- macOS (primary dev target — built on macOS)
- Windows (architecture-ready; CI builds)
- Linux (architecture-ready; CI builds)

All platform-specific work is behind small traits (process/port/editor/terminal
detection), so cross-platform behaviour is designed in from day one.

---

## Installation

> Alpha. Build from source below, or grab a prebuilt installer from the
> [Releases](https://github.com/ArdaTkn/DevDock/releases) page once a release is cut
> (CI builds macOS `.dmg`, Windows `.msi`, and Linux `.AppImage`/`.deb`).

### Prerequisites

- **Rust** toolchain (stable) — e.g. via `rustup` or Homebrew
- **Node.js 18+** and npm
- On macOS: Xcode Command Line Tools (`xcode-select --install`)

### Build & run (development)

```bash
git clone https://github.com/ArdaTkn/DevDock.git
cd DevDock
npm install
npm run tauri dev      # runs the desktop app with hot-reload
```

### Run the scanner standalone (no UI)

```bash
cd src-tauri
cargo run --example sample_scan -- ~/Projects ~/Code
```

### Tests & lint

```bash
# Rust
cd src-tauri
cargo test             # unit + integration tests
cargo clippy -- -D warnings

# Frontend
npm run typecheck
npm run build
```

---

## Architecture

```
┌─────────────────────────────────┐
│          React UI               │  Vite + React + TypeScript (renderer)
└──────────────┬──────────────────┘
               │  Tauri IPC (commands + events)
┌──────────────▼──────────────────┐
│           Rust Core             │  tokio async workers
│                                 │
│  discovery/   project scanner   │
│  git/         git metadata      │
│  system/      editor/terminal   │
│  storage/     SQLite            │
└──────────────┬──────────────────┘
        ┌──────▼───────┐
        │    SQLite    │   metadata + paths only (never source code)
        └──────────────┘
```

- **Renderer** never touches the filesystem or spawns processes. It only calls
  typed Tauri commands and receives typed results.
- **Rust core** owns all filesystem/process/system work and returns serde DTOs.
- **SQLite** stores only paths and metadata.

See [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) and
[`docs/PLAN.md`](docs/PLAN.md) for details.

---

## Project detection

A `ProjectDetector` trait + a registry keep detection independent and testable:

```
ProjectDetector
 ├── GitDetector      (.git)
 ├── NodeDetector     (package.json — reads scripts/deps)
 ├── PythonDetector   (pyproject.toml, requirements.txt, Pipfile, setup.py)
 ├── RustDetector     (Cargo.toml)
 ├── GoDetector       (go.mod)
 ├── FlutterDetector  (pubspec.yaml)
 ├── DotNetDetector   (*.csproj, *.sln)
 ├── DockerDetector   (Dockerfile, compose.yml)
 ├── UnityDetector    (ProjectSettings/Assets/Packages)
 └── JavaDetector     (pom.xml, build.gradle(.kts))
```

Adding a detector = implement the trait + register it. See
[`docs/DETECTORS.md`](docs/DETECTORS.md).

---

## Privacy & security

- **Local-first:** works fully offline. No account, no cloud, no backend.
- **No source upload:** DevDock reads only directory listings, marker files
  (e.g. `package.json` *scripts*), and Git status (read-only). It never reads
  or transmits your source code, environment variables, or keys.
- **No telemetry by default.**
- **No auto-execution:** the app never runs project commands on its own; every
  open/terminal action is user-initiated via explicit argv (no shell), which
  eliminates shell-injection.
- **No elevated privileges:** normal operation never asks for admin/root.

See [`docs/SECURITY.md`](docs/SECURITY.md).

---

## Contributing

Contributions are welcome! Read [`CONTRIBUTING.md`](CONTRIBUTING.md) first —
it covers how to add a detector, the code of conduct, and the dev setup.

## Supporting

DevDock is free and open source (MIT). If it helps your workflow, consider:

- ⭐ starring the repo and sharing it with other developers
- 🐛 filing issues / PRs for bugs, ideas, or new detectors


## License

[MIT](LICENSE) © Arda Tekin
